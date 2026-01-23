mod config;
use config::{TowerType, TowersConfig};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::backend::Backend;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use ratatui_image::StatefulImage;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::{
    fs,
    io::{self, stdout},
};

enum SelectedPanel {
    Towers,
    TowerDetails,
}

#[derive(PartialEq, Clone, Copy)]
enum TowerField {
    Id,
    Name,
    SpritePath,
    ProjectileSprite,
    Cost,
    Range,
    Damage,
    FireRate,
    ProjectileSpeed,
    Description,
}

impl TowerField {
    fn all() -> Vec<TowerField> {
        vec![
            TowerField::Id,
            TowerField::Name,
            TowerField::SpritePath,
            TowerField::ProjectileSprite,
            TowerField::Cost,
            TowerField::Range,
            TowerField::Damage,
            TowerField::FireRate,
            TowerField::ProjectileSpeed,
            TowerField::Description,
        ]
    }

    fn next(&self) -> TowerField {
        let all = Self::all();
        let current_idx = all.iter().position(|f| f == self).unwrap();
        all[(current_idx + 1) % all.len()]
    }

    fn prev(&self) -> TowerField {
        let all = Self::all();
        let current_idx = all.iter().position(|f| f == self).unwrap();
        if current_idx == 0 {
            all[all.len() - 1]
        } else {
            all[current_idx - 1]
        }
    }
}

struct App {
    towers: Vec<TowerType>,
    selected_panel: SelectedPanel,
    tower_list_state: ListState,
    current_tower: Option<TowerType>,
    status_message: String,
    selected_field: TowerField,
    editing: bool,
    edit_buffer: String,
    picker: Picker,
    tower_image: Option<StatefulProtocol>,
    projectile_image: Option<StatefulProtocol>,
}

impl App {
    fn new() -> io::Result<Self> {
        // Load config
        let towers_config =
            TowersConfig::load().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut app = Self {
            towers: towers_config.towers,
            selected_panel: SelectedPanel::Towers,
            tower_list_state: ListState::default(),
            current_tower: None,
            status_message: "q:quit | w:save | a:new tower | x:delete | Tab:switch | ↑/↓:navigate | Enter:edit/save"
                .to_string(),
            selected_field: TowerField::Id,
            editing: false,
            edit_buffer: String::new(),
            picker: Picker::from_query_stdio().unwrap_or(Picker::halfblocks()),
            tower_image: None,
            projectile_image: None,
        };

        if !app.towers.is_empty() {
            app.tower_list_state.select(Some(0));
            app.current_tower = Some(app.towers[0].clone());
            app.load_selected_tower_image();
        }

        Ok(app)
    }

