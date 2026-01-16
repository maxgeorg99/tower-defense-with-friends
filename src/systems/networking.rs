use bevy::prelude::*;
use bevy_spacetimedb::*;
use spacetimedb_sdk::Table;

use crate::module_bindings::user_table::UserTableAccess;
use crate::module_bindings::{DbConnection, User};

/// Type alias for cleaner SpacetimeDB resource access
pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

/// Marker component for the online users UI panel
#[derive(Component)]
pub struct OnlineUsersPanel;

/// Marker component for individual user list items
#[derive(Component)]
pub struct UserListItem;

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

/// Setup the online users UI panel
pub fn setup_online_users_ui(mut commands: Commands) {
    // Create a panel in the top-right corner for online users
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(5.0),
                min_width: Val::Px(150.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            OnlineUsersPanel,
        ))
        .with_children(|parent| {
            // Header
            parent.spawn((
                Text::new("Online Users"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// System to update the online users list UI
pub fn update_online_users_ui(
    stdb: Option<SpacetimeDB>,
    mut commands: Commands,
    panel_query: Query<Entity, With<OnlineUsersPanel>>,
    user_items_query: Query<Entity, With<UserListItem>>,
) {
    let Some(stdb) = stdb else {
        return;
    };

    // Only update if we have a panel
    let Ok(panel_entity) = panel_query.single() else {
        return;
    };

    // Remove existing user list items
    for entity in user_items_query.iter() {
        commands.entity(entity).despawn();
    }

    // Get all online users from the database cache
    let users: Vec<User> = stdb.db().user().iter().filter(|u| u.online).collect();

    // Add user items to the panel
    commands.entity(panel_entity).with_children(|parent| {
        if users.is_empty() {
            parent.spawn((
                Text::new("No users online"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                UserListItem,
            ));
        } else {
            for user in users {
                let name = user.name.as_deref().unwrap_or("Anonymous");
                let display_text = format!("• {}", name);

                parent.spawn((
                    Text::new(display_text),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.0, 1.0, 0.0, 1.0)),
                    UserListItem,
                ));
            }
        }
    });
}
