use bevy::prelude::*;
use bevy_spacetimedb::StdbConnection;
use crate::components::{get_defense_type_icon, DefenseType, Enemy};
use crate::config::{UnitSpawn, UnitType, Wave};
use crate::module_bindings::{DbConnection, start_wave as start_wave_reducer};
use crate::resources::{AppState, EnemySpawner, GameState, WaveConfigs};
use crate::resources::AppState::InGame;

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;
// ============================================================================
// Components
// ============================================================================

#[derive(Component)]
pub struct WavePanel;

#[derive(Component)]
pub struct WaveEnemyItem;

#[derive(Component)]
pub struct WaveTimerText;
// ============================================================================
// Resources
// ============================================================================

#[derive(Resource)]
pub struct WaveManager {
    pub preparation_time: f32,
    pub current_prep_time: f32,
    pub wave_active: bool,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            preparation_time: 30.0, // 15 seconds between waves
            current_prep_time: 15.0,
            wave_active: false,
        }
    }
}

// ============================================================================
// Systems
// ============================================================================

/// Setup the wave panel UI in the top-left corner
pub fn setup_wave_panel(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            width: Val::Px(280.0),
            ..default()
        },
        WavePanel,
    ));
}

/// Update the wave panel with current wave information
pub fn update_wave_panel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    wave_manager: Res<WaveManager>,
    game_state: Option<Res<GameState>>,
    app_state: Res<State<AppState>>,
    spawner: Option<Res<EnemySpawner>>,
    wave_configs: Option<Res<WaveConfigs>>,
    panel_query: Query<Entity, With<WavePanel>>,
    items_query: Query<Entity, With<WaveEnemyItem>>,
) {
    let Ok(panel_entity) = panel_query.single() else {
        return;
    };

    // Early return if required resources aren't loaded yet
    let Some(game_state) = game_state else {
        return;
    };
    let Some(spawner) = spawner else {
        return;
    };
    let Some(wave_configs) = wave_configs else {
        return;
    };

    if !app_state.eq(&InGame) {
        return;
    }
    // Only show panel during preparation phase or when no enemies spawned yet
    if wave_manager.wave_active && spawner.enemies_spawned > 0 {
        return;
    }

    // Only update when relevant resources change
    if !wave_manager.is_changed() && !game_state.is_changed() {
        return;
    }

    // Remove existing items
    for entity in items_query.iter() {
        commands.entity(entity).despawn();
    }

    // Get next wave (current wave index)
    let next_wave_idx = (game_state.wave - 1) as usize;

    if next_wave_idx >= wave_configs.waves.len() {
        return;
    }

    let wave = &wave_configs.waves[next_wave_idx];

    // Rebuild the wave panel
    commands.entity(panel_entity).with_children(|parent| {
        spawn_wave_banner(parent, wave, &wave_configs, &wave_manager, &asset_server);
    });
}

/// Spawns the complete wave banner with all enemy information
fn spawn_wave_banner(
    parent: &mut ChildSpawnerCommands,
    wave: &Wave,
    wave_configs: &WaveConfigs,
    _wave_manager: &WaveManager,
    asset_server: &AssetServer,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Relative,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            WaveEnemyItem,
        ))
        .with_children(|banner| {
            // Banner background image
            banner.spawn((
                ImageNode::new(asset_server.load(
                    "UI Elements/UI Elements/Banners/Banner.png"
                )),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..default()
                },
            ));

            // Content container (on top of background)
            banner
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::top(Val::Px(5.0)),
                    ..default()
                })
                .with_children(|content| {
                    // "NEXT WAVE" text centered
                    content.spawn((
                        Text::new("NEXT WAVE"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.25, 0.1)), // Dark brown
                    ));

                    // Enemy list
                    for spawn in &wave.spawns {
                        if let Some(unit) = wave_configs
                            .units
                            .iter()
                            .find(|u| u.id == spawn.unit_id)
                        {
                            spawn_enemy_row(content, unit, spawn, asset_server);
                        }
                    }
                    //dynamic padding for long lists
                    content.spawn(Node {
                        height: Val::Px(6.0 * wave.spawns.len() as f32),
                        width: Val::Percent(100.0),
                        ..default()
                    });
                });
        });
}

