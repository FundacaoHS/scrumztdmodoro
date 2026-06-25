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

    /// Adiciona ou atualiza uma config com base na chave fornecida
    pub fn add(&mut self, key: ConfigKey) {
        match key {
            ConfigKey::Vault(cfg) => {
                if let Some(path) = cfg.path {
                    self.vault = path;
                }
            }
        }
    }

    /// Verifica se o vault ja existe no disco (privado)
    #[allow(dead_code)]
    fn exists(&self) -> bool {
        self.vault.exists()
    }
}

/// Chave que define o tipo de dado aceito na config
#[derive(Debug, Clone)]
pub enum ConfigKey {
    /// Configuracoes do vault (diretorio de arquivos .md)
    Vault(VaultConfig),
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

fn default_vault_path() -> PathBuf {
    let base = directories::ProjectDirs::from("com", "fundacao", "sm")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./.sm"));
    base.join("vault")
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
    fn test_add_changes_path() {
        let mut cfg = Config::default();
        let new_path = PathBuf::from("/tmp/novo_vault");

        cfg.add(ConfigKey::Vault(VaultConfig::new().path(&new_path)));

        assert_eq!(cfg.vault, new_path);
    }
}
