use bevy::prelude::*;

use crate::components::{AnimationTimer, Enemy, HealthBar};
use crate::constants::{SCALED_TILE_SIZE, WARRIOR_FRAME_SIZE};
use crate::resources::{EnemySpawner, GameState, PathWaypoints, WaveConfigs};

pub fn spawn_enemies(
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
        let mut selected_spawn = None;

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

pub fn move_enemies(
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

pub fn cleanup_dead_enemies(
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
