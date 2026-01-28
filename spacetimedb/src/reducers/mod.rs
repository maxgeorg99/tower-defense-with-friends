// =============================================================================
// Reducers (Client-Callable Actions)
// =============================================================================
//
// Reducers are functions that clients can call to modify server state.
// They are the only way clients can change data - all game actions
// must go through reducers for validation.
//
// Organization:
// - user_actions.rs   : Profile updates (set_name, set_color)
// - chat.rs           : Chat messages
// - tower_actions.rs  : Place tower, upgrade tower, sell tower
// - wave_actions.rs   : Start wave, skip wave
//
// =============================================================================

pub mod user_actions;
pub mod chat;
pub mod tower_actions;
pub mod wave_actions;
pub mod worker_actions;

// Re-export all reducers so they're visible to SpacetimeDB
pub use user_actions::*;
pub use chat::*;
pub use tower_actions::*;
pub use wave_actions::*;
pub use worker_actions::*;
