use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

const CONFIG_FILE: &str = "devfoundry.json";

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub model: Option<String>,
    pub default_agent: Option<String>,
    pub instructions: Vec<String>,
    pub provider: ProviderConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProviderConfig {
    pub kind: String,
    pub base_url: String,
    pub token_env: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            kind: "github-copilot".into(),
            base_url: "https://api.githubcopilot.com".into(),
            token_env: Some("GITHUB_COPILOT_TOKEN".into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid JSON in {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

pub fn load_from_root(root: &Path) -> Result<Config, ConfigError> {
    let path = root.join(CONFIG_FILE);
    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&path).map_err(|source| ConfigError::Read {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|source| ConfigError::Parse { path, source })
}

pub fn discover_root(start: &Path) -> Result<PathBuf, std::io::Error> {
    let start = start.canonicalize()?;
    let directory = if start.is_dir() {
        start
    } else {
        start
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| std::io::Error::other("path has no parent directory"))?
    };

    for candidate in directory.ancestors() {
        if candidate.join(".git").exists() || candidate.join(CONFIG_FILE).exists() {
            return Ok(candidate.to_path_buf());
        }
    }

    Ok(directory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn missing_config_uses_defaults() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(load_from_root(directory.path()).unwrap(), Config::default());
    }

    #[test]
    fn root_discovery_finds_config_ancestor() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("src").join("module");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.path().join(CONFIG_FILE), b"{}").unwrap();
        assert_eq!(
            discover_root(&nested).unwrap(),
            root.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn copilot_provider_settings_deserialize() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join(CONFIG_FILE),
            br#"{"provider":{"kind":"github-copilot","base_url":"http://127.0.0.1:9000","token_env":"TEST_COPILOT_TOKEN"}}"#,
        )
        .unwrap();
        let config = load_from_root(directory.path()).unwrap();
        assert_eq!(config.provider.kind, "github-copilot");
        assert_eq!(config.provider.base_url, "http://127.0.0.1:9000");
        assert_eq!(
            config.provider.token_env.as_deref(),
            Some("TEST_COPILOT_TOKEN")
        );
    }
}
