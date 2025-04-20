use anyhow::{Context, Result};
use clap::Parser;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use octocrab::Octocrab;
use octocrab::models::repos::Asset;
use reqwest::Client;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tar::Archive;

use crate::{Config, tools::*};

#[derive(Parser, Default)]
pub struct Args {
    pub tool: Option<String>,
    pub version: Option<String>,
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

fn extract_binary(path: &PathBuf, install_dir: &PathBuf, tool_name: &str) -> Result<()> {
    let tar_gz = fs::File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    // Find the first file that starts with the tool name
    let mut binary_entry = None;
    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid filename in archive")?;

        if filename.starts_with(tool_name) && entry.header().entry_type().is_file() {
            binary_entry = Some(entry);
            break;
        }
    }

    let mut binary_entry = binary_entry.context("No matching binary found in archive")?;
    let binary_path = install_dir.join(tool_name);
    binary_entry.unpack(&binary_path)?;

    // Make the binary executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

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
    download_binary(&binary_asset.browser_download_url.to_string(), &binary_path).await?;

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
    let tools: Vec<_> = match &args.tool {
        Some(tool) => tool_by_name(&tool).collect(),
        None => all_tools().collect(),
    };

    if tools.is_empty() {
        return Err(anyhow::anyhow!("No tools found to install"));
    }

    for tool in tools {
        println!("Installing {}...", tool.name);
        install_tool(tool, &args.version, config).await?;
    }

    Ok(())
}
