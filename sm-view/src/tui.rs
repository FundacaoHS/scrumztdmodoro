use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use sm_core::{BulletKind, TaskList};
use std::io::stdout;

pub fn run(tasks: &mut TaskList) -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(stdout()))?;

    let mut list_state = ListState::default();
    list_state.select(if tasks.list_tasks().is_empty() {
        None
    } else {
        Some(0)
    });

    loop {
        terminal.draw(|f| draw(f, tasks, &mut list_state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => break,
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
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(i) = list_state.selected() {
                            if let Some(task) = tasks.list_tasks().get(i) {
                                tasks.toggle_task(task.id);
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(i) = list_state.selected() {
                            if let Some(task) = tasks.list_tasks().get(i) {
                                tasks.remove_task(task.id);
                                let len = tasks.list_tasks().len();
                                if i >= len {
                                    list_state.select(Some(len.saturating_sub(1)));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn draw(f: &mut Frame, tasks: &TaskList, list_state: &mut ListState) {
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

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Tasks "))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], list_state);

    let help = Paragraph::new(Line::from(vec![
        Span::styled("↑/k ↓/j ", Style::default().fg(Color::Gray)),
        Span::styled("navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Space/Enter ", Style::default().fg(Color::Gray)),
        Span::styled("toggle  ", Style::default().fg(Color::DarkGray)),
        Span::styled("d ", Style::default().fg(Color::Red)),
        Span::styled("delete  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q ", Style::default().fg(Color::Red)),
        Span::styled("quit", Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Help "));

    f.render_widget(help, chunks[1]);
}
