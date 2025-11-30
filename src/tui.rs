use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::module_bindings::*;
use spacetimedb_sdk::{Identity, Table};

pub struct AppState {
    pub needs_redraw: bool,
    pub hot_rx: std::sync::mpsc::Receiver<()>,
}

impl AppState {
    pub fn new() -> Self {
        let (hot_tx, hot_rx) = std::sync::mpsc::channel();
        subsecond::register_handler(std::sync::Arc::new(move || {
            let _ = hot_tx.send(());
        }));

        Self {
            needs_redraw: false,
            hot_rx,
        }
    }

    pub fn trigger_redraw(&mut self) {
        self.needs_redraw = true;
    }

    pub fn take_redraw_flag(&mut self) -> bool {
        let flag = self.needs_redraw;
        self.needs_redraw = false;
        flag
    }

    pub fn check_hotreload(&self) -> bool {
        self.hot_rx.try_recv().is_ok()
    }
}

pub struct App {
    pub input: String,
    pub input_mode: InputMode,
    pub state: Arc<Mutex<AppState>>,
    pub ctx: DbConnection,
    pub my_identity: Option<Identity>,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Command,
}

impl App {
    pub fn new(
        ctx: DbConnection,
        state: Arc<Mutex<AppState>>,
        my_identity_arc: Arc<Mutex<Option<Identity>>>,
    ) -> Self {
        let my_identity = my_identity_arc.lock().unwrap().clone();
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            state,
            ctx,
            my_identity,
        }
    }

    pub fn submit_input(&mut self) {
        let input = self.input.drain(..).collect::<String>();
        if input.is_empty() {
            return;
        }

        if self.input_mode == InputMode::Command {
            if let Some(name) = input.strip_prefix("name ") {
                let _ = self.ctx.reducers.set_name(name.to_string());
            }
        } else {
            let _ = self.ctx.reducers.send_message(input);
        }
    }
}

pub fn run_tui(
    ctx: DbConnection,
    state: Arc<Mutex<AppState>>,
    my_identity: Arc<Mutex<Option<Identity>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(ctx, state, my_identity);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle quit gracefully
    if let Err(err) = res {
        if err.kind() != io::ErrorKind::Other || err.to_string() != "quit" {
            println!("Error: {err:?}");
        }
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        subsecond::call(|| tick(terminal, app))?;
    }
}

fn tick<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    // Check if we need to redraw
    let should_draw = {
        let mut state = app.state.lock().unwrap();
        state.take_redraw_flag() || state.check_hotreload() || true // Always draw for now to handle input changes
    };

    if should_draw {
        terminal.draw(|f| ui(f, app))?;
    }

    // Poll for events with a timeout
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => {
                                return Err(io::Error::new(io::ErrorKind::Other, "quit"));
                            }
                            KeyCode::Char('i') => app.input_mode = InputMode::Editing,
                            KeyCode::Char('/') => {
                                app.input_mode = InputMode::Command;
                                app.input.clear();
                            }
                            _ => {}
                        },
                        InputMode::Editing | InputMode::Command => match key.code {
                            KeyCode::Enter => {
                                app.submit_input();
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => {
                                app.input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.input.clear();
                            }
                            _ => {}
                        },
                    }
                }
            }
            break;
        }

        // Check for hotreload events
        let has_hotreload = {
            let state = app.state.lock().unwrap();
            state.check_hotreload()
        };
        if has_hotreload {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(chunks[0]);

    // Messages area
    render_messages(f, app, main_chunks[0]);

    // Users area
    render_users(f, app, main_chunks[1]);

    // Input area
    render_input(f, app, chunks[1]);
}

fn render_messages(f: &mut Frame, app: &App, area: Rect) {
    // Get messages directly from the database
    let mut all_messages: Vec<Message> = app.ctx.db.message().iter().collect();
    all_messages.sort_by_key(|m| m.sent);

    // Take the last N messages that fit in the visible area
    let visible_height = area.height.saturating_sub(2) as usize; // -2 for borders
    let start_idx = all_messages.len().saturating_sub(visible_height);

    let messages: Vec<ListItem> = all_messages
        .iter()
        .skip(start_idx)
        .map(|msg| {
            // Get sender name from user table
            let sender_name = app
                .ctx
                .db
                .user()
                .identity()
                .find(&msg.sender)
                .and_then(|u| u.name.clone())
                .unwrap_or_else(|| msg.sender.to_abbreviated_hex().to_string());

            let content = Line::from(vec![
                Span::styled(
                    format!("{}: ", sender_name),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&msg.text),
            ]);
            ListItem::new(content)
        })
        .collect();

    let messages_list =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));

    f.render_widget(messages_list, area);
}

fn render_users(f: &mut Frame, app: &App, area: Rect) {
    // Get online users directly from the database
    let online_users: Vec<User> = app.ctx.db.user().iter().filter(|u| u.online).collect();

    let users: Vec<ListItem> = online_users
        .iter()
        .map(|user| {
            let is_me = app
                .my_identity
                .as_ref()
                .map_or(false, |id| id == &user.identity);

            let display_name = user
                .name
                .clone()
                .unwrap_or_else(|| user.identity.to_abbreviated_hex().to_string());

            let style = if is_me {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(display_name, style)))
        })
        .collect();

    let users_list = List::new(users).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Users ({})", online_users.len())),
    );

    f.render_widget(users_list, area);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let input_text = match app.input_mode {
        InputMode::Normal => {
            Text::from("Presss 'i' to type a message, '/' for command, 'q' to quit")
        }
        InputMode::Editing => Text::from(format!("{}", app.input)),
        InputMode::Command => Text::from(format!("/{}", app.input)),
    };

    let input_style = match app.input_mode {
        InputMode::Normal => Style::default(),
        InputMode::Editing => Style::default().fg(Color::Yellow),
        InputMode::Command => Style::default().fg(Color::Magenta),
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default().borders(Borders::ALL).title("Input"));

    f.render_widget(input, area);

    // Set cursor position when in editing mode
    if app.input_mode != InputMode::Normal {
        let cursor_x = area.x + app.input.len() as u16 + 2; // +2 for border and prefix
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}
