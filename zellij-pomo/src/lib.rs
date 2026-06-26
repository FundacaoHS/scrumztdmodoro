use std::collections::BTreeMap;
use zellij_tile::prelude::*;

#[derive(Default)]
struct SmPomo {
    pomo_text: String,
}

register_plugin!(SmPomo);

impl ZellijPlugin for SmPomo {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        set_timeout(1.0);
        subscribe(&[EventType::RunCommandResult, EventType::Timer]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Timer(_) => {
                run_command(&["sm", "pomo", "status"], BTreeMap::new());
            }
            Event::RunCommandResult(exit_code, stdout, _stderr, _context) => {
                if exit_code == Some(0) {
                    let cleaned = String::from_utf8_lossy(&stdout);
                    let cleaned = cleaned.trim();
                    if cleaned.is_empty() || cleaned.contains("No active") {
                        self.pomo_text.clear();
                    } else {
                        self.pomo_text = cleaned.to_string();
                    }
                } else {
                    self.pomo_text.clear();
                }
                set_timeout(5.0);
            }
            _ => {}
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        if !self.pomo_text.is_empty() {
            println!(" {} ", self.pomo_text);
        }
    }
}
