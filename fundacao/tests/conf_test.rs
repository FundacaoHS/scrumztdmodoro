use std::fs;
use std::path::PathBuf;

use fundacao::{AddStatus, Config, ConfigKey, VaultConfig};

#[test]
fn test_add_creates_vault_on_disk() {
    let dir = std::env::temp_dir().join("int_test_add_create");
    let _ = fs::remove_dir_all(&dir);

    let mut cfg = Config::default();
    let status = cfg
        .add(ConfigKey::Vault(VaultConfig::new().path(&dir)))
        .unwrap();

    assert_eq!(status, AddStatus::Created);
    assert!(dir.exists());
    assert!(dir.join("done").exists());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_add_returns_already_exists() {
    let dir = std::env::temp_dir().join("int_test_add_exists");
    fs::create_dir_all(&dir).unwrap();

    let mut cfg = Config::default();
    let status = cfg
        .add(ConfigKey::Vault(VaultConfig::new().path(&dir)))
        .unwrap();

    assert_eq!(status, AddStatus::AlreadyExists);

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_add_no_path_returns_no_change() {
    let mut cfg = Config::default();
    let original = cfg.vault.clone();

    let status = cfg.add(ConfigKey::Vault(VaultConfig::new())).unwrap();

    assert_eq!(status, AddStatus::NoChange);
    assert_eq!(cfg.vault, original);
}

#[test]
fn test_vault_config_builder() {
    let vcfg = VaultConfig::new().path("/tmp/test_vault");
    assert_eq!(vcfg.path, Some(PathBuf::from("/tmp/test_vault")));
}

#[test]
fn test_default_config_has_vault() {
    let cfg = Config::default();
    assert!(!cfg.vault.as_os_str().is_empty());
    assert_eq!(cfg.project_name, "sm");
    assert!(cfg.scrum_columns.len() == 5);
}
