use bevy::prelude::*;
use bevy_spacetimedb::*;
use spacetimedb_sdk::Table;

use crate::module_bindings::user_table::UserTableAccess;
use crate::module_bindings::{DbConnection, User, Color as PlayerColor};

/// Type alias for cleaner SpacetimeDB resource access
pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

/// Marker component for the online users panel
#[derive(Component)]
pub struct OnlineUsersPanel;

/// Marker component for individual player banner items
#[derive(Component)]
pub struct PlayerBanner;

/// Setup the online users UI panel
pub fn setup_online_users_ui(mut commands: Commands) {
    // Create a panel in the center-right area for online users
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(20.0),
                right: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                width: Val::Px(320.0),
                ..default()
            },
            OnlineUsersPanel,
        ));
}

/// System to update the online users list UI
pub fn update_online_users_ui(
    stdb: Option<SpacetimeDB>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    panel_query: Query<Entity, With<OnlineUsersPanel>>,
    banner_items_query: Query<Entity, With<PlayerBanner>>,
) {
    let Some(stdb) = stdb else {
        return;
    };

    // Only update if we have a panel
    let Ok(panel_entity) = panel_query.single() else {
        return;
    };

    // Remove existing player banners
    for entity in banner_items_query.iter() {
        commands.entity(entity).despawn();
    }

    // Get all online users from the database cache
    let mut users: Vec<User> = stdb.db().user().iter().filter(|u| u.online).collect();

    // Sort users by name for consistent ordering
    users.sort_by(|a, b| {
        let name_a = a.name.as_deref().unwrap_or("Anonymous");
        let name_b = b.name.as_deref().unwrap_or("Anonymous");
        name_a.cmp(name_b)
    });

    // Add player banners to the panel
    commands.entity(panel_entity).with_children(|parent| {
        if users.is_empty() {
            // Show "waiting for players" message
            parent.spawn((
                Text::new("Waiting for players..."),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                PlayerBanner,
            ));
        } else {
            for user in users {
                spawn_player_banner(parent, &user, &asset_server);
            }
        }
    });
}

/// Spawns a single player banner with styled background and avatar
fn spawn_player_banner(parent: &mut ChildSpawner, user: &User, asset_server: &AssetServer) {
    let name = user.name.as_deref().unwrap_or("Anonymous");

    // Get player color paths from enum
    let (ribbon_path, avatar_path) = match user.color {
        PlayerColor::Blue => (
            "UI Elements/UI Elements/Ribbons/Ribbon_Blue.png",
            "UI Elements/UI Elements/Human Avatars/Avatar_Blue.png"
        ),
        PlayerColor::Yellow => (
            "UI Elements/UI Elements/Ribbons/Ribbon_Yellow.png",
            "UI Elements/UI Elements/Human Avatars/Avatar_Yellow.png"
        ),
        PlayerColor::Purple => (
            "UI Elements/UI Elements/Ribbons/Ribbon_Purple.png",
            "UI Elements/UI Elements/Human Avatars/Avatar_Purple.png"
        ),
        PlayerColor::Black => (
            "UI Elements/UI Elements/Ribbons/Ribbon_Black.png",
            "UI Elements/UI Elements/Human Avatars/Avatar_Black.png"
        ),
    };

    parent
        .spawn_with_children((
                                 Node {
                                     width: Val::Percent(100.0),
                                     height: Val::Px(80.0),
                                     align_items: AlignItems::Center,
                                     justify_content: JustifyContent::Start,
                                     position_type: PositionType::Relative,
                                     ..default()
                                 },
                                 PlayerBanner,
                             ), |banner| {
            // Ribbon background image
            banner.spawn((
                ImageNode::new(asset_server.load(ribbon_path)),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));

            // Avatar container (left side) - positioned on top of ribbon
            banner
                .spawn_with_children((
                                         Node {
                                             width: Val::Px(56.0),
                                             height: Val::Px(56.0),
                                             margin: UiRect {
                                                 left: Val::Px(12.0),
                                                 right: Val::Px(15.0),
                                                 ..default()
                                             },
                                             justify_content: JustifyContent::Center,
                                             align_items: AlignItems::Center,
                                             ..default()
                                         },
                                         BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                                         BorderRadius::all(Val::Px(8.0)),
                                     ), |avatar_container| {
                    // Avatar image
                    avatar_container.spawn((
                        ImageNode::new(asset_server.load(avatar_path)),
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                    ));
                });

            banner.spawn((
                Text::new(name.to_uppercase()),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout {
                    justify: Justify::Left,
                    ..default()
                },
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
        });
}