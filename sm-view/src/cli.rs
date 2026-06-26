use clap::{Parser, Subcommand};
use fundacao::Vault;
use sm_core::{has_marker, BulletKind, Pomodoro, TaskList, MARKER_BACKLOG};

#[derive(Parser)]
#[command(name = "sm", about = "Smart Task Manager")]
pub struct Cli {
    #[arg(long, help = "Launch GUI (Tauri)")]
    pub tauri: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Add a new task (goes to backlog by default)")]
    Add {
        #[arg(long)]
        task: String,

        #[arg(long, help = "Comma-separated tags")]
        tags: Option<String>,

        #[arg(long, help = "Add directly to today (with [todo]) instead of backlog")]
        today: bool,
    },

    #[command(about = "Remove a task by ID")]
    Remove {
        #[arg(long)]
        id: u64,
    },

    #[command(about = "Toggle task done/undone")]
    Toggle {
        #[arg(long)]
        id: u64,
    },

    #[command(about = "List daily tasks (without [backlog])")]
    List,

    #[command(about = "Set task bullet type (ex: done, migrated, scheduled, event, note, priority)")]
    Bullet {
        #[arg(long)]
        id: u64,

        #[arg(long, help = "Bullet type: task, done, migrated, scheduled, event, note, priority")]
        r#type: String,
    },

    #[command(about = "List backlog tasks")]
    Backlog,

    #[command(about = "Pull task(s) from backlog to daily")]
    Pull {
        #[arg(long, help = "Task ID to pull")]
        id: Option<u64>,

        #[arg(long, help = "Pull all backlog tasks")]
        all: bool,
    },

    #[command(about = "Mark a task as cancelled")]
    Cancel {
        #[arg(long)]
        id: u64,
    },

    #[command(about = "Pomodoro timer commands", subcommand)]
    Pomo(PomoCommand),
}

#[derive(Subcommand)]
pub enum PomoCommand {
    #[command(about = "Start a focus session")]
    Start,
    #[command(about = "Show current session status")]
    Status,
    #[command(about = "Stop current session")]
    Stop,
    #[command(about = "Skip to next phase")]
    Skip,
    #[command(about = "Show or set timer config")]
    Config {
        #[arg(long, help = "Focus duration in minutes")]
        focus: Option<u64>,
        #[arg(long, help = "Short break duration in minutes")]
        short: Option<u64>,
        #[arg(long, help = "Long break duration in minutes")]
        long: Option<u64>,
    },
}

fn print_task(task: &sm_core::Task) {
    let tags = if task.tags.is_empty() {
        String::new()
    } else {
        format!(" #{}", task.tags.join(" #"))
    };
    println!("{} {} - {}{}", task.bullet.symbol(), task.id, task.description, tags);
}

