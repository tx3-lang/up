use std::process::Command;

use crate::Config;

#[derive(Debug, clap::Parser)]
pub struct Args {
    pub tool: Option<String>,
}

fn print_tool(tool: &crate::tools::Tool, config: &Config) -> anyhow::Result<()> {
    println!("bin path: {}", tool.bin_path(config).display());

    println!(
        "github repo: https://github.com/{}/{}",
        tool.repo_owner, tool.repo_name
    );

    let version = Command::new(tool.bin_path(config))
        .arg(tool.version_cmd())
        .output()?;

    let version = String::from_utf8(version.stdout)?;

    let version = if version.is_empty() {
        "not reported\n".to_string()
    } else {
        version
    };

    println!("version: {}", version);

    Ok(())
}

pub async fn run(_args: &Args, config: &Config) -> anyhow::Result<()> {
    // for each tool, trigger a shell command to print the version
    for tool in crate::tools::all_tools() {
        println!("{}: {}", tool.name, tool.description);
        println!("min version: {}", tool.min_version);

        let ok = print_tool(tool, config);

        if let Err(e) = ok {
            println!("error: {}", e);
        }
    }

    Ok(())
}
