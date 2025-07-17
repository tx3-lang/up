use std::{path::PathBuf, vec};
use std::sync::OnceLock;
use octocrab::Octocrab;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestTool {
    repo_name: String,
    repo_owner: String,
    min_version: String,
    max_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Manifest {
    tools: Vec<ManifestTool>,
}

pub struct Tool {
    pub name: String,
    pub description: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub min_version: String,
    pub max_version: String,
}

impl Tool {
    pub fn bin_path(&self, config: &Config) -> PathBuf {
        config.bin_dir().join(self.name.clone())
    }

    pub fn version_cmd(&self) -> String {
        "--version".to_string()
    }
}

static TOOLS: OnceLock<Vec<Tool>> = OnceLock::new();

fn supported_tools() -> impl Iterator<Item = &'static Tool> {
    TOOLS
        .get_or_init(|| {
            vec![
                Tool {
                    name: "trix".to_string(),
                    description: "The tx3 package manager".to_string(),
                    repo_owner: "tx3-lang".to_string(),
                    repo_name: "trix".to_string(),
                    min_version: "".to_string(),
                    max_version: "".to_string(),
                },
                Tool {
                    name: "tx3-lsp".to_string(),
                    description: "A language server for tx3".to_string(),
                    repo_owner: "tx3-lang".to_string(),
                    repo_name: "lsp".to_string(),
                    min_version: "".to_string(),
                    max_version: "".to_string(),
                },
                Tool {
                    name: "dolos".to_string(),
                    description: "A lightweight Cardano data node".to_string(),
                    repo_owner: "txpipe".to_string(),
                    repo_name: "dolos".to_string(),
                    min_version: "".to_string(),
                    max_version: "".to_string(),
                },
                Tool {
                    name: "cshell".to_string(),
                    description: "A terminal wallet for Cardano".to_string(),
                    repo_owner: "txpipe".to_string(),
                    repo_name: "cshell".to_string(),
                    min_version: "".to_string(),
                    max_version: "".to_string(),
                },
            ]
        })
        .iter()
}

pub async fn all_tools() -> anyhow::Result<Vec<Tool>> {
    let octocrab = Octocrab::builder().build()
        .map_err(|e| anyhow::anyhow!("Failed to create Octocrab client: {}", e))?;
    
    let repo = octocrab.repos("tx3-lang", "toolchain");
    
    let release = repo.releases().get_latest().await
        .map_err(|e| anyhow::anyhow!("Failed to fetch latest release: {}", e))?;

    let manifest_asset = release.assets.iter()
        .find(|asset| asset.name == "manifest.json")
        .ok_or_else(|| anyhow::anyhow!("No manifest asset found in latest release"))?;

    let manifest_content = fetch_manifest_content(manifest_asset.browser_download_url.as_ref()).await
        .map_err(|e| anyhow::anyhow!("Failed to fetch manifest: {}", e))?;

    let manifest: Manifest = serde_json::from_str(&manifest_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse manifest file: {}", e))?;

    let mut tools = vec![];
    for tool in manifest.tools {
        let t = supported_tools()
            .find(|t| t.repo_name == tool.repo_name && t.repo_owner == tool.repo_owner);

        if let Some(t) = t {
            tools.push(Tool {
                name: t.name.clone(),
                description: t.description.clone(),
                repo_owner: tool.repo_owner,
                repo_name: tool.repo_name,
                min_version: tool.min_version,
                max_version: tool.max_version,
            });
        }
    }

    Ok(tools)
}

async fn fetch_manifest_content(url: &str) -> anyhow::Result<String> {
    let client = Client::new();
    let response = client.get(url).send().await
        .map_err(|e| anyhow::anyhow!("Failed to fetch manifest: {}", e))?;
    let data = response.text().await
        .map_err(|e| anyhow::anyhow!("Failed to read manifest response: {}", e))?;
    Ok(data)
}

pub async fn tool_by_name(name: &str) -> anyhow::Result<Vec<Tool>> {
    all_tools().await
        .map(|tools| {
            tools.into_iter()
                .filter(|tool| tool.name == name)
                .collect()
        })
}
