use bevy::prelude::*;
use bevy_spacetimedb::*;

use crate::module_bindings::{DbConnection, User};

/// Type alias for cleaner SpacetimeDB resource access
pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

/// System to handle SpacetimeDB connection events
pub fn on_connected(messages: Option<ReadStdbConnectedMessage>, stdb: Option<SpacetimeDB>) {
    let (Some(mut messages), Some(stdb)) = (messages, stdb) else {
        return;
    };
    for _ in messages.read() {
        info!("Connected to SpacetimeDB!");

        // Subscribe to the user table to get all online users
        stdb.subscription_builder()
            .on_applied(|_| info!("User subscription applied"))
            .on_error(|_, err| error!("User subscription failed: {}", err))
            .subscribe("SELECT * FROM user");

        stdb.subscription_builder()
            .on_applied(|_| info!("My User subscription applied"))
            .on_error(|_, err| error!("My User subscription failed: {}", err))
            .subscribe("SELECT * FROM my_user");
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