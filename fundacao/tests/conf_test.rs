use std::path::PathBuf;

use fundacao::{Config, ConfigKey, VaultConfig};

#[test]
fn test_add_updates_vault_path() {
    let mut cfg = Config::default();
    let original = cfg.vault.clone();

    let new_path = PathBuf::from("./custom_vault");
    cfg.add(ConfigKey::Vault(VaultConfig::new().path(&new_path)));

    assert_eq!(cfg.vault, new_path);
    assert_ne!(cfg.vault, original);
}

#[test]
fn test_add_no_path_keeps_original() {
    let mut cfg = Config::default();
    let original = cfg.vault.clone();

    cfg.add(ConfigKey::Vault(VaultConfig::new()));

    assert_eq!(cfg.vault, original);
}

#[test]
fn test_vault_config_builder() {
    let vcfg = VaultConfig::new()
        .path("/tmp/test_vault");

    assert_eq!(vcfg.path, Some(PathBuf::from("/tmp/test_vault")));
}

#[test]
fn test_default_config_has_vault() {
    let cfg = Config::default();
    assert!(!cfg.vault.as_os_str().is_empty());
    assert_eq!(cfg.project_name, "sm");
    assert!(cfg.scrum_columns.len() == 5);
}
