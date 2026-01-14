mod config;
use config::{TowerType, TowersConfig, UnitSpawn, UnitType, UnitsConfig, Wave, WavesConfig};

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[cfg(feature = "bevy-demo")]
use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};

fn main() {
    // Connect to devtools for hot-reloading
    #[cfg(feature = "bevy-demo")]
    {
        std::thread::spawn(|| {
            dioxus_devtools::connect_subsecond();
        });
    }

    // Load initial configs
    let units = UnitsConfig::load()
        .expect("Failed to load units.toml")
        .units;
    let waves = WavesConfig::load()
        .expect("Failed to load waves.toml")
        .waves;
    let towers = TowersConfig::load()
        .expect("Failed to load towers.toml")
        .towers;

    // Initialize spawner with first wave config
    let spawner = if let Some(first_wave) = waves.first() {
        EnemySpawner::from_wave_config(first_wave)
    } else {
        panic!("No waves defined in waves.toml!");
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Tower Defense Game".to_string(),
            resolution: (1024, 768).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(TiledPlugin::default())
    .init_state::<AppState>()
    .init_resource::<GameState>()
    .insert_resource(spawner)
    .insert_resource(PathWaypoints {
        points: create_path_waypoints(),
    })
    .insert_resource(WaveConfigs { units, waves })
    .insert_resource(TowerConfigs { towers })
    .insert_resource(TowerWheelState {
        active: false,
        position: Vec2::ZERO,
    })
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            spawn_enemies,
            move_enemies,
            animate_sprites,
            show_tower_wheel_menu,
            hide_tower_wheel_menu,
            handle_tower_selection,
            tower_shooting,
            move_projectiles,
            handle_projectile_hits,
            update_health_bars,
        )
        .run_if(in_state(AppState::InGame))
    )
    .add_systems(Update, (cleanup_dead_enemies, update_ui, check_game_over))
    .add_systems(Update, camera_zoom)
    .add_systems(OnEnter(AppState::GameOver), setup_game_over_screen)
    .add_systems(OnExit(AppState::GameOver), cleanup_game_over_screen)
    .add_systems(Update, camera_pan);


    // Setup file watching for hot-reloading
    #[cfg(feature = "bevy-demo")]
    {
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(tx).expect("Failed to create file watcher");

        watcher
            .watch(Path::new("units.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch units.toml");
        watcher
            .watch(Path::new("waves.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch waves.toml");
        watcher
            .watch(Path::new("towers.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch towers.toml");

        app.insert_resource(FileWatcher {
            receiver: Arc::new(Mutex::new(rx)),
            _watcher: Arc::new(Mutex::new(Box::new(watcher))),
        });
        app.add_systems(Update, watch_config_files);
    }

    app.run();
}

// === COMPONENTS ===

#[derive(Component)]
struct Enemy {
    health: f32,
    speed: f32,
    current_waypoint: usize,
    gold_reward: i32,
    damage_to_base: i32,
}

#[derive(Component)]
struct AnimationTimer {
    timer: Timer,
}

#[derive(Component)]
struct Tower {
    tower_type_id: String,
    range: f32,
    damage: f32,
    fire_rate: f32,
    cooldown: f32,
    projectile_sprite: String,
    projectile_speed: f32,
}

#[derive(Component)]
struct Projectile {
    damage: f32,
    speed: f32,
    target: Entity,
}

#[derive(Component)]
struct HealthBar {
    max_health: f32,
}

#[derive(Component)]
struct Castle;

#[derive(Component)]
struct GameUI;

#[derive(Component)]
struct TowerWheelMenu;

#[derive(Component)]
struct TowerWheelOption {
    tower_type_id: String,
}

// === RESOURCES ===

#[derive(Resource)]
struct GameState {
    lives: i32,
    gold: i32,
    wave: i32,
    score: i32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            lives: 20,
            gold: 100,
            wave: 1,
            score: 0,
        }
    }
}
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    InGame,
    GameOver,
}

#[derive(Resource)]
struct EnemySpawner {
    timer: Timer,
    enemies_this_wave: i32,
    enemies_spawned: i32,
}

#[derive(Resource)]
struct PathWaypoints {
    points: Vec<Vec2>,
}

#[cfg(feature = "bevy-demo")]
#[derive(Resource, Clone)]
struct FileWatcher {
    receiver: Arc<Mutex<Receiver<notify::Result<Event>>>>,
    _watcher: Arc<Mutex<Box<dyn Watcher + Send>>>,
}

