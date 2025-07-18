use clap::Parser;
use clap::ValueEnum;

use crate::ArgsCommon;
use crate::{Config, manifest, updates};

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

#[derive(Parser, Default)]
pub struct Args {
    #[arg(short, long)]
    pub silent: bool,

    /// Force
    #[arg(short, long)]
    pub force: bool,

    /// Print details of each update
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub output: Option<OutputFormat>,
}

impl ArgsCommon for Args {
    fn skip_banner(&self) -> bool {
        self.silent || matches!(self.output, Some(OutputFormat::Json))
    }
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

fn text_output(
    updates: &[updates::Update],
    manifest: &manifest::Manifest,
    verbose: bool,
) -> anyhow::Result<()> {
    if updates.is_empty() {
        println!("You are up to date 🎉");
        return Ok(());
    }

    if !verbose {
        println!("You have {} update/s to install 📦", updates.len());
    } else {
        for update in updates {
            print_update(&update, &manifest)?;
        }
    }

    Ok(())
}

fn json_output(updates: &[updates::Update]) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(&updates)?;
    println!("{}", json);
    Ok(())
}

pub async fn run(args: &Args, config: &Config) -> anyhow::Result<()> {
    let manifest = manifest::load_latest_manifest(config, args.force).await?;

    let updates = updates::load_updates(&manifest, config, args.force).await?;

    if args.silent {
        return Ok(());
    }

    let output = args.output.as_ref().unwrap_or(&OutputFormat::Text);

    match output {
        OutputFormat::Json => json_output(&updates)?,
        OutputFormat::Text => text_output(&updates, &manifest, args.verbose)?,
    };

    Ok(())
}
