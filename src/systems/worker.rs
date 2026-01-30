use bevy::prelude::*;
use bevy::ecs::prelude::ChildSpawnerCommands;

use crate::components::{
    AnimationTimer, BuildWorkerOption, Depleted, HarvestTimer, HouseMenu, ResourceNode,
    ResourceType, Worker, WorkerBuilding, WorkerState, WorkerTarget,
};
use crate::constants::SCALED_TILE_SIZE;
use crate::map::{create_blocked_tiles, tile_to_world};
use crate::module_bindings::Color as PlayerColor;
use crate::resources::{GameState, HouseMenuState, RecruitMenuState, SelectedColor, TowerUpgradeMenuState, TowerWheelState};
use crate::systems::{AnimationInfo, DustEffect, SoundEffect};

const PAWN_FRAME_SIZE: UVec2 = UVec2::new(192, 192);
const WORKER_SPEED: f32 = 30.0;
const HARVEST_TIME: f32 = 5.0;
const ARRIVAL_DISTANCE: f32 = 16.0;

// Asset sizes for proper scaling
const HOUSE_SIZE: Vec2 = Vec2::new(128.0, 128.0);

/// Get the color directory name for asset paths
fn get_color_dir(color: PlayerColor) -> &'static str {
    match color {
        PlayerColor::Blue => "Blue",
        PlayerColor::Yellow => "Yellow",
        PlayerColor::Purple => "Purple",
        PlayerColor::Black => "Black",
    }
}