#[derive(Resource)]
struct WaveConfigs {
    units: Vec<UnitType>,
    waves: Vec<Wave>,
}

#[derive(Resource)]
struct TowerConfigs {
    towers: Vec<TowerType>,
}

#[derive(Resource)]
struct TowerWheelState {
    active: bool,
    position: Vec2,
}

impl EnemySpawner {
    fn from_wave_config(wave: &Wave) -> Self {
        let total_enemies: i32 = wave.spawns.iter().map(|s| s.count).sum();
        Self {
            timer: Timer::from_seconds(wave.spawn_interval, TimerMode::Repeating),
            enemies_this_wave: total_enemies,
            enemies_spawned: 0,
        }
    }
}

// === CONSTANTS ===

const TILE_SIZE: f32 = 16.0;
const MAP_SCALE: f32 = 2.0;
const SCALED_TILE_SIZE: f32 = TILE_SIZE * MAP_SCALE; // 32 pixels

// Asset dimensions
const TOWER_SIZE: Vec2 = Vec2::new(128.0, 256.0);
const WARRIOR_FRAME_SIZE: Vec2 = Vec2::new(192.0, 192.0); // Single frame from sprite sheet
const ARROW_SIZE: Vec2 = Vec2::new(64.0, 64.0);
const CASTLE_SIZE: Vec2 = Vec2::new(320.0, 256.0);

// Map dimensions
const MAP_WIDTH: i32 = 30;
const MAP_HEIGHT: i32 = 20;

// === PATH WAYPOINTS ===

fn create_path_waypoints() -> Vec<Vec2> {
    // Based on your tilemap, manually define the path waypoints
    // Starting from left, following the road tiles
    // Converting tile coordinates to world coordinates
    let waypoints = vec![
        (0, 10),  // Start left side, row 10
        (5, 10),  // Move right
        (5, 4),   // Turn up
        (11, 4),  // Move right
        (11, 16), // Move down
        (17, 16), // Move right
        (17, 4),  // Move up
        (23, 4),  // Move right
        (23, 11), // Move down
        (26, 11), // End at castle (right side)
    ];

    // Convert tile coordinates to world positions
    waypoints
        .iter()
        .map(|(x, y)| tile_to_world(*x, *y))
        .collect()
}

fn tile_to_world(tile_x: i32, tile_y: i32) -> Vec2 {
    // Convert tile coordinates to world space
    // Origin of tilemap is at -480, -320 with scale 2.0
    let world_x = -480.0 + (tile_x as f32 * SCALED_TILE_SIZE) + (SCALED_TILE_SIZE / 2.0);
    let world_y =
        -320.0 + ((MAP_HEIGHT - 1 - tile_y) as f32 * SCALED_TILE_SIZE) + (SCALED_TILE_SIZE / 2.0);
    Vec2::new(world_x, world_y)
}

// === SETUP ===

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    // Load and spawn the tilemap
    commands.spawn((
        TiledMap(asset_server.load("map.tmx")),
        Transform::from_xyz(-480.0, -320.0, 0.0).with_scale(Vec3::splat(MAP_SCALE)),
    ));

    // Spawn UI
    commands.spawn((
        Text::new("Lives: 20 | Gold: 100 | Wave: 1 | Left-click to open tower menu"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        GameUI,
    ));

    // Spawn castle at the end of the path (right side)
    let castle_scale = (SCALED_TILE_SIZE * 4.0) / CASTLE_SIZE.x.max(CASTLE_SIZE.y); // 4 tiles tall
    commands.spawn((
        Sprite::from_image(asset_server.load("Decorations/Buildings/Blue Buildings/Castle.png")),
        Transform::from_xyz(400.0, 0.0, 1.0).with_scale(Vec3::splat(castle_scale)),
        Castle,
    ));

}

