// =============================================================================
// Worker Agent
// =============================================================================
//
// Handles worker AI: finding resources, harvesting, returning to base.
// Workers follow a simple state machine:
//
// Idle -> MovingToResource -> Harvesting -> ReturningWithResource -> Idle
//
// =============================================================================

use spacetimedb::{ReducerContext, ScheduleAt, Table, TimeDuration};
use crate::tables::game_entity::{GameEntity, EntityType, game_entity as GameEntityTable};
use crate::tables::worker::{
    WorkerComponent, WorkerBuildingComponent, ResourceNodeComponent,
    WorkerState, ResourceType, HARVEST_TIME, RESOURCES_PER_HARVEST,
    worker_component as WorkerComponentTable,
    worker_building_component as WorkerBuildingComponentTable,
    resource_node_component as ResourceNodeComponentTable,
};
use crate::tables::user::{User, user as UserTable};

/// Worker tick interval in microseconds (100ms = 10Hz)
const WORKER_TICK_US: i64 = 100_000;
/// Delta time in seconds
const DELTA_TIME: f32 = 0.1;
/// Distance threshold for arrival
const ARRIVAL_THRESHOLD: f32 = 10.0;

// =============================================================================
// Timer Table
// =============================================================================

#[spacetimedb::table(name = worker_timer, scheduled(worker_tick, at = scheduled_at))]
pub struct WorkerTimer {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}

// =============================================================================
// Agent Functions
// =============================================================================

pub fn init(ctx: &ReducerContext) {
    ctx.db.worker_timer().insert(WorkerTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(ctx.timestamp),
    });
    log::info!("Worker agent initialized");
}

#[spacetimedb::reducer]
pub fn worker_tick(ctx: &ReducerContext, timer: WorkerTimer) {
    // Delete triggering timer
    ctx.db.worker_timer().scheduled_id().delete(timer.scheduled_id);

    // Process all workers
    process_workers(ctx);

    // Reschedule
    let next_time = ctx.timestamp + TimeDuration::from_micros(WORKER_TICK_US);
    ctx.db.worker_timer().insert(WorkerTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(next_time),
    });
}

fn process_workers(ctx: &ReducerContext) {
    // Get all active worker entities
    let workers: Vec<_> = ctx.db.game_entity()
        .iter()
        .filter(|e| e.entity_type == EntityType::Unit && e.active)
        .filter_map(|e| {
            ctx.db.worker_component().entity_id().find(e.entity_id)
                .map(|w| (e, w))
        })
        .collect();

    for (entity, worker) in workers {
        match worker.state {
            WorkerState::Idle => process_idle_worker(ctx, entity, worker),
            WorkerState::MovingToResource => process_moving_to_resource(ctx, entity, worker),
            WorkerState::Harvesting => process_harvesting(ctx, entity, worker),
            WorkerState::ReturningWithResource => process_returning(ctx, entity, worker),
        }
    }
}

fn process_idle_worker(ctx: &ReducerContext, entity: GameEntity, mut worker: WorkerComponent) {
    // Find nearest available resource
    if let Some((resource_entity, resource_node)) = find_nearest_resource(ctx, &entity) {
        // Set target and transition to moving
        worker.target_resource_id = Some(resource_entity.entity_id);
        worker.target_x = resource_entity.x;
        worker.target_y = resource_entity.y;
        worker.state = WorkerState::MovingToResource;
        ctx.db.worker_component().entity_id().update(worker);

        log::debug!(
            "Worker {} targeting {:?} resource {}",
            entity.entity_id, resource_node.resource_type, resource_entity.entity_id
        );
    }
}

fn process_moving_to_resource(ctx: &ReducerContext, entity: GameEntity, mut worker: WorkerComponent) {
    // Check if target still exists
    let target_id = match worker.target_resource_id {
        Some(id) => id,
        None => {
            // No target, go idle
            worker.state = WorkerState::Idle;
            ctx.db.worker_component().entity_id().update(worker);
            return;
        }
    };

    let target_resource = ctx.db.resource_node_component().entity_id().find(target_id);
    if target_resource.is_none() || target_resource.as_ref().map(|r| r.depleted).unwrap_or(true) {
        // Target gone or depleted, go idle
        worker.target_resource_id = None;
        worker.state = WorkerState::Idle;
        ctx.db.worker_component().entity_id().update(worker);
        return;
    }

    // Move towards target
    let arrived = move_towards(ctx, &entity, &mut worker);

    if arrived {
        // Start harvesting
        worker.state = WorkerState::Harvesting;
        worker.harvest_progress = 0.0;
    }

    ctx.db.worker_component().entity_id().update(worker);
}

