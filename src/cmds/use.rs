use clap::Parser;

use crate::{ArgsCommon, Config, perm_path};

#[derive(Parser)]
pub struct Args {
    #[arg(default_value = "stable")]
    pub new_channel: String,
}

impl ArgsCommon for Args {
    fn skip_banner(&self) -> bool {
        false
    }
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