fn spawn_tower(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    tower_type: &TowerType,
) {
    // Tower is 128x256, we want it to fit exactly 1 tile (32x32 when scaled)
    // Scale factor = desired_size / actual_size
    let scale_x = SCALED_TILE_SIZE / TOWER_SIZE.x; // 32 / 128 = 0.25
    let scale_y = SCALED_TILE_SIZE / TOWER_SIZE.y; // 32 / 256 = 0.125
    let scale = scale_x.min(scale_y); // Use smaller to fit within 1 tile

    commands.spawn((
        Sprite::from_image(asset_server.load(&tower_type.sprite_path)),
        Transform::from_translation(position).with_scale(Vec3::splat(scale)),
        Tower {
            tower_type_id: tower_type.id.clone(),
            range: tower_type.range,
            damage: tower_type.damage,
            fire_rate: tower_type.fire_rate,
            cooldown: 0.0,
            projectile_sprite: tower_type.projectile_sprite.clone(),
            projectile_speed: tower_type.projectile_speed,
        },
    ));
}

// === ENEMY SPAWNING ===

fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
    mut spawner: ResMut<EnemySpawner>,
    mut game_state: ResMut<GameState>,
    waypoints: Res<PathWaypoints>,
    wave_configs: Res<WaveConfigs>,
) {
    spawner.timer.tick(time.delta());

    if spawner.timer.just_finished() && spawner.enemies_spawned < spawner.enemies_this_wave {
        // Get current wave config
        let current_wave_idx = (game_state.wave - 1) as usize;
        if current_wave_idx >= wave_configs.waves.len() {
            // No more waves defined, use last wave with increased difficulty
            return;
        }

        let wave = &wave_configs.waves[current_wave_idx];

        // Calculate which spawn we're on
        let mut total_count = 0;
        let mut selected_spawn: Option<&UnitSpawn> = None;

        for spawn in &wave.spawns {
            total_count += spawn.count;
            if spawner.enemies_spawned < total_count {
                selected_spawn = Some(spawn);
                break;
            }
        }

        if let Some(spawn) = selected_spawn {
            // Find the unit type
            if let Some(unit_type) = wave_configs.units.iter().find(|u| u.id == spawn.unit_id) {
                // Spawn enemy at the start of the path
                let start_pos = waypoints.points.first().copied().unwrap_or(Vec2::ZERO);

                // Calculate health based on unit type and multiplier
                let max_health = unit_type.base_health * spawn.health_multiplier;

                // Create texture atlas layout for the sprite sheet
                // Assuming 6 frames in sprite sheets
                let layout = TextureAtlasLayout::from_grid(UVec2::splat(192), 6, 1, None, None);
                let texture_atlas_layout = texture_atlases.add(layout);

                // Scale to fit 1 tile
                let enemy_scale = SCALED_TILE_SIZE / WARRIOR_FRAME_SIZE.x;

                let enemy_entity = commands
                    .spawn((
                        Sprite::from_atlas_image(
                            asset_server.load(&unit_type.sprite_path),
                            TextureAtlas {
                                layout: texture_atlas_layout,
                                index: 0,
                            },
                        ),
                        Transform::from_xyz(start_pos.x, start_pos.y, 1.0)
                            .with_scale(Vec3::splat(enemy_scale)),
                        Enemy {
                            health: max_health,
                            speed: unit_type.base_speed,
                            current_waypoint: 0,
                            gold_reward: unit_type.gold_reward,
                            damage_to_base: unit_type.damage_to_base,
                        },
                        AnimationTimer {
                            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                        },
                    ))
                    .id();

                // Spawn health bar above enemy as child
                let health_bar = commands
                    .spawn((
                        Sprite {
                            color: Color::srgb(0.0, 1.0, 0.0), // Green
                            custom_size: Some(Vec2::new(SCALED_TILE_SIZE, 4.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, SCALED_TILE_SIZE * 0.6, 0.1),
                        HealthBar { max_health },
                    ))
                    .id();

                commands.entity(enemy_entity).add_child(health_bar);

                spawner.enemies_spawned += 1;

                // Check if wave is complete
                if spawner.enemies_spawned >= total_count {
                    spawner.enemies_spawned = 0;
                    game_state.wave += 1;

                    // Update spawn interval for next wave if available
                    if (game_state.wave - 1) < wave_configs.waves.len() as i32 {
                        let next_wave = &wave_configs.waves[(game_state.wave - 1) as usize];
                        spawner.timer =
                            Timer::from_seconds(next_wave.spawn_interval, TimerMode::Repeating);

                        // Calculate total enemies for next wave
                        spawner.enemies_this_wave = next_wave.spawns.iter().map(|s| s.count).sum();
                    }
                }
            }
        }
    }
}

// === ENEMY MOVEMENT ===

fn move_enemies(
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy, Option<&Children>)>,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    waypoints: Res<PathWaypoints>,
) {
    for (entity, mut transform, mut enemy, children) in enemies.iter_mut() {
        // Get current and next waypoint
        if enemy.current_waypoint >= waypoints.points.len() {
            // Reached the end (castle) - despawn enemy and deal damage
            if let Some(children) = children {
                for child in children.iter() {
                    commands.queue_silenced(move |world: &mut World| {
                        if let Ok(entity_mut) = world.get_entity_mut(child) {
                            entity_mut.despawn();
                        }
                    });
                }
            }
            let entity_to_despawn = entity;
            commands.queue_silenced(move |world: &mut World| {
                if let Ok(entity_mut) = world.get_entity_mut(entity_to_despawn) {
                    entity_mut.despawn();
                }
            });
            game_state.lives -= enemy.damage_to_base;
            continue;
        }

        let target = waypoints.points[enemy.current_waypoint];
        let current_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let direction = (target - current_pos).normalize_or_zero();

        // Move towards current waypoint
        let movement = direction * enemy.speed * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        // Check if reached current waypoint
        let distance_to_waypoint = current_pos.distance(target);
        if distance_to_waypoint < 5.0 {
            // Move to next waypoint
            enemy.current_waypoint += 1;
        }
    }
}

// === SPRITE ANIMATION ===

fn animate_sprites(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % 6; // Cycle through 6 frames
            }
        }
    }
}

