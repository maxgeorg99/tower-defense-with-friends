use bevy::prelude::*;
use bevy_spacetimedb::StdbConnection;
use spacetimedb_sdk::Table;
use crate::components::{Enemy, Tower, TowerWheelMenu, TowerWheelOption, Projectile};
use crate::config::TowerType;
use crate::constants::{ARROW_SIZE, EXPLORE_COST, EXPLORE_RADIUS, SCALED_TILE_SIZE, TOWER_SIZE};
use crate::map::world_to_tile;
use crate::module_bindings;
use crate::module_bindings::{DbConnection, MyUserTableAccess, UserTableAccess};
use crate::resources::{BlockedTiles, FogOfWar, GameState, RecruitMenuState, TowerConfigs, TowerWheelState};

//TODO Display for generated Types?!
impl module_bindings::Color {
    fn as_str(&self) -> &str {
        match self {
            module_bindings::Color::Blue => "Blue",
            module_bindings::Color::Yellow => "Yellow",
            module_bindings::Color::Purple => "Purple",
            module_bindings::Color::Black => "Black",
        }
    }
}
pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

pub fn spawn_tower(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    tower_type: &TowerType,
    stdb: Option<SpacetimeDB>,
) {
    // Tower is 128x256, we want it to fit exactly 1 tile (32x32 when scaled)
    // Scale factor = desired_size / actual_size
    let scale_x = SCALED_TILE_SIZE / TOWER_SIZE.x; // 32 / 128 = 0.25
    let scale_y = SCALED_TILE_SIZE / TOWER_SIZE.y; // 32 / 256 = 0.125
    let scale = scale_x.min(scale_y); // Use smaller to fit within 1 tile

    let path = get_tower_sprite_path(tower_type, stdb.as_ref());
    commands.spawn((
        Sprite::from_image(asset_server.load(path)),
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

pub fn show_tower_wheel_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut wheel_state: ResMut<TowerWheelState>,
    tower_configs: Res<TowerConfigs>,
    fog: Res<FogOfWar>,
    recruit_menu_state: Res<RecruitMenuState>,
    blocked_tiles: Res<BlockedTiles>,
    existing_menus: Query<Entity, With<TowerWheelMenu>>,
    stdb: Option<SpacetimeDB>,
) {
    // Don't show if recruit menu is active
    if mouse_button.just_pressed(MouseButton::Left) && !wheel_state.active && !recruit_menu_state.active {
        let Ok(window) = windows.single() else { return };
        let Ok((camera, camera_transform)) = camera.single() else {
            return;
        };

        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                let (tile_x, tile_y) = world_to_tile(world_pos);

                // Don't show tower wheel on castle (recruit menu handles that)
                if blocked_tiles.is_castle(tile_x, tile_y) {
                    return;
                }

                // Don't show tower wheel on road tiles
                if blocked_tiles.is_road(tile_x, tile_y) {
                    return;
                }

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

                // Check if clicked tile is in fog
                let is_in_fog = !fog.is_explored(tile_x, tile_y);

                if is_in_fog {
                    // Show explore option only
                    let circle_entity = commands
                        .spawn((
                            Sprite {
                                color: Color::srgba(0.1, 0.5, 0.1, 0.8),
                                custom_size: Some(Vec2::splat(70.0)),
                                ..default()
                            },
                            Transform::from_xyz(world_pos.x, world_pos.y - 60.0, 10.0),
                            TowerWheelMenu,
                            TowerWheelOption {
                                tower_type_id: "_explore".to_string(),
                            },
                        ))
                        .id();

                    // Add explore text
                    let name_entity = commands
                        .spawn((
                            Text2d::new("Explore"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                            Transform::from_xyz(0.0, 0.0, 0.1),
                        ))
                        .id();
                    commands.entity(circle_entity).add_child(name_entity);

                    // Add cost label
                    let cost_entity = commands
                        .spawn((
                            Text2d::new(format!("{}g", EXPLORE_COST)),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 0.0)),
                            Transform::from_xyz(0.0, -20.0, 0.1),
                        ))
                        .id();
                    commands.entity(circle_entity).add_child(cost_entity);
                } else {
                    // Show tower options (existing code)
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
                                Transform::from_xyz(
                                    world_pos.x + offset_x,
                                    world_pos.y + offset_y,
                                    10.0,
                                ),
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
                                Sprite::from_image(asset_server.load(get_tower_sprite_path(tower_type, stdb.as_ref()))),
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

pub fn hide_tower_wheel_menu(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut wheel_state: ResMut<TowerWheelState>,
    menu_entities: Query<Entity, With<TowerWheelMenu>>,
) {
    if (mouse_button.just_pressed(MouseButton::Right)
        || mouse_button.just_pressed(MouseButton::Middle))
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

pub fn handle_tower_selection(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut wheel_state: ResMut<TowerWheelState>,
    mut game_state: ResMut<GameState>,
    tower_configs: Res<TowerConfigs>,
    mut fog: ResMut<FogOfWar>,
    menu_options: Query<(&Transform, &TowerWheelOption), With<TowerWheelMenu>>,
    menu_entities: Query<Entity, With<TowerWheelMenu>>,
    stdb: Option<SpacetimeDB>,
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

                // If an option was selected
                if let Some((option, _)) = closest_option {
                    // Handle explore option
                    if option.tower_type_id == "_explore" {
                        if game_state.gold >= EXPLORE_COST {
                            let (tile_x, tile_y) = world_to_tile(wheel_state.position);
                            fog.explore_rect(tile_x, tile_y, EXPLORE_RADIUS);
                            game_state.gold -= EXPLORE_COST;
                        }
                    } else if let Some(tower_type) = tower_configs
                        .towers
                        .iter()
                        .find(|t| t.id == option.tower_type_id)
                    {
                        // Snap to tile grid
                        let snapped_x =
                            (wheel_state.position.x / SCALED_TILE_SIZE).round() * SCALED_TILE_SIZE;
                        let snapped_y =
                            (wheel_state.position.y / SCALED_TILE_SIZE).round() * SCALED_TILE_SIZE;
                        let snapped_pos = Vec3::new(snapped_x, snapped_y, 1.0);

                        // Check if tile is explored (not in fog)
                        let (tile_x, tile_y) = world_to_tile(Vec2::new(snapped_x, snapped_y));
                        let is_explored = fog.is_explored(tile_x, tile_y);

                        if game_state.gold >= tower_type.cost && is_explored {
                            spawn_tower(&mut commands, &asset_server, snapped_pos, tower_type, stdb);
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

pub fn tower_shooting(
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

fn get_user_color(stdb: Option<&SpacetimeDB>) -> module_bindings::Color {
    stdb
        .and_then(|stdb| stdb.db().user().iter().next().map(|user| user.color))
        .unwrap_or(module_bindings::Color::Blue)
}

fn get_tower_sprite_path(tower_type: &TowerType, stdb: Option<&SpacetimeDB>) -> String {
    let color = get_user_color(stdb);
    tower_type.sprite_path.replace("Blue", color.as_str())
}
