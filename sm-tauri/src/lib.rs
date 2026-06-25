use fundacao::{Config, Vault};
use sm_core::{BulletKind, Pomodoro, TaskList};
use std::sync::Mutex;

struct AppState {
    tasks: Mutex<TaskList>,
    vault: Vault,
    pomo: Mutex<Pomodoro>,
}

#[tauri::command]
fn list_tasks(state: tauri::State<AppState>) -> Result<Vec<sm_core::Task>, String> {
    let tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    Ok(tasks.list_tasks().to_vec())
}

#[tauri::command]
fn add_task(state: tauri::State<AppState>, description: String, tags: Vec<String>) -> Result<sm_core::Task, String> {
    let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    let task = tasks.add_task(description, tags).clone();
    tasks.save_to_vault(&state.vault).map_err(|e| e.to_string())?;
    Ok(task)
}

#[tauri::command]
fn remove_task(state: tauri::State<AppState>, id: u64) -> Result<(), String> {
    let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    tasks.remove_task(id).ok_or("Task not found")?;
    tasks.save_to_vault(&state.vault).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn toggle_task(state: tauri::State<AppState>, id: u64) -> Result<sm_core::Task, String> {
    let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    let task = tasks.toggle_task(id).ok_or("Task not found")?.clone();
    tasks.save_to_vault(&state.vault).map_err(|e| e.to_string())?;
    Ok(task)
}

#[tauri::command]
fn set_bullet(state: tauri::State<AppState>, id: u64, bullet: String) -> Result<sm_core::Task, String> {
    let bullet_kind = match bullet.as_str() {
        "task" => BulletKind::Task,
        "done" => BulletKind::Done,
        "migrated" => BulletKind::Migrated,
        "scheduled" => BulletKind::Scheduled,
        "event" => BulletKind::Event,
        "note" => BulletKind::Note,
        "priority" => BulletKind::Priority,
        _ => return Err("Invalid bullet type".into()),
    };
    let mut tasks = state.tasks.lock().map_err(|e| e.to_string())?;
    let task = tasks.set_bullet(id, bullet_kind).ok_or("Task not found")?.clone();
    tasks.save_to_vault(&state.vault).map_err(|e| e.to_string())?;
    Ok(task)
}

#[tauri::command]
fn pomo_status(state: tauri::State<AppState>) -> Result<String, String> {
    let pomo = state.pomo.lock().map_err(|e| e.to_string())?;
    if !pomo.is_active() {
        return Ok("idle".into());
    }
    let state_name = match &pomo.state {
        sm_core::SessionState::Focusing { .. } => "focusing",
        sm_core::SessionState::ShortBreak { .. } => "short_break",
        sm_core::SessionState::LongBreak { .. } => "long_break",
        _ => "idle",
    };
    let remaining = pomo.remaining_seconds();
    Ok(format!("{}:{}", state_name, remaining))
}

#[tauri::command]
fn pomo_start(state: tauri::State<AppState>) -> Result<(), String> {
    let mut pomo = state.pomo.lock().map_err(|e| e.to_string())?;
    pomo.start_focus();
    pomo.save(&state.vault).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn pomo_stop(state: tauri::State<AppState>) -> Result<(), String> {
    let mut pomo = state.pomo.lock().map_err(|e| e.to_string())?;
    pomo.stop();
    pomo.save(&state.vault).map_err(|e| e.to_string())?;
    Ok(())
}

fn init_vault() -> (Vault, TaskList, Pomodoro) {
    let conf_path = fundacao::default_conf_path();
    let mut cfg = Config::load(&conf_path).unwrap_or_default();
    if !cfg.vault.exists() {
        cfg.vault = fundacao::default_vault_path();
    }
    let vault = Vault::new(&cfg.vault);
    let tasks = TaskList::load_from_vault(&vault).unwrap_or_default();
    let pomo = Pomodoro::load(&vault).unwrap_or_default();
    (vault, tasks, pomo)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (vault, tasks, pomo) = init_vault();

    tauri::Builder::default()
        .manage(AppState {
            tasks: Mutex::new(tasks),
            vault,
            pomo: Mutex::new(pomo),
        })
        .invoke_handler(tauri::generate_handler![
            list_tasks,
            add_task,
            remove_task,
            toggle_task,
            set_bullet,
            pomo_status,
            pomo_start,
            pomo_stop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
