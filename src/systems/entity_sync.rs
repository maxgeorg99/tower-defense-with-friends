use bevy::prelude::*;
use std::collections::HashMap;
use bevy_spacetimedb::*;
use spacetimedb_sdk::Table;

use crate::components::{AttackType, DefenseType, Enemy, Projectile, Tower, TowerLevel};
use crate::constants::SCALED_TILE_SIZE;
use crate::module_bindings::{
    GameEntity, TowerComponent as DbTowerComponent, EnemyComponent as DbEnemyComponent,
    ProjectileComponent as DbProjectileComponent, EntityType,
};
use crate::resources::SelectedColor;

/// Maps server entity IDs to Bevy entities for syncing
#[derive(Resource, Default)]
pub struct EntityMap {
    pub server_to_bevy: HashMap<u64, Entity>,
    pub bevy_to_server: HashMap<Entity, u64>,
}

impl EntityMap {
    pub fn insert(&mut self, server_id: u64, bevy_entity: Entity) {
        self.server_to_bevy.insert(server_id, bevy_entity);
        self.bevy_to_server.insert(bevy_entity, server_id);
    }

    pub fn remove_by_server_id(&mut self, server_id: u64) -> Option<Entity> {
        if let Some(bevy_entity) = self.server_to_bevy.remove(&server_id) {
            self.bevy_to_server.remove(&bevy_entity);
            Some(bevy_entity)
        } else {
            None
        }
    }

    pub fn get_bevy_entity(&self, server_id: u64) -> Option<Entity> {
        self.server_to_bevy.get(&server_id).copied()
    }
}

/// Marker component for entities synced from the server
#[derive(Component)]
pub struct ServerEntity {
    pub server_id: u64,
}

// =============================================================================
// Game Entity Sync - Position Updates
// =============================================================================

/// Handle new game entities being inserted
pub fn on_game_entity_inserted(
    mut commands: Commands,
    messages: Option<ReadInsertMessage<GameEntity>>,
    mut entity_map: ResMut<EntityMap>,
) {
    let Some(mut messages) = messages else { return };

    for msg in messages.read() {
        let entity = &msg.row;

        // Skip inactive entities
        if !entity.active {
            continue;
        }

        // Skip if we already have this entity
        if entity_map.get_bevy_entity(entity.entity_id).is_some() {
            continue;
        }

        let position = Vec3::new(entity.x, entity.y, 0.0);

        match entity.entity_type {
            EntityType::Tower => {
                // Tower will be fully spawned when we receive TowerComponent
                // For now, spawn a placeholder that will be updated
                let bevy_entity = commands.spawn((
                    ServerEntity { server_id: entity.entity_id },
                    Transform::from_translation(position),
                    Visibility::Hidden, // Hidden until we get component data
                )).id();
                entity_map.insert(entity.entity_id, bevy_entity);
                info!("Spawned tower placeholder for server entity {}", entity.entity_id);
            }
            EntityType::Enemy => {
                // Enemy will be fully spawned when we receive EnemyComponent
                let bevy_entity = commands.spawn((
                    ServerEntity { server_id: entity.entity_id },
                    Transform::from_translation(position),
                    Visibility::Hidden,
                )).id();
                entity_map.insert(entity.entity_id, bevy_entity);
                info!("Spawned enemy placeholder for server entity {}", entity.entity_id);
            }
            EntityType::Projectile => {
                let bevy_entity = commands.spawn((
                    ServerEntity { server_id: entity.entity_id },
                    Transform::from_translation(position),
                    Visibility::Hidden,
                )).id();
                entity_map.insert(entity.entity_id, bevy_entity);
                info!("Spawned projectile placeholder for server entity {}", entity.entity_id);
            }
            EntityType::Unit => {
                // Worker units - handle separately if needed
                info!("Unit entity {} - not yet implemented", entity.entity_id);
            }
        }
    }
}

/// Handle game entities being updated (position changes)
pub fn on_game_entity_updated(
    messages: Option<ReadUpdateMessage<GameEntity>>,
    entity_map: Res<EntityMap>,
    mut transforms: Query<&mut Transform>,
) {
    let Some(mut messages) = messages else { return };

    for msg in messages.read() {
        let entity = &msg.new;

        if let Some(bevy_entity) = entity_map.get_bevy_entity(entity.entity_id) {
            if let Ok(mut transform) = transforms.get_mut(bevy_entity) {
                transform.translation.x = entity.x;
                transform.translation.y = entity.y;
            }
        }
    }
}

