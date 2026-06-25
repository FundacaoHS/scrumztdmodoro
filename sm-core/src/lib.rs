use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub description: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskList {
    pub tasks: Vec<Task>,
}

impl TaskList {
    pub fn new() -> Self {
        TaskList { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, description: String, tags: Vec<String>) -> &Task {
        let id = self.tasks.len() as u64 + 1;
        self.tasks.push(Task {
            id,
            description,
            done: false,
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
        task.done = !task.done;
        Some(task)
    }

    pub fn list_tasks(&self) -> &[Task] {
        &self.tasks
    }
}
