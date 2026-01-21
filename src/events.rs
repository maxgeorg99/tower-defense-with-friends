use bevy::prelude::*;

/// Event fired when an enemy is killed
#[derive(Message)]
pub struct EnemyKilled {
    pub gold_reward: i32,
}

/// Event fired when an enemy reaches the end of the path
#[derive(Message)]
pub struct EnemyReachedEnd {
    pub damage: i32,
}

/// Event fired when a wave is complete
#[derive(Message)]
pub struct WaveComplete {
    pub wave: i32,
}

/// Event fired when a tower is placed
#[derive(Message)]
pub struct TowerPlaced {
    pub tower_type: String,
    pub position: Vec2,
}

/// Event fired when a projectile is fired
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
            app.add_event::<EnemyKilled>()
                .add_event::<EnemyReachedEnd>()
                .add_event::<WaveComplete>()
                .add_event::<TowerPlaced>()
                .add_event::<ProjectileFired>();
        }
    }
}
