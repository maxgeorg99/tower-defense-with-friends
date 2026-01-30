// =============================================================================
// Tower Actions
// =============================================================================

use spacetimedb::{ReducerContext, Table};
use crate::tables::game_entity::{GameEntity, EntityType, game_entity as GameEntityTable};
use crate::tables::components::{TowerComponent, AttackType, tower_component as TowerComponentTable};
use crate::tables::wave::{TowerTypeDef, tower_type_def as TowerTypeDefTable};
use crate::tables::user::{User, user as UserTable};

/// Place a new tower at the specified position
#[spacetimedb::reducer]
pub fn place_tower(
    ctx: &ReducerContext,
    tower_type: String,
    x: f32,
    y: f32,
) -> Result<(), String> {
    let owner = ctx.sender;

    // Get tower type definition
    let tower_def = ctx.db.tower_type_def().tower_type().find(&tower_type)
        .ok_or_else(|| format!("Unknown tower type: {}", tower_type))?;

    // Get player and check if they have enough gold
    let mut player = ctx.db.user().identity().find(owner)
        .ok_or("Player not found")?;

    if player.gold < tower_def.cost {
        return Err(format!(
            "Not enough gold. Need {}, have {}",
            tower_def.cost, player.gold
        ));
    }

    // Deduct gold from player
    player.gold -= tower_def.cost;
    let remaining_gold = player.gold;
    ctx.db.user().identity().update(player);

    // Create the base entity
    let entity = GameEntity::new_tower(owner, x, y);
    let entity = ctx.db.game_entity().insert(entity);
    let entity_id = entity.entity_id;

    // Create the tower component
    let tower_component = TowerComponent {
        entity_id,
        tower_type: tower_type.clone(),
        range: tower_def.base_range,
        damage: tower_def.base_damage,
        fire_rate: tower_def.base_fire_rate,
        cooldown: 0.0,
        projectile_speed: tower_def.projectile_speed,
        attack_type: tower_def.attack_type,
        damage_level: 1,
        range_level: 1,
        fire_rate_level: 1,
    };
    ctx.db.tower_component().insert(tower_component);

    log::info!(
        "Player {:?} placed {} tower at ({}, {}) - {} gold remaining",
        owner, tower_type, x, y, remaining_gold
    );

    Ok(())
}

/// Upgrade a tower's damage
#[spacetimedb::reducer]
pub fn upgrade_tower_damage(ctx: &ReducerContext, entity_id: u64) -> Result<(), String> {
    const UPGRADE_COST: i32 = 30;
    const DAMAGE_MULTIPLIER: f32 = 1.25;

    upgrade_tower(ctx, entity_id, UPGRADE_COST, |tower| {
        tower.damage *= DAMAGE_MULTIPLIER;
        tower.damage_level += 1;
    })
}

/// Upgrade a tower's range
#[spacetimedb::reducer]
pub fn upgrade_tower_range(ctx: &ReducerContext, entity_id: u64) -> Result<(), String> {
    const UPGRADE_COST: i32 = 25;
    const RANGE_MULTIPLIER: f32 = 1.20;

    upgrade_tower(ctx, entity_id, UPGRADE_COST, |tower| {
        tower.range *= RANGE_MULTIPLIER;
        tower.range_level += 1;
    })
}

/// Upgrade a tower's fire rate
#[spacetimedb::reducer]
pub fn upgrade_tower_fire_rate(ctx: &ReducerContext, entity_id: u64) -> Result<(), String> {
    const UPGRADE_COST: i32 = 35;
    const FIRE_RATE_MULTIPLIER: f32 = 0.80; // Lower = faster

    upgrade_tower(ctx, entity_id, UPGRADE_COST, |tower| {
        tower.fire_rate *= FIRE_RATE_MULTIPLIER;
        tower.fire_rate_level += 1;
    })
}

/// Helper function for tower upgrades
fn upgrade_tower<F>(
    ctx: &ReducerContext,
    entity_id: u64,
    wood_cost: i32,
    apply_upgrade: F,
) -> Result<(), String>
where
    F: FnOnce(&mut TowerComponent),
{
    let owner = ctx.sender;

    // Verify entity exists and is owned by caller
    let entity = ctx.db.game_entity().entity_id().find(entity_id)
        .ok_or("Tower not found")?;

    if entity.owner != Some(owner) {
        return Err("You don't own this tower".to_string());
    }

    if entity.entity_type != EntityType::Tower {
        return Err("Entity is not a tower".to_string());
    }

    // Check player's resources
    let mut player = ctx.db.user().identity().find(owner)
        .ok_or("Player not found")?;

    if player.wood < wood_cost {
        return Err(format!(
            "Not enough wood. Need {}, have {}",
            wood_cost, player.wood
        ));
    }

    // Get tower component
    let mut tower = ctx.db.tower_component().entity_id().find(entity_id)
        .ok_or("Tower component not found")?;

    // Deduct cost from player and apply upgrade
    player.wood -= wood_cost;
    ctx.db.user().identity().update(player);

    apply_upgrade(&mut tower);
    ctx.db.tower_component().entity_id().update(tower);

    log::info!("Tower {} upgraded by {:?}", entity_id, owner);
    Ok(())
}

/// Sell a tower for partial refund
#[spacetimedb::reducer]
pub fn sell_tower(ctx: &ReducerContext, entity_id: u64) -> Result<(), String> {
    const SELL_REFUND_PERCENT: f32 = 0.5;

    let owner = ctx.sender;

    // Verify entity exists and is owned by caller
    let mut entity = ctx.db.game_entity().entity_id().find(entity_id)
        .ok_or("Tower not found")?;

    if entity.owner != Some(owner) {
        return Err("You don't own this tower".to_string());
    }

    if entity.entity_type != EntityType::Tower {
        return Err("Entity is not a tower".to_string());
    }

    // Get tower component for refund calculation
    let tower = ctx.db.tower_component().entity_id().find(entity_id)
        .ok_or("Tower component not found")?;

    // Get original cost
    let tower_def = ctx.db.tower_type_def().tower_type().find(&tower.tower_type);
    let refund = tower_def
        .map(|def| (def.cost as f32 * SELL_REFUND_PERCENT) as i32)
        .unwrap_or(0);

    // Add refund to player's gold
    let mut player = ctx.db.user().identity().find(owner)
        .ok_or("Player not found")?;
    player.gold += refund;
    ctx.db.user().identity().update(player);

    // Mark entity as inactive (will be cleaned up by cleanup agent)
    entity.active = false;
    ctx.db.game_entity().entity_id().update(entity);

    // Delete tower component
    ctx.db.tower_component().entity_id().delete(entity_id);

    log::info!("Tower {} sold by {:?} for {} gold", entity_id, owner, refund);
    Ok(())
}