/// Setup resource gathering - spawns building, trees, gold mines, and sheep
pub fn setup_resource_gathering(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    selected_color: Res<SelectedColor>,
) {
    let color = selected_color.0;
    let color_dir = get_color_dir(color);

    // Spawn worker building (House1) close to castle (castle is at ~tile 27, 10)
    let building_pos = tile_to_world(27, 6);
    let building_path = format!("Decorations/Buildings/{} Buildings/House1.png", color_dir);
    // Scale house like towers: use min of x/y scale to fit in 1 tile
    let house_scale_x = SCALED_TILE_SIZE / HOUSE_SIZE.x;
    let house_scale_y = SCALED_TILE_SIZE / HOUSE_SIZE.y;
    let house_scale = house_scale_x.min(house_scale_y);
    let _building_entity = commands
        .spawn((
            Sprite::from_image(asset_server.load(&building_path)),
            Transform::from_xyz(building_pos.x, building_pos.y, 1.0)
                .with_scale(Vec3::splat(house_scale)),
            WorkerBuilding {
                spawn_timer: Timer::from_seconds(3.0, TimerMode::Repeating),
                worker_capacity: 1,
                current_workers: 0,
            },
        ))
        .id();

    const TREE_FRAME_SIZE: UVec2 = UVec2::new(192, 256);
    let tree_scale = SCALED_TILE_SIZE / TREE_FRAME_SIZE.x as f32;

    // Get blocked tiles (road and castle) to avoid placing trees there
    let (blocked_tiles, _) = create_blocked_tiles();

    // Tree positions distributed across the map, avoiding the road path
    // Original cluster near worker building
    let mut tree_positions: Vec<(i32, i32)> = vec![
        (22, 15), (22, 14), (21, 16),
    ];

    // Additional tree clusters in various areas of the map
    // Top-left area (y=0-3)
    tree_positions.extend([(1, 1), (2, 2), (3, 1), (1, 3), (8, 1), (9, 2), (14, 1), (15, 2)]);

    // Left side (avoid road at x=5, y around 4-10)
    tree_positions.extend([(1, 6), (2, 7), (1, 13), (2, 14), (3, 15), (1, 17), (2, 18)]);

    // Bottom area (y=17-19)
    tree_positions.extend([(7, 18), (8, 19), (9, 18), (13, 18), (14, 19), (20, 18), (21, 19)]);

    // Middle-top area (between road segments)
    tree_positions.extend([(7, 1), (8, 2), (13, 1), (14, 2), (19, 1), (20, 2), (21, 1)]);

    // Right side area (near castle but not blocking)
    tree_positions.extend([(28, 2), (29, 3), (28, 17), (29, 18)]);

    // Center areas (between road bends)
    tree_positions.extend([(8, 7), (9, 8), (13, 8), (14, 7), (19, 8), (20, 7)]);

    // Filter out any positions that overlap with blocked tiles
    let tree_positions: Vec<(i32, i32)> = tree_positions
        .into_iter()
        .filter(|pos| !blocked_tiles.contains(pos))
        .collect();

    for (i, (tx, ty)) in tree_positions.iter().enumerate() {
        let tree_pos = tile_to_world(*tx, *ty);
        let tree_sprite = format!("Terrain/Resources/Wood/Trees/Tree{}.png", (i % 4) + 1);

        let texture = asset_server.load(&tree_sprite);
        let layout = TextureAtlasLayout::from_grid(TREE_FRAME_SIZE, 8, 1, None, None);
        let texture_atlas_layout = asset_server.add(layout);

        commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                },
            ),
            Transform::from_xyz(tree_pos.x, tree_pos.y, 1.0)
                .with_scale(Vec3::splat(tree_scale)),
            ResourceNode {
                resource_type: ResourceType::Wood,
                remaining: 5,
            },
            AnimationTimer {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            },
            AnimationInfo {
                frame_count: 8,
            },
        ));
    }

    // === GOLD: Gold stones (animated highlight sprite sheets, 6 frames of 128x128) ===
    const GOLD_FRAME_SIZE: UVec2 = UVec2::new(128, 128);
    let gold_scale = SCALED_TILE_SIZE / GOLD_FRAME_SIZE.x as f32;
    let gold_positions = [(24, 17), (25, 17), (24, 18)];
    for (i, (gx, gy)) in gold_positions.iter().enumerate() {
        let gold_pos = tile_to_world(*gx, *gy);
        // Use larger gold stones with highlight animation (4, 5, 6)
        let gold_sprite = format!("Terrain/Resources/Gold/Gold Stones/Gold Stone {}_Highlight.png", (i % 3) + 4);

        let texture = asset_server.load(&gold_sprite);
        let layout = TextureAtlasLayout::from_grid(GOLD_FRAME_SIZE, 6, 1, None, None);
        let texture_atlas_layout = asset_server.add(layout);

        commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                },
            ),
            Transform::from_xyz(gold_pos.x, gold_pos.y, 1.0)
                .with_scale(Vec3::splat(gold_scale)),
            ResourceNode {
                resource_type: ResourceType::Gold,
                remaining: 8,
            },
            AnimationTimer {
                timer: Timer::from_seconds(0.25, TimerMode::Repeating),
            },
            AnimationInfo {
                frame_count: 6,
            },
        ));
    }

    // === MEAT: Sheep (animated sprite sheets, 6 frames of 128x128) ===
    const SHEEP_FRAME_SIZE: UVec2 = UVec2::new(128, 128);
    let sheep_scale = SCALED_TILE_SIZE / SHEEP_FRAME_SIZE.x as f32;
    let sheep_positions = [(20, 12), (19, 13), (20, 14)];
    for (_i, (sx, sy)) in sheep_positions.iter().enumerate() {
        let sheep_pos = tile_to_world(*sx, *sy);

        // Create animated sheep sprite
        let texture = asset_server.load("Terrain/Resources/Meat/Sheep/Sheep_Idle.png");
        let layout = TextureAtlasLayout::from_grid(SHEEP_FRAME_SIZE, 6, 1, None, None);
        let texture_atlas_layout = asset_server.add(layout);

        commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                },
            ),
            Transform::from_xyz(sheep_pos.x, sheep_pos.y, 1.0)
                .with_scale(Vec3::splat(sheep_scale)),
            ResourceNode {
                resource_type: ResourceType::Meat,
                remaining: 3,
            },
            AnimationTimer {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            },
            AnimationInfo {
                frame_count: 6,
            },
        ));
    }
}