// === TOWER WHEEL MENU ===

fn show_tower_wheel_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut wheel_state: ResMut<TowerWheelState>,
    tower_configs: Res<TowerConfigs>,
    existing_menus: Query<Entity, With<TowerWheelMenu>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) && !wheel_state.active {
        let Ok(window) = windows.single() else { return };
        let Ok((camera, camera_transform)) = camera.single() else {
            return;
        };

        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                // Clean up any existing menus
                for entity in existing_menus.iter() {
                    commands.queue_silenced(move |world: &mut World| {
                        if let Ok(entity_mut) = world.get_entity_mut(entity) {
                            entity_mut.despawn();
                        }
                    });
                }

                // Store the world position where we want to place the tower
                wheel_state.active = true;
                wheel_state.position = world_pos;

                // Create wheel menu
                let num_towers = tower_configs.towers.len();
                let radius = 80.0; // Distance from center to each option

                for (i, tower_type) in tower_configs.towers.iter().enumerate() {
                    let angle = (i as f32 / num_towers as f32) * std::f32::consts::TAU;
                    let offset_x = angle.cos() * radius;
                    let offset_y = angle.sin() * radius;

                    // Create background circle
                    let circle_entity = commands
                        .spawn((
                            Sprite {
                                color: Color::srgba(0.2, 0.2, 0.8, 0.7),
                                custom_size: Some(Vec2::splat(60.0)),
                                ..default()
                            },
                            Transform::from_xyz(world_pos.x + offset_x, world_pos.y + offset_y, 10.0),
                            TowerWheelMenu,
                            TowerWheelOption {
                                tower_type_id: tower_type.id.clone(),
                            },
                        ))
                        .id();

                    // Add tower sprite on top
                    let scale = 40.0 / TOWER_SIZE.x.max(TOWER_SIZE.y);
                    let sprite_entity = commands
                        .spawn((
                            Sprite::from_image(asset_server.load(&tower_type.sprite_path)),
                            Transform::from_xyz(0.0, 0.0, 0.1).with_scale(Vec3::splat(scale)),
                        ))
                        .id();

                    commands.entity(circle_entity).add_child(sprite_entity);

                    // Add tower name below
                    let name_entity = commands
                        .spawn((
                            Text2d::new(&tower_type.name),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                            Transform::from_xyz(0.0, -40.0, 0.1),
                            TowerWheelMenu,
                        ))
                        .id();
                    commands.entity(circle_entity).add_child(name_entity);

                    // Add cost label above sprite
                    let cost_entity = commands
                        .spawn((
                            Text2d::new(format!("{}g", tower_type.cost)),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 0.0)),
                            Transform::from_xyz(0.0, 35.0, 0.1),
                            TowerWheelMenu,
                        ))
                        .id();
                    commands.entity(circle_entity).add_child(cost_entity);
                }

                // Add center indicator
                commands.spawn((
                    Sprite {
                        color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                    TowerWheelMenu,
                ));
            }
        }
    }
}