/// Spawns a single enemy row with avatar and count
fn spawn_enemy_row(
    parent: &mut ChildSpawnerCommands,
    unit: &UnitType,
    spawn: &UnitSpawn,
    asset_server: &AssetServer,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            column_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(6.0)),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
                .with_children(|left| {
                    left.spawn(Node {
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        position_type: PositionType::Relative,
                        ..default()
                    })
                        .with_children(|avatar| {
                            // Avatar image
                            avatar.spawn((
                                ImageNode::new(asset_server.load(&unit.avatar_path)),
                                Node {
                                    width: Val::Px(50.0),
                                    height: Val::Px(50.0),
                                    ..default()
                                },
                            ));

                            // Defense type icon
                            avatar.spawn((
                                ImageNode::new(asset_server.load(
                                    get_defense_type_icon(DefenseType::from_str(&unit.defense_type))
                                )),
                                Node {
                                    width: Val::Px(18.0),
                                    height: Val::Px(18.0),
                                    position_type: PositionType::Absolute,
                                    right: Val::Px(-4.0),
                                    bottom: Val::Px(-4.0),
                                    ..default()
                                },
                            ));
                        });

                    // Enemy name
                    left.spawn((
                        Text::new(&unit.name),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.25, 0.1)),
                    ));
                });

            // Right side: Count
            row.spawn((
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                BorderRadius::all(Val::Px(16.0)),
            ))
                .with_child((
                    Text::new(format!("{}", spawn.count)),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        });
}

/// Update the timer text every frame
pub fn update_wave_timer(
    wave_manager: Res<WaveManager>,
    mut timer_query: Query<&mut Text, With<WaveTimerText>>,
) {
    if wave_manager.wave_active {
        return;
    }

    for mut text in timer_query.iter_mut() {
        text.0 = format!("Time: {}s", wave_manager.current_prep_time.ceil() as i32);
    }
}

/// Countdown the wave timer and start wave automatically
pub fn countdown_wave_timer(
    time: Res<Time>,
    mut wave_manager: ResMut<WaveManager>,
    spawner: Option<ResMut<EnemySpawner>>,
    stdb: Option<SpacetimeDB>,
) {
    if wave_manager.wave_active {
        return;
    }

    let Some(mut spawner) = spawner else {
        return;
    };

    wave_manager.current_prep_time -= time.delta_secs();

    if wave_manager.current_prep_time <= 0.0 {
        // Call server reducer to start wave (server handles wave state authoritatively)
        if let Some(ref stdb) = stdb {
            if let Err(e) = stdb.conn().reducers.start_wave() {
                warn!("Failed to start wave via server: {:?}", e);
                // Fallback to local wave start
                start_wave_local(&mut wave_manager, &mut spawner);
            } else {
                info!("Wave start requested from server");
                // Also start locally for immediate feedback
                // Server sync will correct if needed
                start_wave_local(&mut wave_manager, &mut spawner);
            }
        } else {
            // No server connection - start locally
            start_wave_local(&mut wave_manager, &mut spawner);
        }
    }
}

/// Start the wave locally (called by timer or button, or as fallback from server)
fn start_wave_local(wave_manager: &mut WaveManager, spawner: &mut EnemySpawner) {
    wave_manager.wave_active = true;
    wave_manager.current_prep_time = 0.0;
    spawner.timer.reset(); // Start spawning immediately
}

/// Reset wave manager when wave completes (called from spawn_enemies)
pub fn check_wave_completion(
    spawner: Option<Res<EnemySpawner>>,
    mut wave_manager: ResMut<WaveManager>,
    enemy_query: Query<&Enemy>,
) {
    let Some(spawner) = spawner else {
        return;
    };

    // Wave is complete when all enemies spawned AND no enemies remain on field
    if wave_manager.wave_active
        && spawner.enemies_spawned >= spawner.enemies_this_wave
        && enemy_query.is_empty()
    {
        wave_manager.wave_active = false;
        wave_manager.current_prep_time = wave_manager.preparation_time;
    }
}


// ============================================================================
// Plugin
// ============================================================================

pub struct WaveManagerPlugin;

impl Plugin for WaveManagerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<WaveManager>()
            .add_systems(Startup, setup_wave_panel)
            .add_systems(Update, (
                update_wave_panel,
                update_wave_timer,
                countdown_wave_timer,
                check_wave_completion,
            ));
    }
}