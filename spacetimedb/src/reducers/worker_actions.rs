// =============================================================================
// Worker Actions
// =============================================================================

use spacetimedb::{ReducerContext, Table};
use crate::tables::game_entity::{GameEntity, EntityType, game_entity as GameEntityTable};
use crate::tables::worker::{
    WorkerComponent, WorkerBuildingComponent, ResourceNodeComponent, WorkerState, ResourceType,
    worker_component as WorkerComponentTable,
    worker_building_component as WorkerBuildingComponentTable,
    resource_node_component as ResourceNodeComponentTable,
    WORKER_COST, WORKER_SPEED,
};
use crate::tables::game_state::{GameState, game_state as GameStateTable};

/// Buy a new worker for a building
#[spacetimedb::reducer]
pub fn buy_worker(ctx: &ReducerContext, building_entity_id: u64) -> Result<(), String> {
    let owner = ctx.sender;

    // Verify building exists and is owned by caller
    let building_entity = ctx.db.game_entity().entity_id().find(building_entity_id)
        .ok_or("Building not found")?;

    if building_entity.owner != Some(owner) {
        return Err("You don't own this building".to_string());
    }

    // Get building component
    let mut building = ctx.db.worker_building_component().entity_id().find(building_entity_id)
        .ok_or("Not a worker building")?;

    // Check capacity
    if building.current_workers >= building.worker_capacity {
        return Err(format!(
            "Building at capacity ({}/{})",
            building.current_workers, building.worker_capacity
        ));
    }

    // Check gold
    let mut game_state = ctx.db.game_state().id().find(0)
        .ok_or("Game state not found")?;

    if game_state.gold < WORKER_COST {
        return Err(format!(
            "Not enough gold. Need {}, have {}",
            WORKER_COST, game_state.gold
        ));
    }

    // Deduct gold
    game_state.gold -= WORKER_COST;
    ctx.db.game_state().id().update(game_state);

    // Create worker entity at building position
    let worker_entity = GameEntity::new_unit(owner, building_entity.x, building_entity.y);
    let worker_entity = ctx.db.game_entity().insert(worker_entity);
    let entity_id = worker_entity.entity_id;

    // Create worker component
    let worker = WorkerComponent {
        entity_id,
        speed: WORKER_SPEED,
        home_building_id: building_entity_id,
        state: WorkerState::Idle,
        carrying_resource: None,
        target_resource_id: None,
        target_x: building_entity.x,
        target_y: building_entity.y,
        harvest_progress: 0.0,
    };
    ctx.db.worker_component().insert(worker);

    // Update building worker count
    building.current_workers += 1;
    ctx.db.worker_building_component().entity_id().update(building);

    log::info!(
        "Player {:?} bought worker {} for building {}",
        owner, entity_id, building_entity_id
    );

    Ok(())
}

/// Place a worker building (house)
#[spacetimedb::reducer]
pub fn place_worker_building(
    ctx: &ReducerContext,
    x: f32,
    y: f32,
    initial_capacity: i32,
) -> Result<(), String> {
    let owner = ctx.sender;

    // Create building entity
    let entity = GameEntity {
        entity_id: 0,
        owner: Some(owner),
        entity_type: EntityType::Tower, // Reuse Tower type for buildings
        x,
        y,
        active: true,
    };
    let entity = ctx.db.game_entity().insert(entity);
    let entity_id = entity.entity_id;

    // Create building component
    let building = WorkerBuildingComponent {
        entity_id,
        worker_capacity: initial_capacity,
        current_workers: 0,
        spawn_cooldown: 0.0,
    };
    ctx.db.worker_building_component().insert(building);

    log::info!(
        "Player {:?} placed worker building {} at ({}, {})",
        owner, entity_id, x, y
    );

    Ok(())
}

/// Spawn a resource node (for map initialization)
#[spacetimedb::reducer]
pub fn spawn_resource_node(
    ctx: &ReducerContext,
    resource_type: ResourceType,
    x: f32,
    y: f32,
    initial_amount: i32,
) -> Result<(), String> {
    // Create resource entity (no owner)
    let entity = GameEntity {
        entity_id: 0,
        owner: None,
        entity_type: EntityType::Unit, // Reuse for resource nodes
        x,
        y,
        active: true,
    };
    let entity = ctx.db.game_entity().insert(entity);
    let entity_id = entity.entity_id;

    // Create resource node component
    let resource_node = crate::tables::worker::ResourceNodeComponent {
        entity_id,
        resource_type,
        remaining: initial_amount,
        depleted: false,
    };
    ctx.db.resource_node_component().insert(resource_node);

    log::info!(
        "Spawned {:?} resource node {} at ({}, {}) with {} resources",
        resource_type, entity_id, x, y, initial_amount
    );

    Ok(())
}
