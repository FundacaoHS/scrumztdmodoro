mod cli;
mod tui;

use clap::Parser;
use fundacao::{Config, Vault};
use sm_core::TaskList;

fn init() -> (TaskList, Vault, Config) {
    let conf_path = fundacao::default_conf_path();
    let mut cfg = match Config::load(&conf_path) {
        Ok(c) => c,
        Err(_) => Config::default(),
    };
    if !cfg.vault.exists() {
        cfg.vault = fundacao::default_vault_path();
    }
    let vault = Vault::new(&cfg.vault);
    let today_path = vault.today_todo();

    let tasks = if !today_path.exists() {
        let mut today_tasks = TaskList::new();
        if let Some(last_path) = vault.last_day_path() {
            eprintln!("→ New day! Migrating unfinished tasks from {}...", last_path.file_stem().unwrap().to_string_lossy());
            if let Err(e) = today_tasks.migrate_untouched(&last_path, &vault) {
                eprintln!("  Migration error: {}", e);
            }
        }
        today_tasks
    } else {
        TaskList::load_from_vault(&vault).unwrap_or_default()
    };

    (tasks, vault, cfg)
}

fn main() {
    let cli = cli::Cli::parse();

    let (mut tasks, vault, cfg) = init();

    if cli.tauri {
        #[cfg(feature = "tauri")]
        launch_tauri();
        #[cfg(not(feature = "tauri"))]
        eprintln!("Tauri GUI not available. Rebuild with --features tauri or build the Tauri app.");
        return;
    }

    match cli.command {
        Some(cmd) => {
            let result = match cmd {
                cli::Command::Pomo(pomo_cmd) => cli::handle_pomo(pomo_cmd, &vault),
                other => {
                    cli::handle_command(other, &mut tasks);
                    tasks.save_to_vault(&vault).ok();
                    Ok(())
                }
            };
            if let Err(e) = result {
                eprintln!("Error: {}", e);
            }
        }
        None => {
            if let Err(e) = tui::run(&mut tasks, &vault, &cfg.project_name) {
                eprintln!("TUI error: {}", e);
            }
        }
    }
}

#[allow(dead_code)]
fn launch_tauri() {
    println!("Launching Tauri GUI... (not yet implemented)");
}
