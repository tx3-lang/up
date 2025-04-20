pub struct Tool {
    pub name: String,
    pub description: String,
    pub min_version: String,
    pub repo_owner: String,
    pub repo_name: String,
}

use std::sync::OnceLock;

static TOOLS: OnceLock<Vec<Tool>> = OnceLock::new();

pub fn all_tools() -> impl Iterator<Item = &'static Tool> {
    TOOLS
        .get_or_init(|| {
            vec![
                Tool {
                    name: "trix".to_string(),
                    description: "The tx3 package manager".to_string(),
                    min_version: "0.1.0".to_string(),
                    repo_owner: "tx3-lang".to_string(),
                    repo_name: "trix".to_string(),
                },
                Tool {
                    name: "lsp".to_string(),
                    description: "A language server for tx3".to_string(),
                    min_version: "0.1.0".to_string(),
                    repo_owner: "tx3-lang".to_string(),
                    repo_name: "lsp".to_string(),
                },
                Tool {
                    name: "dolos".to_string(),
                    description: "A lightweight Cardano data node".to_string(),
                    min_version: "0.1.0".to_string(),
                    repo_owner: "txpipe".to_string(),
                    repo_name: "dolos".to_string(),
                },
                Tool {
                    name: "cshell".to_string(),
                    description: "A terminal wallet for Cardano".to_string(),
                    min_version: "0.1.0".to_string(),
                    repo_owner: "txpipe".to_string(),
                    repo_name: "cshell".to_string(),
                },
            ]
        })
        .iter()
}

pub fn tool_by_name(name: &str) -> impl Iterator<Item = &'static Tool> {
    all_tools().filter(move |t| t.name == name)
}
