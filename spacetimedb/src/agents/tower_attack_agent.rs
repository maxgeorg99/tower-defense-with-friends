// =============================================================================
// Tower Attack Agent
// =============================================================================
//
// Handles tower targeting and attacks.
// Towers find closest enemy in range and either:
// - Fire a projectile (normal towers)
// - Deal instant damage (holy tower)
//
// =============================================================================

use spacetimedb::{ReducerContext, ScheduleAt, Table, TimeDuration};
use crate::tables::game_entity::{GameEntity, EntityType, game_entity as GameEntityTable};
use crate::tables::components::{
    TowerComponent, EnemyComponent, ProjectileComponent, AttackType,
    get_damage_multiplier,
    tower_component as TowerComponentTable,
    enemy_component as EnemyComponentTable,
    projectile_component as ProjectileComponentTable,
};
use crate::tables::game_state::{GameState, game_state as GameStateTable};

/// Attack tick interval in microseconds (50ms = 20Hz)
const ATTACK_TICK_US: i64 = 50_000;
/// Delta time in seconds
const DELTA_TIME: f32 = 0.05;

// =============================================================================
// Timer Table
// =============================================================================

#[spacetimedb::table(name = tower_attack_timer, scheduled(tower_attack_tick, at = scheduled_at))]
pub struct TowerAttackTimer {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}

// =============================================================================
// Agent Functions
// =============================================================================

pub fn init(ctx: &ReducerContext) {
    ctx.db.tower_attack_timer().insert(TowerAttackTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(ctx.timestamp),
    });
    log::info!("Tower attack agent initialized");
}

#[spacetimedb::reducer]
pub fn tower_attack_tick(ctx: &ReducerContext, timer: TowerAttackTimer) {
    // Delete triggering timer
    ctx.db.tower_attack_timer().scheduled_id().delete(timer.scheduled_id);

    // Process tower attacks
    process_tower_attacks(ctx);

    // Reschedule
    let next_time = ctx.timestamp + TimeDuration::from_micros(ATTACK_TICK_US);
    ctx.db.tower_attack_timer().insert(TowerAttackTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(next_time),
    });
}

fn process_tower_attacks(ctx: &ReducerContext) {
    // Get all active towers
    let towers: Vec<_> = ctx.db.game_entity()
        .iter()
        .filter(|e| e.entity_type == EntityType::Tower && e.active)
        .collect();

    // Get all active enemies with positions
    let enemies: Vec<(GameEntity, EnemyComponent)> = ctx.db.game_entity()
        .iter()
        .filter(|e| e.entity_type == EntityType::Enemy && e.active)
        .filter_map(|e| {
            ctx.db.enemy_component().entity_id().find(e.entity_id)
                .map(|ec| (e, ec))
        })
        .collect();

    for tower_entity in towers {
        let Some(mut tower) = ctx.db.tower_component().entity_id().find(tower_entity.entity_id) else {
            continue;
        };

        // Update cooldown
        tower.cooldown -= DELTA_TIME;

        if tower.cooldown <= 0.0 {
            // Find closest enemy in range
            if let Some((target_entity, target_enemy)) = find_closest_enemy(
                &tower_entity,
                &tower,
                &enemies,
            ) {
                // Attack!
                if tower.tower_type == "holy" {
                    // Holy tower: instant damage
                    deal_instant_damage(ctx, &tower, target_entity.entity_id, &target_enemy);
                } else {
                    // Normal tower: spawn projectile
                    spawn_projectile(ctx, &tower_entity, &tower, target_entity.entity_id);
                }

                // Reset cooldown
                tower.cooldown = tower.fire_rate;
            }
        }

        ctx.db.tower_component().entity_id().update(tower);
    }
}

fn find_closest_enemy<'a>(
    tower_entity: &GameEntity,
    tower: &TowerComponent,
    enemies: &'a [(GameEntity, EnemyComponent)],
) -> Option<(&'a GameEntity, &'a EnemyComponent)> {
    let mut closest: Option<(&GameEntity, &EnemyComponent, f32)> = None;

    for (enemy_entity, enemy_component) in enemies {
        let dx = enemy_entity.x - tower_entity.x;
        let dy = enemy_entity.y - tower_entity.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance <= tower.range {
            if let Some((_, _, closest_dist)) = closest {
                if distance < closest_dist {
                    closest = Some((enemy_entity, enemy_component, distance));
                }
            } else {
                closest = Some((enemy_entity, enemy_component, distance));
            }
        }
    }

    closest.map(|(e, c, _)| (e, c))
}

fn deal_instant_damage(
    ctx: &ReducerContext,
    tower: &TowerComponent,
    target_id: u64,
    target_enemy: &EnemyComponent,
) {
    let multiplier = get_damage_multiplier(tower.attack_type, target_enemy.defense_type);
    let final_damage = tower.damage * multiplier;

    // Apply damage
    if let Some(mut enemy) = ctx.db.enemy_component().entity_id().find(target_id) {
        enemy.health -= final_damage;

        if enemy.health <= 0.0 {
            // Enemy killed
            handle_enemy_death(ctx, target_id, &enemy);
        } else {
            ctx.db.enemy_component().entity_id().update(enemy);
        }
    }

    log::debug!("Holy tower dealt {} damage to enemy {}", final_damage, target_id);
}

fn spawn_projectile(
    ctx: &ReducerContext,
    tower_entity: &GameEntity,
    tower: &TowerComponent,
    target_id: u64,
) {
    // Create projectile entity
    let entity = GameEntity::new_projectile(tower_entity.owner, tower_entity.x, tower_entity.y);
    let entity = ctx.db.game_entity().insert(entity);
    let entity_id = entity.entity_id;

    // Create projectile component
    let projectile = ProjectileComponent {
        entity_id,
        target_id,
        damage: tower.damage,
        speed: tower.projectile_speed,
        attack_type: tower.attack_type,
    };
    ctx.db.projectile_component().insert(projectile);

    log::debug!("Tower spawned projectile {} targeting enemy {}", entity_id, target_id);
}

fn handle_enemy_death(ctx: &ReducerContext, entity_id: u64, enemy: &EnemyComponent) {
    // Award gold and score
    if let Some(mut game_state) = ctx.db.game_state().id().find(0) {
        game_state.gold += enemy.gold_reward;
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

    log::debug!("Enemy {} killed, awarded {} gold", entity_id, enemy.gold_reward);
}
