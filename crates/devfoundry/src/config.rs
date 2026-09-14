use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

const CONFIG_FILE: &str = "devfoundry.json";
const GLOBAL_CONFIG_DIR: &str = "devfoundry";
const GLOBAL_CONFIG_FILE: &str = "config.json";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    Boss,
    Build,
    Plan,
    Review,
    Test,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AgentProfileConfig {
    pub name: String,
    pub description: String,
    pub mode: AgentMode,
    pub model: Option<String>,
    pub step_limit: u32,
    pub max_workers: u32,
    pub allow_worker_spawn: bool,
}

pub fn built_in_profiles() -> Vec<AgentProfileConfig> {
    [
        (
            "boss",
            "Coordinate explicit worker assignments",
            AgentMode::Boss,
            3,
            true,
        ),
        (
            "build",
            "Implement approved code changes",
            AgentMode::Build,
            1,
            false,
        ),
        (
            "plan",
            "Inspect and plan without mutation",
            AgentMode::Plan,
            1,
            false,
        ),
        (
            "review",
            "Review changes and report risks",
            AgentMode::Review,
            1,
            false,
        ),
        ("test", "Run and diagnose tests", AgentMode::Test, 1, false),
    ]
    .into_iter()
    .map(
        |(name, description, mode, max_workers, allow_worker_spawn)| AgentProfileConfig {
            name: name.into(),
            description: description.into(),
            mode,
            model: None,
            step_limit: 12,
            max_workers,
            allow_worker_spawn,
        },
    )
    .collect()
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub model: Option<String>,
    pub default_agent: Option<String>,
    pub instructions: Vec<String>,
    pub provider: ProviderConfig,
    pub agents: AgentConfig,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct AgentConfig {
    pub default: Option<String>,
    pub profiles: Vec<AgentProfileConfig>,
    pub max_workers: Option<u32>,
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
    #[error("unknown agent profile: {0}")]
    UnknownAgent(String),
}

pub fn load_from_root(root: &Path) -> Result<Config, ConfigError> {
    let path = root.join(CONFIG_FILE);
    let mut config = load_optional(&global_config_path())?.unwrap_or_default();
    if let Some(project) = load_optional(&path)? {
        merge_config(&mut config, project);
    }
    validate_profiles(&config)?;
    Ok(config)
}

fn load_optional(path: &Path) -> Result<Option<Config>, ConfigError> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&contents)
        .map(Some)
        .map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })
}

fn global_config_path() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join(GLOBAL_CONFIG_DIR)
        .join(GLOBAL_CONFIG_FILE)
}

fn merge_config(base: &mut Config, project: Config) {
    if project.model.is_some() {
        base.model = project.model;
    }
    if project.default_agent.is_some() {
        base.default_agent = project.default_agent;
    }
    if !project.instructions.is_empty() {
        base.instructions = project.instructions;
    }
    if project.provider != ProviderConfig::default() {
        base.provider = project.provider;
    }
    if project.agents.default.is_some() {
        base.agents.default = project.agents.default;
    }
    if project.agents.max_workers.is_some() {
        base.agents.max_workers = project.agents.max_workers;
    }
    if !project.agents.profiles.is_empty() {
        base.agents.profiles = project.agents.profiles;
    }
}

fn validate_profiles(config: &Config) -> Result<(), ConfigError> {
    let names = built_in_profiles()
        .into_iter()
        .map(|profile| profile.name)
        .collect::<std::collections::HashSet<_>>();
    for profile in &config.agents.profiles {
        if !names.contains(&profile.name) {
            return Err(ConfigError::UnknownAgent(profile.name.clone()));
        }
    }
    if let Some(agent) = config
        .agents
        .default
        .as_deref()
        .or(config.default_agent.as_deref())
        && !names.contains(agent)
    {
        return Err(ConfigError::UnknownAgent(agent.into()));
    }
    Ok(())
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

    #[test]
    fn built_in_profiles_are_deterministic() {
        let profiles = built_in_profiles();
        assert_eq!(profiles.len(), 5);
        assert_eq!(profiles[0].name, "boss");
        assert!(profiles[0].allow_worker_spawn);
        assert_eq!(profiles[2].mode, AgentMode::Plan);
    }

    #[test]
    fn project_agent_override_is_loaded() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join(CONFIG_FILE),
            br#"{"agents":{"default":"plan","max_workers":2}}"#,
        )
        .unwrap();
        let config = load_from_root(directory.path()).unwrap();
        assert_eq!(config.agents.default.as_deref(), Some("plan"));
        assert_eq!(config.agents.max_workers, Some(2));
    }

    #[test]
    fn unknown_agent_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join(CONFIG_FILE),
            br#"{"agents":{"default":"custom"}}"#,
        )
        .unwrap();
        assert!(matches!(
            load_from_root(directory.path()),
            Err(ConfigError::UnknownAgent(name)) if name == "custom"
        ));
    }
}