/// Handle game entities being deleted
pub fn on_game_entity_deleted(
    mut commands: Commands,
    messages: Option<ReadDeleteMessage<GameEntity>>,
    mut entity_map: ResMut<EntityMap>,
) {
    let Some(mut messages) = messages else { return };

    for msg in messages.read() {
        let entity = &msg.row;

        if let Some(bevy_entity) = entity_map.remove_by_server_id(entity.entity_id) {
            if let Ok(mut entity_commands) = commands.get_entity(bevy_entity) {
                entity_commands.despawn();
            }
            info!("Despawned entity for server entity {}", entity.entity_id);
        }
    }
}

// =============================================================================
// Tower Component Sync
// =============================================================================

/// Handle tower components being inserted - complete the tower entity
pub fn on_tower_component_inserted(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    messages: Option<ReadInsertMessage<DbTowerComponent>>,
    entity_map: Res<EntityMap>,
    mut query: Query<(&mut Visibility, &Transform)>,
    selected_color: Res<SelectedColor>,
) {
    let Some(mut messages) = messages else { return };

    // Get user's color from local resource
    let color_str = match selected_color.0 {
        crate::module_bindings::Color::Blue => "Blue",
        crate::module_bindings::Color::Yellow => "Yellow",
        crate::module_bindings::Color::Purple => "Purple",
        crate::module_bindings::Color::Black => "Black",
    };

    for msg in messages.read() {
        let tower_comp = &msg.row;

        if let Some(bevy_entity) = entity_map.get_bevy_entity(tower_comp.entity_id) {
            // Get existing transform to preserve position
            let existing_pos = query.get(bevy_entity)
                .map(|(_, t)| t.translation)
                .unwrap_or(Vec3::ZERO);

            // Get tower sprite based on type and user's color
            // Paths match towers.toml config: Decorations/Buildings/{Color} Buildings/{Building}.png
            let sprite_path = match tower_comp.tower_type.as_str() {
                "archer" => format!("Decorations/Buildings/{} Buildings/Archery.png", color_str),
                "catapult" => format!("Decorations/Buildings/{} Buildings/Barracks.png", color_str),
                "holy" => format!("Decorations/Buildings/{} Buildings/Monastery.png", color_str),
                "tower" => format!("Decorations/Buildings/{} Buildings/Tower.png", color_str),
                _ => format!("Decorations/Buildings/{} Buildings/Archery.png", color_str),
            };

            let scale = SCALED_TILE_SIZE / 128.0; // Tower sprites are typically 128x256

            commands.entity(bevy_entity).insert((
                Sprite::from_image(asset_server.load(&sprite_path)),
                Transform::from_translation(existing_pos).with_scale(Vec3::splat(scale)),
                Tower {
                    tower_type_id: tower_comp.tower_type.clone(),
                    range: tower_comp.range,
                    damage: tower_comp.damage,
                    fire_rate: tower_comp.fire_rate,
                    cooldown: tower_comp.cooldown,
                    projectile_sprite: "Units/Blue Units/Archer/Arrow.png".to_string(),
                    projectile_speed: tower_comp.projectile_speed,
                    attack_type: match tower_comp.attack_type {
                        crate::module_bindings::AttackType::Blunt => AttackType::Blunt,
                        crate::module_bindings::AttackType::Pierce => AttackType::Pierce,
                        crate::module_bindings::AttackType::Divine => AttackType::Divine,
                    },
                },
                TowerLevel {
                    damage_level: tower_comp.damage_level,
                    range_level: tower_comp.range_level,
                    fire_rate_level: tower_comp.fire_rate_level,
                },
            ));

            // Make visible now that we have the full data
            if let Ok((mut visibility, _)) = query.get_mut(bevy_entity) {
                *visibility = Visibility::Visible;
            }

            info!("Tower {} fully initialized: type={}, color={}", tower_comp.entity_id, tower_comp.tower_type, color_str);
        }
    }
}

/// Handle tower component updates
pub fn on_tower_component_updated(
    messages: Option<ReadUpdateMessage<DbTowerComponent>>,
    entity_map: Res<EntityMap>,
    mut towers: Query<(&mut Tower, &mut TowerLevel)>,
) {
    let Some(mut messages) = messages else { return };

    for msg in messages.read() {
        let tower_comp = &msg.new;

        if let Some(bevy_entity) = entity_map.get_bevy_entity(tower_comp.entity_id) {
            if let Ok((mut tower, mut level)) = towers.get_mut(bevy_entity) {
                tower.damage = tower_comp.damage;
                tower.range = tower_comp.range;
                tower.fire_rate = tower_comp.fire_rate;
                tower.cooldown = tower_comp.cooldown;
                level.damage_level = tower_comp.damage_level;
                level.range_level = tower_comp.range_level;
                level.fire_rate_level = tower_comp.fire_rate_level;
            }
        }
    }
}

// =============================================================================
// Enemy Component Sync
// =============================================================================

