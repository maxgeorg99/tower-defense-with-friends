use bevy::prelude::*;

use crate::components::{Enemy, Projectile};
use crate::resources::GameState;

pub fn move_projectiles(
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

pub fn handle_projectile_hits(
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