pub fn handle_command(command: Command, tasks: &mut TaskList) {
    match command {
        Command::Add { task, tags, today } => {
            let tag_list = tags
                .as_deref()
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let t = if today {
                tasks.add_today(task, tag_list)
            } else {
                tasks.add_backlog(task, tag_list)
            };
            let label = if has_marker(&t.description, MARKER_BACKLOG) { "backlog" } else { "today" };
            println!("✓ Task #{} added to {}: {}", t.id, label, t.description);
        }
        Command::Remove { id } => match tasks.remove_task(id) {
            Some(t) => println!("✓ Task #{} removed: {}", t.id, t.description),
            None => eprintln!("✗ Task #{} not found", id),
        },
        Command::Toggle { id } => match tasks.toggle_task(id) {
            Some(t) => {
                let status = if t.bullet == BulletKind::Done { "done" } else { "undone" };
                println!("✓ Task #{} set to {}", t.id, status);
            }
            None => eprintln!("✗ Task #{} not found", id),
        },
        Command::List => {
            let daily = tasks.list_daily();
            if daily.is_empty() {
                println!("No daily tasks. Use 'sm backlog' to see backlog.");
                return;
            }
            for task in &daily {
                print_task(task);
            }
        }
        Command::Bullet { id, r#type } => {
            let bullet = match parse_bullet_type(&r#type) {
                Some(b) => b,
                None => {
                    eprintln!("✗ Invalid bullet type '{}'. Options: task, done, migrated, scheduled, event, note, priority", r#type);
                    return;
                }
            };
            match tasks.set_bullet(id, bullet) {
                Some(t) => println!("✓ Task #{} bullet set to {} ({})", t.id, t.bullet.symbol(), r#type),
                None => eprintln!("✗ Task #{} not found", id),
            }
        }
        Command::Backlog => {
            let backlog = tasks.list_backlog();
            if backlog.is_empty() {
                println!("No backlog tasks. Use 'sm add \"task\"' to add one.");
                return;
            }
            for task in &backlog {
                print_task(task);
            }
        }
        Command::Pull { id, all } => {
            if all {
                let count = tasks.list_backlog().len();
                tasks.pull_all();
                println!("✓ Pulled {} task(s) from backlog to daily.", count);
            } else if let Some(id) = id {
                match tasks.pull_task(id) {
                    Some(t) => println!("✓ Task #{} pulled to daily: {}", t.id, t.description),
                    None => eprintln!("✗ Task #{} not found or not in backlog.", id),
                }
            } else {
                eprintln!("Use --id <id> or --all to pull tasks.");
            }
        }
        Command::Cancel { id } => match tasks.mark_cancelled(id) {
            Some(t) => println!("✓ Task #{} cancelled: {}", t.id, t.description),
            None => eprintln!("✗ Task #{} not found.", id),
        },
        Command::Pomo(_) => unreachable!(),
    }
}

fn parse_bullet_type(s: &str) -> Option<BulletKind> {
    match s {
        "task" | "•" => Some(BulletKind::Task),
        "done" | "x" => Some(BulletKind::Done),
        "migrated" | ">" => Some(BulletKind::Migrated),
        "scheduled" | "<" => Some(BulletKind::Scheduled),
        "event" | "o" => Some(BulletKind::Event),
        "note" | "-" => Some(BulletKind::Note),
        "priority" | "*" => Some(BulletKind::Priority),
        _ => None,
    }
}

pub fn handle_pomo(cmd: PomoCommand, vault: &Vault) -> Result<(), Box<dyn std::error::Error>> {
    let mut pomo = Pomodoro::load(vault)?;

    match cmd {
        PomoCommand::Start => {
            if pomo.is_active() {
                println!("Timer is already running. Stop it first or use 'skip'.");
                return Ok(());
            }
            pomo.start_focus();
            pomo.save(vault)?;
            println!("▶ Focus session started ({} min)", pomo.config.focus_duration);
        }
        PomoCommand::Status => {
            if !pomo.is_active() {
                println!("⏸ No active session. Use 'sm pomo start' to begin.");
                return Ok(());
            }
            let state = match pomo.state {
                sm_core::SessionState::Focusing { .. } => "Focusing",
                sm_core::SessionState::ShortBreak { .. } => "Short Break",
                sm_core::SessionState::LongBreak { .. } => "Long Break",
                _ => "Idle",
            };
            let remaining = pomo.remaining_seconds();
            let mins = remaining / 60;
            let secs = remaining % 60;
            println!("{} — {:02}:{:02} remaining", state, mins, secs);
        }
        PomoCommand::Stop => {
            if !pomo.is_active() {
                println!("No active session.");
                return Ok(());
            }
            pomo.stop();
            pomo.save(vault)?;
            println!("⏹ Session stopped.");
        }
        PomoCommand::Skip => {
            pomo.skip();
            pomo.save(vault)?;
            let state = match pomo.state {
                sm_core::SessionState::Focusing { .. } => "Focusing",
                sm_core::SessionState::ShortBreak { .. } => "Short Break",
                sm_core::SessionState::LongBreak { .. } => "Long Break",
                _ => "Idle",
            };
            println!("⏭ Skipped to {}", state);
        }
        PomoCommand::Config { focus, short, long } => {
            let changed = focus.is_some() || short.is_some() || long.is_some();
            if let Some(v) = focus {
                pomo.config.focus_duration = v;
            }
            if let Some(v) = short {
                pomo.config.short_break = v;
            }
            if let Some(v) = long {
                pomo.config.long_break = v;
            }
            if changed {
                pomo.save(vault)?;
            }
            println!("Pomodoro config:");
            println!("  Focus:      {} min", pomo.config.focus_duration);
            println!("  Short break: {} min", pomo.config.short_break);
            println!("  Long break:  {} min", pomo.config.long_break);
            println!("  Cycles:      {}", pomo.config.cycles_before_long);
        }
    }

    Ok(())
}
