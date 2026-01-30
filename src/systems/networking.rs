use bevy::prelude::*;
use bevy_spacetimedb::*;
use spacetimedb_sdk::Table;

use crate::module_bindings::{
    DbConnection, User, GameState as DbGameState,
    MyUserTableAccess,
};
use crate::resources::GameState;

/// Type alias for cleaner SpacetimeDB resource access
pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

/// System to handle SpacetimeDB connection events
pub fn on_connected(messages: Option<ReadStdbConnectedMessage>, stdb: Option<SpacetimeDB>) {
    let (Some(mut messages), Some(stdb)) = (messages, stdb) else {
        return;
    };
    for _ in messages.read() {
        info!("Connected to SpacetimeDB!");

        // Subscribe to user tables
        stdb.subscription_builder()
            .on_applied(|_| info!("User subscription applied"))
            .on_error(|_, err| error!("User subscription failed: {}", err))
            .subscribe("SELECT * FROM user");

        stdb.subscription_builder()
            .on_applied(|_| info!("My User subscription applied"))
            .on_error(|_, err| error!("My User subscription failed: {}", err))
            .subscribe("SELECT * FROM my_user");

        // Subscribe to game state (singleton - resources, lives, score)
        stdb.subscription_builder()
            .on_applied(|_| info!("Game state subscription applied"))
            .on_error(|_, err| error!("Game state subscription failed: {}", err))
            .subscribe("SELECT * FROM game_state");

        // Subscribe to wave state
        stdb.subscription_builder()
            .on_applied(|_| info!("Wave state subscription applied"))
            .on_error(|_, err| error!("Wave state subscription failed: {}", err))
            .subscribe("SELECT * FROM wave_state");

        // Subscribe to all game entities (towers, enemies, projectiles)
        stdb.subscription_builder()
            .on_applied(|_| info!("Game entity subscription applied"))
            .on_error(|_, err| error!("Game entity subscription failed: {}", err))
            .subscribe("SELECT * FROM game_entity");

        // Subscribe to tower components
        stdb.subscription_builder()
            .on_applied(|_| info!("Tower component subscription applied"))
            .on_error(|_, err| error!("Tower component subscription failed: {}", err))
            .subscribe("SELECT * FROM tower_component");

        // Subscribe to enemy components
        stdb.subscription_builder()
            .on_applied(|_| info!("Enemy component subscription applied"))
            .on_error(|_, err| error!("Enemy component subscription failed: {}", err))
            .subscribe("SELECT * FROM enemy_component");

        // Subscribe to projectile components
        stdb.subscription_builder()
            .on_applied(|_| info!("Projectile component subscription applied"))
            .on_error(|_, err| error!("Projectile component subscription failed: {}", err))
            .subscribe("SELECT * FROM projectile_component");

        // Subscribe to path waypoints (for enemy movement)
        stdb.subscription_builder()
            .on_applied(|_| info!("Path waypoint subscription applied"))
            .on_error(|_, err| error!("Path waypoint subscription failed: {}", err))
            .subscribe("SELECT * FROM path_waypoint");

        // Subscribe to tower type definitions
        stdb.subscription_builder()
            .on_applied(|_| info!("Tower type def subscription applied"))
            .on_error(|_, err| error!("Tower type def subscription failed: {}", err))
            .subscribe("SELECT * FROM tower_type_def");

        // Subscribe to enemy type definitions
        stdb.subscription_builder()
            .on_applied(|_| info!("Enemy type def subscription applied"))
            .on_error(|_, err| error!("Enemy type def subscription failed: {}", err))
            .subscribe("SELECT * FROM enemy_type_def");
    }
}

/// System to handle SpacetimeDB disconnection events
pub fn on_disconnected(messages: Option<ReadStdbDisconnectedMessage>) {
    let Some(mut messages) = messages else {
        return;
    };
    for _ in messages.read() {
        warn!("Disconnected from SpacetimeDB");
    }
}

/// System to handle SpacetimeDB connection errors
pub fn on_connection_error(messages: Option<ReadStdbConnectionErrorMessage>) {
    let Some(mut messages) = messages else {
        return;
    };
    for msg in messages.read() {
        error!("SpacetimeDB connection error: {:?}", msg.err);
    }
}

/// System to handle new users being inserted
pub fn on_user_inserted(messages: Option<ReadInsertMessage<User>>) {
    let Some(mut messages) = messages else {
        return;
    };
    for msg in messages.read() {
        let name = msg.row.name.as_deref().unwrap_or("Anonymous");
        info!(
            "User joined: {} (online: {})",
            name, msg.row.online
        );
    }
}

/// System to handle users being updated (name change, online status)
pub fn on_user_updated(messages: Option<ReadUpdateMessage<User>>) {
    let Some(mut messages) = messages else {
        return;
    };
    for msg in messages.read() {
        let old_name = msg.old.name.as_deref().unwrap_or("Anonymous");
        let new_name = msg.new.name.as_deref().unwrap_or("Anonymous");
        info!(
            "User updated: {} -> {} (online: {} -> {})",
            old_name, new_name, msg.old.online, msg.new.online
        );
    }
}

/// System to handle users being deleted
pub fn on_user_deleted(messages: Option<ReadDeleteMessage<User>>) {
    let Some(mut messages) = messages else {
        return;
    };
    for msg in messages.read() {
        let name = msg.row.name.as_deref().unwrap_or("Anonymous");
        info!("User removed: {}", name);
    }
}

// =============================================================================
// Game State Sync
// =============================================================================

/// Sync game state from server to local resource (lives and score only - resources are per-player)
pub fn sync_game_state(
    messages: Option<ReadInsertUpdateMessage<DbGameState>>,
    mut game_state: ResMut<GameState>,
) {
    let Some(mut messages) = messages else {
        return;
    };
    for msg in messages.read() {
        let db_state = &msg.new;
        game_state.lives = db_state.lives;
        game_state.score = db_state.score;
        info!(
            "Game state synced: lives={}, score={}",
            db_state.lives, db_state.score
        );
    }
}

/// Sync player resources (gold, wood, meat) from the User table
pub fn sync_player_resources(
    messages: Option<ReadInsertUpdateMessage<User>>,
    mut game_state: ResMut<GameState>,
    stdb: Option<SpacetimeDB>,
) {
    // Check for direct updates to our user
    if let Some(mut messages) = messages {
        for msg in messages.read() {
            let user = &msg.new;
            // This fires for all user updates, but we only want our own
            // Use the my_user view for this
            if let Some(ref stdb) = stdb {
                if let Some(my_user) = stdb.db().my_user().iter().next() {
                    if user.identity == my_user.identity {
                        game_state.gold = user.gold;
                        game_state.wood = user.wood;
                        game_state.meat = user.meat;
                        info!(
                            "Player resources synced: gold={}, wood={}, meat={}",
                            user.gold, user.wood, user.meat
                        );
                    }
                }
            }
        }
    }
}

/// WASM-only system to process SpacetimeDB messages each frame.
/// On native, message processing happens in a background thread via run_threaded().
/// On WASM, we don't have threads, so we process messages synchronously each frame.
#[cfg(target_arch = "wasm32")]
pub fn process_stdb_messages(stdb: Option<SpacetimeDB>) {
    let Some(stdb) = stdb else {
        return;
    };

    // Process all pending WebSocket messages via the underlying connection
    if let Err(e) = stdb.conn().frame_tick() {
        // Only log if it's not a normal disconnection
        warn!("SpacetimeDB frame_tick error: {:?}", e);
    }
}