fn hide_tower_wheel_menu(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut wheel_state: ResMut<TowerWheelState>,
    menu_entities: Query<Entity, With<TowerWheelMenu>>,
) {
    if (mouse_button.just_pressed(MouseButton::Right) || mouse_button.just_pressed(MouseButton::Middle))
        && wheel_state.active
    {
        // Clean up menu
        for entity in menu_entities.iter() {
            commands.queue_silenced(move |world: &mut World| {
                if let Ok(entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.despawn();
                }
            });
        }
        wheel_state.active = false;
    }
}

fn handle_tower_selection(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut wheel_state: ResMut<TowerWheelState>,
    mut game_state: ResMut<GameState>,
    tower_configs: Res<TowerConfigs>,
    menu_options: Query<(&Transform, &TowerWheelOption), With<TowerWheelMenu>>,
    menu_entities: Query<Entity, With<TowerWheelMenu>>,
) {
    if mouse_button.just_released(MouseButton::Left) && wheel_state.active {
        let Ok(window) = windows.single() else { return };
        let Ok((camera, camera_transform)) = camera.single() else {
            return;
        };

        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(mouse_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                // Check which option is closest to the mouse
                let mut closest_option: Option<(&TowerWheelOption, f32)> = None;

                for (transform, option) in menu_options.iter() {
                    let distance = transform.translation.truncate().distance(mouse_world_pos);
                    if distance < 40.0 {
                        // Within click range
                        if let Some((_, closest_dist)) = closest_option {
                            if distance < closest_dist {
                                closest_option = Some((option, distance));
                            }
                        } else {
                            closest_option = Some((option, distance));
                        }
                    }
                }

                // If an option was selected, try to place that tower
                if let Some((option, _)) = closest_option {
                    if let Some(tower_type) = tower_configs
                        .towers
                        .iter()
                        .find(|t| t.id == option.tower_type_id)
                    {
                        if game_state.gold >= tower_type.cost {
                            // Snap to tile grid
                            let snapped_x =
                                (wheel_state.position.x / SCALED_TILE_SIZE).round() * SCALED_TILE_SIZE;
                            let snapped_y =
                                (wheel_state.position.y / SCALED_TILE_SIZE).round() * SCALED_TILE_SIZE;
                            let snapped_pos = Vec3::new(snapped_x, snapped_y, 1.0);

                            spawn_tower(&mut commands, &asset_server, snapped_pos, tower_type);
                            game_state.gold -= tower_type.cost;
                        }
                    }
                }

                // Clean up menu
                for entity in menu_entities.iter() {
                    commands.queue_silenced(move |world: &mut World| {
                        if let Ok(entity_mut) = world.get_entity_mut(entity) {
                            entity_mut.despawn();
                        }
                    });
                }
                wheel_state.active = false;
            }
        }
    }
}

// === TOWER SHOOTING ===

fn tower_shooting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut towers: Query<(&Transform, &mut Tower)>,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<Tower>)>,
    time: Res<Time>,
) {
    for (tower_transform, mut tower) in towers.iter_mut() {
        tower.cooldown -= time.delta_secs();

        if tower.cooldown <= 0.0 {
            // Find closest enemy in range
            let mut closest_enemy: Option<(Entity, f32)> = None;

            for (enemy_entity, enemy_transform) in enemies.iter() {
                let distance = tower_transform
                    .translation
                    .distance(enemy_transform.translation);

                if distance <= tower.range {
                    if let Some((_, closest_dist)) = closest_enemy {
                        if distance < closest_dist {
                            closest_enemy = Some((enemy_entity, distance));
                        }
                    } else {
                        closest_enemy = Some((enemy_entity, distance));
                    }
                }
            }

            // Shoot at closest enemy
            if let Some((target_entity, _)) = closest_enemy {
                // Scale projectile to be about half a tile
                let projectile_scale = (SCALED_TILE_SIZE * 0.5) / ARROW_SIZE.x;

                commands.spawn((
                    Sprite::from_image(asset_server.load(&tower.projectile_sprite)),
                    Transform::from_translation(tower_transform.translation)
                        .with_scale(Vec3::splat(projectile_scale)),
                    Projectile {
                        damage: tower.damage,
                        speed: tower.projectile_speed,
                        target: target_entity,
                    },
                ));

                tower.cooldown = tower.fire_rate;
            }
        }
    }
}

