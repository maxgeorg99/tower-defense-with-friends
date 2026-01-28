// =============================================================================
// User Actions
// =============================================================================

use spacetimedb::ReducerContext;
use crate::tables::user::{Color, user as UserTable};
use crate::helpers::validation;

/// Update the player's display name
#[spacetimedb::reducer]
pub fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    // Validate name
    let validated_name = validation::validate_name(&name)?;

    // Find and update user
    let identity = ctx.sender;
    if let Some(mut user) = ctx.db.user().identity().find(identity) {
        user.name = Some(validated_name);
        ctx.db.user().identity().update(user);
        log::info!("User {:?} changed name to: {}", identity, name);
        Ok(())
    } else {
        Err("User not found".to_string())
    }
}

/// Update the player's color
#[spacetimedb::reducer]
pub fn set_color(ctx: &ReducerContext, color: Color) -> Result<(), String> {
    let identity = ctx.sender;

    if let Some(mut user) = ctx.db.user().identity().find(identity) {
        user.color = color;
        ctx.db.user().identity().update(user);
        log::info!("User {:?} changed color to: {:?}", identity, color);
        Ok(())
    } else {
        Err("User not found".to_string())
    }
}
