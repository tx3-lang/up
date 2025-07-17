use anyhow::Result;
use clap::Parser;
use std::os::unix::fs::symlink;
use std::{fs, path::Path};

use crate::{Config, perm_path};

#[derive(Parser)]
pub struct Args {
    #[arg(default_value = "stable")]
    pub new_channel: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            new_channel: "stable".to_string(),
        }
    }
}

pub async fn run(args: &Args, config: &Config) -> anyhow::Result<()> {
    config.set_fixed_channel(&args.new_channel)?;
    println!("Set fixed channel to {}", args.new_channel);

    println!("updating PATH variable");
    perm_path::check_or_update(config)?;
    Ok(())
}
