use anyhow::Context;
use octocrab::{Octocrab, models::repos::Release, repos::RepoHandler};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};
use tokio::fs;

use crate::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub version: String,
}

impl Tool {
    pub fn bin_path(&self, config: &Config) -> PathBuf {
        config.bin_dir().join(self.name.clone())
    }

    pub fn version_cmd(&self) -> String {
        "--version".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    tools: Vec<Tool>,
}

impl Manifest {
    pub fn tools(&self) -> impl Iterator<Item = &Tool> {
        self.tools.iter()
    }

    pub fn tool_by_name(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|tool| tool.name == name)
    }
}

async fn fetch_manifest_content(url: &str) -> anyhow::Result<String> {
    let client = Client::new();

    let response = client.get(url).send().await.context("fetching manifest")?;

    let data = response.text().await.context("reading manifest response")?;

    Ok(data)
}

async fn define_release(
    repo: &RepoHandler<'_>,
    explicit_tag: Option<&str>,
) -> anyhow::Result<Release> {
    if let Some(explicit) = explicit_tag {
        return repo
            .releases()
            .get_by_tag(explicit)
            .await
            .context("fetching release");
    } else {
        repo.releases()
            .get_latest()
            .await
            .context("fetching latest release")
    }
}

pub async fn download_remote_manifest(
    config: &Config,
    explicit_tag: Option<&str>,
) -> anyhow::Result<()> {
    let octocrab = Octocrab::builder()
        .build()
        .context("building octocrab client")?;

    let repo = octocrab.repos("tx3-lang", "toolchain");

    let release = define_release(&repo, explicit_tag).await?;

    let manifest_name = format!("manifest-{}.json", config.ensure_channel());

    let manifest_asset = release
        .assets
        .iter()
        .find(|asset| asset.name == manifest_name)
        .ok_or_else(|| anyhow::anyhow!("No manifest asset found in latest release"))?;

    let manifest_content = fetch_manifest_content(manifest_asset.browser_download_url.as_ref())
        .await
        .context("fetching manifest")?;

    // ensure manifest is valid json and matches the format
    let _: Manifest = serde_json::from_str(&manifest_content).context("parsing manifest file")?;

    fs::create_dir_all(config.channel_dir())
        .await
        .context("creating channel dir")?;

    fs::write(config.manifest_file(), manifest_content)
        .await
        .context("writing manifest file")?;

    Ok(())
}

pub async fn load_local_manifest(config: &Config) -> anyhow::Result<Option<Manifest>> {
    let manifest_file = config.manifest_file();

    if !manifest_file.exists() {
        return Ok(None);
    }

    let manifest_content = fs::read_to_string(manifest_file)
        .await
        .context("reading manifest file")?;

    let manifest: Manifest =
        serde_json::from_str(&manifest_content).context("parsing manifest file")?;

    Ok(Some(manifest))
}

async fn check_manifest_timestamp(config: &Config) -> anyhow::Result<Option<SystemTime>> {
    let manifest_file = config.manifest_file();

    if !manifest_file.exists() {
        return Ok(None);
    }

    let metadata = fs::metadata(manifest_file)
        .await
        .context("getting manifest file metadata")?;

    let modified = metadata
        .modified()
        .context("getting manifest file modified time")?;

    Ok(Some(modified))
}

const MANIFEST_STALE_THRESHOLD: Duration = Duration::from_secs(60 * 60 * 24);

fn manifest_is_stale(timestamp: Option<SystemTime>) -> bool {
    timestamp.is_none() || timestamp.unwrap() < SystemTime::now() - MANIFEST_STALE_THRESHOLD
}

pub async fn load_latest_manifest(
    config: &Config,
    force_download: bool,
) -> anyhow::Result<Manifest> {
    let timestamp = check_manifest_timestamp(config).await?;

    if manifest_is_stale(timestamp) || force_download {
        download_remote_manifest(config, None).await?;
    }

    let manifest = load_local_manifest(config)
        .await?
        .ok_or(anyhow::anyhow!("Manifest file should exist"))?;

    Ok(manifest)
}

pub async fn load_tagged_manifest(config: &Config, explicit_tag: &str) -> anyhow::Result<Manifest> {
    download_remote_manifest(config, Some(explicit_tag)).await?;

    let manifest = load_local_manifest(config)
        .await?
        .ok_or(anyhow::anyhow!("Manifest file should exist"))?;

    Ok(manifest)
}
