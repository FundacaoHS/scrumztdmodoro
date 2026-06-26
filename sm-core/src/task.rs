use chrono::{DateTime, Local, Utc};
use fundacao::Vault;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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

    pub fn is_unfinished(&self) -> bool {
        matches!(self, BulletKind::Task | BulletKind::Scheduled | BulletKind::Priority)
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

// --- Helpers de marcador ---

pub const MARKER_BACKLOG: &str = "[backlog]";
pub const MARKER_TODO: &str = "[todo]";
pub const MARKER_CANCELLED: &str = "[cancelled]";

pub fn has_marker(text: &str, marker: &str) -> bool {
    text.contains(marker)
}

pub fn add_marker(text: &str, marker: &str) -> String {
    if has_marker(text, marker) {
        return text.to_string();
    }
    format!("{} {}", text, marker)
}

pub fn remove_marker(text: &str, marker: &str) -> String {
    text.replace(marker, "").split_whitespace().collect::<Vec<_>>().join(" ")
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

    /// Add task que cai no backlog (description ganha [backlog])
    pub fn add_backlog(&mut self, description: String, tags: Vec<String>) -> &Task {
        let desc = add_marker(&description, MARKER_BACKLOG);
        self.add_task(desc, tags)
    }

    /// Add task pro dia de hoje (description ganha [todo])
    pub fn add_today(&mut self, description: String, tags: Vec<String>) -> &Task {
        let desc = add_marker(&description, MARKER_TODO);
        self.add_task(desc, tags)
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

    /// Retorna tasks sem [backlog] (daily view)
    pub fn list_daily(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|t| !has_marker(&t.description, MARKER_BACKLOG)).collect()
    }

    /// Retorna tasks com [backlog]
    pub fn list_backlog(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|t| has_marker(&t.description, MARKER_BACKLOG)).collect()
    }

    /// Puxa task do backlog pro daily: remove [backlog], add [todo]
    pub fn pull_task(&mut self, id: u64) -> Option<&Task> {
        let task = self.tasks.iter_mut().find(|t| t.id == id)?;
        if !has_marker(&task.description, MARKER_BACKLOG) {
            return Some(task);
        }
        task.description = remove_marker(&task.description, MARKER_BACKLOG);
        task.description = add_marker(&task.description, MARKER_TODO);
        Some(task)
    }

    /// Puxa todas tasks do backlog
    pub fn pull_all(&mut self) {
        let ids: Vec<u64> = self.tasks.iter()
            .filter(|t| has_marker(&t.description, MARKER_BACKLOG))
            .map(|t| t.id)
            .collect();
        for id in ids {
            self.pull_task(id);
        }
    }

    /// Marca task como cancelada
    pub fn mark_cancelled(&mut self, id: u64) -> Option<&Task> {
        let task = self.tasks.iter_mut().find(|t| t.id == id)?;
        task.description = remove_marker(&task.description, MARKER_TODO);
        task.description = remove_marker(&task.description, MARKER_BACKLOG);
        task.description = add_marker(&task.description, MARKER_CANCELLED);
        Some(task)
    }

    /// Le tasks de um arquivo .md no formato Bullet Journal
    pub fn from_md(path: impl AsRef<Path>) -> Result<Self, TaskError> {
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

    /// Gera o texto markdown com data de hoje
    pub fn to_md(&self) -> String {
        let date = Local::now().format("%Y-%m-%d").to_string();
        self.to_md_for_date(&date)
    }

    /// Gera o texto markdown com data especifica
    pub fn to_md_for_date(&self, date: &str) -> String {
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

    /// Salva tasks num path especifico com data especifica no cabecalho
    pub fn save_to_path(&self, path: &Path, date: &str) -> Result<(), TaskError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = self.to_md_for_date(date);
        fs::write(path, content)?;
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

    /// Migra tasks inacabadas do ultimo dia para hoje.
    /// - Bullet inacabado ({Task, Scheduled, Priority}) sem [backlog] → copia pra hoje com [todo], vira > no ultimo dia
    /// - Bullet Note → move pra notes/YYYY-MM-DD.md, vira > no ultimo dia
    /// - Bullet Done/Migrated ou com [backlog] → permanece como esta
    pub fn migrate_untouched(&mut self, last_path: &Path, vault: &Vault) -> Result<(), TaskError> {
        let date_stem = last_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let mut last_tasks = TaskList::from_md(last_path)?;
        let mut notes = Vec::new();

        for task in &last_tasks.tasks {
            let has_backlog = has_marker(&task.description, MARKER_BACKLOG);
            let has_cancelled = has_marker(&task.description, MARKER_CANCELLED);

            if task.bullet.is_unfinished() && !has_backlog && !has_cancelled {
                let desc = add_marker(&task.description, MARKER_TODO);
                let desc = remove_marker(&desc, MARKER_BACKLOG);
                self.add_task(desc, task.tags.clone());
            }

            if task.bullet == BulletKind::Note && !has_cancelled {
                notes.push(task.description.clone());
            }
        }

        // Salva notas
        if !notes.is_empty() {
            let notes_path = vault.notes_file_for(date_stem);
            if let Some(parent) = notes_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let notes_content = format!(
                "---\ncreated: {}\ntags: [note]\n---\n\n# Notas\n\n",
                date_stem
            );
            let notes_body = notes
                .iter()
                .map(|n| format!("- {}", n))
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(&notes_path, format!("{}{}\n", notes_content, notes_body))?;
        }

        // Marca tasks migradas como > no ultimo dia
        for task in &mut last_tasks.tasks {
            let has_backlog = has_marker(&task.description, MARKER_BACKLOG);
            let has_cancelled = has_marker(&task.description, MARKER_CANCELLED);
            if (task.bullet.is_unfinished() || task.bullet == BulletKind::Note)
                && !has_backlog
                && !has_cancelled
            {
                task.bullet = BulletKind::Migrated;
            }
        }

        // Salva ultimo dia atualizado
        last_tasks.save_to_path(last_path, date_stem)?;

        Ok(())
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
