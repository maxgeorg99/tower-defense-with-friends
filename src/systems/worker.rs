use bevy::prelude::*;
use bevy_spacetimedb::*;
use spacetimedb_sdk::Table;

use crate::components::{
    AnimationTimer, Depleted, HarvestTimer, ResourceNode, ResourceType, Worker, WorkerBuilding,
    WorkerState, WorkerTarget,
};
use crate::constants::SCALED_TILE_SIZE;
use crate::map::tile_to_world;
use crate::module_bindings::{Color as PlayerColor, DbConnection, UserTableAccess};
use crate::resources::GameState;
use crate::systems::AnimationInfo;

/// Type alias for cleaner SpacetimeDB resource access
pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

const PAWN_FRAME_SIZE: UVec2 = UVec2::new(192, 192);
const WORKER_SPEED: f32 = 60.0;
const HARVEST_TIME: f32 = 2.0;
const ARRIVAL_DISTANCE: f32 = 16.0;

// Asset sizes for proper scaling (like towers: use min of x/y scale to fit in 1 tile)
// Doubled to make them half size
const HOUSE_SIZE: Vec2 = Vec2::new(128.0, 128.0);
const TREE_SIZE: Vec2 = Vec2::new(128.0, 128.0);

/// Get the color directory name for asset paths
fn get_color_dir(color: PlayerColor) -> &'static str {
    match color {
        PlayerColor::Blue => "Blue",
        PlayerColor::Yellow => "Yellow",
        PlayerColor::Purple => "Purple",
        PlayerColor::Black => "Black",
    }
}

/// Get the current player's color from SpacetimeDB, defaulting to Blue
fn get_player_color(stdb: &Option<SpacetimeDB>) -> PlayerColor {
    stdb.as_ref()
        .and_then(|db| db.db().user().iter().next())
        .map(|user| user.color)
        .unwrap_or(PlayerColor::Blue)
}

