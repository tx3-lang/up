use std::process::Command;

use crate::{Config, manifest};

#[derive(Debug, clap::Parser)]
pub struct Args {
    pub tool: Option<String>,
}

fn print_tool(tool: &crate::manifest::Tool, config: &Config) -> anyhow::Result<()> {
    println!("bin path: {}", tool.bin_path(config).display());

    println!(
        "github repo: https://github.com/{}/{}",
        tool.repo_owner, tool.repo_name
    );

    println!("required version: {}", tool.version);

    let version = Command::new(tool.bin_path(config))
        .arg(tool.version_cmd())
        .output()?;

    let version = String::from_utf8(version.stdout)?;

    let version = if version.is_empty() {
        "not reported\n".to_string()
    } else {
        version
    };

    println!("installed version: {}", version);

    Ok(())
}

pub async fn run(_args: &Args, config: &Config) -> anyhow::Result<()> {
    let manifest = manifest::load_manifest(config, false).await?;

    for tool in manifest.tools() {
        println!("{}: {}", tool.name, tool.description);

        let ok = print_tool(&tool, config);

        if let Err(e) = ok {
            println!("error: {}", e);
        }
    }

    Ok(())
}
