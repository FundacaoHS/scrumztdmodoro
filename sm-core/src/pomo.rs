use chrono::{DateTime, Utc};
use fundacao::Vault;
use serde::{Deserialize, Serialize};
use std::fs;

/// Configuração do timer Pomodoro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroConfig {
    pub focus_duration: u64,
    pub short_break: u64,
    pub long_break: u64,
    pub cycles_before_long: u32,
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            focus_duration: 25,
            short_break: 5,
            long_break: 15,
            cycles_before_long: 4,
        }
    }
}

/// Estado atual da sessão Pomodoro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    Idle,
    Focusing { started_at: DateTime<Utc>, cycle: u32 },
    ShortBreak { started_at: DateTime<Utc>, cycle: u32 },
    LongBreak { started_at: DateTime<Utc>, cycle: u32 },
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Timer Pomodoro com controle de estado e persistência
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pomodoro {
    pub config: PomodoroConfig,
    pub state: SessionState,
}

impl Default for Pomodoro {
    fn default() -> Self {
        Self {
            config: PomodoroConfig::default(),
            state: SessionState::Idle,
        }
    }
}

impl Pomodoro {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retorna os segundos restantes da sessão atual (0 se Idle)
    pub fn remaining_seconds(&self) -> i64 {
        match &self.state {
            SessionState::Idle => 0,
            SessionState::Focusing { started_at, .. } => {
                let elapsed = (Utc::now() - *started_at).num_seconds();
                let total = (self.config.focus_duration as i64) * 60;
                (total - elapsed).max(0)
            }
            SessionState::ShortBreak { started_at, .. } => {
                let elapsed = (Utc::now() - *started_at).num_seconds();
                let total = (self.config.short_break as i64) * 60;
                (total - elapsed).max(0)
            }
            SessionState::LongBreak { started_at, .. } => {
                let elapsed = (Utc::now() - *started_at).num_seconds();
                let total = (self.config.long_break as i64) * 60;
                (total - elapsed).max(0)
            }
        }
    }

    /// Indica se o timer está rodando
    pub fn is_active(&self) -> bool {
        !matches!(self.state, SessionState::Idle)
    }

    /// Inicia um foco Pomodoro
    pub fn start_focus(&mut self) {
        let cycle = match &self.state {
            SessionState::Idle => 1,
            SessionState::ShortBreak { cycle, .. }
            | SessionState::LongBreak { cycle, .. } => *cycle + 1,
            SessionState::Focusing { .. } => return,
        };
        self.state = SessionState::Focusing {
            started_at: Utc::now(),
            cycle,
        };
    }

    /// Inicia pausa curta ou longa automaticamente baseado no ciclo
    pub fn start_break(&mut self) {
        let cycle = match &self.state {
            SessionState::Focusing { cycle, .. } => *cycle,
            _ => return,
        };

        if cycle % self.config.cycles_before_long == 0 {
            self.state = SessionState::LongBreak {
                started_at: Utc::now(),
                cycle,
            };
        } else {
            self.state = SessionState::ShortBreak {
                started_at: Utc::now(),
                cycle,
            };
        }
    }

    /// Volta ao estado Idle
    pub fn stop(&mut self) {
        self.state = SessionState::Idle;
    }

    /// Pula para o próximo estado (foco → pausa → foco)
    pub fn skip(&mut self) {
        match &self.state {
            SessionState::Focusing { .. } => self.start_break(),
            SessionState::ShortBreak { .. } | SessionState::LongBreak { .. } => self.start_focus(),
            SessionState::Idle => self.start_focus(),
        }
    }

    /// Salva estado atual no vault como JSON
    pub fn save(&self, vault: &Vault) -> Result<(), PomoError> {
        vault.ensure()?;
        let path = vault.task_file("pomo_state");
        let data = serde_json::to_string(self)?;
        fs::write(&path, data)?;
        Ok(())
    }

    /// Carrega estado do vault
    pub fn load(vault: &Vault) -> Result<Self, PomoError> {
        let path = vault.task_file("pomo_state");
        if !path.exists() {
            return Ok(Self::new());
        }
        let data = fs::read_to_string(&path)?;
        let mut pomo: Self = serde_json::from_str(&data)?;
        // Se o timer já expirou, avança automaticamente
        if pomo.remaining_seconds() == 0 && pomo.is_active() {
            pomo.skip();
        }
        Ok(pomo)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PomoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
