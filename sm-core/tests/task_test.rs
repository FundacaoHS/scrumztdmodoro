use std::fs;

use fundacao::{Config, ConfigKey, ProjectConfig, VaultConfig};
use sm_core::{BulletKind, TaskList};

#[test]
fn test_full_flow_create_vault_add_tasks() {
    let dir = std::env::temp_dir().join("sm_int_test_full");
    let _ = fs::remove_dir_all(&dir);

    // 1. Configura vault
    let mut cfg = Config::default();
    cfg.add(ConfigKey::Vault(VaultConfig::new().path(&dir)))
        .unwrap();
    assert!(dir.exists());

    // 2. Configura projeto
    cfg.add(ConfigKey::Project(ProjectConfig::new().name("meu-proj")))
        .unwrap();
    assert_eq!(cfg.project_name, "meu-proj");

    // 3. Cria tasks
    let vault = fundacao::Vault::new(&dir);
    let mut tasks = TaskList::new();
    tasks.add_task("Comprar pao".into(), vec!["mercado".into()]);
    tasks.add_task("Estudar Rust".into(), vec!["estudo".into()]);
    tasks.add_task("Revisar PR".into(), vec![]);

    assert_eq!(tasks.list_tasks().len(), 3);

    // 4. Salva no vault
    tasks.save_to_vault(&vault).unwrap();
    let saved_path = vault.today_todo();
    assert!(saved_path.exists());

    // 5. Le do vault
    let loaded = TaskList::load_from_vault(&vault).unwrap();
    assert_eq!(loaded.list_tasks().len(), 3);

    let t0 = &loaded.list_tasks()[0];
    assert_eq!(t0.description, "Comprar pao");
    assert_eq!(t0.tags, vec!["mercado"]);
    assert_eq!(t0.bullet, BulletKind::Task);

    // 6. Toggle task
    let t1_id = loaded.list_tasks()[1].id;
    let mut loaded = loaded;
    loaded.toggle_task(t1_id);
    assert_eq!(loaded.list_tasks()[1].bullet, BulletKind::Done);

    // 7. Salva denovo
    loaded.save_to_vault(&vault).unwrap();

    // 8. Le e confirma toggle persistiu
    let reloaded = TaskList::load_from_vault(&vault).unwrap();
    assert_eq!(reloaded.list_tasks()[1].bullet, BulletKind::Done);

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_parse_markdown_roundtrip() {
    let md = "\
# 2026-06-25

- • Comprar pao  #mercado
- x Estudar Rust  #estudo
- > Revisar PR  #work
- * Pagar conta
- o Reuniao  #work
- - Nota qualquer
";

    let tasks = TaskList::parse_md(md).unwrap();
    assert_eq!(tasks.list_tasks().len(), 6);

    assert_eq!(tasks.list_tasks()[0].bullet, BulletKind::Task);
    assert_eq!(tasks.list_tasks()[0].description, "Comprar pao");
    assert_eq!(tasks.list_tasks()[0].tags, vec!["mercado"]);

    assert_eq!(tasks.list_tasks()[1].bullet, BulletKind::Done);
    assert_eq!(tasks.list_tasks()[1].description, "Estudar Rust");

    assert_eq!(tasks.list_tasks()[2].bullet, BulletKind::Migrated);
    assert_eq!(tasks.list_tasks()[3].bullet, BulletKind::Priority);
    assert_eq!(tasks.list_tasks()[4].bullet, BulletKind::Event);
    assert_eq!(tasks.list_tasks()[5].bullet, BulletKind::Note);

    // Roundtrip: to_md → parse_md
    let md2 = tasks.to_md();
    let tasks2 = TaskList::parse_md(&md2).unwrap();
    assert_eq!(tasks2.list_tasks().len(), 6);
    assert_eq!(tasks2.list_tasks()[0].description, "Comprar pao");
    assert_eq!(tasks2.list_tasks()[2].bullet, BulletKind::Migrated);
}