// === PROJECTILE MOVEMENT ===

fn move_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &Projectile)>,
    enemies: Query<&Transform, (With<Enemy>, Without<Projectile>)>,
    time: Res<Time>,
) {
    for (projectile_entity, mut projectile_transform, projectile) in projectiles.iter_mut() {
        // Get target position
        if let Ok(enemy_transform) = enemies.get(projectile.target) {
            let direction =
                (enemy_transform.translation - projectile_transform.translation).normalize();
            projectile_transform.translation += direction * projectile.speed * time.delta_secs();

            // Rotate projectile to face target
            let angle = direction.y.atan2(direction.x);
            projectile_transform.rotation = Quat::from_rotation_z(angle);

            // Check if hit
            let distance = projectile_transform
                .translation
                .distance(enemy_transform.translation);
            if distance < 10.0 {
                commands.queue_silenced(move |world: &mut World| {
                    if let Ok(entity_mut) = world.get_entity_mut(projectile_entity) {
                        entity_mut.despawn();
                    }
                });
            }
        } else {
            // Target died, remove projectile
            commands.queue_silenced(move |world: &mut World| {
                if let Ok(entity_mut) = world.get_entity_mut(projectile_entity) {
                    entity_mut.despawn();
                }
            });
        }
    }
}

// === PROJECTILE HITS ===

fn handle_projectile_hits(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &Projectile)>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy, Option<&Children>)>,
    mut game_state: ResMut<GameState>,
) {
    for (projectile_entity, projectile_transform, projectile) in projectiles.iter() {
        if let Ok((enemy_entity, enemy_transform, mut enemy, children)) =
            enemies.get_mut(projectile.target)
        {
            let distance = projectile_transform
                .translation
                .distance(enemy_transform.translation);

            if distance < 10.0 {
                enemy.health -= projectile.damage;
                commands.queue_silenced(move |world: &mut World| {
                    if let Ok(entity_mut) = world.get_entity_mut(projectile_entity) {
                        entity_mut.despawn();
                    }
                });

                // Enemy died
                if enemy.health <= 0.0 {
                    // Despawn children (health bar) first
                    if let Some(children) = children {
                        for child in children.iter() {
                            commands.queue_silenced(move |world: &mut World| {
                                if let Ok(entity_mut) = world.get_entity_mut(child) {
                                    entity_mut.despawn();
                                }
                            });
                        }
                    }
                    let entity_to_despawn = enemy_entity;
                    commands.queue_silenced(move |world: &mut World| {
                        if let Ok(entity_mut) = world.get_entity_mut(entity_to_despawn) {
                            entity_mut.despawn();
                        }
                    });
                    game_state.gold += enemy.gold_reward;
                    game_state.score += enemy.gold_reward;
                }
            }
        }
    }
}

// === HEALTH BAR UPDATE ===

fn update_health_bars(
    enemies: Query<(&Enemy, &Children)>,
    mut health_bars: Query<(&mut Sprite, &HealthBar)>,
) {
    for (enemy, children) in enemies.iter() {
        for child in children.iter() {
            if let Ok((mut sprite, health_bar)) = health_bars.get_mut(child) {
                // Update health bar width based on current health
                let health_percent = (enemy.health / health_bar.max_health).clamp(0.0, 1.0);
                sprite.custom_size = Some(Vec2::new(SCALED_TILE_SIZE * health_percent, 4.0));

                // Update color based on health percentage
                sprite.color = if health_percent > 0.6 {
                    Color::srgb(0.0, 1.0, 0.0) // Green
                } else if health_percent > 0.3 {
                    Color::srgb(1.0, 0.6, 0.0) // Orange
                } else {
                    Color::srgb(1.0, 0.0, 0.0) // Red
                };
            }
        }
    }
}

// === CLEANUP ===

fn cleanup_dead_enemies(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy, Option<&Children>)>,
) {
    for (entity, enemy, children) in enemies.iter() {
        if enemy.health <= 0.0 {
            // Despawn children (health bar) first
            if let Some(children) = children {
                for child in children.iter() {
                    commands.queue_silenced(move |world: &mut World| {
                        if let Ok(entity_mut) = world.get_entity_mut(child) {
                            entity_mut.despawn();
                        }
                    });
                }
            }
            commands.queue_silenced(move |world: &mut World| {
                if let Ok(entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.despawn();
                }
            });
        }
    }
}

