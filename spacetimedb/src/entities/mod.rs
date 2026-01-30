// =============================================================================
// Database Tables
// =============================================================================
//
// All SpacetimeDB table definitions are organized here.
// Each table has its own file for clarity.
//
// Tables:
// - user.rs     : Player accounts and profiles
// - message.rs  : Chat messages
//
// Future tables for tower defense:
// - game_session.rs : Active game sessions
// - tower.rs        : Placed towers
// - enemy.rs        : Active enemies
// - wave.rs         : Wave state
//
// =============================================================================

pub mod user;
pub mod message;

// Re-export tables for convenience
pub use user::*;
pub use message::*;