fn process_harvesting(ctx: &ReducerContext, entity: GameEntity, mut worker: WorkerComponent) {
    // Update harvest progress
    worker.harvest_progress += DELTA_TIME;

    if worker.harvest_progress >= HARVEST_TIME {
        // Harvest complete
        let target_id = match worker.target_resource_id {
            Some(id) => id,
            None => {
                worker.state = WorkerState::Idle;
                ctx.db.worker_component().entity_id().update(worker);
                return;
            }
        };

        // Get resource node
        if let Some(mut resource) = ctx.db.resource_node_component().entity_id().find(target_id) {
            // Take resource
            let resource_type = resource.resource_type;
            resource.remaining -= RESOURCES_PER_HARVEST;

            if resource.remaining <= 0 {
                resource.depleted = true;
                // Mark entity as inactive
                if let Some(mut res_entity) = ctx.db.game_entity().entity_id().find(target_id) {
                    res_entity.active = false;
                    ctx.db.game_entity().entity_id().update(res_entity);
                }
            }
            ctx.db.resource_node_component().entity_id().update(resource);

            // Worker now carrying resource
            worker.carrying_resource = Some(resource_type);
        }

        // Get home building position
        if let Some(home_entity) = ctx.db.game_entity().entity_id().find(worker.home_building_id) {
            worker.target_x = home_entity.x;
            worker.target_y = home_entity.y;
        }

        // Start returning
        worker.state = WorkerState::ReturningWithResource;
        worker.target_resource_id = None;
        worker.harvest_progress = 0.0;
    }

    ctx.db.worker_component().entity_id().update(worker);
}

fn process_returning(ctx: &ReducerContext, entity: GameEntity, mut worker: WorkerComponent) {
    // Move towards home
    let arrived = move_towards(ctx, &entity, &mut worker);

    if arrived {
        // Deposit resource to the worker's owner
        if let Some(resource_type) = worker.carrying_resource {
            if let Some(owner) = entity.owner {
                if let Some(mut player) = ctx.db.user().identity().find(owner) {
                    match resource_type {
                        ResourceType::Wood => player.wood += RESOURCES_PER_HARVEST,
                        ResourceType::Gold => player.gold += RESOURCES_PER_HARVEST,
                        ResourceType::Meat => player.meat += RESOURCES_PER_HARVEST,
                    }
                    ctx.db.user().identity().update(player);

                    log::debug!(
                        "Worker {} deposited {:?} for player {:?}",
                        entity.entity_id, resource_type, owner
                    );
                }
            }
        }

        // Clear carrying and go idle
        worker.carrying_resource = None;
        worker.state = WorkerState::Idle;
    }

    ctx.db.worker_component().entity_id().update(worker);
}

/// Move entity towards worker's target position
/// Returns true if arrived
fn move_towards(ctx: &ReducerContext, entity: &GameEntity, worker: &mut WorkerComponent) -> bool {
    let dx = worker.target_x - entity.x;
    let dy = worker.target_y - entity.y;
    let distance = (dx * dx + dy * dy).sqrt();

    if distance <= ARRIVAL_THRESHOLD {
        return true;
    }

    // Move towards target
    let dir_x = dx / distance;
    let dir_y = dy / distance;
    let move_dist = worker.speed * DELTA_TIME;

    let new_x = entity.x + dir_x * move_dist.min(distance);
    let new_y = entity.y + dir_y * move_dist.min(distance);

    // Update entity position
    let mut updated = entity.clone();
    updated.x = new_x;
    updated.y = new_y;
    ctx.db.game_entity().entity_id().update(updated);

    false
}

/// Find the nearest non-depleted resource node
fn find_nearest_resource(
    ctx: &ReducerContext,
    worker_entity: &GameEntity,
) -> Option<(GameEntity, ResourceNodeComponent)> {
    let mut nearest: Option<(GameEntity, ResourceNodeComponent, f32)> = None;

    // Get all active resource nodes
    for resource in ctx.db.resource_node_component().iter() {
        if resource.depleted {
            continue;
        }

        let Some(entity) = ctx.db.game_entity().entity_id().find(resource.entity_id) else {
            continue;
        };

        if !entity.active {
            continue;
        }

        let dx = entity.x - worker_entity.x;
        let dy = entity.y - worker_entity.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if let Some((_, _, nearest_dist)) = &nearest {
            if distance < *nearest_dist {
                nearest = Some((entity, resource, distance));
            }
        } else {
            nearest = Some((entity, resource, distance));
        }
    }

    nearest.map(|(e, r, _)| (e, r))
}
