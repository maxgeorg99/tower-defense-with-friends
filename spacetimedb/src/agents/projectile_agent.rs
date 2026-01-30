// =============================================================================
// Projectile Agent
// =============================================================================
//
// Handles projectile movement and collision detection.
//
// =============================================================================

use spacetimedb::{Identity, ReducerContext, ScheduleAt, Table, TimeDuration};
use crate::tables::game_entity::{GameEntity, EntityType, game_entity as GameEntityTable};
use crate::tables::components::{
    ProjectileComponent, EnemyComponent,
    get_damage_multiplier,
    projectile_component as ProjectileComponentTable,
    enemy_component as EnemyComponentTable,
};
use crate::tables::game_state::{GameState, game_state as GameStateTable};
use crate::tables::user::{User, user as UserTable};

/// Projectile tick interval in microseconds (50ms = 20Hz)
const PROJECTILE_TICK_US: i64 = 50_000;
/// Delta time in seconds
const DELTA_TIME: f32 = 0.05;
/// Hit detection radius
const HIT_RADIUS: f32 = 16.0;

// =============================================================================
// Timer Table
// =============================================================================

#[spacetimedb::table(name = projectile_timer, scheduled(projectile_tick, at = scheduled_at))]
pub struct ProjectileTimer {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}

// =============================================================================
// Agent Functions
// =============================================================================

pub fn init(ctx: &ReducerContext) {
    ctx.db.projectile_timer().insert(ProjectileTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(ctx.timestamp),
    });
    log::info!("Projectile agent initialized");
}

#[spacetimedb::reducer]
pub fn projectile_tick(ctx: &ReducerContext, timer: ProjectileTimer) {
    // Delete triggering timer
    ctx.db.projectile_timer().scheduled_id().delete(timer.scheduled_id);

    // Process projectiles
    process_projectiles(ctx);

    // Reschedule
    let next_time = ctx.timestamp + TimeDuration::from_micros(PROJECTILE_TICK_US);
    ctx.db.projectile_timer().insert(ProjectileTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(next_time),
    });
}

fn process_projectiles(ctx: &ReducerContext) {
    // Get all active projectiles
    let projectiles: Vec<_> = ctx.db.game_entity()
        .iter()
        .filter(|e| e.entity_type == EntityType::Projectile && e.active)
        .collect();

    for proj_entity in projectiles {
        let Some(projectile) = ctx.db.projectile_component().entity_id().find(proj_entity.entity_id) else {
            continue;
        };

        // Get target entity
        let Some(target_entity) = ctx.db.game_entity().entity_id().find(projectile.target_id) else {
            // Target no longer exists, despawn projectile
            despawn_projectile(ctx, proj_entity.entity_id);
            continue;
        };

        if !target_entity.active {
            // Target died, despawn projectile
            despawn_projectile(ctx, proj_entity.entity_id);
            continue;
        }

        // Calculate distance to target
        let dx = target_entity.x - proj_entity.x;
        let dy = target_entity.y - proj_entity.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance <= HIT_RADIUS {
            // Hit! Apply damage
            handle_projectile_hit(ctx, &projectile, projectile.target_id, proj_entity.owner);
            despawn_projectile(ctx, proj_entity.entity_id);
        } else {
            // Move towards target
            let dir_x = dx / distance;
            let dir_y = dy / distance;

            let move_dist = projectile.speed * DELTA_TIME;

            // Check for overshoot
            let (new_x, new_y) = if move_dist >= distance {
                (target_entity.x, target_entity.y)
            } else {
                (
                    proj_entity.x + dir_x * move_dist,
                    proj_entity.y + dir_y * move_dist,
                )
            };

            // Update position
            let mut updated = proj_entity.clone();
            updated.x = new_x;
            updated.y = new_y;
            ctx.db.game_entity().entity_id().update(updated);
        }
    }
}

fn handle_projectile_hit(ctx: &ReducerContext, projectile: &ProjectileComponent, target_id: u64, owner: Option<Identity>) {
    let Some(mut enemy) = ctx.db.enemy_component().entity_id().find(target_id) else {
        return;
    };

    // Calculate damage with type effectiveness
    let multiplier = get_damage_multiplier(projectile.attack_type, enemy.defense_type);
    let final_damage = projectile.damage * multiplier;

    enemy.health -= final_damage;

    if enemy.health <= 0.0 {
        // Enemy killed
        handle_enemy_death(ctx, target_id, &enemy, owner);
    } else {
        ctx.db.enemy_component().entity_id().update(enemy);
    }

    log::debug!("Projectile hit enemy {} for {} damage", target_id, final_damage);
}

fn despawn_projectile(ctx: &ReducerContext, entity_id: u64) {
    // Mark entity as inactive
    if let Some(mut entity) = ctx.db.game_entity().entity_id().find(entity_id) {
        entity.active = false;
        ctx.db.game_entity().entity_id().update(entity);
    }

    // Delete projectile component
    ctx.db.projectile_component().entity_id().delete(entity_id);
}

fn handle_enemy_death(ctx: &ReducerContext, entity_id: u64, enemy: &EnemyComponent, owner: Option<Identity>) {
    // Award gold to the player who killed the enemy
    if let Some(owner_identity) = owner {
        if let Some(mut player) = ctx.db.user().identity().find(owner_identity) {
            player.gold += enemy.gold_reward;
            ctx.db.user().identity().update(player);
            log::debug!("Awarded {} gold to player {:?}", enemy.gold_reward, owner_identity);
        }
    }

    // Update game state score and kill count (score is still shared/global)
    if let Some(mut game_state) = ctx.db.game_state().id().find(0) {
        game_state.score += enemy.gold_reward;
        game_state.enemies_killed += 1;
        ctx.db.game_state().id().update(game_state);
    }

    // Mark entity as inactive
    if let Some(mut entity) = ctx.db.game_entity().entity_id().find(entity_id) {
        entity.active = false;
        ctx.db.game_entity().entity_id().update(entity);
    }

    // Delete enemy component
    ctx.db.enemy_component().entity_id().delete(entity_id);

    log::debug!("Enemy {} killed", entity_id);
}