    fn load_selected_tower_image(&mut self) {
        if let Some(idx) = self.tower_list_state.selected() {
            if let Some(unit) = self.towers.get(idx) {
                let sprite_path = format!("assets/{}", unit.sprite_path);
                if let Ok(dyn_img) = image::ImageReader::open(&sprite_path).and_then(|r| {
                    r.decode()
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                }) {
                    self.tower_image = Some(self.picker.new_resize_protocol(dyn_img));
                }
                let projectile_path = format!("assets/{}", unit.projectile_sprite);
                if let Ok(projectile_img) = image::ImageReader::open(&projectile_path).and_then(|r| {
                    r.decode()
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                }) {
                    self.projectile_image = Some(self.picker.new_resize_protocol(projectile_img));
                }
            }
        }
    }
    fn next_tower(&mut self) {
        let i = match self.tower_list_state.selected() {
            Some(i) => {
                if i >= self.towers.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.tower_list_state.select(Some(i));
        self.current_tower = Some(self.towers[i].clone());
        self.load_selected_tower_image();
    }

    fn previous_tower(&mut self) {
        let i = match self.tower_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.towers.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.tower_list_state.select(Some(i));
        self.current_tower = Some(self.towers[i].clone());
        self.load_selected_tower_image();
    }

    fn save(&mut self) -> io::Result<()> {
        // Save towers config
        let towers_config = TowersConfig {
            towers: self.towers.clone(),
        };
        let towers_toml = toml::to_string_pretty(&towers_config)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write("towers.toml", towers_toml)?;

        self.status_message = "✓ Saved successfully!".to_string();
        Ok(())
    }

    fn add_new_tower(&mut self) {
        let new_tower_id = format!("tower_{}", self.towers.len() + 1);
        let new_tower = TowerType {
            id: new_tower_id.clone(),
            name: format!("New Tower {}", self.towers.len() + 1),
            sprite_path: "Decorations/Buildings/Blue Buildings/Tower.png".to_string(),
            cost: 50,
            range: 256.0,
            damage: 25.0,
            fire_rate: 0.5,
            projectile_sprite: "Units/Blue Units/Archer/Arrow.png".to_string(),
            projectile_speed: 300.0,
            description: "A new tower".to_string(),
        };
        self.towers.push(new_tower.clone());
        self.tower_list_state.select(Some(self.towers.len() - 1));
        self.current_tower = Some(new_tower);
        self.status_message = format!("Added {}", new_tower_id);
    }

    fn delete_current_tower(&mut self) {
        if let Some(idx) = self.tower_list_state.selected() {
            if !self.towers.is_empty() {
                self.towers.remove(idx);
                self.status_message = "Tower deleted".to_string();

                if self.towers.is_empty() {
                    self.current_tower = None;
                    self.tower_list_state.select(None);
                } else {
                    let new_idx = idx.min(self.towers.len() - 1);
                    self.tower_list_state.select(Some(new_idx));
                    self.current_tower = Some(self.towers[new_idx].clone());
                }
            }
        }
    }

    fn next_field(&mut self) {
        self.selected_field = self.selected_field.next();
    }

    fn prev_field(&mut self) {
        self.selected_field = self.selected_field.prev();
    }

    fn start_editing(&mut self) {
        if let Some(tower) = &self.current_tower {
            self.editing = true;
            self.edit_buffer = match self.selected_field {
                TowerField::Id => tower.id.clone(),
                TowerField::Name => tower.name.clone(),
                TowerField::SpritePath => tower.sprite_path.clone(),
                TowerField::ProjectileSprite => tower.projectile_sprite.clone(),
                TowerField::Cost => tower.cost.to_string(),
                TowerField::Range => tower.range.to_string(),
                TowerField::Damage => tower.damage.to_string(),
                TowerField::FireRate => tower.fire_rate.to_string(),
                TowerField::ProjectileSpeed => tower.projectile_speed.to_string(),
                TowerField::Description => tower.description.clone(),
            };
            self.status_message = "Editing (Enter to save, Esc to cancel)".to_string();
        }
    }

    fn confirm_edit(&mut self) {
        if let Some(tower_idx) = self.tower_list_state.selected() {
            let result = match self.selected_field {
                TowerField::Id => {
                    if !self.edit_buffer.is_empty() {
                        self.towers[tower_idx].id = self.edit_buffer.clone();
                        Ok(format!("ID updated"))
                    } else {
                        Err("ID cannot be empty".to_string())
                    }
                }
                TowerField::Name => {
                    if !self.edit_buffer.is_empty() {
                        self.towers[tower_idx].name = self.edit_buffer.clone();
                        Ok(format!("Name updated"))
                    } else {
                        Err("Name cannot be empty".to_string())
                    }
                }
                TowerField::SpritePath => {
                    if !self.edit_buffer.is_empty() {
                        self.towers[tower_idx].sprite_path = self.edit_buffer.clone();
                        self.load_selected_tower_image();
                        Ok(format!("Sprite path updated"))
                    } else {
                        Err("Sprite path cannot be empty".to_string())
                    }
                }
                TowerField::ProjectileSprite => {
                    if !self.edit_buffer.is_empty() {
                        self.towers[tower_idx].projectile_sprite = self.edit_buffer.clone();
                        self.load_selected_tower_image();
                        Ok(format!("Projectile sprite updated"))
                    } else {
                        Err("Projectile sprite cannot be empty".to_string())
                    }
                }
                TowerField::Cost => {
                    if let Ok(value) = self.edit_buffer.parse::<i32>() {
                        self.towers[tower_idx].cost = value;
                        Ok(format!("Cost set to {}", value))
                    } else {
                        Err("Invalid number".to_string())
                    }
                }
                TowerField::Range => {
                    if let Ok(value) = self.edit_buffer.parse::<f32>() {
                        self.towers[tower_idx].range = value;
                        Ok(format!("Range set to {}", value))
                    } else {
                        Err("Invalid number".to_string())
                    }
                }
                TowerField::Damage => {
                    if let Ok(value) = self.edit_buffer.parse::<f32>() {
                        self.towers[tower_idx].damage = value;
                        Ok(format!("Damage set to {}", value))
                    } else {
                        Err("Invalid number".to_string())
                    }
                }
                TowerField::FireRate => {
                    if let Ok(value) = self.edit_buffer.parse::<f32>() {
                        self.towers[tower_idx].fire_rate = value;
                        Ok(format!("Fire rate set to {}", value))
                    } else {
                        Err("Invalid number".to_string())
                    }
                }
                TowerField::ProjectileSpeed => {
                    if let Ok(value) = self.edit_buffer.parse::<f32>() {
                        self.towers[tower_idx].projectile_speed = value;
                        Ok(format!("Projectile speed set to {}", value))
                    } else {
                        Err("Invalid number".to_string())
                    }
                }
                TowerField::Description => {
                    self.towers[tower_idx].description = self.edit_buffer.clone();
                    Ok(format!("Description updated"))
                }
            };

            match result {
                Ok(msg) => {
                    self.current_tower = Some(self.towers[tower_idx].clone());
                    self.status_message = msg;
                    self.editing = false;
                }
                Err(msg) => {
                    self.status_message = msg;
                }
            }
        }
        self.edit_buffer.clear();
    }

    fn cancel_edit(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
        self.status_message = "Edit cancelled".to_string();
    }
}

// === UI RENDERING ===

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[0]);

