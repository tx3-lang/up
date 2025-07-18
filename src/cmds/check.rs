use clap::Parser;

use crate::{Config, manifest, updates};

#[derive(Parser, Default)]
pub struct Args {
    #[arg(short, long)]
    pub silent: bool,

    #[arg(short, long)]
    pub force: bool,

    #[arg(short, long)]
    pub verbose: bool,
}

fn print_update(update: &updates::Update, manifest: &manifest::Manifest) -> anyhow::Result<()> {
    let tool = manifest.tool_by_name(&update.tool).unwrap();

    if let Some(current) = update.current()? {
        println!("\nYour version of {} needs to be updated 😬", tool.name);
        println!("  Current version: {current}");
        println!("  Requested version: {}", update.requested);
    } else {
        println!("\nYour need to install {} 📦", tool.name);
    }

    Ok(())
}

pub async fn run(args: &Args, config: &Config) -> anyhow::Result<()> {
    let manifest = manifest::load_latest_manifest(config, args.force).await?;

    let updates = updates::load_updates(&manifest, config, args.force).await?;

    if args.silent {
        return Ok(());
    }

    if updates.is_empty() {
        println!("You are up to date 🎉");
        return Ok(());
    }

    if !args.verbose {
        println!("You have {} update/s to install 📦", updates.len());
    } else {
        for update in updates {
            print_update(&update, &manifest)?;
        }
    }

    Ok(())
}
