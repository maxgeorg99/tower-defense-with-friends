use bevy::prelude::*;

use crate::components::{AnimationTimer, DefenseType, Enemy, HealthBar, HealthBarFill};
use crate::constants::{SCALED_TILE_SIZE, WARRIOR_FRAME_SIZE};
use crate::resources::{EnemySpawner, GameState, PathWaypoints, WaveConfigs};
use crate::systems::{SoundEffect, WaveManager};

#[derive(Component)]
pub struct AnimationInfo {
    pub frame_count: usize,
}


// ============================================================================
// Modified spawn_enemies integration
// ============================================================================

/// Modified version of your spawn_enemies system
pub fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
    mut spawner: ResMut<EnemySpawner>,
    mut game_state: ResMut<GameState>,
    waypoints: Res<PathWaypoints>,
    wave_configs: Res<WaveConfigs>,
    wave_manager: Res<WaveManager>,
) {
    // Only spawn enemies during active wave
    if !wave_manager.wave_active {
        return;
    }

    spawner.timer.tick(time.delta());

    if spawner.timer.just_finished() && spawner.enemies_spawned < spawner.enemies_this_wave {
        let current_wave_idx = (game_state.wave - 1) as usize;
        if current_wave_idx >= wave_configs.waves.len() {
            return;
        }

        let wave = &wave_configs.waves[current_wave_idx];

        let mut cumulative_count = 0;
        let mut selected_spawn = None;

        for spawn in &wave.spawns {
            let spawn_end = cumulative_count + spawn.count;
            if spawner.enemies_spawned >= cumulative_count && spawner.enemies_spawned < spawn_end {
                selected_spawn = Some(spawn);
                break;
            }
            cumulative_count = spawn_end;
        }

        let total_count = wave.spawns.iter().map(|s| s.count).sum::<i32>();

        if let Some(spawn) = selected_spawn {
            if let Some(unit_type) = wave_configs.units.iter().find(|u| u.id == spawn.unit_id) {
                let start_pos = waypoints.points.first().copied().unwrap_or(Vec2::ZERO);
                let max_health = unit_type.base_health * spawn.health_multiplier;

                let [frame_width, frame_height] = unit_type.frame_size;
                let layout = TextureAtlasLayout::from_grid(
                    UVec2::new(frame_width, frame_height),
                    unit_type.frame_count as u32,
                    1,
                    None,
                    None,
                );
                let texture_atlas_layout = texture_atlases.add(layout);

                let enemy_scale = SCALED_TILE_SIZE / frame_width as f32;

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
                            defense_type: DefenseType::from_str(&unit_type.defense_type),
                        },
                        AnimationTimer {
                            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                        },
                        AnimationInfo {
                            frame_count: unit_type.frame_count,
                        },
                    ))
                    .id();

                spawn_health_bar(&mut commands, &asset_server, enemy_entity, max_health, SCALED_TILE_SIZE);

                spawner.enemies_spawned += 1;

                if spawner.enemies_spawned >= total_count {
                    spawner.enemies_spawned = 0;
                    game_state.wave += 1;

                    if (game_state.wave - 1) < wave_configs.waves.len() as i32 {
                        let next_wave = &wave_configs.waves[(game_state.wave - 1) as usize];
                        spawner.timer =
                            Timer::from_seconds(next_wave.spawn_interval, TimerMode::Repeating);
                        spawner.enemies_this_wave = next_wave.spawns.iter().map(|s| s.count).sum();
                    }
                }
            }
        }
    }
}
fn spawn_health_bar(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    parent_entity: Entity,
    max_health: f32,
    scaled_tile_size: f32,
) -> Entity {

    // The health bar background is 320x64
    // Make it much bigger for visibility
    let bar_width = scaled_tile_size * 18.0; // Increased from 5 to 10
    let bar_height = bar_width * (64.0 / 320.0); // Maintain aspect ratio

    // Make the fill thicker and more visible
    let fill_height = 9.0;

    // Create the background sprite (the border/frame)
    let health_bar_bg = commands
        .spawn((
            Sprite {
                image: asset_server.load("UI Elements/UI Elements/Bars/SmallBar_Base.png"),
                color: Color::srgb(0.3, 0.3, 0.3), // Gray background as fallback
                custom_size: Some(Vec2::new(bar_width, bar_height)),
                ..default()
            },
            Transform::from_xyz(0.0, scaled_tile_size * 2.5, 5.0), // Position above enemy, higher z
            HealthBar { max_health },
        ))
        .id();

    // Create the fill (green bar that shrinks)
    // Position it centered within the background
    // The actual fill area inside the frame is much smaller than the total width
    let fill_width = bar_width * 0.2; // Much smaller to fit inside the frame border
    let health_bar_fill = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.0, 1.0, 0.0), // Green
                custom_size: Some(Vec2::new(fill_width, fill_height)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1), // Centered on the background
            HealthBarFill { max_width: fill_width },
        ))
        .id();

    // Make fill a child of background
    commands.entity(health_bar_bg).add_child(health_bar_fill);

    // Make background a child of enemy
    commands.entity(parent_entity).add_child(health_bar_bg);

    health_bar_bg
}

pub fn move_enemies(
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy, Option<&Children>)>,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    waypoints: Res<PathWaypoints>,
    mut sound_events: EventWriter<SoundEffect>,
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
            sound_events.write(SoundEffect::CastleDamage);
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
