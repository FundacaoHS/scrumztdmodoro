use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_COLUMNS: &[&str] = &["backlog", "todo", "doing", "testing", "done"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Caminho do vault (diretorio com arquivos .md)
    pub vault: PathBuf,

    /// Colunas do scrum board
    pub scrum_columns: Vec<String>,

    /// Nome do projeto
    pub project_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault: default_vault_path(),
            scrum_columns: DEFAULT_COLUMNS.iter().map(|s| s.to_string()).collect(),
            project_name: String::from("sm"),
        }
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Carrega config de um arquivo YAML
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }

    /// Salva config em um arquivo YAML
    pub fn save(&self, path: impl Into<PathBuf>) -> Result<(), ConfigError> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Adiciona uma config com base na chave fornecida.
    /// Se o vault nao existir no disco, cria + salva .conf.
    /// Se ja existir, retorna o config atual.
    pub fn add(&mut self, key: ConfigKey) -> Result<&Config, ConfigError> {
        match key {
            ConfigKey::Vault(cfg) => {
                if let Some(path) = cfg.path {
                    self.vault = path;
                    if !self.exists() {
                        let vault = crate::Vault::new(&self.vault);
                        vault.ensure()?;
                        let conf_path = default_conf_path();
                        self.save(conf_path)?;
                    }
                }
                Ok(self)
            }
            ConfigKey::Column(columns) => {
                self.scrum_columns = columns;
                Ok(self)
            }
            ConfigKey::Todo(_cfg) => {
                Ok(self)
            }
            ConfigKey::Project(cfg) => {
                if let Some(name) = cfg.name {
                    self.project_name = name;
                }
                Ok(self)
            }
        }
    }

    /// Verifica se o vault ja existe no disco (privado)
    fn exists(&self) -> bool {
        self.vault.exists()
    }
}

/// Chave que define o tipo de dado aceito na config
#[derive(Debug, Clone)]
pub enum ConfigKey {
    /// Configuracoes do vault (diretorio de arquivos .md)
    Vault(VaultConfig),
    /// Colunas do scrum board
    Column(Vec<String>),
    /// Configuracoes de todo
    Todo(TodoConfig),
    /// Configuracoes do projeto
    Project(ProjectConfig),
}

/// Configuracoes do vault
#[derive(Debug, Clone, Default)]
pub struct VaultConfig {
    /// Caminho do diretorio vault
    pub path: Option<PathBuf>,
}

impl VaultConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// Configuracoes de todo (scaffold)
#[derive(Debug, Clone, Default)]
pub struct TodoConfig {
    pub file_name: Option<String>,
}

impl TodoConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = Some(name.into());
        self
    }
}

/// Configuracoes do projeto
#[derive(Debug, Clone, Default)]
pub struct ProjectConfig {
    pub name: Option<String>,
}

impl ProjectConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    vault: Option<PathBuf>,
    scrum_columns: Option<Vec<String>>,
    project_name: Option<String>,
}

impl ConfigBuilder {
    pub fn vault(mut self, path: impl Into<PathBuf>) -> Self {
        self.vault = Some(path.into());
        self
    }

    pub fn scrum_columns(mut self, columns: impl Into<Vec<String>>) -> Self {
        self.scrum_columns = Some(columns.into());
        self
    }

    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    pub fn build(self) -> Config {
        Config {
            vault: self.vault.unwrap_or_else(default_vault_path),
            scrum_columns: self
                .scrum_columns
                .unwrap_or_else(|| DEFAULT_COLUMNS.iter().map(|s| s.to_string()).collect()),
            project_name: self.project_name.unwrap_or_else(|| String::from("sm")),
        }
    }
}

pub fn default_vault_path() -> PathBuf {
    let base = directories::ProjectDirs::from("com", "fundacao", "sm")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./.sm"));
    base.join("vault")
}

pub fn default_conf_path() -> PathBuf {
    let base = directories::ProjectDirs::from("com", "fundacao", "sm")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./.sm"));
    base.join("sm.yaml")
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Parse(#[from] serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_exists_returns_false_when_vault_missing() {
        let cfg = Config {
            vault: PathBuf::from("/tmp/non_existent_vault_12345"),
            scrum_columns: Default::default(),
            project_name: "test".into(),
        };
        assert!(!cfg.exists());
    }

    #[test]
    fn test_exists_returns_true_when_vault_exists() {
        let dir = std::env::temp_dir().join("test_vault_exists_12345");
        fs::create_dir_all(&dir).unwrap();

        let cfg = Config {
            vault: dir.clone(),
            scrum_columns: Default::default(),
            project_name: "test".into(),
        };
        assert!(cfg.exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_add_creates_vault_and_returns_config() {
        let dir = std::env::temp_dir().join("unit_test_add_create");
        let _ = fs::remove_dir_all(&dir);

        let mut cfg = Config::default();
        let result = cfg
            .add(ConfigKey::Vault(VaultConfig::new().path(&dir)))
            .unwrap();

        assert_eq!(result.vault, dir);
        assert!(dir.exists());
        assert_eq!(result.project_name, "sm");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_add_with_existing_vault_returns_config() {
        let dir = std::env::temp_dir().join("unit_test_add_exists");
        fs::create_dir_all(&dir).unwrap();

        let mut cfg = Config::default();
        let result = cfg
            .add(ConfigKey::Vault(VaultConfig::new().path(&dir)))
            .unwrap();

        assert_eq!(result.vault, dir);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_add_no_path_returns_config_unchanged() {
        let mut cfg = Config::default();
        let original = cfg.vault.clone();
        let result = cfg.add(ConfigKey::Vault(VaultConfig::new())).unwrap();

        assert_eq!(result.vault, original);
    }

    #[test]
    fn test_add_column_updates_scrum_columns() {
        let mut cfg = Config::default();
        let cols = vec!["backlog".into(), "todo".into(), "testing".into()];

        let result = cfg.add(ConfigKey::Column(cols.clone())).unwrap();

        assert_eq!(result.scrum_columns, cols);
    }

    #[test]
    fn test_add_project_updates_name() {
        let mut cfg = Config::default();

        let result = cfg
            .add(ConfigKey::Project(ProjectConfig::new().name("meu-proj")))
            .unwrap();

        assert_eq!(result.project_name, "meu-proj");
    }

    #[test]
    fn test_add_todo_returns_config() {
        let mut cfg = Config::default();

        let result = cfg
            .add(ConfigKey::Todo(TodoConfig::new().file_name("tasks.md")))
            .unwrap();

        assert_eq!(result.project_name, "sm");
    }
}
