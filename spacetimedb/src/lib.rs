// =============================================================================
// Tower Defense MMO - SpacetimeDB Server Module
// =============================================================================
//
// This is the main entry point for the SpacetimeDB WebAssembly module.
// The server is organized into the following modules:
//
// - tables/    : Database table definitions
// - reducers/  : Client-callable actions
// - agents/    : Scheduled game loops (wave spawning, combat, etc.)
// - helpers/   : Shared utility functions
//
// =============================================================================

use spacetimedb::{ReducerContext, Table};

// Module declarations
pub mod tables;
pub mod reducers;
pub mod agents;
pub mod helpers;

// Re-export commonly used types for client bindings
pub use tables::user::{User, Color};
pub use tables::message::Message;

// Import table access traits
use tables::user::user as UserTable;
use tables::game_state::{GameState, game_state as GameStateTable};
use tables::wave::{WaveState, wave_state as WaveStateTable};

// =============================================================================
// Lifecycle Hooks
// =============================================================================

/// Called once when the module is first published
#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    log::info!("Tower Defense server initialized!");

    // Initialize singleton tables
    init_game_state(ctx);
    init_wave_state(ctx);

    // Initialize all agents (scheduled game loops)
    agents::init(ctx);
}

/// Initialize the game state singleton
fn init_game_state(ctx: &ReducerContext) {
    if ctx.db.game_state().id().find(0).is_none() {
        ctx.db.game_state().insert(GameState::default());
        log::info!("Game state initialized");
    }
}

/// Initialize the wave state singleton
fn init_wave_state(ctx: &ReducerContext) {
    if ctx.db.wave_state().id().find(0).is_none() {
        ctx.db.wave_state().insert(WaveState::default());
        log::info!("Wave state initialized");
    }
}

/// Called when a client connects
#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    let identity = ctx.sender;

    // Check if user already exists
    if let Some(mut user) = ctx.db.user().identity().find(identity) {
        // Returning user - update online status
        user.online = true;
        ctx.db.user().identity().update(user);
        log::info!("User reconnected: {:?}", identity);
    } else {
        // New user - create with default values
        let new_user = User {
            identity,
            name: None,
            color: Color::Purple,
            online: true,
        };
        ctx.db.user().insert(new_user);
        log::info!("New user connected: {:?}", identity);
    }
}

/// Called when a client disconnects
#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    let identity = ctx.sender;

    if let Some(mut user) = ctx.db.user().identity().find(identity) {
        user.online = false;
        ctx.db.user().identity().update(user);
        log::info!("User disconnected: {:?}", identity);
    } else {
        log::warn!("Disconnect for unknown user: {:?}", identity);
    }
}
