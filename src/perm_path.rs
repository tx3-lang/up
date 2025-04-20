use dirs;
use std::env;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::Config;

/// Common profile configuration file names in order of preference
const PROFILE_FILES: &[&str] = &[
    //".bash_profile",
    //".bash_login",
    //".profile",
    ".zshrc",
    //".zprofile",
];

pub fn append_file(profile_path: PathBuf, home_dir: PathBuf) -> io::Result<()> {
    println!(
        "Appending to profile file: {}",
        profile_path.to_str().unwrap()
    );

    let mut profile = OpenOptions::new()
        .append(true)
        .create(false)
        .open(profile_path)?;

    writeln!(profile, "\nexport TX3_ROOT={}", home_dir.to_str().unwrap())?;
    writeln!(profile, "export PATH=\"$TX3_ROOT/default/bin:$PATH\"")?;
    profile.flush()
}

#[cfg(target_family = "unix")]
fn update_all_profiles(config: &Config) -> anyhow::Result<()> {
    for profile_file in PROFILE_FILES {
        let profile_path = dirs::home_dir().unwrap().join(profile_file);

        if Path::new(&profile_path).exists() {
            append_file(profile_path, config.root_dir())?;
        }
    }

    Ok(())
}

pub fn check_or_update(config: &Config) -> anyhow::Result<()> {
    let path = env::var("PATH").unwrap();

    if path.contains(".tx3/default/bin") {
        println!("Tx3 toolchain already in $PATH");
        return Ok(());
    }

    update_all_profiles(config)?;

    println!("\nRestart your shell or run:");
    println!(
        "export PATH=\"{}/default/bin:$PATH\"",
        config.root_dir().display()
    );

    Ok(())
}
