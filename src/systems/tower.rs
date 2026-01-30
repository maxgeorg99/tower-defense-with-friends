use bevy::prelude::*;
use bevy::ecs::prelude::ChildSpawnerCommands;
use bevy_spacetimedb::StdbConnection;
use spacetimedb_sdk::Table;
use crate::components::{get_attack_type_icon, get_damage_multiplier, AttackType, AnimationTimer, Enemy, HolyTowerEffect, Tower, TowerLevel, TowerUpgradeMenu, TowerUpgradeOption, TowerWheelMenu, TowerWheelOption, Projectile, UpgradeType, WorkerBuilding};
use crate::systems::{AnimationInfo, SoundEffect};
use crate::config::TowerType;
use crate::constants::{ARROW_SIZE, EXPLORE_COST, EXPLORE_RADIUS, SCALED_TILE_SIZE, TOWER_SIZE};
use crate::map::world_to_tile;
use crate::module_bindings;
use crate::module_bindings::{
    DbConnection, MyUserTableAccess, UserTableAccess,
    place_tower, upgrade_tower_damage, upgrade_tower_range, upgrade_tower_fire_rate
};
use crate::systems::entity_sync::EntityMap;
use crate::resources::{BlockedTiles, FogOfWar, GameState, HouseMenuState, RecruitMenuState, TowerConfigs, TowerUpgradeMenuState, TowerWheelState};

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
            attack_type: AttackType::from_str(&tower_type.attack_type),
        },
        TowerLevel::default(),
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
    house_menu_state: Res<HouseMenuState>,
    upgrade_menu_state: Res<TowerUpgradeMenuState>,
    blocked_tiles: Res<BlockedTiles>,
    existing_menus: Query<Entity, With<TowerWheelMenu>>,
    existing_towers: Query<&Transform, With<Tower>>,
    worker_buildings: Query<&Transform, With<WorkerBuilding>>,
    stdb: Option<SpacetimeDB>,
) {
    // Don't show if any menu is active
    if mouse_button.just_pressed(MouseButton::Left)
        && !wheel_state.active
        && !recruit_menu_state.active
        && !house_menu_state.active
        && !upgrade_menu_state.active
    {
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

                // Don't show tower wheel if clicking on existing tower (upgrade menu handles that)
                for tower_transform in existing_towers.iter() {
                    let tower_pos = tower_transform.translation.truncate();
                    if world_pos.distance(tower_pos) < SCALED_TILE_SIZE / 2.0 {
                        return;
                    }
                }

                // Don't show tower wheel if clicking on worker building (house menu handles that)
                for building_transform in worker_buildings.iter() {
                    let building_pos = building_transform.translation.truncate();
                    if world_pos.distance(building_pos) < SCALED_TILE_SIZE / 2.0 {
                        return;
                    }
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

                // Load paper background texture
                let paper_texture = asset_server.load("UI Elements/UI Elements/Papers/SpecialPaper.png");

                if is_in_fog {
                    // Show explore option only
                    let circle_entity = commands
                        .spawn((
                            Sprite {
                                image: paper_texture.clone(),
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

                        // Create background with paper texture
                        let circle_entity = commands
                            .spawn((
                                Sprite {
                                    image: paper_texture.clone(),
                                    custom_size: Some(Vec2::splat(70.0)),
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

                        // Add damage type badge (similar to wave_manager_ui defense type badge)
                        let attack_type = AttackType::from_str(&tower_type.attack_type);
                        let badge_entity = commands
                            .spawn((
                                Sprite {
                                    image: asset_server.load(get_attack_type_icon(attack_type)),
                                    custom_size: Some(Vec2::splat(18.0)),
                                    ..default()
                                },
                                // Position at bottom-right corner, slightly outside
                                Transform::from_xyz(22.0, -22.0, 0.2),
                            ))
                            .id();

                        commands.entity(circle_entity).add_child(badge_entity);

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

                        // Check if tile is explored (not in fog)
                        let (tile_x, tile_y) = world_to_tile(Vec2::new(snapped_x, snapped_y));
                        let is_explored = fog.is_explored(tile_x, tile_y);

                        // Only attempt placement if tile is explored and we might have gold
                        // Server will validate gold and handle deduction
                        if game_state.gold >= tower_type.cost && is_explored {
                            if let Some(ref stdb) = stdb {
                                // Call the server reducer to place tower
                                // Server will validate gold, deduct it, and create entities
                                // Entity sync will spawn the visual representation
                                if let Err(e) = stdb.conn().reducers.place_tower(
                                    tower_type.id.clone(),
                                    snapped_x,
                                    snapped_y,
                                ) {
                                    error!("Failed to place tower: {:?}", e);
                                } else {
                                    info!("Tower placement requested: {} at ({}, {})",
                                        tower_type.id, snapped_x, snapped_y);
                                }
                            } else {
                                // Fallback for offline/non-networked mode - spawn locally
                                let snapped_pos = Vec3::new(snapped_x, snapped_y, 1.0);
                                spawn_tower(&mut commands, &asset_server, snapped_pos, tower_type, None);
                                game_state.gold -= tower_type.cost;
                            }
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

// Holy tower heal effect constants
const HEAL_FRAME_SIZE: UVec2 = UVec2::new(192, 192);
const HEAL_FRAME_COUNT: usize = 11;
const HEAL_ANIMATION_DURATION: f32 = 1.1; // 11 frames at 0.1s each

pub fn tower_shooting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut towers: Query<(&Transform, &mut Tower)>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy), Without<Tower>>,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    stdb: Option<SpacetimeDB>,
    mut sound_events: EventWriter<SoundEffect>,
) {
    for (tower_transform, mut tower) in towers.iter_mut() {
        tower.cooldown -= time.delta_secs();

        if tower.cooldown <= 0.0 {
            // Find closest enemy in range
            let mut closest_enemy: Option<(Entity, f32)> = None;

            for (enemy_entity, enemy_transform, _) in enemies.iter() {
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

            // Handle attack based on tower type
            if let Some((target_entity, _)) = closest_enemy {
                // Holy tower: instant damage with holy effect on enemy
                if tower.tower_type_id == "holy" {
                    // Deal instant damage to target
                    if let Ok((_enemy_entity, enemy_transform, mut enemy)) = enemies.get_mut(target_entity) {
                        let multiplier = get_damage_multiplier(tower.attack_type, enemy.defense_type);
                        let final_damage = tower.damage * multiplier;
                        enemy.health -= final_damage;

                        // Spawn holy effect at enemy position
                        spawn_holy_tower_effect(
                            &mut commands,
                            &asset_server,
                            &mut texture_atlases,
                            enemy_transform.translation,
                        );

                        // Check if enemy died from instant damage
                        if enemy.health <= 0.0 {
                            game_state.gold += enemy.gold_reward;
                            game_state.score += enemy.gold_reward;
                        }
                    }
                } else {
                    // Regular towers: spawn projectile
                    let projectile_scale = (SCALED_TILE_SIZE * 0.5) / ARROW_SIZE.x;

                    commands.spawn((
                        Sprite::from_image(asset_server.load(&tower.projectile_sprite)),
                        Transform::from_translation(tower_transform.translation)
                            .with_scale(Vec3::splat(projectile_scale)),
                        Projectile {
                            damage: tower.damage,
                            speed: tower.projectile_speed,
                            target: target_entity,
                            attack_type: tower.attack_type,
                        },
                    ));
                    sound_events.write(SoundEffect::ArrowShoot);
                }

                tower.cooldown = tower.fire_rate;
            }
        }
    }
}

/// Spawn holy smite effect on the enemy (gold-tinted heal effect)
fn spawn_holy_tower_effect(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    enemy_position: Vec3,
) {
    // Use Yellow monk's heal effect as base (will be tinted gold)
    let heal_effect_path = "Units/Yellow Units/Monk/Heal_Effect.png";

    // Create texture atlas layout for heal animation
    let layout = TextureAtlasLayout::from_grid(HEAL_FRAME_SIZE, HEAL_FRAME_COUNT as u32, 1, None, None);
    let effect_atlas_layout = texture_atlases.add(layout);

    let effect_scale = SCALED_TILE_SIZE * 1.5 / HEAL_FRAME_SIZE.x as f32;

    // Gold/holy tint color
    let holy_gold_tint = Color::srgb(1.0, 0.85, 0.3);

    // Spawn holy effect on enemy (gold-tinted aura)
    commands.spawn((
        Sprite {
            image: asset_server.load(heal_effect_path),
            color: holy_gold_tint,
            texture_atlas: Some(TextureAtlas {
                layout: effect_atlas_layout,
                index: 0,
            }),
            ..default()
        },
        Transform::from_translation(enemy_position + Vec3::new(0.0, 0.0, 5.0))
            .with_scale(Vec3::splat(effect_scale)),
        AnimationTimer {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        },
        AnimationInfo {
            frame_count: HEAL_FRAME_COUNT,
        },
        HolyTowerEffect {
            lifetime: Timer::from_seconds(HEAL_ANIMATION_DURATION, TimerMode::Once),
        },
    ));
}

/// Update and cleanup holy tower visual effects
pub fn update_holy_tower_effects(
    mut commands: Commands,
    mut effects: Query<(Entity, &mut HolyTowerEffect)>,
    time: Res<Time>,
) {
    for (entity, mut effect) in effects.iter_mut() {
        effect.lifetime.tick(time.delta());

        if effect.lifetime.finished() {
            commands.queue_silenced(move |world: &mut World| {
                if let Ok(entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.despawn();
                }
            });
        }
    }
}

fn get_user_color(stdb: Option<&SpacetimeDB>) -> module_bindings::Color {
    stdb
        .and_then(|stdb| stdb.db().my_user().iter().next().map(|user| user.color))
        .unwrap_or(module_bindings::Color::Blue)
}

fn get_tower_sprite_path(tower_type: &TowerType, stdb: Option<&SpacetimeDB>) -> String {
    let color = get_user_color(stdb);
    tower_type.sprite_path.replace("Blue", color.as_str())
}

// ==================== Tower Upgrade Menu Systems ====================

const UPGRADE_DAMAGE_COST: i32 = 30;
const UPGRADE_RANGE_COST: i32 = 25;
const UPGRADE_FIRE_RATE_COST: i32 = 35;

/// Show upgrade menu when clicking on an existing tower
pub fn show_tower_upgrade_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut upgrade_menu_state: ResMut<TowerUpgradeMenuState>,
    wheel_state: Res<TowerWheelState>,
    recruit_menu_state: Res<RecruitMenuState>,
    house_menu_state: Res<HouseMenuState>,
    towers: Query<(Entity, &Transform, &Tower, &TowerLevel)>,
    existing_menus: Query<Entity, With<TowerUpgradeMenu>>,
) {
    // Don't show if any other menu is active
    if !mouse_button.just_pressed(MouseButton::Left)
        || upgrade_menu_state.active
        || wheel_state.active
        || recruit_menu_state.active
        || house_menu_state.active
    {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    // Check if clicked on a tower
    for (tower_entity, tower_transform, tower, tower_level) in towers.iter() {
        let tower_pos = tower_transform.translation.truncate();
        if world_pos.distance(tower_pos) < SCALED_TILE_SIZE / 2.0 {
            // Clean up existing menus
            for entity in existing_menus.iter() {
                commands.entity(entity).despawn();
            }

            upgrade_menu_state.active = true;
            upgrade_menu_state.selected_tower = Some(tower_entity);
            spawn_tower_upgrade_menu(&mut commands, &asset_server, tower, tower_level);
            return;
        }
    }
}

fn spawn_tower_upgrade_menu(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    tower: &Tower,
    tower_level: &TowerLevel,
) {
    let wood_icon = asset_server.load("Terrain/Resources/Wood/Wood Resource/Wood Resource.png");

    // Main menu container (centered overlay)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            TowerUpgradeMenu,
        ))
        .with_children(|parent| {
            // Menu background panel
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(15.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.15, 0.2, 0.95)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new(format!("Upgrade {}", tower.tower_type_id.to_uppercase())),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Current stats display
                    panel.spawn((
                        Text::new(format!(
                            "DMG: {:.0}  RNG: {:.0}  SPD: {:.1}s",
                            tower.damage, tower.range, tower.fire_rate
                        )),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    ));

                    // Upgrade options container
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(15.0),
                            ..default()
                        })
                        .with_children(|cards_row| {
                            // Damage upgrade
                            spawn_upgrade_card(
                                cards_row,
                                &wood_icon,
                                "DAMAGE",
                                "+25%",
                                format!("Lv.{}", tower_level.damage_level),
                                UpgradeType::Damage,
                                UPGRADE_DAMAGE_COST,
                            );

                            // Range upgrade
                            spawn_upgrade_card(
                                cards_row,
                                &wood_icon,
                                "RANGE",
                                "+20%",
                                format!("Lv.{}", tower_level.range_level),
                                UpgradeType::Range,
                                UPGRADE_RANGE_COST,
                            );

                            // Fire Rate upgrade
                            spawn_upgrade_card(
                                cards_row,
                                &wood_icon,
                                "SPEED",
                                "-20%",
                                format!("Lv.{}", tower_level.fire_rate_level),
                                UpgradeType::FireRate,
                                UPGRADE_FIRE_RATE_COST,
                            );
                        });

                    // Close hint
                    panel.spawn((
                        Text::new("Right-click or ESC to close"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));
                });
        });
}

fn spawn_upgrade_card(
    parent: &mut ChildSpawnerCommands,
    wood_icon: &Handle<Image>,
    name: &str,
    bonus: &str,
    level: String,
    upgrade_type: UpgradeType,
    cost: i32,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(6.0),
                width: Val::Px(90.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.35, 0.45, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|card: &mut ChildSpawnerCommands| {
            // Upgrade name
            card.spawn((
                Text::new(name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Bonus text
            card.spawn((
                Text::new(bonus),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 1.0, 0.4)),
            ));

            // Current level
            card.spawn((
                Text::new(level),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
            ));

            // Upgrade button
            card.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.4, 0.3, 1.0)),
                BorderRadius::all(Val::Px(4.0)),
                TowerUpgradeOption {
                    upgrade_type,
                    wood_cost: cost,
                },
                Button,
            ))
            .with_children(|button: &mut ChildSpawnerCommands| {
                button.spawn((
                    Text::new("Upgrade"),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                // Cost row with icon
                button
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|cost_row: &mut ChildSpawnerCommands| {
                        cost_row.spawn((
                            Text::new(format!("{}", cost)),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.7, 0.4)),
                        ));

                        cost_row.spawn((
                            ImageNode::new(wood_icon.clone()),
                            Node {
                                width: Val::Px(14.0),
                                height: Val::Px(14.0),
                                ..default()
                            },
                        ));
                    });
            });
        });
}

/// Hide upgrade menu on right-click or escape
pub fn hide_tower_upgrade_menu(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<TowerUpgradeMenuState>,
    menu_entities: Query<Entity, With<TowerUpgradeMenu>>,
) {
    if !menu_state.active {
        return;
    }

    if mouse_button.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
        for entity in menu_entities.iter() {
            commands.entity(entity).despawn();
        }
        menu_state.active = false;
        menu_state.selected_tower = None;
    }
}

