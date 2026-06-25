use clap::{Parser, Subcommand};
use sm_core::TaskList;

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
    #[command(about = "Add a new task")]
    Add {
        #[arg(long)]
        task: String,

        #[arg(long, help = "Comma-separated tags")]
        tags: Option<String>,
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

    #[command(about = "List all tasks")]
    List,
}

pub fn handle_command(command: Command, tasks: &mut TaskList) {
    match command {
        Command::Add { task, tags } => {
            let tag_list = tags
                .as_deref()
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let t = tasks.add_task(task, tag_list);
            println!("✓ Task #{} added: {}", t.id, t.description);
        }
        Command::Remove { id } => match tasks.remove_task(id) {
            Some(t) => println!("✓ Task #{} removed: {}", t.id, t.description),
            None => eprintln!("✗ Task #{} not found", id),
        },
        Command::Toggle { id } => match tasks.toggle_task(id) {
            Some(t) => println!("✓ Task #{} {} set to {}", t.id, t.description, if t.done { "done" } else { "undone" }),
            None => eprintln!("✗ Task #{} not found", id),
        },
        Command::List => {
            if tasks.list_tasks().is_empty() {
                println!("No tasks yet.");
                return;
            }
            for task in tasks.list_tasks() {
                let status = if task.done { "[x]" } else { "[ ]" };
                let tags = if task.tags.is_empty() {
                    String::new()
                } else {
                    format!(" #{}", task.tags.join(" #"))
                };
                println!("{} {} - {}{}", status, task.id, task.description, tags);
            }
        }
    }
}
