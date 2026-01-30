// =============================================================================
// Database Tables
// =============================================================================
//
// All SpacetimeDB table definitions are organized here.
// Each table has its own file for clarity.
//
// Tables:
// - user.rs        : Player accounts and profiles
// - message.rs     : Chat messages
// - game_entity.rs : Base entity table (ECS-style)
// - components.rs  : Entity component tables (Tower, Enemy, Projectile, Unit)
// - wave.rs        : Wave state and configuration
// - game_state.rs  : Global game state (resources, lives, score)
//
// =============================================================================

pub mod user;
pub mod message;
pub mod game_entity;
pub mod components;
pub mod wave;
pub mod game_state;
pub mod worker;

// Re-export tables for convenience
pub use user::*;
pub use message::*;
pub use game_entity::*;
pub use components::*;
pub use wave::*;
pub use game_state::*;
pub use worker::*;