    let tower_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(main_chunks[1]);

    let tower_image_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(tower_chunks[0]);

    // Render towers list
    render_towers_list(f, app, main_chunks[0]);

    // Render tower image preview
    render_tower_image(f, app, tower_image_chunks[0]);

    // Render projectile image preview
    render_projectile_image(f, app, tower_image_chunks[1]);

    // Render tower details
    render_tower_details(f, app, tower_chunks[1]);

    // Render status bar
    render_status_bar(f, app, chunks[1]);
}

fn render_projectile_image(f: &mut Frame, app: &mut App, area: Rect) {
    let is_selected = matches!(app.selected_panel, SelectedPanel::TowerDetails);
    let border_style = if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Projectile Preview")
        .border_style(border_style);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(ref mut image_protocol) = app.projectile_image {
        let image_widget = StatefulImage::default();
        f.render_stateful_widget(image_widget, inner_area, image_protocol);
    } else {
        let placeholder =
            Paragraph::new("No image available").style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, inner_area);
    }
}

fn render_tower_image(f: &mut Frame, app: &mut App, area: Rect) {
    let is_selected = matches!(app.selected_panel, SelectedPanel::TowerDetails);
    let border_style = if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Tower Preview")
        .border_style(border_style);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(ref mut image_protocol) = app.tower_image {
        let image_widget = StatefulImage::default();
        f.render_stateful_widget(image_widget, inner_area, image_protocol);
    } else {
        let placeholder =
            Paragraph::new("No image available").style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, inner_area);
    }
}