/// Handle clicking on upgrade buttons
pub fn handle_tower_upgrade(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &TowerUpgradeOption),
        (Changed<Interaction>, With<Button>),
    >,
    game_state: Res<GameState>,
    mut menu_state: ResMut<TowerUpgradeMenuState>,
    menu_entities: Query<Entity, With<TowerUpgradeMenu>>,
    entity_map: Res<EntityMap>,
    stdb: Option<SpacetimeDB>,
    mut sound_events: EventWriter<SoundEffect>,
) {
    for (interaction, option) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            sound_events.write(SoundEffect::ButtonClick);
            if game_state.wood >= option.wood_cost {
                if let Some(tower_entity) = menu_state.selected_tower {
                    // Look up the server entity_id from the Bevy entity
                    if let Some(server_id) = entity_map.bevy_to_server.get(&tower_entity) {
                        if let Some(ref stdb) = stdb {
                            // Call the appropriate reducer based on upgrade type
                            let result = match option.upgrade_type {
                                UpgradeType::Damage => {
                                    info!("Requesting damage upgrade for tower {}", server_id);
                                    stdb.conn().reducers.upgrade_tower_damage(*server_id)
                                }
                                UpgradeType::Range => {
                                    info!("Requesting range upgrade for tower {}", server_id);
                                    stdb.conn().reducers.upgrade_tower_range(*server_id)
                                }
                                UpgradeType::FireRate => {
                                    info!("Requesting fire rate upgrade for tower {}", server_id);
                                    stdb.conn().reducers.upgrade_tower_fire_rate(*server_id)
                                }
                            };

                            if let Err(e) = result {
                                error!("Failed to upgrade tower: {:?}", e);
                            }
                        } else {
                            info!("No network connection - upgrades require server");
                        }

                        // Close menu after requesting upgrade
                        // (server sync will update the tower stats)
                        for entity in menu_entities.iter() {
                            commands.entity(entity).despawn();
                        }
                        menu_state.active = false;
                        menu_state.selected_tower = None;
                    } else {
                        warn!("Tower entity not found in entity map - may be a local tower");
                    }
                }
            } else {
                info!(
                    "Not enough wood to upgrade. Need {}, have {}",
                    option.wood_cost, game_state.wood
                );
            }
        }
    }
}
