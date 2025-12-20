use serde::Deserialize;
use std::sync::OnceLock;

static PROJECT_CONFIG: OnceLock<ProjectConfig> = OnceLock::new();

const CONFIG_TOML: &str = include_str!("../config.toml");

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub project: Project,
    pub binaries: Binaries,
    pub release: Release,
    pub components: Vec<Component>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub version: String,
    pub edition: String,
    pub license: String,
    pub authors: Vec<String>,
    pub repository: String,
    pub homepage: String,
}

#[derive(Debug, Deserialize)]
pub struct Binaries {
    pub cli: String,
    pub indexer_cli: String,
    pub indexer_http: String,
    pub indexer_mcp: String,
    pub tasks_cli: String,
    pub tasks_http: String,
    pub tasks_mcp: String,
    #[serde(default)]
    pub tarminal: String,
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub targets: Vec<String>,
    pub cli_tag_pattern: String,
    pub indexer_tag_pattern: String,
    #[serde(default)]
    pub tasks_tag_pattern: String,
}

#[derive(Debug, Deserialize)]
pub struct Component {
    pub name: String,
    pub binary: String,
    pub description: String,
    #[serde(rename = "crate")]
    pub crate_name: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub repo: String,
}

impl Component {
    pub fn parse_repo(&self) -> (&str, &str) {
        let parts: Vec<&str> = self.repo.split('/').collect();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            panic!("Invalid repo format: {}", self.repo)
        }
    }
}

impl ProjectConfig {
    pub fn get() -> &'static ProjectConfig {
        PROJECT_CONFIG
            .get_or_init(|| toml::from_str(CONFIG_TOML).expect("Failed to parse config.toml"))
    }

    /// Parse repository URL to get owner and repo name
    /// Example: "https://github.com/adi-family/cli" -> ("adi-family", "cli")
    pub fn parse_repository(&self) -> (&str, &str) {
        let url = self.project.repository.trim_end_matches('/');
        let parts: Vec<&str> = url.split('/').collect();

        if parts.len() >= 2 {
            let owner = parts[parts.len() - 2];
            let repo = parts[parts.len() - 1];
            (owner, repo)
        } else {
            panic!("Invalid repository URL format: {}", self.project.repository)
        }
    }

    /// Get component by name
    pub fn get_component(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loads() {
        let config = ProjectConfig::get();
        assert_eq!(config.project.version, "0.8.3");
        assert_eq!(config.binaries.cli, "adi");
        assert_eq!(config.binaries.indexer_cli, "adi-indexer-cli");
    }

    #[test]
    fn test_parse_repository() {
        let config = ProjectConfig::get();
        let (owner, repo) = config.parse_repository();
        assert_eq!(owner, "adi-family");
        assert_eq!(repo, "adi-cli");
    }

    #[test]
    fn test_get_component() {
        let config = ProjectConfig::get();
        let component = config.get_component("indexer-cli").unwrap();
        assert_eq!(component.binary, "adi-indexer-cli");
        assert_eq!(component.crate_name, "adi-indexer-cli");
        assert_eq!(component.version, "0.8.3");
        assert_eq!(component.dependencies, Vec::<String>::new());
    }
}
