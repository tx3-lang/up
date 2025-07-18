use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod banner;
mod bin;
mod cmds;
mod manifest;
mod perm_path;
mod updates;

#[derive(Parser)]
#[command(author, version, about, long_about = Some(banner::BANNER))]
struct Cli {
    #[arg(global = true, short, long, env = "TX3_ROOT")]
    root_dir: Option<PathBuf>,

    #[arg(global = true, short, long, env = "TX3_CHANNEL")]
    channel: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install or update the tx3 toolchain
    Install(cmds::install::Args),
    /// Check for updates
    Check(cmds::check::Args),
    /// Uninstall the tx3 toolchain
    Uninstall,
    /// Set the default channel
    #[command(alias("default"))]
    Use(cmds::r#use::Args),
    /// Show the version of the tx3 toolchain
    Show(cmds::show::Args),
}

pub trait ArgsCommon {
    fn skip_banner(&self) -> bool;
}

impl Commands {
    fn skip_banner(&self) -> bool {
        match self {
            Commands::Install(x) => x.skip_banner(),
            Commands::Check(x) => x.skip_banner(),
            Commands::Use(x) => x.skip_banner(),
            Commands::Show(x) => x.skip_banner(),
            Commands::Uninstall => true,
        }
    }
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

    pub fn fixed_channel_dir(&self) -> PathBuf {
        self.root_dir().join("default")
    }

    pub fn fixed_channel(&self) -> anyhow::Result<Option<String>> {
        let fixed_channel_dir = self.fixed_channel_dir();

        if !fixed_channel_dir.exists() {
            return Ok(None);
        }

        let target = std::fs::read_link(fixed_channel_dir).context("reading fixed channel dir")?;

        let channel = target
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("no fixed channel dir"))?;

        Ok(Some(channel.to_str().unwrap().to_string()))
    }

    fn set_fixed_channel(&self, channel: &str) -> Result<()> {
        let fixed_channel_dir = self.fixed_channel_dir();
        let channel_dir = self.root_dir().join(channel);

        // Remove existing symlink if it exists
        if fixed_channel_dir.exists() {
            std::fs::remove_file(&fixed_channel_dir)?;
        }

        std::fs::create_dir_all(&channel_dir)?;

        std::os::unix::fs::symlink(&channel_dir, &fixed_channel_dir)?;

        Ok(())
    }

    pub fn channel(&self) -> anyhow::Result<String> {
        let explicit = self.channel.clone();

        if let Some(explicit) = explicit {
            return Ok(explicit);
        }

        if let Some(default) = self.fixed_channel()? {
            return Ok(default);
        }

        Err(anyhow::anyhow!("no channel set"))
    }

    pub fn ensure_channel(&self) -> String {
        match self.channel() {
            Ok(channel) => channel,
            Err(_) => {
                self.set_fixed_channel("stable").unwrap();
                "stable".to_string()
            }
        }
    }

    pub fn channel_dir(&self) -> PathBuf {
        let channel = self.ensure_channel();
        self.root_dir().join(channel)
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.channel_dir().join("bin")
    }

    pub fn manifest_file(&self) -> PathBuf {
        self.channel_dir().join("manifest.json")
    }

    pub fn updates_file(&self) -> PathBuf {
        self.channel_dir().join("updates.json")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = Config {
        root_dir: cli.root_dir,
        channel: cli.channel,
    };

    let skip_banner = cli.command.as_ref().map_or(false, |c| c.skip_banner());

    if !skip_banner {
        banner::print_banner(&config);
    }

    if let Some(command) = cli.command {
        match command {
            Commands::Install(args) => cmds::install::run(&args, &config).await?,
            Commands::Check(args) => cmds::check::run(&args, &config).await?,
            Commands::Use(args) => cmds::r#use::run(&args, &config).await?,
            Commands::Show(args) => cmds::show::run(&args, &config).await?,
            Commands::Uninstall => todo!(),
        }
    } else {
        cmds::install::run(&cmds::install::Args::default(), &config).await?;
    }

    Ok(())
}
