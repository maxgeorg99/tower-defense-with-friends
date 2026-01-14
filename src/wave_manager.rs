mod config;
use config::{UnitSpawn, UnitType, UnitsConfig, Wave, WavesConfig};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use ratatui_image::{picker::{Picker, ProtocolType}, protocol::StatefulProtocol, StatefulImage};
use std::{
    fs,
    io::{self, stdout},
    path::Path,
};

// === APP STATE ===

enum SelectedPanel {
    Waves,
    WaveDetails,
    Units,
}

#[derive(PartialEq, Clone, Copy)]
enum WaveDetailField {
    SpawnInterval,
    Spawn(usize), // Index of spawn
}

impl WaveDetailField {
    fn is_spawn(&self) -> bool {
        matches!(self, WaveDetailField::Spawn(_))
    }

    fn spawn_index(&self) -> Option<usize> {
        match self {
            WaveDetailField::Spawn(idx) => Some(*idx),
            _ => None,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum SpawnField {
    UnitType,
    Count,
    HealthMultiplier,
}

impl SpawnField {
    fn all() -> Vec<SpawnField> {
        vec![
            SpawnField::UnitType,
            SpawnField::Count,
            SpawnField::HealthMultiplier,
        ]
    }

    fn next(&self) -> SpawnField {
        let all = Self::all();
        let current_idx = all.iter().position(|f| f == self).unwrap();
        all[(current_idx + 1) % all.len()]
    }

    fn prev(&self) -> SpawnField {
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
    units: Vec<UnitType>,
    waves: Vec<Wave>,
    selected_panel: SelectedPanel,
    wave_list_state: ListState,
    unit_list_state: ListState,
    current_wave: Option<Wave>,
    status_message: String,
    selected_field: WaveDetailField,
    selected_spawn_field: SpawnField,
    editing: bool,
    edit_buffer: String,
    picker: Picker,
    unit_image: Option<StatefulProtocol>,
}

impl App {
    fn new() -> io::Result<Self> {
        // Load configs
        let units_config =
            UnitsConfig::load().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let waves_config =
            WavesConfig::load().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let picker = Picker::from_query_stdio().unwrap_or(Picker::halfblocks());

        let mut app = Self {
            units: units_config.units,
            waves: waves_config.waves,
            selected_panel: SelectedPanel::Waves,
            wave_list_state: ListState::default(),
            unit_list_state: ListState::default(),
            current_wave: None,
            status_message: "q:quit | w:save | a:new wave | x:delete | Tab:switch | ↑/↓:navigate | Enter:edit/save"
                .to_string(),
            selected_field: WaveDetailField::SpawnInterval,
            selected_spawn_field: SpawnField::UnitType,
            editing: false,
            edit_buffer: String::new(),
            picker,
            unit_image: None,
        };

        if !app.waves.is_empty() {
            app.wave_list_state.select(Some(0));
            app.current_wave = Some(app.waves[0].clone());
        }

        if !app.units.is_empty() {
            app.unit_list_state.select(Some(0));
            app.load_selected_unit_image();
        }

        Ok(app)
    }

    fn next_wave(&mut self) {
        let i = match self.wave_list_state.selected() {
            Some(i) => {
                if i >= self.waves.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.wave_list_state.select(Some(i));
        self.current_wave = Some(self.waves[i].clone());
    }

    fn previous_wave(&mut self) {
        let i = match self.wave_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.waves.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.wave_list_state.select(Some(i));
        self.current_wave = Some(self.waves[i].clone());
    }

    fn next_unit(&mut self) {
        let i = match self.unit_list_state.selected() {
            Some(i) => {
                if i >= self.units.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.unit_list_state.select(Some(i));
        self.load_selected_unit_image();
    }

    fn previous_unit(&mut self) {
        let i = match self.unit_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.units.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.unit_list_state.select(Some(i));
        self.load_selected_unit_image();
    }

    fn load_selected_unit_image(&mut self) {
        if let Some(idx) = self.unit_list_state.selected() {
            if let Some(unit) = self.units.get(idx) {
                let sprite_path = format!("assets/{}", unit.sprite_path);
                if let Ok(dyn_img) = image::ImageReader::open(&sprite_path)
                    .and_then(|r| r.decode().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
                {
                    // Crop to first frame (assuming square frames based on height)
                    let height = dyn_img.height();
                    let frame_width = height; // Assume square frames
                    let cropped = dyn_img.crop_imm(0, 0, frame_width, height);
                    self.unit_image = Some(self.picker.new_resize_protocol(cropped));
                } else {
                    self.unit_image = None;
                }
            }
        }
    }

    fn save(&mut self) -> io::Result<()> {
        // Save waves config
        let waves_config = WavesConfig {
            waves: self.waves.clone(),
        };
        let waves_toml = toml::to_string_pretty(&waves_config)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write("waves.toml", waves_toml)?;

        // Save units config
        let units_config = UnitsConfig {
            units: self.units.clone(),
        };
        let units_toml = toml::to_string_pretty(&units_config)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write("units.toml", units_toml)?;

        self.status_message = "✓ Saved successfully!".to_string();
        Ok(())
    }

    fn add_new_wave(&mut self) {
        let new_wave_number = self.waves.iter().map(|w| w.wave_number).max().unwrap_or(0) + 1;
        let new_wave = Wave {
            wave_number: new_wave_number,
            spawn_interval: 2.0,
            spawns: vec![UnitSpawn {
                unit_id: "warrior".to_string(),
                count: 5,
                health_multiplier: 1.0,
            }],
        };
        self.waves.push(new_wave.clone());
        self.wave_list_state.select(Some(self.waves.len() - 1));
        self.current_wave = Some(new_wave);
        self.status_message = format!("Added Wave {}", new_wave_number);
    }

    fn delete_current_wave(&mut self) {
        if let Some(idx) = self.wave_list_state.selected() {
            if !self.waves.is_empty() {
                self.waves.remove(idx);
                self.status_message = "Wave deleted".to_string();

                if self.waves.is_empty() {
                    self.current_wave = None;
                    self.wave_list_state.select(None);
                } else {
                    let new_idx = idx.min(self.waves.len() - 1);
                    self.wave_list_state.select(Some(new_idx));
                    self.current_wave = Some(self.waves[new_idx].clone());
                }
            }
        }
    }

    fn add_spawn_to_current_wave(&mut self) {
        if let Some(idx) = self.wave_list_state.selected() {
            let new_spawn = UnitSpawn {
                unit_id: self
                    .units
                    .first()
                    .map(|u| u.id.clone())
                    .unwrap_or("warrior".to_string()),
                count: 5,
                health_multiplier: 1.0,
            };
            self.waves[idx].spawns.push(new_spawn);
            self.current_wave = Some(self.waves[idx].clone());
            // Select the newly added spawn
            let new_spawn_idx = self.waves[idx].spawns.len() - 1;
            self.selected_field = WaveDetailField::Spawn(new_spawn_idx);
            self.status_message = "Added new spawn to wave".to_string();
        }
    }

    fn remove_current_spawn(&mut self) {
        if let Some(wave_idx) = self.wave_list_state.selected() {
            if let Some(spawn_idx) = self.selected_field.spawn_index() {
                if spawn_idx < self.waves[wave_idx].spawns.len()
                    && !self.waves[wave_idx].spawns.is_empty()
                {
                    self.waves[wave_idx].spawns.remove(spawn_idx);
                    self.current_wave = Some(self.waves[wave_idx].clone());

                    // Adjust selected field
                    if self.waves[wave_idx].spawns.is_empty() {
                        self.selected_field = WaveDetailField::SpawnInterval;
                    } else if spawn_idx > 0 {
                        self.selected_field = WaveDetailField::Spawn(spawn_idx - 1);
                    } else if !self.waves[wave_idx].spawns.is_empty() {
                        self.selected_field = WaveDetailField::Spawn(0);
                    }

                    self.status_message = "Removed spawn from wave".to_string();
                }
            }
        }
    }

    fn next_field(&mut self) {
        if let Some(wave) = &self.current_wave {
            match &self.selected_field {
                WaveDetailField::SpawnInterval => {
                    if !wave.spawns.is_empty() {
                        self.selected_field = WaveDetailField::Spawn(0);
                        self.selected_spawn_field = SpawnField::UnitType;
                    }
                }
                WaveDetailField::Spawn(idx) => {
                    if *idx < wave.spawns.len() - 1 {
                        self.selected_field = WaveDetailField::Spawn(idx + 1);
                        self.selected_spawn_field = SpawnField::UnitType;
                    }
                }
            }
        }
    }

    fn prev_field(&mut self) {
        match &self.selected_field {
            WaveDetailField::SpawnInterval => {}
            WaveDetailField::Spawn(idx) => {
                if *idx == 0 {
                    self.selected_field = WaveDetailField::SpawnInterval;
                } else {
                    self.selected_field = WaveDetailField::Spawn(idx - 1);
                    self.selected_spawn_field = SpawnField::UnitType;
                }
            }
        }
    }

    fn next_spawn_field(&mut self) {
        if self.selected_field.is_spawn() {
            self.selected_spawn_field = self.selected_spawn_field.next();
        }
    }

    fn prev_spawn_field(&mut self) {
        if self.selected_field.is_spawn() {
            self.selected_spawn_field = self.selected_spawn_field.prev();
        }
    }

    fn cycle_unit_for_current_spawn(&mut self) {
        if let Some(wave_idx) = self.wave_list_state.selected() {
            if let Some(spawn_idx) = self.selected_field.spawn_index() {
                if spawn_idx < self.waves[wave_idx].spawns.len() {
                    let current_unit_id = &self.waves[wave_idx].spawns[spawn_idx].unit_id;

                    // Find current unit index and cycle to next
                    if let Some(current_idx) =
                        self.units.iter().position(|u| &u.id == current_unit_id)
                    {
                        let next_idx = (current_idx + 1) % self.units.len();
                        self.waves[wave_idx].spawns[spawn_idx].unit_id =
                            self.units[next_idx].id.clone();
                        self.current_wave = Some(self.waves[wave_idx].clone());
                        self.status_message =
                            format!("Changed unit to {}", self.units[next_idx].name);
                    }
                }
            }
        }
    }

    fn start_editing(&mut self) {
        if let Some(wave) = &self.current_wave {
            self.editing = true;

            match &self.selected_field {
                WaveDetailField::SpawnInterval => {
                    self.edit_buffer = wave.spawn_interval.to_string();
                }
                WaveDetailField::Spawn(idx) => {
                    if *idx < wave.spawns.len() {
                        self.edit_buffer = match self.selected_spawn_field {
                            SpawnField::UnitType => wave.spawns[*idx].unit_id.clone(),
                            SpawnField::Count => wave.spawns[*idx].count.to_string(),
                            SpawnField::HealthMultiplier => {
                                wave.spawns[*idx].health_multiplier.to_string()
                            }
                        };
                    }
                }
            }

            self.status_message = "Editing (Enter to save, Esc to cancel)".to_string();
        }
    }

    fn confirm_edit(&mut self) {
        if let Some(wave_idx) = self.wave_list_state.selected() {
            let result = match &self.selected_field {
                WaveDetailField::SpawnInterval => {
                    if let Ok(value) = self.edit_buffer.parse::<f32>() {
                        self.waves[wave_idx].spawn_interval = value;
                        Ok(format!("Spawn interval set to {}", value))
                    } else {
                        Err("Invalid number".to_string())
                    }
                }
                WaveDetailField::Spawn(spawn_idx) => {
                    if *spawn_idx < self.waves[wave_idx].spawns.len() {
                        match self.selected_spawn_field {
                            SpawnField::UnitType => {
                                // Validate unit ID exists
                                if self.units.iter().any(|u| u.id == self.edit_buffer) {
                                    self.waves[wave_idx].spawns[*spawn_idx].unit_id =
                                        self.edit_buffer.clone();
                                    Ok("Unit type updated".to_string())
                                } else {
                                    Err(format!("Unit '{}' not found", self.edit_buffer))
                                }
                            }
                            SpawnField::Count => {
                                if let Ok(value) = self.edit_buffer.parse::<i32>() {
                                    self.waves[wave_idx].spawns[*spawn_idx].count = value;
                                    Ok(format!("Count set to {}", value))
                                } else {
                                    Err("Invalid number".to_string())
                                }
                            }
                            SpawnField::HealthMultiplier => {
                                if let Ok(value) = self.edit_buffer.parse::<f32>() {
                                    self.waves[wave_idx].spawns[*spawn_idx].health_multiplier =
                                        value;
                                    Ok(format!("Health multiplier set to {}", value))
                                } else {
                                    Err("Invalid number".to_string())
                                }
                            }
                        }
                    } else {
                        Err("Invalid spawn index".to_string())
                    }
                }
            };

            match result {
                Ok(msg) => {
                    self.current_wave = Some(self.waves[wave_idx].clone());
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
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(chunks[0]);

    // Render waves list
    render_waves_list(f, app, main_chunks[0]);

    // Render wave details
    render_wave_details(f, app, main_chunks[1]);

    // Split units panel into image preview (top) and units list (bottom)
    let units_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(main_chunks[2]);

    // Render unit image preview
    render_unit_image(f, app, units_chunks[0]);

    // Render units list
    render_units_list(f, app, units_chunks[1]);

    // Render status bar
    render_status_bar(f, app, chunks[1]);
}

fn render_waves_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .waves
        .iter()
        .map(|wave| {
            let total_units: i32 = wave.spawns.iter().map(|s| s.count).sum();
            let content = Line::from(vec![
                Span::styled(
                    format!("Wave {}", wave.wave_number),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::styled(
                    format!("{} units", total_units),
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    let is_selected = matches!(app.selected_panel, SelectedPanel::Waves);
    let border_style = if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Waves (↑/↓ to navigate, 'n' to add)")
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.wave_list_state);
}

fn render_wave_details(f: &mut Frame, app: &App, area: Rect) {
    let is_selected = matches!(app.selected_panel, SelectedPanel::WaveDetails);
    let border_style = if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    if let Some(wave) = &app.current_wave {
        let mut lines = vec![Line::from(vec![
            Span::styled("Wave Number: ", Style::default().fg(Color::Cyan)),
            Span::raw(wave.wave_number.to_string()),
        ])];

        // Spawn Interval field
        let is_interval_selected =
            matches!(app.selected_field, WaveDetailField::SpawnInterval) && is_selected;
        let is_interval_editing = app.editing && is_interval_selected;
        let mut interval_spans = vec![
            Span::styled(
                if is_interval_selected { ">> " } else { "   " },
                Style::default().fg(Color::Green),
            ),
            Span::styled("Spawn Interval: ", Style::default().fg(Color::Cyan)),
        ];
        if is_interval_editing {
            interval_spans.push(Span::styled(
                app.edit_buffer.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            interval_spans.push(Span::raw(format!("{:.1}s", wave.spawn_interval)));
        }
        lines.push(Line::from(interval_spans));

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Spawns (↑/↓:navigate spawns | ←/→:navigate fields):",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));

        for (idx, spawn) in wave.spawns.iter().enumerate() {
            if let Some(unit) = app.units.iter().find(|u| u.id == spawn.unit_id) {
                let health = unit.base_health * spawn.health_multiplier;
                let is_spawn_selected = matches!(app.selected_field, WaveDetailField::Spawn(i) if i == idx)
                    && is_selected;

                lines.push(Line::from(""));

                // Unit Type field
                let is_unit_field =
                    is_spawn_selected && matches!(app.selected_spawn_field, SpawnField::UnitType);
                let is_editing_unit = app.editing && is_unit_field;
                let mut unit_spans = vec![
                    Span::styled(
                        if is_unit_field { ">> " } else { "   " },
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled("Unit: ", Style::default().fg(Color::White)),
                ];
                if is_editing_unit {
                    unit_spans.push(Span::styled(
                        app.edit_buffer.clone(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    unit_spans.push(Span::styled(&unit.name, Style::default().fg(Color::Cyan)));
                }
                lines.push(Line::from(unit_spans));

                // Count field
                let is_count_field =
                    is_spawn_selected && matches!(app.selected_spawn_field, SpawnField::Count);
                let is_editing_count = app.editing && is_count_field;
                let mut count_spans = vec![
                    Span::styled(
                        if is_count_field { ">> " } else { "   " },
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled("Count: ", Style::default().fg(Color::White)),
                ];
                if is_editing_count {
                    count_spans.push(Span::styled(
                        app.edit_buffer.clone(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    count_spans.push(Span::styled(
                        spawn.count.to_string(),
                        Style::default().fg(Color::Yellow),
                    ));
                }
                lines.push(Line::from(count_spans));

                // Health Multiplier field
                let is_health_field = is_spawn_selected
                    && matches!(app.selected_spawn_field, SpawnField::HealthMultiplier);
                let is_editing_health = app.editing && is_health_field;
                let mut health_spans = vec![
                    Span::styled(
                        if is_health_field { ">> " } else { "   " },
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled("Health Mult: ", Style::default().fg(Color::White)),
                ];
                if is_editing_health {
                    health_spans.push(Span::styled(
                        app.edit_buffer.clone(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    health_spans.push(Span::raw(format!(
                        "x{:.1} ({:.0} HP)",
                        spawn.health_multiplier, health
                    )));
                }
                lines.push(Line::from(health_spans));

                // Unit stats (read-only)
                lines.push(Line::from(vec![
                    Span::raw("   Speed: "),
                    Span::styled(
                        format!("{:.0}", unit.base_speed),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::raw(" | Damage: "),
                    Span::styled(
                        unit.damage_to_base.to_string(),
                        Style::default().fg(Color::Gray),
                    ),
                ]));
            }
        }

        let title = if app.editing {
            "Wave Details (Editing - Enter to save, Esc to cancel)"
        } else {
            "Wave Details (Enter:edit | Space:cycle unit | Insert:add | Delete:remove)"
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
        let paragraph = Paragraph::new("No wave selected").block(
            Block::default()
                .borders(Borders::ALL)
                .title("Wave Details")
                .border_style(border_style),
        );
        f.render_widget(paragraph, area);
    }
}

fn render_unit_image(f: &mut Frame, app: &mut App, area: Rect) {
    let is_selected = matches!(app.selected_panel, SelectedPanel::Units);
    let border_style = if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Unit Preview")
        .border_style(border_style);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(ref mut image_protocol) = app.unit_image {
        let image_widget = StatefulImage::default();
        f.render_stateful_widget(image_widget, inner_area, image_protocol);
    } else {
        let placeholder = Paragraph::new("No image available")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, inner_area);
    }
}

fn render_units_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .units
        .iter()
        .map(|unit| {
            let content = vec![
                Line::from(vec![Span::styled(
                    &unit.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                    Span::raw("  HP: "),
                    Span::styled(
                        format!("{:.0}", unit.base_health),
                        Style::default().fg(Color::Red),
                    ),
                    Span::raw(" | Spd: "),
                    Span::styled(
                        format!("{:.0}", unit.base_speed),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  Dmg: "),
                    Span::styled(
                        unit.damage_to_base.to_string(),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::raw(" | Gold: "),
                    Span::styled(
                        unit.gold_reward.to_string(),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  Asset: "),
                    Span::styled(
                        Path::new(&unit.sprite_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&unit.sprite_path),
                        Style::default().fg(Color::Gray),
                    ),
                ]),
                Line::from(""),
            ];
            ListItem::new(content)
        })
        .collect();

    let is_selected = matches!(app.selected_panel, SelectedPanel::Units);
    let border_style = if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Available Units")
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.unit_list_state);
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

    if let Err(err) = res {
        println!("Error: {err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()>
where
    B::Error: std::fmt::Debug,
{
    loop {
        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

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
                        app.add_new_wave();
                    }
                    KeyCode::Char('x') => {
                        if matches!(app.selected_panel, SelectedPanel::Waves) {
                            app.delete_current_wave();
                        }
                    }
                    KeyCode::Insert => {
                        if matches!(app.selected_panel, SelectedPanel::WaveDetails) {
                            app.add_spawn_to_current_wave();
                        }
                    }
                    KeyCode::Delete => {
                        if matches!(app.selected_panel, SelectedPanel::WaveDetails) {
                            app.remove_current_spawn();
                        }
                    }
                    KeyCode::Char(' ') => {
                        if matches!(app.selected_panel, SelectedPanel::WaveDetails) {
                            app.cycle_unit_for_current_spawn();
                        }
                    }
                    KeyCode::Enter => {
                        if matches!(app.selected_panel, SelectedPanel::WaveDetails) {
                            app.start_editing();
                        }
                    }
                    KeyCode::Down => match app.selected_panel {
                        SelectedPanel::Waves => app.next_wave(),
                        SelectedPanel::WaveDetails => app.next_field(),
                        SelectedPanel::Units => app.next_unit(),
                    },
                    KeyCode::Up => match app.selected_panel {
                        SelectedPanel::Waves => app.previous_wave(),
                        SelectedPanel::WaveDetails => app.prev_field(),
                        SelectedPanel::Units => app.previous_unit(),
                    },
                    KeyCode::Left => {
                        if matches!(app.selected_panel, SelectedPanel::WaveDetails) {
                            app.prev_spawn_field();
                        }
                    }
                    KeyCode::Right => {
                        if matches!(app.selected_panel, SelectedPanel::WaveDetails) {
                            app.next_spawn_field();
                        }
                    }
                    KeyCode::Tab => {
                        app.selected_panel = match app.selected_panel {
                            SelectedPanel::Waves => SelectedPanel::WaveDetails,
                            SelectedPanel::WaveDetails => SelectedPanel::Units,
                            SelectedPanel::Units => SelectedPanel::Waves,
                        };
                    }
                    _ => {}
                }
            }
        }
    }
}
