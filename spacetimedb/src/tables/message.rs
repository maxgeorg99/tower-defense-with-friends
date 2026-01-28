// =============================================================================
// Message Table
// =============================================================================

use spacetimedb::{Identity, Timestamp};

/// Chat message
#[spacetimedb::table(name = message, public)]
pub struct Message {
    pub sender: Identity,
    pub sent: Timestamp,
    pub text: String,
}