/// Spawn workers from building
pub fn spawn_workers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut buildings: Query<(Entity, &mut WorkerBuilding, &Transform)>,
    selected_color: Res<SelectedColor>,
) {
    let color = selected_color.0;
    let color_dir = get_color_dir(color);

    for (building_entity, mut building, building_transform) in buildings.iter_mut() {
        building.spawn_timer.tick(time.delta());

        if building.spawn_timer.just_finished() && building.current_workers < building.worker_capacity {
            let spawn_pos = building_transform.translation.truncate();

            // Load pawn idle sprite sheet with dynamic color
            let texture_path = format!("Units/{} Units/Pawn/Pawn_Idle.png", color_dir);
            let texture = asset_server.load(&texture_path);
            let layout = TextureAtlasLayout::from_grid(PAWN_FRAME_SIZE, 8, 1, None, None);
            let texture_atlas_layout = asset_server.add(layout);

            // Scale pawns same as enemies: SCALED_TILE_SIZE / frame_size
            let pawn_scale = SCALED_TILE_SIZE / PAWN_FRAME_SIZE.x as f32;

            commands.spawn((
                Sprite::from_atlas_image(
                    texture,
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: 0,
                    },
                ),
                Transform::from_xyz(spawn_pos.x, spawn_pos.y, 2.0)
                    .with_scale(Vec3::splat(pawn_scale)),
                Worker {
                    speed: WORKER_SPEED,
                    home_building: building_entity,
                    current_resource: None,
                },
                WorkerState::Idle,
                AnimationTimer {
                    timer: Timer::from_seconds(0.15, TimerMode::Repeating),
                },
                AnimationInfo {
                    frame_count: 6,
                },
            ));

            building.current_workers += 1;
        }
    }
}

/// Assign idle workers to nearby resource nodes
pub fn worker_find_resource(
    mut commands: Commands,
    mut workers: Query<(Entity, &Transform, &mut Worker), (With<WorkerState>, Without<WorkerTarget>)>,
    worker_states: Query<&WorkerState>,
    resources: Query<(Entity, &Transform, &ResourceNode), Without<Depleted>>,
) {
    for (worker_entity, worker_transform, mut worker) in workers.iter_mut() {
        let state = worker_states.get(worker_entity).unwrap();
        if *state != WorkerState::Idle {
            continue;
        }

        // Find nearest undepleted resource
        let worker_pos = worker_transform.translation.truncate();
        let mut nearest: Option<(Entity, Vec2, f32, ResourceType)> = None;

        for (res_entity, res_transform, resource) in resources.iter() {
            let res_pos = res_transform.translation.truncate();
            let dist = worker_pos.distance(res_pos);

            if nearest.is_none() || dist < nearest.unwrap().2 {
                nearest = Some((res_entity, res_pos, dist, resource.resource_type));
            }
        }

        if let Some((target_entity, target_pos, _, resource_type)) = nearest {
            worker.current_resource = Some(resource_type);
            commands.entity(worker_entity).insert((
                WorkerTarget {
                    target_entity: Some(target_entity),
                    target_position: target_pos,
                },
                WorkerState::MovingToResource,
            ));
        }
    }
}

/// Move workers toward their target
pub fn worker_movement(
    time: Res<Time>,
    mut workers: Query<(&Worker, &WorkerState, &WorkerTarget, &mut Transform, &mut Sprite)>,
) {
    for (worker, state, target, mut transform, mut sprite) in workers.iter_mut() {
        if *state != WorkerState::MovingToResource && *state != WorkerState::ReturningWithResource {
            continue;
        }

        let current_pos = transform.translation.truncate();
        let direction = (target.target_position - current_pos).normalize_or_zero();

        // Move toward target
        let movement = direction * worker.speed * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        // Flip sprite based on movement direction
        if direction.x != 0.0 {
            sprite.flip_x = direction.x < 0.0;
        }
    }
}

