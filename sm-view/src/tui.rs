use std::io::stdout;
use std::fs;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use chrono::Local;
use fundacao::Vault;
use sm_core::{has_marker, BulletKind, TaskList};

enum InputMode {
    Browsing,
    WhichKey,
    WhichKeyBullet,
    Adding { input: String, cursor: usize },
    Backlog,
    Notes { content: String },
}

pub fn run(tasks: &mut TaskList, vault: &Vault, project_name: &str) -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(stdout()))?;

    let mut list_state = ListState::default();
    let daily_count = tasks.list_daily().len();
    list_state.select(if daily_count == 0 {
        None
    } else {
        Some(0)
    });

    let mut mode = InputMode::Browsing;

    loop {
        terminal.draw(|f| draw(f, tasks, &mut list_state, &mode, project_name))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match &mut mode {
                InputMode::Browsing => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let len = tasks.list_daily().len();
                        let i = list_state.selected().unwrap_or(0);
                        list_state.select(Some(i.saturating_sub(1).min(len.saturating_sub(1))));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let len = tasks.list_daily().len();
                        let i = list_state.selected().unwrap_or(0);
                        if len > 0 && i + 1 < len {
                            list_state.select(Some(i + 1));
                        }
                    }
                    KeyCode::Char(' ') => {
                        mode = InputMode::WhichKey;
                    }
                    _ => {}
                },
                InputMode::WhichKey => match key.code {
                    KeyCode::Esc => mode = InputMode::Browsing,
                    KeyCode::Char('a') => {
                        mode = InputMode::Adding {
                            input: String::new(),
                            cursor: 0,
                        };
                    }
                    KeyCode::Char('d') => {
                        if let Some(i) = list_state.selected() {
                            let daily = tasks.list_daily();
                            if let Some(task) = daily.get(i) {
                                tasks.remove_task(task.id);
                                tasks.save_to_vault(vault).ok();
                                let len = tasks.list_daily().len();
                                list_state.select(if len == 0 { None } else { Some(i.min(len.saturating_sub(1))) });
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('t') | KeyCode::Enter => {
                        if let Some(i) = list_state.selected() {
                            let daily = tasks.list_daily();
                            if let Some(task) = daily.get(i) {
                                tasks.toggle_task(task.id);
                                tasks.save_to_vault(vault).ok();
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('b') => {
                        mode = InputMode::WhichKeyBullet;
                    }
                    KeyCode::Char('c') => {
                        if let Some(i) = list_state.selected() {
                            let daily = tasks.list_daily();
                            if let Some(task) = daily.get(i) {
                                tasks.mark_cancelled(task.id);
                                tasks.save_to_vault(vault).ok();
                                let len = tasks.list_daily().len();
                                list_state.select(if len == 0 { None } else { Some(i.min(len.saturating_sub(1))) });
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('B') => {
                        list_state.select(Some(0));
                        mode = InputMode::Backlog;
                    }
                    KeyCode::Char('N') => {
                        let notes_path = vault.notes_file();
                        let content = if notes_path.exists() {
                            fs::read_to_string(&notes_path).unwrap_or_default()
                        } else {
                            "No notes for today yet.".to_string()
                        };
                        mode = InputMode::Notes { content };
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                },
                InputMode::WhichKeyBullet => match key.code {
                    KeyCode::Esc => mode = InputMode::WhichKey,
                    KeyCode::Char('m') => {
                        if let Some(i) = list_state.selected() {
                            let daily = tasks.list_daily();
                            if let Some(task) = daily.get(i) {
                                tasks.set_bullet(task.id, BulletKind::Migrated);
                                tasks.save_to_vault(vault).ok();
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('s') => {
                        if let Some(i) = list_state.selected() {
                            let daily = tasks.list_daily();
                            if let Some(task) = daily.get(i) {
                                tasks.set_bullet(task.id, BulletKind::Scheduled);
                                tasks.save_to_vault(vault).ok();
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('e') => {
                        if let Some(i) = list_state.selected() {
                            let daily = tasks.list_daily();
                            if let Some(task) = daily.get(i) {
                                tasks.set_bullet(task.id, BulletKind::Event);
                                tasks.save_to_vault(vault).ok();
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('n') => {
                        if let Some(i) = list_state.selected() {
                            let daily = tasks.list_daily();
                            if let Some(task) = daily.get(i) {
                                tasks.set_bullet(task.id, BulletKind::Note);
                                tasks.save_to_vault(vault).ok();
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('p') => {
                        if let Some(i) = list_state.selected() {
                            let daily = tasks.list_daily();
                            if let Some(task) = daily.get(i) {
                                tasks.set_bullet(task.id, BulletKind::Priority);
                                tasks.save_to_vault(vault).ok();
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    _ => {}
                },
                InputMode::Adding { input, cursor } => match key.code {
                    KeyCode::Esc => {
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Enter => {
                        if !input.is_empty() {
                            let (desc, tags) = parse_input(input);
                            tasks.add_backlog(desc, tags);
                            tasks.save_to_vault(vault).ok();
                            let daily_len = tasks.list_daily().len();
                            list_state.select(if daily_len == 0 { None } else { Some(daily_len.saturating_sub(1)) });
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Backspace => {
                        if *cursor > 0 {
                            let idx = *cursor - 1;
                            input.remove(idx);
                            *cursor -= 1;
                        }
                    }
                    KeyCode::Delete => {
                        if *cursor < input.len() {
                            input.remove(*cursor);
                        }
                    }
                    KeyCode::Left => {
                        if *cursor > 0 {
                            *cursor -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if *cursor < input.len() {
                            *cursor += 1;
                        }
                    }
                    KeyCode::Home => {
                        *cursor = 0;
                    }
                    KeyCode::End => {
                        *cursor = input.len();
                    }
                    KeyCode::Char(c) => {
                        input.insert(*cursor, c);
                        *cursor += 1;
                    }
                    _ => {}
                },
                InputMode::Backlog => match key.code {
                    KeyCode::Esc => {
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let len = tasks.list_backlog().len();
                        let i = list_state.selected().unwrap_or(0);
                        list_state.select(Some(i.saturating_sub(1).min(len.saturating_sub(1))));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let len = tasks.list_backlog().len();
                        let i = list_state.selected().unwrap_or(0);
                        if len > 0 && i + 1 < len {
                            list_state.select(Some(i + 1));
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(i) = list_state.selected() {
                            let backlog = tasks.list_backlog();
                            if let Some(task) = backlog.get(i) {
                                tasks.pull_task(task.id);
                                tasks.save_to_vault(vault).ok();
                                let len = tasks.list_backlog().len();
                                list_state.select(if len == 0 { None } else { Some(i.min(len.saturating_sub(1))) });
                                if tasks.list_backlog().is_empty() {
                                    mode = InputMode::Browsing;
                                }
                            }
                        }
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                },
                InputMode::Notes { .. } => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('N') => {
                        mode = InputMode::Browsing;
                    }
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn parse_input(text: &str) -> (String, Vec<String>) {
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_x = (r.width * percent_x) / 100;
    let popup_y = (r.height * percent_y) / 100;
    let x = r.x + (r.width - popup_x) / 2;
    let y = r.y + (r.height - popup_y) / 2;
    Rect::new(x, y, popup_x, popup_y)
}

fn draw(f: &mut Frame, tasks: &TaskList, list_state: &mut ListState, mode: &InputMode, project_name: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(f.area());

    let is_backlog = matches!(mode, InputMode::Backlog);
    let display_tasks: Vec<&sm_core::Task> = if is_backlog {
        tasks.list_backlog()
    } else {
        tasks.list_daily()
    };

    let items: Vec<ListItem> = display_tasks.iter().map(|t| {
        let style = if t.bullet == BulletKind::Done || has_marker(&t.description, "[cancelled]") {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default().fg(Color::White)
        };
        let tags = if t.tags.is_empty() {
            String::new()
        } else {
            format!(" #{}", t.tags.join(" #"))
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", t.bullet.symbol()), style),
            Span::styled(format!("{}", t.id), Style::default().fg(Color::Cyan)),
            Span::styled(" - ", style),
            Span::styled(format!("{}{}", t.description, tags), style),
        ]))
    }).collect();

    let date = Local::now().format("%Y-%m-%d").to_string();
    let daily_count = tasks.list_daily().len();
    let backlog_count = tasks.list_backlog().len();
    let view_label = if is_backlog { " BACKLOG " } else { " DAILY " };
    let title = format!(" {} | {} | {} tasks | {} backlog {} ", project_name, date, daily_count, backlog_count, view_label);
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], list_state);

    match mode {
        InputMode::Browsing => {
            let help = Paragraph::new(Line::from(vec![
                Span::styled("↑/k ↓/j ", Style::default().fg(Color::Gray)),
                Span::styled("nav  ", Style::default().fg(Color::DarkGray)),
                Span::styled("<Space> ", Style::default().fg(Color::Green)),
                Span::styled("commands", Style::default().fg(Color::DarkGray)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Help "));
            f.render_widget(help, chunks[1]);
        }
        InputMode::WhichKey => {
            let help = Paragraph::new(Line::from(vec![
                Span::styled("which-key: ", Style::default().fg(Color::Yellow)),
                Span::styled("a", Style::default().fg(Color::Green)),
                Span::styled("dd  ", Style::default().fg(Color::DarkGray)),
                Span::styled("t", Style::default().fg(Color::Green)),
                Span::styled("oggle  ", Style::default().fg(Color::DarkGray)),
                Span::styled("c", Style::default().fg(Color::Red)),
                Span::styled("ancel  ", Style::default().fg(Color::DarkGray)),
                Span::styled("b", Style::default().fg(Color::Cyan)),
                Span::styled("ullet  ", Style::default().fg(Color::DarkGray)),
                Span::styled("B", Style::default().fg(Color::Yellow)),
                Span::styled("acklog  ", Style::default().fg(Color::DarkGray)),
                Span::styled("N", Style::default().fg(Color::Green)),
                Span::styled("otes  ", Style::default().fg(Color::DarkGray)),
                Span::styled("d", Style::default().fg(Color::Red)),
                Span::styled("el  ", Style::default().fg(Color::DarkGray)),
                Span::styled("q", Style::default().fg(Color::Red)),
                Span::styled("uit  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Gray)),
                Span::styled("cancel", Style::default().fg(Color::DarkGray)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Command "));
            f.render_widget(help, chunks[1]);

            let area = centered_rect(40, 50, f.area());
            f.render_widget(Clear, area);

            let items = vec![
                ListItem::new(Line::from(vec![
                    Span::styled("  a  ", Style::default().fg(Color::Green)),
                    Span::styled("Add task (backlog)", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  d  ", Style::default().fg(Color::Red)),
                    Span::styled("Delete task", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  t  ", Style::default().fg(Color::Yellow)),
                    Span::styled("Toggle (", Style::default().fg(Color::White)),
                    Span::styled("\u{2022}", Style::default().fg(Color::Yellow)),
                    Span::styled("/", Style::default().fg(Color::White)),
                    Span::styled("x", Style::default().fg(Color::Yellow)),
                    Span::styled(")", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  c  ", Style::default().fg(Color::Red)),
                    Span::styled("Cancel (", Style::default().fg(Color::White)),
                    Span::styled("[cancelled]", Style::default().fg(Color::Red)),
                    Span::styled(")", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  b  ", Style::default().fg(Color::Cyan)),
                    Span::styled("Bullet journal...", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  B  ", Style::default().fg(Color::Yellow)),
                    Span::styled("Backlog view (", Style::default().fg(Color::White)),
                    Span::styled("Enter", Style::default().fg(Color::Green)),
                    Span::styled(" pulls to daily)", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  N  ", Style::default().fg(Color::Green)),
                    Span::styled("Notes of the day", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  q  ", Style::default().fg(Color::Red)),
                    Span::styled("Quit", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  Esc", Style::default().fg(Color::Gray)),
                    Span::styled("  Cancel", Style::default().fg(Color::DarkGray)),
                ])),
            ];
            let popup = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Which Key "))
                .highlight_style(Style::default());
            f.render_widget(popup, area);
        }
        InputMode::WhichKeyBullet => {
            let help = Paragraph::new(Line::from(vec![
                Span::styled("bullet: ", Style::default().fg(Color::Yellow)),
                Span::styled("m", Style::default().fg(Color::Green)),
                Span::styled("igrate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("s", Style::default().fg(Color::Green)),
                Span::styled("chedule  ", Style::default().fg(Color::DarkGray)),
                Span::styled("e", Style::default().fg(Color::Green)),
                Span::styled("vent  ", Style::default().fg(Color::DarkGray)),
                Span::styled("n", Style::default().fg(Color::Green)),
                Span::styled("ote  ", Style::default().fg(Color::DarkGray)),
                Span::styled("p", Style::default().fg(Color::Green)),
                Span::styled("riority  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Gray)),
                Span::styled("back", Style::default().fg(Color::DarkGray)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Command "));
            f.render_widget(help, chunks[1]);

            let area = centered_rect(36, 38, f.area());
            f.render_widget(Clear, area);

            let items = vec![
                ListItem::new(Line::from(vec![
                    Span::styled("  m  ", Style::default().fg(Color::Green)),
                    Span::styled("Migrate ", Style::default().fg(Color::White)),
                    Span::styled(">", Style::default().fg(Color::Yellow)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  s  ", Style::default().fg(Color::Green)),
                    Span::styled("Schedule ", Style::default().fg(Color::White)),
                    Span::styled("<", Style::default().fg(Color::Yellow)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  e  ", Style::default().fg(Color::Green)),
                    Span::styled("Event ", Style::default().fg(Color::White)),
                    Span::styled("o", Style::default().fg(Color::Yellow)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  n  ", Style::default().fg(Color::Green)),
                    Span::styled("Note ", Style::default().fg(Color::White)),
                    Span::styled("-", Style::default().fg(Color::Yellow)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  p  ", Style::default().fg(Color::Green)),
                    Span::styled("Priority ", Style::default().fg(Color::White)),
                    Span::styled("*", Style::default().fg(Color::Red)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  Esc", Style::default().fg(Color::Gray)),
                    Span::styled("  Back", Style::default().fg(Color::DarkGray)),
                ])),
            ];
            let popup = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Bullet Journal "))
                .highlight_style(Style::default());
            f.render_widget(popup, area);
        }
        InputMode::Adding { input, cursor } => {
            let display = if *cursor < input.len() {
                let mut s = input.clone();
                s.insert(*cursor, '\u{2588}');
                s
            } else {
                format!("{}\u{2588}", input)
            };
            let input_widget = Paragraph::new(Line::from(vec![
                Span::styled("Task: ", Style::default().fg(Color::Green)),
                Span::styled(display, Style::default().fg(Color::White)),
                Span::styled("  #tag1 #tag2 (goes to backlog)", Style::default().fg(Color::DarkGray)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Add Task (Esc cancel) "));
            f.render_widget(input_widget, chunks[1]);
        }
        InputMode::Backlog => {
            let help = Paragraph::new(Line::from(vec![
                Span::styled("backlog: ", Style::default().fg(Color::Yellow)),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::styled(" pull to daily  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Gray)),
                Span::styled(" back  ", Style::default().fg(Color::DarkGray)),
                Span::styled("q", Style::default().fg(Color::Red)),
                Span::styled(" uit", Style::default().fg(Color::DarkGray)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Backlog "));
            f.render_widget(help, chunks[1]);
        }
        InputMode::Notes { content } => {
            let notes_widget = Paragraph::new(content.as_str())
                .block(Block::default().borders(Borders::ALL).title(" Notes (Esc/N to close) "));
            f.render_widget(notes_widget, chunks[1]);
        }
    }
}
