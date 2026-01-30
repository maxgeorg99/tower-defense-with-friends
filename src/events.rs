use bevy::prelude::*;

// These events are registered for future use

/// Event fired when an enemy is killed
#[allow(dead_code)]
#[derive(Message)]
pub struct EnemyKilled {
    pub gold_reward: i32,
}

/// Event fired when an enemy reaches the end of the path
#[allow(dead_code)]
#[derive(Message)]
pub struct EnemyReachedEnd {
    pub damage: i32,
}

/// Event fired when a wave is complete
#[allow(dead_code)]
#[derive(Message)]
pub struct WaveComplete {
    pub wave: i32,
}

/// Event fired when a tower is placed
#[allow(dead_code)]
#[derive(Message)]
pub struct TowerPlaced {
    pub tower_type: String,
    pub position: Vec2,
}

/// Event fired when a projectile is fired
#[allow(dead_code)]
#[derive(Message)]
pub struct ProjectileFired {
    pub tower: Entity,
    pub target: Entity,
}

/// Plugin that registers all game events
pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        #[allow(deprecated)]
        {
            app.add_message::<EnemyKilled>()
                .add_message::<EnemyReachedEnd>()
                .add_message::<WaveComplete>()
                .add_message::<TowerPlaced>()
                .add_message::<ProjectileFired>();
        }
    }
}
