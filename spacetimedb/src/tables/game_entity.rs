// =============================================================================
// Game Entity - Base Entity Table
// =============================================================================
//
// All game objects (towers, enemies, projectiles, units) are entities.
// This table stores the common data shared by all entities.
// Type-specific data is stored in component tables that reference entity_id.
//
// =============================================================================

use spacetimedb::{Identity, SpacetimeType};

/// Entity type discriminator
#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Tower,
    Enemy,
    Projectile,
    Unit,  // Player-recruited units
}

/// Base entity table - all game objects have an entry here
#[spacetimedb::table(name = game_entity, public)]
#[derive(Clone)]
pub struct GameEntity {
    #[primary_key]
    #[auto_inc]
    pub entity_id: u64,

    /// Owner of this entity (for towers/units) or None (for enemies)
    pub owner: Option<Identity>,

    /// Type discriminatFor for component lookup
    pub entity_type: EntityType,

    /// Position in world coordinates
    pub x: f32,
    pub y: f32,

    /// Whether this entity is active (false = pending deletion)
    pub active: bool,
}

impl GameEntity {
    pub fn new_tower(owner: Identity, x: f32, y: f32) -> Self {
        Self {
            entity_id: 0, // auto_inc
            owner: Some(owner),
            entity_type: EntityType::Tower,
            x,
            y,
            active: true,
        }
    }

    pub fn new_enemy(x: f32, y: f32) -> Self {
        Self {
            entity_id: 0,
            owner: None,
            entity_type: EntityType::Enemy,
            x,
            y,
            active: true,
        }
    }

    pub fn new_projectile(owner: Option<Identity>, x: f32, y: f32) -> Self {
        Self {
            entity_id: 0,
            owner,
            entity_type: EntityType::Projectile,
            x,
            y,
            active: true,
        }
    }

    pub fn new_unit(owner: Identity, x: f32, y: f32) -> Self {
        Self {
            entity_id: 0,
            owner: Some(owner),
            entity_type: EntityType::Unit,
            x,
            y,
            active: true,
        }
    }
}
