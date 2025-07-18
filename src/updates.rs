use std::time::{Duration, SystemTime};

use anyhow::Context as _;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    Config, bin,
    manifest::{Manifest, Tool},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub tool: String,
    pub current: Option<String>,
    pub requested: String,
}

impl Update {
    pub fn requested(&self) -> anyhow::Result<VersionReq> {
        VersionReq::parse(&self.requested).context("parsing requested version")
    }

    pub fn current(&self) -> anyhow::Result<Option<Version>> {
        if let Some(current) = &self.current {
            Ok(Some(
                Version::parse(current).context("parsing current version")?,
            ))
        } else {
            Ok(None)
        }
    }
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

async fn evaluate_update(tool: &Tool, config: &Config) -> anyhow::Result<Option<Update>> {
    let current = find_installed_version(tool, config).await?;
    let requested = VersionReq::parse(&tool.version)?;

    if let Some(current) = &current {
        if requested.matches(&current) {
            return Ok(None);
        }
    }

    Ok(Some(Update {
        tool: tool.name.clone(),
        current: current.map(|v| v.to_string()),
        requested: requested.to_string(),
    }))
}

async fn save_updates(updates: &[Update], config: &Config) -> anyhow::Result<()> {
    fs::create_dir_all(config.channel_dir())
        .await
        .context("creating channel dir")?;

    fs::write(
        config.updates_file(),
        serde_json::to_string(&updates)?.as_bytes(),
    )
    .await
    .context("writing updates file")?;

    Ok(())
}

pub async fn clear_updates(config: &Config) -> anyhow::Result<()> {
    let updates_file = config.updates_file();

    if !updates_file.exists() {
        return Ok(());
    }

    fs::remove_file(updates_file)
        .await
        .context("removing updates file")?;

    Ok(())
}

pub async fn check_updates(manifest: &Manifest, config: &Config) -> anyhow::Result<Vec<Update>> {
    let mut updates = vec![];

    for tool in manifest.tools() {
        if let Some(update) = evaluate_update(tool, config).await? {
            updates.push(update);
        }
    }

    if updates.is_empty() {
        clear_updates(config).await?;
    } else {
        save_updates(&updates, config).await?;
    }

    Ok(updates)
}

async fn check_updates_timestamp(config: &Config) -> anyhow::Result<Option<SystemTime>> {
    let updates_file = config.updates_file();

    if !updates_file.exists() {
        return Ok(None);
    }

    let metadata = fs::metadata(updates_file)
        .await
        .context("getting updates file metadata")?;

    let modified = metadata
        .modified()
        .context("getting updates file modified time")?;

    Ok(Some(modified))
}

const UPDATES_STALE_THRESHOLD: Duration = Duration::from_secs(60 * 60 * 24);

fn updates_are_stale(timestamp: Option<SystemTime>) -> bool {
    timestamp.is_none() || timestamp.unwrap() < SystemTime::now() - UPDATES_STALE_THRESHOLD
}

pub async fn load_updates(
    manifest: &Manifest,
    config: &Config,
    force_check: bool,
) -> anyhow::Result<Vec<Update>> {
    let timestamp = check_updates_timestamp(config).await?;

    if force_check || updates_are_stale(timestamp) {
        check_updates(manifest, config).await?;
    }

    let updates_file = config.updates_file();

    if !updates_file.exists() {
        return Ok(vec![]);
    }

    let updates = fs::read_to_string(updates_file)
        .await
        .context("reading updates file")?;

    let updates: Vec<Update> = serde_json::from_str(&updates)?;

    Ok(updates)
}
