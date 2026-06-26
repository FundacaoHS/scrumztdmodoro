use std::fs;

use fundacao::Vault;
use sm_core::{has_marker, BulletKind, MARKER_BACKLOG, MARKER_CANCELLED, MARKER_TODO, TaskList};

#[test]
fn test_backlog_add_list_pull() {
    let dir = std::env::temp_dir().join("sm_int_backlog");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let _vault = Vault::new(&dir);
    let mut tasks = TaskList::new();

    // add_backlog + add_today
    let t1_id = tasks.add_backlog("Comprar pao".into(), vec!["mercado".into()]).id;
    let t2_id = tasks.add_today("Reuniao".into(), vec!["work".into()]).id;

    assert!(has_marker(&tasks.list_tasks()[0].description, MARKER_BACKLOG));
    assert!(has_marker(&tasks.list_tasks()[1].description, MARKER_TODO));
    assert_eq!(tasks.list_tasks().len(), 2);

    // list_daily — only the [todo] one
    assert_eq!(tasks.list_daily().len(), 1);
    assert_eq!(tasks.list_daily()[0].id, t2_id);

    // list_backlog — only the [backlog] one
    assert_eq!(tasks.list_backlog().len(), 1);
    assert_eq!(tasks.list_backlog()[0].id, t1_id);

    // pull_task
    tasks.pull_task(t1_id);
    assert_eq!(tasks.list_backlog().len(), 0);
    assert_eq!(tasks.list_daily().len(), 2);
    assert!(has_marker(&tasks.list_daily()[0].description, MARKER_TODO));

    // pull_all
    tasks.add_backlog("Terceira".into(), vec![]);
    tasks.add_backlog("Quarta".into(), vec![]);
    assert_eq!(tasks.list_backlog().len(), 2);
    tasks.pull_all();
    assert_eq!(tasks.list_backlog().len(), 0);
    assert_eq!(tasks.list_daily().len(), 4);

    // mark_cancelled
    let t4_id = tasks.list_tasks()[3].id;
    tasks.mark_cancelled(t4_id);
    assert!(has_marker(&tasks.list_tasks()[3].description, MARKER_CANCELLED));

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_migrate_untouched() {
    let dir = std::env::temp_dir().join("sm_int_migrate");
    let _ = fs::remove_dir_all(&dir);

    let yesterday = "2026-06-24";
    let yesterday_path = dir.join(format!("{}.md", yesterday));
    fs::create_dir_all(&dir).unwrap();

    let yesterday_md = format!(
        "\
# {date}

- • Comprar pao  #mercado
- x Estudar Rust  #estudo
- > Ja migrada
- • Task com [backlog] #backlog
- - Nota qualquer
- * Prioridade urgente
",
        date = yesterday
    );
    fs::write(&yesterday_path, yesterday_md).unwrap();

    let vault = Vault::new(&dir);
    let mut today_tasks = TaskList::new();

    today_tasks.migrate_untouched(&yesterday_path, &vault).unwrap();

    // Hoje: tasks inacabadas (• sem backlog, *) copiadas com [todo]
    assert_eq!(today_tasks.list_tasks().len(), 2);
    assert_eq!(today_tasks.list_backlog().len(), 0);
    assert_eq!(today_tasks.list_daily().len(), 2);

    let t0 = &today_tasks.list_tasks()[0];
    assert!(has_marker(&t0.description, MARKER_TODO));
    assert!(!has_marker(&t0.description, MARKER_BACKLOG));

    // Ontem: tasks inacabadas migradas viraram >
    let reloaded = TaskList::from_md(&yesterday_path).unwrap();
    assert_eq!(reloaded.list_tasks().len(), 6);

    assert_eq!(reloaded.list_tasks()[0].bullet, BulletKind::Migrated); // Comprar pao
    assert_eq!(reloaded.list_tasks()[1].bullet, BulletKind::Done);     // Estudar Rust
    assert_eq!(reloaded.list_tasks()[2].bullet, BulletKind::Migrated); // Ja migrada
    assert_eq!(reloaded.list_tasks()[3].bullet, BulletKind::Task);     // Task com [backlog]
    assert_eq!(reloaded.list_tasks()[4].bullet, BulletKind::Migrated); // Nota qualquer
    assert_eq!(reloaded.list_tasks()[5].bullet, BulletKind::Migrated); // Prioridade

    // Notas salvas em notes/
    let notes_path = vault.notes_file_for(yesterday);
    assert!(notes_path.exists());
    let notes_content = fs::read_to_string(&notes_path).unwrap();
    assert!(notes_content.contains("# Notas"));
    assert!(notes_content.contains("- Nota qualquer"));

    // Task com [backlog] preservou o marcador e bullet
    assert!(has_marker(&reloaded.list_tasks()[3].description, MARKER_BACKLOG));

    fs::remove_dir_all(&dir).unwrap();
}
