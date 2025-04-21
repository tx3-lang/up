use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod cmds;
mod perm_path;
mod tools;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, env = "TX3_ROOT")]
    root_dir: Option<PathBuf>,

    #[arg(short, long, env = "TX3_CHANNEL")]
    channel: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install the tx3 toolchain
    Install(cmds::install::Args),
    /// Update the tx3 toolchain to the latest version
    Update,
    /// Uninstall the tx3 toolchain
    Uninstall,
    /// Set the default channel
    Default(cmds::default::Args),
    /// Show the version of the tx3 toolchain
    Show(cmds::show::Args),
}

pub struct Config {
    root_dir: Option<PathBuf>,
    channel: Option<String>,
}

impl Config {
    pub fn default_root_dir() -> Result<PathBuf> {
        let mut path = if cfg!(target_os = "windows") {
            dirs::data_local_dir()
        } else {
            dirs::home_dir()
        }
        .context("Could not determine home directory")?;

        path.push(".tx3");

        Ok(path)
    }

    pub fn root_dir(&self) -> PathBuf {
        self.root_dir
            .clone()
            .unwrap_or_else(|| Self::default_root_dir().unwrap())
    }

    pub fn channel(&self) -> String {
        self.channel.clone().unwrap_or_else(|| "stable".to_string())
    }

    pub fn channel_dir(&self) -> PathBuf {
        self.root_dir().join(self.channel())
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.channel_dir().join("bin")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config {
        root_dir: cli.root_dir,
        channel: cli.channel,
    };

    if let Some(command) = cli.command {
        match command {
            Commands::Install(args) => cmds::install::run(&args, &config).await?,
            Commands::Update => todo!(),
            Commands::Uninstall => todo!(),
            Commands::Default(args) => cmds::default::run(&args, &config).await?,
            Commands::Show(args) => cmds::show::run(&args, &config).await?,
        }
    } else {
        cmds::install::run(&cmds::install::Args::default(), &config).await?;
        cmds::default::run(&cmds::default::Args::default(), &config).await?;
    }

    Ok(())
}