/// Check if workers have arrived at their target
pub fn worker_arrive_check(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    buildings: Query<&Transform, With<WorkerBuilding>>,
    mut workers: Query<(Entity, &Transform, &mut WorkerState, &WorkerTarget, &mut Worker)>,
    mut sound_events: MessageWriter<SoundEffect>,
) {
    for (worker_entity, worker_transform, mut state, target, mut worker) in workers.iter_mut() {
        let worker_pos = worker_transform.translation.truncate();

        match *state {
            WorkerState::MovingToResource => {
                // Check distance to resource target
                let dist = worker_pos.distance(target.target_position);
                if dist < ARRIVAL_DISTANCE {
                    // Start harvesting
                    *state = WorkerState::Harvesting;
                    commands
                        .entity(worker_entity)
                        .insert(HarvestTimer(Timer::from_seconds(HARVEST_TIME, TimerMode::Once)));
                }
            }
            WorkerState::ReturningWithResource => {
                // Check distance to the ACTUAL building, not target.target_position
                // (target might not be updated yet due to deferred commands)
                if let Ok(building_transform) = buildings.get(worker.home_building) {
                    let building_pos = building_transform.translation.truncate();
                    let dist_to_building = worker_pos.distance(building_pos);

                    if dist_to_building < ARRIVAL_DISTANCE {
                        // Deposit resource and go idle
                        let had_resource = worker.current_resource.is_some();
                        match worker.current_resource {
                            Some(ResourceType::Wood) => game_state.wood += 1,
                            Some(ResourceType::Gold) => game_state.gold += 5,
                            Some(ResourceType::Meat) => game_state.meat += 1,
                            None => {}
                        }
                        if had_resource {
                            sound_events.write(SoundEffect::Reward);
                        }
                        worker.current_resource = None;
                        commands.entity(worker_entity).remove::<WorkerTarget>();
                        *state = WorkerState::Idle;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Handle harvesting timer and resource depletion
pub fn worker_harvest(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    buildings: Query<&Transform, With<WorkerBuilding>>,
    mut workers: Query<(
        Entity,
        &mut WorkerState,
        &mut HarvestTimer,
        &WorkerTarget,
        &Worker,
    )>,
    mut resources: Query<(Entity, &mut ResourceNode, &mut Sprite)>,
) {
    for (worker_entity, mut state, mut harvest_timer, target, worker) in workers.iter_mut() {
        if *state != WorkerState::Harvesting {
            continue;
        }

        harvest_timer.0.tick(time.delta());

        if harvest_timer.0.just_finished() {
            // Decrement resource
            if let Some(target_entity) = target.target_entity {
                if let Ok((res_entity, mut resource, mut res_sprite)) =
                    resources.get_mut(target_entity)
                {
                    resource.remaining -= 1;

                    if resource.remaining <= 0 {
                        // Resource depleted - handle based on type
                        match resource.resource_type {
                            ResourceType::Wood => {
                                // Wood turns into a stump (persists but marked depleted)
                                // Remove animation components since stump is static
                                commands.entity(res_entity)
                                    .insert(Depleted)
                                    .remove::<AnimationTimer>()
                                    .remove::<AnimationInfo>();
                                // Change to static stump image
                                res_sprite.image =
                                    asset_server.load("Terrain/Resources/Wood/Trees/Stump 1.png");
                                // Remove texture atlas to use as regular sprite
                                res_sprite.texture_atlas = None;
                            }
                            ResourceType::Gold => {
                                // Gold stone disappears when fully harvested
                                commands.entity(res_entity).despawn();
                            }
                            ResourceType::Meat => {
                                // Sheep disappears when fully harvested
                                commands.entity(res_entity).despawn();
                            }
                        }
                    }
                }
            }

            // Return to building with resource
            if let Ok(building_transform) = buildings.get(worker.home_building) {
                let building_pos = building_transform.translation.truncate();
                commands.entity(worker_entity).remove::<HarvestTimer>();
                commands.entity(worker_entity).insert(WorkerTarget {
                    target_entity: None,
                    target_position: building_pos,
                });
                *state = WorkerState::ReturningWithResource;
            }
        }
    }
}

pub fn worker_sprite_update(
    asset_server: Res<AssetServer>,
    mut workers: Query<(&WorkerState, &Worker, &mut Sprite, &mut AnimationInfo), Changed<WorkerState>>,
    selected_color: Res<SelectedColor>,
    mut sound_events: MessageWriter<SoundEffect>,
) {
    let color = selected_color.0;
    let color_dir = get_color_dir(color);

    for (state, worker, mut sprite, mut anim_info) in workers.iter_mut() {
        // Play harvesting sounds when entering Harvesting state
        if *state == WorkerState::Harvesting {
            match worker.current_resource {
                Some(ResourceType::Wood) => { sound_events.write(SoundEffect::AxeHit); }
                Some(ResourceType::Gold) => { sound_events.write(SoundEffect::PickaxeHit); }
                Some(ResourceType::Meat) => { sound_events.write(SoundEffect::SheepHarvest); }
                None => {}
            }
        }

        // Returns (sprite_name, frame_count)
        let (sprite_name, frame_count) = match (state, worker.current_resource) {
            (WorkerState::Idle, _) => ("Pawn_Idle.png", 8),
            // Moving to resource - use tool based on resource type
            (WorkerState::MovingToResource, Some(ResourceType::Wood)) => ("Pawn_Run Axe.png", 6),
            (WorkerState::MovingToResource, Some(ResourceType::Gold)) => ("Pawn_Run Pickaxe.png", 6),
            (WorkerState::MovingToResource, Some(ResourceType::Meat)) => ("Pawn_Run Knife.png", 6),
            (WorkerState::MovingToResource, None) => ("Pawn_Run.png", 8),
            // Harvesting - use interact animation based on resource type
            (WorkerState::Harvesting, Some(ResourceType::Wood)) => ("Pawn_Interact Axe.png", 6),
            (WorkerState::Harvesting, Some(ResourceType::Gold)) => ("Pawn_Interact Pickaxe.png", 6),
            (WorkerState::Harvesting, Some(ResourceType::Meat)) => ("Pawn_Interact Knife.png", 4), // Only 4 frames!
            (WorkerState::Harvesting, None) => ("Pawn_Idle.png", 8),
            // Returning with resource - carry the resource
            (WorkerState::ReturningWithResource, Some(ResourceType::Wood)) => ("Pawn_Run Wood.png", 6),
            (WorkerState::ReturningWithResource, Some(ResourceType::Gold)) => ("Pawn_Run Gold.png", 6),
            (WorkerState::ReturningWithResource, Some(ResourceType::Meat)) => ("Pawn_Run Meat.png", 6),
            (WorkerState::ReturningWithResource, None) => ("Pawn_Run.png", 6),
        };

        let texture_path = format!("Units/{} Units/Pawn/{}", color_dir, sprite_name);

        // Update the sprite image
        sprite.image = asset_server.load(&texture_path);

        // Update frame count for this animation
        anim_info.frame_count = frame_count;

        // Reset to first frame when state changes
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = 0;
        }
    }
}

// Separate system to animate frames
pub fn animate_worker_sprites(
    time: Res<Time>,
    mut workers: Query<(&mut Sprite, &mut AnimationTimer, &AnimationInfo)>,
) {
    for (mut sprite, mut timer, info) in workers.iter_mut() {
        timer.timer.tick(time.delta());

        if timer.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % info.frame_count;
            }
        }
    }
}

// ==================== House Menu Systems ====================

const WORKER_GOLD_COST: i32 = 50;

/// Show house menu when clicking on the worker building
pub fn show_house_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut house_menu_state: ResMut<HouseMenuState>,
    recruit_menu_state: Res<RecruitMenuState>,
    tower_wheel_state: Res<TowerWheelState>,
    upgrade_menu_state: Res<TowerUpgradeMenuState>,
    buildings: Query<&Transform, With<WorkerBuilding>>,
    existing_menus: Query<Entity, With<HouseMenu>>,
    selected_color: Res<SelectedColor>,
) {
    if !mouse_button.just_pressed(MouseButton::Left)
        || house_menu_state.active
        || recruit_menu_state.active
        || tower_wheel_state.active
        || upgrade_menu_state.active
    {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    for building_transform in buildings.iter() {
        let building_pos = building_transform.translation.truncate();
        if world_pos.distance(building_pos) < SCALED_TILE_SIZE / 2.0 {
            for entity in existing_menus.iter() {
                commands.entity(entity).despawn();
            }
            house_menu_state.active = true;
            spawn_house_menu(&mut commands, &asset_server, selected_color.0);
            return;
        }
    }
}

fn spawn_house_menu(commands: &mut Commands, asset_server: &Res<AssetServer>, player_color: PlayerColor) {
    let gold_icon = asset_server.load("UI Elements/UI Elements/Icons/Gold_Icon.png");
    let color_dir = get_color_dir(player_color);
    let pawn_icon: Handle<Image> = asset_server.load(format!("Units/{} Units/Pawn/Pawn_Avatar.png", color_dir));

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
            HouseMenu,
        ))
        .with_children(|parent| {
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
                    panel.spawn((
                        Text::new("Worker House"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                    ));

                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(15.0)),
                                row_gap: Val::Px(8.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.35, 0.45, 0.9)),
                            BorderRadius::all(Val::Px(8.0)),
                        ))
                        .with_children(|card: &mut ChildSpawnerCommands| {
                            // Pawn sprite preview (first frame of the sprite sheet)
                            card.spawn((
                                ImageNode::new(pawn_icon.clone()),
                                Node {
                                    width: Val::Px(64.0),
                                    height: Val::Px(64.0),
                                    ..default()
                                },
                            ));

                            card.spawn((
                                Text::new("WORKER"),
                                TextFont { font_size: 14.0, ..default() },
                                TextColor(Color::WHITE),
                            ));

                            card.spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.15, 0.4, 0.3, 1.0)),
                                BorderRadius::all(Val::Px(4.0)),
                                BuildWorkerOption { gold_cost: WORKER_GOLD_COST },
                                Button,
                            ))
                            .with_children(|button: &mut ChildSpawnerCommands| {
                                button.spawn((
                                    Text::new("Build"),
                                    TextFont { font_size: 12.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));

                                button
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(4.0),
                                        ..default()
                                    })
                                    .with_children(|cost_row: &mut ChildSpawnerCommands| {
                                        cost_row.spawn((
                                            Text::new(format!("{}", WORKER_GOLD_COST)),
                                            TextFont { font_size: 11.0, ..default() },
                                            TextColor(Color::srgb(1.0, 0.85, 0.0)),
                                        ));
                                        cost_row.spawn((
                                            ImageNode::new(gold_icon.clone()),
                                            Node { width: Val::Px(16.0), height: Val::Px(16.0), ..default() },
                                        ));
                                    });
                            });
                        });

                    panel.spawn((
                        Text::new("Right-click or ESC to close"),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));
                });
        });
}

