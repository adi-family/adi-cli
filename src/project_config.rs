use serde::Deserialize;
use std::sync::OnceLock;

static PROJECT_CONFIG: OnceLock<ProjectConfig> = OnceLock::new();

const CONFIG_TOML: &str = include_str!("../config.toml");

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub project: Project,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub repository: String,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repository() {
        let config = ProjectConfig::get();
        let (owner, repo) = config.parse_repository();
        assert_eq!(owner, "adi-family");
        assert_eq!(repo, "adi-cli");
    }
}