/// Handle enemy components being inserted
pub fn on_enemy_component_inserted(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    messages: Option<ReadInsertMessage<DbEnemyComponent>>,
    entity_map: Res<EntityMap>,
    mut query: Query<(&mut Visibility, &Transform)>,
) {
    let Some(mut messages) = messages else { return };

    for msg in messages.read() {
        let enemy_comp = &msg.row;

        if let Some(bevy_entity) = entity_map.get_bevy_entity(enemy_comp.entity_id) {
            // Get existing transform to preserve position
            let existing_pos = query.get(bevy_entity)
                .map(|(_, t)| t.translation)
                .unwrap_or(Vec3::ZERO);

            // Get enemy sprite based on type
            let sprite_path = match enemy_comp.enemy_type.as_str() {
                "goblin" => "Units/Black Units/Pawn/BlackPawn.png",
                "skeleton" => "Units/Black Units/Pawn/BlackPawn.png",
                "orc" => "Units/Black Units/Pawn/BlackPawn.png",
                _ => "Units/Black Units/Pawn/BlackPawn.png",
            };

            let scale = SCALED_TILE_SIZE / 64.0; // Enemy sprites are typically 64x64

            commands.entity(bevy_entity).insert((
                Sprite::from_image(asset_server.load(sprite_path)),
                Transform::from_translation(existing_pos).with_scale(Vec3::splat(scale)),
                Enemy {
                    health: enemy_comp.health,
                    speed: enemy_comp.speed,
                    current_waypoint: enemy_comp.current_waypoint as usize,
                    gold_reward: enemy_comp.gold_reward,
                    damage_to_base: enemy_comp.damage_to_base,
                    defense_type: match enemy_comp.defense_type {
                        crate::module_bindings::DefenseType::Armor => DefenseType::Armor,
                        crate::module_bindings::DefenseType::Agility => DefenseType::Agility,
                        crate::module_bindings::DefenseType::Mystical => DefenseType::Mystical,
                    },
                },
            ));

            // Make visible
            if let Ok((mut visibility, _)) = query.get_mut(bevy_entity) {
                *visibility = Visibility::Visible;
            }

            info!("Enemy {} fully initialized: type={}", enemy_comp.entity_id, enemy_comp.enemy_type);
        }
    }
}

/// Handle enemy component updates (health, waypoint)
pub fn on_enemy_component_updated(
    messages: Option<ReadUpdateMessage<DbEnemyComponent>>,
    entity_map: Res<EntityMap>,
    mut enemies: Query<&mut Enemy>,
) {
    let Some(mut messages) = messages else { return };

    for msg in messages.read() {
        let enemy_comp = &msg.new;

        if let Some(bevy_entity) = entity_map.get_bevy_entity(enemy_comp.entity_id) {
            if let Ok(mut enemy) = enemies.get_mut(bevy_entity) {
                enemy.health = enemy_comp.health;
                enemy.current_waypoint = enemy_comp.current_waypoint as usize;
            }
        }
    }
}

// =============================================================================
// Projectile Component Sync
// =============================================================================

/// Handle projectile components being inserted
pub fn on_projectile_component_inserted(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    messages: Option<ReadInsertMessage<DbProjectileComponent>>,
    entity_map: Res<EntityMap>,
    mut query: Query<(&mut Visibility, &Transform)>,
) {
    let Some(mut messages) = messages else { return };

    for msg in messages.read() {
        let proj_comp = &msg.row;

        if let Some(bevy_entity) = entity_map.get_bevy_entity(proj_comp.entity_id) {
            // Get existing transform to preserve position from GameEntity
            let existing_pos = query.get(bevy_entity)
                .map(|(_, t)| t.translation)
                .unwrap_or(Vec3::ZERO);

            commands.entity(bevy_entity).insert((
                Sprite::from_image(asset_server.load("Units/Blue Units/Archer/Arrow.png")),
                Transform::from_translation(existing_pos).with_scale(Vec3::splat(0.5)),
                Projectile {
                    target: Entity::PLACEHOLDER, // Server handles targeting
                    damage: proj_comp.damage,
                    speed: proj_comp.speed,
                    attack_type: match proj_comp.attack_type {
                        crate::module_bindings::AttackType::Blunt => AttackType::Blunt,
                        crate::module_bindings::AttackType::Pierce => AttackType::Pierce,
                        crate::module_bindings::AttackType::Divine => AttackType::Divine,
                    },
                },
            ));

            // Make visible
            if let Ok((mut visibility, _)) = query.get_mut(bevy_entity) {
                *visibility = Visibility::Visible;
            }

            info!("Projectile {} initialized at {:?}", proj_comp.entity_id, existing_pos);
        }
    }
}