pub fn hide_house_menu(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<HouseMenuState>,
    menu_entities: Query<Entity, With<HouseMenu>>,
) {
    if !menu_state.active {
        return;
    }
    if mouse_button.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
        for entity in menu_entities.iter() {
            commands.entity(entity).despawn();
        }
        menu_state.active = false;
    }
}

pub fn handle_build_worker(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut interaction_query: Query<(&Interaction, &BuildWorkerOption), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
    mut menu_state: ResMut<HouseMenuState>,
    menu_entities: Query<Entity, With<HouseMenu>>,
    mut buildings: Query<(Entity, &mut WorkerBuilding, &Transform)>,
    selected_color: Res<SelectedColor>,
    mut sound_events: MessageWriter<SoundEffect>,
) {
    for (interaction, option) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            sound_events.write(SoundEffect::ButtonClick);
            if game_state.gold >= option.gold_cost {
                game_state.gold -= option.gold_cost;

                if let Some((building_entity, mut building, building_transform)) = buildings.iter_mut().next() {
                    let spawn_pos = building_transform.translation.truncate();
                    let color = selected_color.0;
                    let color_dir = get_color_dir(color);

                    let texture_path = format!("Units/{} Units/Pawn/Pawn_Idle.png", color_dir);
                    let texture = asset_server.load(&texture_path);
                    let layout = TextureAtlasLayout::from_grid(PAWN_FRAME_SIZE, 8, 1, None, None);
                    let texture_atlas_layout = asset_server.add(layout);
                    let pawn_scale = SCALED_TILE_SIZE / PAWN_FRAME_SIZE.x as f32;

                    // Spawn dust effect at worker spawn position
                    let dust_frame_count = 8;
                    let dust_frame_time = 0.08;
                    let dust_layout = TextureAtlasLayout::from_grid(
                        UVec2::new(64, 64),
                        dust_frame_count,
                        1,
                        None,
                        None,
                    );
                    let dust_atlas_layout = texture_atlases.add(dust_layout);
                    let dust_scale = SCALED_TILE_SIZE / 64.0;

                    commands.spawn((
                        Sprite::from_atlas_image(
                            asset_server.load("Particle FX/Dust_01.png"),
                            TextureAtlas {
                                layout: dust_atlas_layout,
                                index: 0,
                            },
                        ),
                        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 1.5)
                            .with_scale(Vec3::splat(dust_scale)),
                        DustEffect {
                            frame_count: dust_frame_count as usize,
                            timer: Timer::from_seconds(dust_frame_time, TimerMode::Repeating),
                        },
                    ));

                    commands.spawn((
                        Sprite::from_atlas_image(texture, TextureAtlas { layout: texture_atlas_layout, index: 0 }),
                        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 2.0).with_scale(Vec3::splat(pawn_scale)),
                        Worker { speed: WORKER_SPEED, home_building: building_entity, current_resource: None },
                        WorkerState::Idle,
                        AnimationTimer { timer: Timer::from_seconds(0.15, TimerMode::Repeating) },
                        AnimationInfo { frame_count: 6 },
                    ));

                    building.current_workers += 1;
                    building.worker_capacity += 1;
                    info!("Worker built for {} gold! Total: {}", option.gold_cost, building.current_workers);
                }

                for entity in menu_entities.iter() {
                    commands.entity(entity).despawn();
                }
                menu_state.active = false;
            }
        }
    }
}