fn render_towers_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .towers
        .iter()
        .map(|tower| {
            let content = Line::from(vec![
                Span::styled(
                    &tower.name,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::styled(
                    format!("{}g", tower.cost),
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    let is_selected = matches!(app.selected_panel, SelectedPanel::Towers);
    let border_style = if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Towers (↑/↓ to navigate, 'n' to add)")
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.tower_list_state);
}

fn render_tower_details(f: &mut Frame, app: &mut App, area: Rect) {
    let is_selected = matches!(app.selected_panel, SelectedPanel::TowerDetails);
    let border_style = if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    if let Some(tower) = &app.current_tower {
        let make_field_line = |field: TowerField, label: String, value: String, color: Color| {
            let is_field_selected = app.selected_field == field && is_selected;
            let is_editing = app.editing && is_field_selected;

            let mut spans = vec![
                Span::styled(
                    if is_field_selected { ">> " } else { "   " },
                    Style::default().fg(Color::Green),
                ),
                Span::styled(label, Style::default().fg(color)),
            ];

            if is_editing {
                spans.push(Span::styled(
                    app.edit_buffer.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw(value));
            }

            Line::from(spans)
        };

        let lines = vec![
            make_field_line(
                TowerField::Id,
                "ID: ".to_string(),
                tower.id.clone(),
                Color::Cyan,
            ),
            make_field_line(
                TowerField::Name,
                "Name: ".to_string(),
                tower.name.clone(),
                Color::Cyan,
            ),
            Line::from(""),
            make_field_line(
                TowerField::SpritePath,
                "Sprite Path: ".to_string(),
                tower.sprite_path.clone(),
                Color::Gray,
            ),
            make_field_line(
                TowerField::ProjectileSprite,
                "Projectile Sprite: ".to_string(),
                tower.projectile_sprite.clone(),
                Color::Gray,
            ),
            Line::from(""),
            make_field_line(
                TowerField::Cost,
                "Cost: ".to_string(),
                format!("{} gold", tower.cost),
                Color::Yellow,
            ),
            make_field_line(
                TowerField::Range,
                "Range: ".to_string(),
                format!("{:.0} px ({:.1} tiles)", tower.range, tower.range / 32.0),
                Color::Cyan,
            ),
            make_field_line(
                TowerField::Damage,
                "Damage: ".to_string(),
                format!("{:.1} HP", tower.damage),
                Color::Red,
            ),
            make_field_line(
                TowerField::FireRate,
                "Fire Rate: ".to_string(),
                format!("{:.2} seconds", tower.fire_rate),
                Color::Magenta,
            ),
            make_field_line(
                TowerField::ProjectileSpeed,
                "Projectile Speed: ".to_string(),
                format!("{:.0} px/s", tower.projectile_speed),
                Color::Blue,
            ),
            Line::from(""),
            make_field_line(
                TowerField::Description,
                "Description: ".to_string(),
                tower.description.clone(),
                Color::Green,
            ),
        ];

        let title = if app.editing {
            "Tower Details (Editing - Enter to save, Esc to cancel)"
        } else {
            "Tower Details (Enter to edit selected field)"
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No tower selected").block(
            Block::default()
                .borders(Borders::ALL)
                .title("Tower Details")
                .border_style(border_style),
        );
        f.render_widget(paragraph, area);
    }
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(app.status_message.clone())
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Green));
    f.render_widget(status, area);
}

// === MAIN ===

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()>
where
    std::io::Error: From<<B as Backend>::Error>,
{
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Handle editing mode separately
            if app.editing {
                match key.code {
                    KeyCode::Enter => {
                        app.confirm_edit();
                    }
                    KeyCode::Esc => {
                        app.cancel_edit();
                    }
                    KeyCode::Backspace => {
                        app.edit_buffer.pop();
                    }
                    KeyCode::Char(c) => {
                        app.edit_buffer.push(c);
                    }
                    _ => {}
                }
            } else {
                // Normal mode input handling
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('w') => {
                        app.save()?;
                    }
                    KeyCode::Char('a') => {
                        app.add_new_tower();
                    }
                    KeyCode::Char('x') => {
                        if matches!(app.selected_panel, SelectedPanel::Towers) {
                            app.delete_current_tower();
                        }
                    }
                    KeyCode::Enter => {
                        if matches!(app.selected_panel, SelectedPanel::TowerDetails) {
                            app.start_editing();
                        }
                    }
                    KeyCode::Down => match app.selected_panel {
                        SelectedPanel::Towers => app.next_tower(),
                        SelectedPanel::TowerDetails => app.next_field(),
                    },
                    KeyCode::Up => match app.selected_panel {
                        SelectedPanel::Towers => app.previous_tower(),
                        SelectedPanel::TowerDetails => app.prev_field(),
                    },
                    KeyCode::Tab => {
                        app.selected_panel = match app.selected_panel {
                            SelectedPanel::Towers => SelectedPanel::TowerDetails,
                            SelectedPanel::TowerDetails => SelectedPanel::Towers,
                        };
                    }
                    _ => {}
                }
            }
        }
    }
}
