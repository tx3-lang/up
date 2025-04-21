use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use crate::{Config, perm_path};

#[derive(Parser, Default)]
pub struct Args {
    #[arg(default_value = "stable")]
    pub channel: String,
}

fn set_default_channel(tx3_root: &PathBuf, channel: &str) -> Result<()> {
    let default_path = tx3_root.join("default");
    let channel_path = tx3_root.join(channel);

    // Remove existing symlink if it exists
    if default_path.exists() {
        fs::remove_file(&default_path)?;
    }

    // Create new symlink
    symlink(&channel_path, &default_path)?;

    Ok(())
}

pub async fn run(args: &Args, config: &Config) -> anyhow::Result<()> {
    set_default_channel(&config.root_dir(), &args.channel)?;
    println!("Set default channel to {}", args.channel);

    println!("updating PATH variable");
    perm_path::check_or_update(config)?;
    Ok(())
}
