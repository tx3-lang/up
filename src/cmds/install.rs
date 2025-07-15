use anyhow::{Context, Result};
use clap::Parser;
use flate2::read::GzDecoder;
use octocrab::Octocrab;
use octocrab::models::repos::Asset;
use reqwest::Client;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tar::Archive;
use xz2::read::XzDecoder;
use serde::{Deserialize, Serialize};

use crate::{Config, tools::*};

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
                "\rDownloading: {:.1}% ({}/{})",
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

fn find_arch_asset<'a>(tool_name: &str, assets: &'a [Asset]) -> Option<&'a Asset> {
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
    assets.iter().find(|asset| asset.name.contains(&target))
}

pub async fn install_tool(tool: &Tool, version: &Option<String>, config: &Config) -> Result<()> {
    let octocrab = Octocrab::builder().build()?;
    let owner = tool.repo_owner.clone();
    let repo = tool.repo_name.clone();
    let repo = octocrab.repos(owner, repo);
    let releases = repo.releases();

    // Get the latest release or specific version
    let release = if let Some(version) = version {
        releases.get_by_tag(&format!("v{}", version)).await?
    } else {
        releases.get_latest().await?
    };

    println!("Found release: {}", release.tag_name);

    // Find the binary asset
    let binary_asset = find_arch_asset(&tool.name, &release.assets)
        .context("No binary asset found for current platform")?;

    println!("Downloading binary: {}", binary_asset.name);

    // Create installation directory
    let install_dir = config.bin_dir().clone();
    fs::create_dir_all(&install_dir)?;

    // Download the binary
    let binary_path = install_dir.join(&binary_asset.name);
    download_binary(binary_asset.browser_download_url.as_ref(), &binary_path).await?;

    println!("Extracting binary...");
    extract_binary(&binary_path, &install_dir, &tool.name)?;

    // Clean up the tar.gz file
    fs::remove_file(&binary_path)?;

    println!(
        "Successfully installed {} to {}",
        tool.name,
        install_dir.join(&tool.name).display()
    );

    Ok(())
}

pub async fn run(args: &Args, config: &Config) -> anyhow::Result<()> {
    let tools = match &args.tool {
        Some(tool) => tool_by_name(tool).await?,
        None => all_tools().await?,
    };

    if tools.is_empty() {
        return Err(anyhow::anyhow!("No tools found to install"));
    }

    for tool in &tools {
        println!("Installing {}...", tool.name);
        install_tool(&tool, &args.version, config).await?;
    }

    let versions_content = VersionsFile {
        tools: tools
            .into_iter()
            .map(|tool| ToolVersion {
                repo_name: tool.repo_name,
                repo_owner: tool.repo_owner,
                version: tool.min_version,
            })
            .collect(),
    };

    let content = serde_json::to_string_pretty(&versions_content)
        .map_err(|e| anyhow::anyhow!("Failed to serialize update info: {}", e))?;

    std::fs::write(config.versions_file(), content)
        .map_err(|e| anyhow::anyhow!("Failed to write update file: {}", e))?;

    Ok(())
}
