use anyhow::{Context, Result};
use clap::Parser;
use flate2::read::GzDecoder;
use octocrab::Octocrab;
use octocrab::models::repos::Asset;
use octocrab::models::repos::Release;
use reqwest::Client;
use semver::Version;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tar::Archive;
use xz2::read::XzDecoder;

use crate::bin;
use crate::manifest;
use crate::{Config, manifest::*};

#[derive(Parser, Default)]
pub struct Args {
    pub tool: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionsFile {
    tools: Vec<ToolVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolVersion {
    repo_name: String,
    repo_owner: String,
    version: String,
}

pub async fn download_binary(url: &str, path: &PathBuf) -> Result<()> {
    let client = Client::new();
    let mut response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(0);
    let mut file = fs::File::create(path)?;
    let mut downloaded = 0;

    while let Some(chunk_result) = response.chunk().await? {
        file.write_all(&chunk_result)?;
        downloaded += chunk_result.len() as u64;

        if total_size > 0 {
            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            print!(
                "\r> Downloading: {:.1}% ({}/{})",
                progress, downloaded, total_size
            );
            std::io::stdout().flush()?;
        }
    }
    println!(); // New line after progress

    Ok(())
}

// Find the first file that starts with the tool name
fn extract_binary_main_entry<R: std::io::Read>(
    mut archive: Archive<R>,
    install_dir: &Path,
    tool_name: &str,
) -> anyhow::Result<()> {
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid filename in archive")?;

        if filename.eq(tool_name) && entry.header().entry_type().is_file() {
            let binary_path = install_dir.join(filename);
            entry.unpack(&binary_path)?;

            // Make the binary executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&binary_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&binary_path, perms)?;
            }

            return Ok(());
        }
    }

    anyhow::bail!("No matching binary found in archive")
}

fn extract_binary(path: &Path, install_dir: &Path, tool_name: &str) -> Result<()> {
    let file = fs::File::open(path)?;

    let extension = path.extension().and_then(|s| s.to_str());

    match extension {
        Some("gz") => {
            let archive = Archive::new(GzDecoder::new(file));
            extract_binary_main_entry(archive, install_dir, tool_name)?;
        }
        Some("xz") => {
            let archive = Archive::new(XzDecoder::new(file));
            extract_binary_main_entry(archive, install_dir, tool_name)?;
        }
        _ => anyhow::bail!("Unsupported archive format. Expected .gz or .xz"),
    };

    Ok(())
}

fn find_arch_asset(tool_name: &str, release: Release) -> Option<Asset> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    // Map Rust's arch to our convention
    let arch = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return None,
    };

    // Map Rust's OS to our convention
    let os = match os {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        _ => return None,
    };

    let target = format!("{}-{}-{}", tool_name, arch, os);

    release
        .assets
        .iter()
        .find(|asset| asset.name.contains(&target))
        .cloned()
}

pub async fn download_tool_from_asset(tool: &Tool, asset: &Asset, config: &Config) -> Result<()> {
    println!("> Downloading binary: {}", asset.name);

    // Create installation directory
    let install_dir = config.bin_dir().clone();
    fs::create_dir_all(&install_dir)?;

    // Download the binary
    let binary_path = install_dir.join(&asset.name);
    download_binary(asset.browser_download_url.as_ref(), &binary_path).await?;

    println!("> Extracting binary...");
    extract_binary(&binary_path, &install_dir, &tool.name)?;

    // Clean up the tar.gz file
    fs::remove_file(&binary_path)?;

    println!(
        "Successfully installed {} to {}",
        tool.name,
        install_dir.join(&tool.name).display()
    );
    println!();

    Ok(())
}

async fn find_matching_release(
    tool: &Tool,
    requested: &VersionReq,
) -> anyhow::Result<Option<(Version, Release)>> {
    let octocrab = Octocrab::builder().build()?;
    let owner = tool.repo_owner.clone();
    let repo = tool.repo_name.clone();
    let repo = octocrab.repos(owner, repo);

    let mut page = repo
        .releases()
        .list()
        .send()
        .await
        .context("Failed to list releases")?;

    for release in page.take_items() {
        let sanitized = if release.tag_name.starts_with("v") {
            release.tag_name[1..].to_string()
        } else {
            release.tag_name.clone()
        };

        let Ok(version) = Version::parse(&sanitized) else {
            continue;
        };

        if requested.matches(&version) {
            return Ok(Some((version, release)));
        }
    }

    Ok(None)
}

async fn find_installed_version(tool: &Tool, config: &Config) -> anyhow::Result<Option<Version>> {
    if !bin::is_installed(tool, config).await? {
        return Ok(None);
    }

    let current_version = bin::check_current_version(tool, config).await;

    match current_version {
        Ok(version) => Ok(Some(version)),
        Err(_) => {
            // if the version command fails, we assume there's something wrong with the
            // binary and respond as if it wasn't installed
            Ok(None)
        }
    }
}

async fn install_tool(tool: &Tool, requested: &VersionReq, config: &Config) -> anyhow::Result<()> {
    println!("\n> Installing {} at version {}", tool.name, requested);

    let Some((version, release)) = find_matching_release(tool, &requested).await? else {
        return Err(anyhow::anyhow!("No release found for {}", tool.name));
    };

    let Some(asset) = find_arch_asset(&tool.name, release) else {
        return Err(anyhow::anyhow!("No asset found for {}", tool.name));
    };

    println!("\nFound version of {} to install 🎉", tool.name);
    println!("  Version: {version}");
    println!("  Asset: {}", asset.name);

    download_tool_from_asset(tool, &asset, config).await?;

    Ok(())
}

pub async fn run(args: &Args, config: &Config) -> anyhow::Result<()> {
    let manifest = manifest::load_manifest(config, true).await?;

    let to_install: Vec<_> = if let Some(filter) = &args.tool {
        manifest.tools().filter(|x| x.name == *filter).collect()
    } else {
        manifest.tools().collect()
    };

    if to_install.is_empty() {
        return Err(anyhow::anyhow!("No tools found to install"));
    }

    for tool in to_install.iter() {
        let current = find_installed_version(tool, config).await?;
        let requested = VersionReq::parse(&tool.version)?;

        if let Some(current) = current {
            if requested.matches(&current) {
                println!("\nYour version of {} is up to date 👌", tool.name);

                continue;
            } else {
                println!("\nYour version of {} needs to be updated 😬", tool.name);
                println!("  Current version: {current}");
                println!("  Requested version: {requested}");
            }
        } else {
            println!("\nYour need to install {} 📦", tool.name);
        }

        install_tool(tool, &requested, config).await?;
    }

    Ok(())
}
