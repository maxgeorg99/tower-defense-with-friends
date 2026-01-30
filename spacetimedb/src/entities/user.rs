// =============================================================================
// User Table
// =============================================================================

use spacetimedb::{Identity, SpacetimeType};

/// Player color selection
#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Blue,
    Yellow,
    Purple,
    Black,
}

/// Player account and profile
#[spacetimedb::table(name = user, public)]
pub struct User {
    #[primary_key]
    pub identity: Identity,
    pub name: Option<String>,
    pub color: Color,
    pub online: bool,
}

/// View that returns the current user's profile
/// Allows clients to query only their own data
#[spacetimedb::view(name = my_user, public)]
fn my_user(ctx: &spacetimedb::ViewContext) -> Option<User> {
    ctx.db.user().identity().find(ctx.sender)
}
