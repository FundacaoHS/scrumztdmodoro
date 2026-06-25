mod cli;
mod tui;

use clap::Parser;
use sm_core::TaskList;

fn main() {
    let cli = cli::Cli::parse();

    let mut tasks = TaskList::new();

    if cli.tauri {
        #[cfg(feature = "tauri")]
        launch_tauri();
        #[cfg(not(feature = "tauri"))]
        eprintln!("Tauri GUI not available. Rebuild with --features tauri or build the Tauri app.");
        return;
    }

    match cli.command {
        Some(cmd) => cli::handle_command(cmd, &mut tasks),
        None => {
            if let Err(e) = tui::run(&mut tasks) {
                eprintln!("TUI error: {}", e);
            }
        }
    }
}

#[allow(dead_code)]
fn launch_tauri() {
    println!("Launching Tauri GUI... (not yet implemented)");
}
