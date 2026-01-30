use bevy::prelude::*;
use bevy::ecs::prelude::ChildSpawnerCommands;
use bevy_spacetimedb::StdbConnection;
use spacetimedb_sdk::Table;
use crate::components::{RecruitMenu, RecruitOption};
use crate::map::world_to_tile;
use crate::module_bindings::{DbConnection, Color as PlayerColor, MyUserTableAccess};
use crate::resources::{BlockedTiles, GameState, RecruitMenuState, TowerWheelState};
use crate::systems::SoundEffect;

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

/// Recruitable unit definition
struct RecruitableUnit {
    id: &'static str,
    name: &'static str,
    /// Sprite path template with {color} placeholder
    sprite_path: &'static str,
    frame_size: (u32, u32),
    meat_cost: i32,
}

const RECRUITABLE_UNITS: &[RecruitableUnit] = &[
    RecruitableUnit {
        id: "warrior",
        name: "WARRIOR",
        sprite_path: "Units/{color} Units/Warrior/Warrior_Idle.png",
        frame_size: (192, 192),
        meat_cost: 5,
    },
    RecruitableUnit {
        id: "lancer",
        name: "LANCER",
        sprite_path: "Units/{color} Units/Lancer/Lancer_Idle.png",
        frame_size: (320, 320),
        meat_cost: 3,
    },
    RecruitableUnit {
        id: "archer",
        name: "ARCHER",
        sprite_path: "Units/{color} Units/Archer/Archer_Idle.png",
        frame_size: (192, 192),
        meat_cost: 2,
    },
    RecruitableUnit {
        id: "monk",
        name: "MONK",
        sprite_path: "Units/{color} Units/Monk/Idle.png",
        frame_size: (192, 192),
        meat_cost: 3,
    },
];

fn get_color_dir(color: PlayerColor) -> &'static str {
    match color {
        PlayerColor::Blue => "Blue",
        PlayerColor::Yellow => "Yellow",
        PlayerColor::Purple => "Purple",
        PlayerColor::Black => "Black",
    }
}

fn get_player_color(stdb: &Option<SpacetimeDB>) -> PlayerColor {
    stdb.as_ref()
        .and_then(|db| db.db().my_user().iter().next())
        .map(|user| user.color)
        .unwrap_or(PlayerColor::Blue)
}

/// Show recruit menu when clicking on the castle
pub fn show_recruit_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut menu_state: ResMut<RecruitMenuState>,
    tower_wheel_state: Res<TowerWheelState>,
    blocked_tiles: Res<BlockedTiles>,
    existing_menus: Query<Entity, With<RecruitMenu>>,
    stdb: Option<SpacetimeDB>,
) {
    // Don't show if tower wheel is active or recruit menu already open
    if !mouse_button.just_pressed(MouseButton::Left) || menu_state.active || tower_wheel_state.active {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    // Check if clicked on castle tile
    let (tile_x, tile_y) = world_to_tile(world_pos);
    if blocked_tiles.is_castle(tile_x, tile_y) {
        // Clean up existing menus
        for entity in existing_menus.iter() {
            commands.entity(entity).despawn();
        }

        menu_state.active = true;
        let color = get_player_color(&stdb);
        spawn_recruit_menu(&mut commands, &asset_server, &mut texture_atlases, color);
    }
}

fn spawn_recruit_menu(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    player_color: PlayerColor,
) {
    // Load meat icon for cost display
    let meat_icon = asset_server.load("Terrain/Resources/Meat/Meat Resource/Meat Resource.png");
    let color_dir = get_color_dir(player_color);

    // Pre-load unit textures and create atlas layouts
    let mut unit_images: Vec<(Handle<Image>, Handle<TextureAtlasLayout>)> = Vec::new();
    for unit in RECRUITABLE_UNITS {
        let path = unit.sprite_path.replace("{color}", color_dir);
        let texture = asset_server.load(&path);
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(unit.frame_size.0, unit.frame_size.1),
            1, 1, // We only want the first frame
            None, None,
        );
        let layout_handle = texture_atlases.add(layout);
        unit_images.push((texture, layout_handle));
    }

    // Main menu container (centered overlay)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            RecruitMenu,
        ))
        .with_children(|parent| {
            // Menu background panel
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(15.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.15, 0.2, 0.95)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("Recruit Units"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Unit cards container
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(15.0),
                            ..default()
                        })
                        .with_children(|cards_row| {
                            for (i, unit) in RECRUITABLE_UNITS.iter().enumerate() {
                                let (texture, layout) = unit_images[i].clone();
                                spawn_unit_card(cards_row, &meat_icon, unit, texture, layout);
                            }
                        });

                    // Close hint
                    panel.spawn((
                        Text::new("Right-click or ESC to close"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));
                });
        });
}

fn spawn_unit_card(
    parent: &mut ChildSpawnerCommands,
    meat_icon: &Handle<Image>,
    unit: &RecruitableUnit,
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(8.0),
                width: Val::Px(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.35, 0.45, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|card: &mut ChildSpawnerCommands| {
            // Avatar image (first frame of sprite sheet)
            card.spawn((
                ImageNode::from_atlas_image(texture, TextureAtlas::from(layout)),
                Node {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    ..default()
                },
            ));

            // Unit name
            card.spawn((
                Text::new(unit.name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Build button
            card.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.4, 0.3, 1.0)),
                BorderRadius::all(Val::Px(4.0)),
                RecruitOption {
                    unit_id: unit.id.to_string(),
                    meat_cost: unit.meat_cost,
                },
                Button,
            ))
            .with_children(|button: &mut ChildSpawnerCommands| {
                button.spawn((
                    Text::new("Build"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                // Cost row with icon
                button
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|cost_row: &mut ChildSpawnerCommands| {
                        cost_row.spawn((
                            Text::new(format!("{}", unit.meat_cost)),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.6, 0.6)),
                        ));

                        cost_row.spawn((
                            ImageNode::new(meat_icon.clone()),
                            Node {
                                width: Val::Px(16.0),
                                height: Val::Px(16.0),
                                ..default()
                            },
                        ));
                    });
            });
        });
}

/// Hide recruit menu on right-click or escape
pub fn hide_recruit_menu(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<RecruitMenuState>,
    menu_entities: Query<Entity, With<RecruitMenu>>,
) {
    if !menu_state.active {
        return;
    }

    if mouse_button.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
        for entity in menu_entities.iter() {
            commands.entity(entity).despawn();
        }
        menu_state.active = false;
    }
}

/// Handle clicking on recruit buttons
pub fn handle_recruit_selection(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &RecruitOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<GameState>,
    mut menu_state: ResMut<RecruitMenuState>,
    menu_entities: Query<Entity, With<RecruitMenu>>,
    mut sound_events: EventWriter<SoundEffect>,
) {
    for (interaction, option) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            sound_events.write(SoundEffect::ButtonClick);
            if game_state.meat >= option.meat_cost {
                game_state.meat -= option.meat_cost;
                info!(
                    "Recruited unit: {} for {} meat",
                    option.unit_id, option.meat_cost
                );

                // TODO: Actually spawn the recruited unit

                // Close menu after successful recruitment
                for entity in menu_entities.iter() {
                    commands.entity(entity).despawn();
                }
                menu_state.active = false;
            } else {
                info!(
                    "Not enough meat to recruit {}. Need {}, have {}",
                    option.unit_id, option.meat_cost, game_state.meat
                );
            }
        }
    }
}
