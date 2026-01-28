// =============================================================================
// Chat Reducers
// =============================================================================

use spacetimedb::{ReducerContext, Table};
use crate::tables::message::{Message, message as MessageTable};
use crate::helpers::validation;

/// Send a chat message
#[spacetimedb::reducer]
pub fn send_message(ctx: &ReducerContext, text: String) -> Result<(), String> {
    // Validate message
    let validated_text = validation::validate_message(&text)?;

    // Insert message
    ctx.db.message().insert(Message {
        sender: ctx.sender,
        sent: ctx.timestamp,
        text: validated_text,
    });

    log::info!("Message from {:?}: {}", ctx.sender, text);
    Ok(())
}
