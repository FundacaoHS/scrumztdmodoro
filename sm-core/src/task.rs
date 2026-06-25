use chrono::{DateTime, Local, Utc};
use fundacao::Vault;
use serde::{Deserialize, Serialize};
use std::fs;

/// Simbolos do Bullet Journal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulletKind {
    /// • — Task pendente
    Task,
    /// x — Task concluida
    Done,
    /// > — Task migrada para outro dia
    Migrated,
    /// < — Task agendada
    Scheduled,
    /// o — Evento
    Event,
    /// - — Nota
    Note,
    /// * — Prioridade
    Priority,
}

impl BulletKind {
    pub fn symbol(&self) -> &'static str {
        match self {
            BulletKind::Task => "\u{2022}",
            BulletKind::Done => "x",
            BulletKind::Migrated => ">",
            BulletKind::Scheduled => "<",
            BulletKind::Event => "o",
            BulletKind::Note => "-",
            BulletKind::Priority => "*",
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '•' => Some(BulletKind::Task),
            'x' | 'X' => Some(BulletKind::Done),
            '>' => Some(BulletKind::Migrated),
            '<' => Some(BulletKind::Scheduled),
            'o' | 'O' => Some(BulletKind::Event),
            '-' => Some(BulletKind::Note),
            '*' => Some(BulletKind::Priority),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub bullet: BulletKind,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskList {
    pub tasks: Vec<Task>,
    pub next_id: u64,
}

impl TaskList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_task(&mut self, description: String, tags: Vec<String>) -> &Task {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push(Task {
            id,
            bullet: BulletKind::Task,
            description,
            created_at: Utc::now(),
            tags,
        });
        self.tasks.last().unwrap()
    }

    pub fn remove_task(&mut self, id: u64) -> Option<Task> {
        let pos = self.tasks.iter().position(|t| t.id == id)?;
        Some(self.tasks.remove(pos))
    }

    pub fn toggle_task(&mut self, id: u64) -> Option<&Task> {
        let task = self.tasks.iter_mut().find(|t| t.id == id)?;
        task.bullet = match task.bullet {
            BulletKind::Done => BulletKind::Task,
            _ => BulletKind::Done,
        };
        Some(task)
    }

    pub fn set_bullet(&mut self, id: u64, bullet: BulletKind) -> Option<&Task> {
        let task = self.tasks.iter_mut().find(|t| t.id == id)?;
        task.bullet = bullet;
        Some(task)
    }

    pub fn list_tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Le tasks de um arquivo .md no formato Bullet Journal
    pub fn from_md(path: impl AsRef<std::path::Path>) -> Result<Self, TaskError> {
        let content = fs::read_to_string(path.as_ref())?;
        Self::parse_md(&content)
    }

    /// Parseia texto markdown para tasks
    pub fn parse_md(text: &str) -> Result<Self, TaskError> {
        let mut list = TaskList::new();

        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with("- ") {
                continue;
            }
            let rest = &line[2..];
            if rest.is_empty() {
                continue;
            }

            let bullet_char = rest.chars().next().unwrap();
            let bullet = match BulletKind::from_char(bullet_char) {
                Some(b) => b,
                None => continue,
            };

            let desc_text = rest[bullet_char.len_utf8()..].trim();
            let (description, tags) = parse_tags(desc_text);

            list.tasks.push(Task {
                id: list.next_id,
                bullet,
                description,
                created_at: Utc::now(),
                tags,
            });
            list.next_id += 1;
        }

        Ok(list)
    }

    /// Gera o texto markdown no formato Bullet Journal
    pub fn to_md(&self) -> String {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let mut out = format!("# {}\n\n", date);

        for task in &self.tasks {
            let tags = if task.tags.is_empty() {
                String::new()
            } else {
                format!(" #{}", task.tags.join(" #"))
            };
            out.push_str(&format!("- {} {}{}\n", task.bullet.symbol(), task.description, tags));
        }

        out
    }

    /// Salva tasks no vault do dia
    pub fn save_to_vault(&self, vault: &Vault) -> Result<(), TaskError> {
        vault.ensure()?;
        let path = vault.today_todo();
        let content = self.to_md();
        fs::write(&path, content)?;
        Ok(())
    }

    /// Carrega tasks do vault do dia
    pub fn load_from_vault(vault: &Vault) -> Result<Self, TaskError> {
        let path = vault.today_todo();
        if path.exists() {
            Self::from_md(path)
        } else {
            Ok(TaskList::new())
        }
    }
}

fn parse_tags(text: &str) -> (String, Vec<String>) {
    let mut desc = String::new();
    let mut tags = Vec::new();

    for word in text.split_whitespace() {
        if word.starts_with('#') {
            tags.push(word[1..].to_string());
        } else {
            if !desc.is_empty() {
                desc.push(' ');
            }
            desc.push_str(word);
        }
    }

    (desc, tags)
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