/// Setup resource gathering - spawns building and trees
pub fn setup_resource_gathering(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stdb: Option<SpacetimeDB>,
) {
    let color = get_player_color(&stdb);
    let color_dir = get_color_dir(color);

    // Spawn worker building (House1) close to castle (castle is at ~tile 27, 10)
    let building_pos = tile_to_world(27, 6);
    let building_path = format!("Decorations/Buildings/{} Buildings/House1.png", color_dir);
    // Scale house like towers: use min of x/y scale to fit in 1 tile
    let house_scale_x = SCALED_TILE_SIZE / HOUSE_SIZE.x;
    let house_scale_y = SCALED_TILE_SIZE / HOUSE_SIZE.y;
    let house_scale = house_scale_x.min(house_scale_y);
    let building_entity = commands
        .spawn((
            Sprite::from_image(asset_server.load(&building_path)),
            Transform::from_xyz(building_pos.x, building_pos.y, 1.0)
                .with_scale(Vec3::splat(house_scale)),
            WorkerBuilding {
                spawn_timer: Timer::from_seconds(3.0, TimerMode::Repeating),
                max_workers: 4,
                spawned_workers: 0,
            },
        ))
        .id();

    // Spawn trees in a cluster near the castle (not in a line)
    // Scale trees like towers: use min of x/y scale to fit in 1 tile
    let tree_scale_x = SCALED_TILE_SIZE / TREE_SIZE.x;
    let tree_scale_y = SCALED_TILE_SIZE / TREE_SIZE.y;
    let tree_scale = tree_scale_x.min(tree_scale_y);
    let tree_positions = [(22, 15), (22, 14), (21, 16)];
    for (i, (tx, ty)) in tree_positions.iter().enumerate() {
        let tree_pos = tile_to_world(*tx, *ty);
        let tree_sprite = format!("Terrain/Resources/Wood/Trees/Tree{}.png", (i % 4) + 1);

        commands.spawn((
            Sprite::from_image(asset_server.load(&tree_sprite)),
            Transform::from_xyz(tree_pos.x, tree_pos.y, 1.0)
                .with_scale(Vec3::splat(tree_scale)),
            ResourceNode {
                resource_type: ResourceType::Wood,
                remaining: 5,
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
    stdb: Option<SpacetimeDB>,
) {
    let color = get_player_color(&stdb);
    let color_dir = get_color_dir(color);

    for (building_entity, mut building, building_transform) in buildings.iter_mut() {
        building.spawn_timer.tick(time.delta());

        if building.spawn_timer.just_finished() && building.spawned_workers < building.max_workers {
            let spawn_pos = building_transform.translation.truncate();

            // Load pawn idle sprite sheet with dynamic color
            let texture_path = format!("Units/{} Units/Pawn/Pawn_Idle.png", color_dir);
            let texture = asset_server.load(&texture_path);
            let layout = TextureAtlasLayout::from_grid(PAWN_FRAME_SIZE, 6, 1, None, None);
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
                },
                WorkerState::Idle,
                AnimationTimer {
                    timer: Timer::from_seconds(0.15, TimerMode::Repeating),
                },
                AnimationInfo {
                    frame_count: 6,
                },
            ));

            building.spawned_workers += 1;
            info!(
                building.spawned_workers, building.max_workers
            );
        }
    }
}

/// Assign idle workers to nearby resource nodes
pub fn worker_find_resource(
    mut commands: Commands,
    workers: Query<(Entity, &Transform, &Worker), (With<WorkerState>, Without<WorkerTarget>)>,
    worker_states: Query<&WorkerState>,
    resources: Query<(Entity, &Transform, &ResourceNode), Without<Depleted>>,
) {
    for (worker_entity, worker_transform, worker) in workers.iter() {
        let state = worker_states.get(worker_entity).unwrap();
        if *state != WorkerState::Idle {
            continue;
        }

        // Find nearest undepleted resource
        let worker_pos = worker_transform.translation.truncate();
        let mut nearest: Option<(Entity, Vec2, f32)> = None;

        for (res_entity, res_transform, _resource) in resources.iter() {
            let res_pos = res_transform.translation.truncate();
            let dist = worker_pos.distance(res_pos);

            if nearest.is_none() || dist < nearest.unwrap().2 {
                nearest = Some((res_entity, res_pos, dist));
            }
        }

        if let Some((target_entity, target_pos, _)) = nearest {
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
    mut workers: Query<(Entity, &Transform, &mut WorkerState, &WorkerTarget, &Worker)>,
) {
    for (worker_entity, worker_transform, mut state, target, worker) in workers.iter_mut() {
        let worker_pos = worker_transform.translation.truncate();
        let dist = worker_pos.distance(target.target_position);

        if dist < ARRIVAL_DISTANCE {
            match *state {
                WorkerState::MovingToResource => {
                    // Start harvesting
                    *state = WorkerState::Harvesting;
                    commands
                        .entity(worker_entity)
                        .insert(HarvestTimer(Timer::from_seconds(HARVEST_TIME, TimerMode::Once)));
                }
                WorkerState::ReturningWithResource => {
                    // Deposit resource and go idle
                    if buildings.get(worker.home_building).is_ok() {
                        game_state.wood += 1;
                        commands.entity(worker_entity).remove::<WorkerTarget>();
                        *state = WorkerState::Idle;
                    }
                }
                _ => {}
            }
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
                        // Resource depleted - change to stump
                        commands.entity(res_entity).insert(Depleted);
                        res_sprite.image =
                            asset_server.load("Terrain/Resources/Wood/Trees/Stump 1.png");
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
    mut workers: Query<(&WorkerState, &mut Sprite), Changed<WorkerState>>,
    stdb: Option<SpacetimeDB>,
) {
    let color = get_player_color(&stdb);
    let color_dir = get_color_dir(color);

    for (state, mut sprite) in workers.iter_mut() {
        let sprite_name = match state {
            WorkerState::Idle => "Pawn_Idle.png",
            WorkerState::MovingToResource => "Pawn_Run Axe.png",
            WorkerState::Harvesting => "Pawn_Interact Axe.png",
            WorkerState::ReturningWithResource => "Pawn_Run Wood.png",
        };

        let texture_path = format!("Units/{} Units/Pawn/{}", color_dir, sprite_name);

        // Update the sprite image
        sprite.image = asset_server.load(&texture_path);

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