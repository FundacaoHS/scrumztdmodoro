mod cli;
mod tui;

use clap::Parser;
use fundacao::{Config, Vault};
use sm_core::TaskList;

fn init() -> (TaskList, Vault) {
    let conf_path = fundacao::default_conf_path();
    let mut cfg = match Config::load(&conf_path) {
        Ok(c) => c,
        Err(_) => Config::default(),
    };
    if !cfg.vault.exists() {
        cfg.vault = fundacao::default_vault_path();
    }
    let vault = Vault::new(&cfg.vault);
    let tasks = TaskList::load_from_vault(&vault).unwrap_or_default();
    (tasks, vault)
}

fn main() {
    let cli = cli::Cli::parse();

    let (mut tasks, vault) = init();

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
            if let Err(e) = tui::run(&mut tasks) {
                eprintln!("TUI error: {}", e);
            }
            tasks.save_to_vault(&vault).ok();
        }
    }
}

#[allow(dead_code)]
fn launch_tauri() {
    println!("Launching Tauri GUI... (not yet implemented)");
}
