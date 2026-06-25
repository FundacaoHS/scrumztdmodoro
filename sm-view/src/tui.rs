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
use sm_core::{BulletKind, TaskList};
use std::io::stdout;

enum InputMode {
    Browsing,
    WhichKey,
    WhichKeyBullet,
    Adding { input: String, cursor: usize },
}

pub fn run(tasks: &mut TaskList, vault: &Vault, project_name: &str) -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(stdout()))?;

    let mut list_state = ListState::default();
    list_state.select(if tasks.list_tasks().is_empty() {
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
                        let i = list_state.selected().unwrap_or(0);
                        list_state.select(Some(i.saturating_sub(1)));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let len = tasks.list_tasks().len();
                        let i = list_state.selected().unwrap_or(0);
                        if i + 1 < len {
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
                            if let Some(task) = tasks.list_tasks().get(i) {
                                tasks.remove_task(task.id);
                                tasks.save_to_vault(vault).ok();
                                let len = tasks.list_tasks().len();
                                if i >= len {
                                    list_state.select(Some(len.saturating_sub(1)));
                                }
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('t') | KeyCode::Enter => {
                        if let Some(i) = list_state.selected() {
                            if let Some(task) = tasks.list_tasks().get(i) {
                                tasks.toggle_task(task.id);
                                tasks.save_to_vault(vault).ok();
                            }
                        }
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('b') => {
                        mode = InputMode::WhichKeyBullet;
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                },
                InputMode::WhichKeyBullet => match key.code {
                    KeyCode::Esc => mode = InputMode::WhichKey,
                    KeyCode::Char('m') => {
                        set_bullet(tasks, &list_state, BulletKind::Migrated);
                        tasks.save_to_vault(vault).ok();
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('s') => {
                        set_bullet(tasks, &list_state, BulletKind::Scheduled);
                        tasks.save_to_vault(vault).ok();
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('e') => {
                        set_bullet(tasks, &list_state, BulletKind::Event);
                        tasks.save_to_vault(vault).ok();
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('n') => {
                        set_bullet(tasks, &list_state, BulletKind::Note);
                        tasks.save_to_vault(vault).ok();
                        mode = InputMode::Browsing;
                    }
                    KeyCode::Char('p') => {
                        set_bullet(tasks, &list_state, BulletKind::Priority);
                        tasks.save_to_vault(vault).ok();
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
                            tasks.add_task(desc, tags);
                            tasks.save_to_vault(vault).ok();
                            list_state.select(Some(tasks.list_tasks().len().saturating_sub(1)));
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
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn set_bullet(tasks: &mut TaskList, list_state: &ListState, bullet: BulletKind) {
    if let Some(i) = list_state.selected() {
        if let Some(task) = tasks.tasks.get_mut(i) {
            task.bullet = bullet;
        }
    }
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

    let items: Vec<ListItem> = tasks
        .list_tasks()
        .iter()
        .map(|t| {
            let is_done = t.bullet == BulletKind::Done;
            let style = if is_done {
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
        })
        .collect();

    let date = Local::now().format("%Y-%m-%d").to_string();
    let title = format!(" {} | {} | {} ", project_name, date, tasks.list_tasks().len());
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
                Span::styled("b", Style::default().fg(Color::Cyan)),
                Span::styled("ullet  ", Style::default().fg(Color::DarkGray)),
                Span::styled("d", Style::default().fg(Color::Red)),
                Span::styled("el  ", Style::default().fg(Color::DarkGray)),
                Span::styled("q", Style::default().fg(Color::Red)),
                Span::styled("uit  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Gray)),
                Span::styled("cancel", Style::default().fg(Color::DarkGray)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Command "));
            f.render_widget(help, chunks[1]);

            let area = centered_rect(36, 38, f.area());
            f.render_widget(Clear, area);

            let items = vec![
                ListItem::new(Line::from(vec![
                    Span::styled("  a  ", Style::default().fg(Color::Green)),
                    Span::styled("Add task", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  d  ", Style::default().fg(Color::Red)),
                    Span::styled("Delete task", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  t  ", Style::default().fg(Color::Yellow)),
                    Span::styled("Toggle (", Style::default().fg(Color::White)),
                    Span::styled("•", Style::default().fg(Color::Yellow)),
                    Span::styled("/", Style::default().fg(Color::White)),
                    Span::styled("x", Style::default().fg(Color::Yellow)),
                    Span::styled(")", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  b  ", Style::default().fg(Color::Cyan)),
                    Span::styled("Bullet journal...", Style::default().fg(Color::White)),
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
                s.insert(*cursor, '█');
                s
            } else {
                format!("{}█", input)
            };
            let input_widget = Paragraph::new(Line::from(vec![
                Span::styled("Task: ", Style::default().fg(Color::Green)),
                Span::styled(display, Style::default().fg(Color::White)),
                Span::styled("  #tag1 #tag2", Style::default().fg(Color::DarkGray)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Add Task (Esc cancel) "));
            f.render_widget(input_widget, chunks[1]);
        }
    }
}
