use tokio::process::Command;

use anyhow::Context;
use semver::Version;

use crate::{Config, manifest::Tool};

pub async fn run_version_cmd(tool: &Tool, config: &Config) -> anyhow::Result<String> {
    let version = tool.version_cmd();

    let bin_path = tool.bin_path(config);

    let output = Command::new(bin_path)
        .arg(version)
        .output()
        .await
        .context("running version command")?;

    String::from_utf8(output.stdout).context("parsing version output")
}

pub async fn check_current_version(tool: &Tool, config: &Config) -> anyhow::Result<Version> {
    let raw_version = run_version_cmd(tool, config).await?;

    let raw_version = raw_version
        .split_whitespace()
        .last()
        .context("no version found in output")?;

    let version = Version::parse(raw_version).context("parsing version")?;

    Ok(version)
}