fn cleanup_game_over_screen(
    mut commands: Commands,
    query: Query<Entity, With<GameOverScreen>>,
) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

#[derive(Component)]
struct GameOverScreen;

fn setup_game_over_screen(mut commands: Commands) {
    commands.spawn((
        Node {
            // Stretch over whole screen
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.7)),
        GameOverScreen,
    )).with_children(|parent| {

        // GAME OVER text
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font_size: 80.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

// === UI UPDATE ===

fn update_ui(game_state: Res<GameState>, mut ui_query: Query<&mut Text, With<GameUI>>) {
    for mut text in ui_query.iter_mut() {
        **text = format!(
            "Lives: {} | Gold: {} | Wave: {} | Score: {}",
            game_state.lives, game_state.gold, game_state.wave, game_state.score
        );
    }
}

// === GAME OVER ===

fn check_game_over(
    game_state: Res<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if game_state.lives <= 0 {
        next_state.set(AppState::GameOver);
    }
}

// === CAMERA ZOOM ===

fn camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    for event in scroll_events.read() {
        for mut transform in query.iter_mut() {
            // Zoom in/out based on scroll direction
            let zoom_delta = event.y * 0.1;

            // Update camera scale (larger = zoomed in, smaller = zoomed out)
            let new_scale = (transform.scale.x + zoom_delta).clamp(0.3, 3.0);
            transform.scale = Vec3::splat(new_scale);
        }
    }
}

// === CONFIG FILE WATCHING ===

#[cfg(feature = "bevy-demo")]
fn watch_config_files(
    file_watcher: Res<FileWatcher>,
    mut wave_configs: ResMut<WaveConfigs>,
    mut tower_configs: ResMut<TowerConfigs>,
) {
    // Check for file change events (non-blocking)
    let receiver = file_watcher.receiver.lock().unwrap();
    while let Ok(Ok(event)) = receiver.try_recv() {
        use notify::EventKind;

        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                // Check which file was modified
                for path in event.paths {
                    if path.ends_with("units.toml") {
                        match UnitsConfig::load() {
                            Ok(config) => {
                                wave_configs.units = config.units;
                                info!(
                                    "Hot-reloaded units.toml - {} units loaded",
                                    wave_configs.units.len()
                                );
                            }
                            Err(e) => error!("Failed to reload units.toml: {}", e),
                        }
                    } else if path.ends_with("waves.toml") {
                        match WavesConfig::load() {
                            Ok(config) => {
                                wave_configs.waves = config.waves;
                                info!(
                                    "Hot-reloaded waves.toml - {} waves loaded",
                                    wave_configs.waves.len()
                                );
                            }
                            Err(e) => error!("Failed to reload waves.toml: {}", e),
                        }
                    } else if path.ends_with("towers.toml") {
                        match TowersConfig::load() {
                            Ok(config) => {
                                tower_configs.towers = config.towers;
                                info!(
                                    "Hot-reloaded towers.toml - {} towers loaded",
                                    tower_configs.towers.len()
                                );
                            }
                            Err(e) => error!("Failed to reload towers.toml: {}", e),
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// === CAMERA PAN ===

fn camera_pan(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut motion_events: MessageReader<CursorMoved>,
    mut query: Query<&mut Transform, With<Camera2d>>,
    mut last_pos: Local<Option<Vec2>>,
) {
    // Check if right mouse button or middle mouse button is pressed
    let is_dragging =
        mouse_button.pressed(MouseButton::Right) || mouse_button.pressed(MouseButton::Middle);

    if is_dragging {
        for event in motion_events.read() {
            if let Some(last) = *last_pos {
                for mut transform in query.iter_mut() {
                    // Calculate delta movement
                    let delta = event.position - last;

                    // Move camera in opposite direction (inverted controls feel more natural)
                    // Scale movement by camera scale so panning speed feels consistent
                    transform.translation.x -= delta.x * transform.scale.x;
                    transform.translation.y += delta.y * transform.scale.y; // Y is inverted in screen space
                }
            }
            *last_pos = Some(event.position);
        }
    } else {
        // Reset last position when not dragging
        if mouse_button.just_released(MouseButton::Right)
            || mouse_button.just_released(MouseButton::Middle)
        {
            *last_pos = None;
        }
        // Update last position even when not dragging to prevent jumps
        for event in motion_events.read() {
            *last_pos = Some(event.position);
        }
    }
}
