use anyhow::Context;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::Config;

// improve things by borrowing from the rustup approach
// https://github.com/rust-lang/rustup/blob/bcfac6278c7c2f16a41294f7533aeee2f7f88d07/src/cli/self_update/shell.rs

enum KnownShell {
    Posix,
    Bash,
    Zsh,
}

impl KnownShell {
    fn rc_files(&self) -> Vec<&str> {
        match self {
            KnownShell::Posix => vec![".profile"],
            KnownShell::Bash => vec![".bash_profile", ".bash_login", ".bashrc"],
            KnownShell::Zsh => vec![".zshrc"],
        }
    }
}

fn known_shells() -> Vec<KnownShell> {
    vec![KnownShell::Posix, KnownShell::Bash, KnownShell::Zsh]
}

fn source_cmd(root_dir: &Path) -> String {
    format!(
        r#"
export TX3_ROOT="{}"
export PATH="$TX3_ROOT/default/bin:$PATH"
"#,
        root_dir.to_str().unwrap()
    )
}

fn file_contains(profile_path: &Path, source_cmd: &str) -> bool {
    let contents = std::fs::read_to_string(profile_path).unwrap();
    contents.contains(source_cmd)
}

fn append_file(profile_path: &Path, source_cmd: &str) -> anyhow::Result<()> {
    println!(
        "Appending to profile file: {}",
        profile_path.to_str().unwrap()
    );

    let mut profile = OpenOptions::new()
        .append(true)
        .create(false)
        .open(profile_path)?;

    write!(profile, "{}", source_cmd)?;
    profile.flush()?;

    Ok(())
}

fn update_all_profiles(config: &Config) -> anyhow::Result<()> {
    for sh in known_shells() {
        let source_cmd = source_cmd(&config.root_dir());

        for rc in sh.rc_files() {
            let profile_path = dirs::home_dir()
                .context("can't find user's home dir")?
                .join(rc);

            if !profile_path.exists() {
                continue;
            }

            if file_contains(&profile_path, &source_cmd) {
                println!(
                    "{} already contains the source command",
                    profile_path.to_str().unwrap()
                );
                continue;
            }

            append_file(&profile_path, &source_cmd)?;
        }
    }

    Ok(())
}

pub fn check_or_update(config: &Config) -> anyhow::Result<()> {
    update_all_profiles(config)?;

    println!("\nRestart your shell or run:");
    println!("{}", source_cmd(&config.root_dir()));

    Ok(())
}
