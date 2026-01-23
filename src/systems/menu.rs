use bevy::prelude::*;
use crate::resources::AppState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    #[default]
    BigBlue,
    BigRed,
    SmallBlueRound,
    SmallRedRound,
    SmallBlueSquare,
    SmallRedSquare,
}

impl ButtonStyle {
    pub fn regular_texture(&self) -> &'static str {
        match self {
            Self::BigBlue => "UI Elements/UI Elements/Buttons/BigBlueButton_Regular.png",
            Self::BigRed => "UI Elements/UI Elements/Buttons/BigRedButton_Regular.png",
            Self::SmallBlueRound => "UI Elements/UI Elements/Buttons/SmallBlueRoundButton_Regular.png",
            Self::SmallRedRound => "UI Elements/UI Elements/Buttons/SmallRedRoundButton_Regular.png",
            Self::SmallBlueSquare => "UI Elements/UI Elements/Buttons/SmallBlueSquareButton_Regular.png",
            Self::SmallRedSquare => "UI Elements/UI Elements/Buttons/SmallRedSquareButton_Regular.png",
        }
    }

    pub fn pressed_texture(&self) -> &'static str {
        match self {
            Self::BigBlue => "UI Elements/UI Elements/Buttons/BigBlueButton_Pressed.png",
            Self::BigRed => "UI Elements/UI Elements/Buttons/BigRedButton_Pressed.png",
            Self::SmallBlueRound => "UI Elements/UI Elements/Buttons/SmallBlueRoundButton_Pressed.png",
            Self::SmallRedRound => "UI Elements/UI Elements/Buttons/SmallRedRoundButton_Pressed.png",
            Self::SmallBlueSquare => "UI Elements/UI Elements/Buttons/SmallBlueSquareButton_Pressed.png",
            Self::SmallRedSquare => "UI Elements/UI Elements/Buttons/SmallRedSquareButton_Pressed.png",
        }
    }

    pub fn grid_params(&self) -> (f32, f32) {
        match self {
            Self::BigBlue | Self::BigRed => (105.0, 2.5),
            _ => (42.67, 0.0),
        }
    }

    pub fn default_size(&self) -> (Val, Val) {
        match self {
            Self::BigBlue | Self::BigRed => (Val::Px(280.0), Val::Px(80.0)),
            Self::SmallBlueRound | Self::SmallRedRound => (Val::Px(200.0), Val::Px(55.0)),
            Self::SmallBlueSquare | Self::SmallRedSquare => (Val::Px(160.0), Val::Px(45.0)),
        }
    }

    pub fn corner_display_size(&self) -> f32 {
        match self {
            Self::BigBlue | Self::BigRed => 20.0,
            _ => 14.0,
        }
    }

    pub fn default_font_size(&self) -> f32 {
        match self {
            Self::BigBlue | Self::BigRed => 32.0,
            Self::SmallBlueRound | Self::SmallRedRound => 22.0,
            Self::SmallBlueSquare | Self::SmallRedSquare => 18.0,
        }
    }
}

fn tile_rect(index: usize, tile_size: f32, gap: f32) -> Rect {
    let row = (index / 3) as f32;
    let col = (index % 3) as f32;
    let stride = tile_size + gap;
    Rect::new(
        col * stride,
        row * stride,
        col * stride + tile_size,
        row * stride + tile_size,
    )
}

#[derive(Component)]
pub struct UIButton {
    pub style: ButtonStyle,
}

#[derive(Component)]
struct NineSlicePart;

#[derive(Component)]
struct MenuUI;

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct QuitButton;

#[derive(Component)]
pub struct LoginButton;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<LoginRequestEvent>()
            .add_systems(OnEnter(AppState::MainMenu), setup_menu)
            .add_systems(
                Update,
                (
                    button_interaction::<PlayButton>,
                    button_interaction::<SettingsButton>,
                    button_interaction::<QuitButton>,
                    button_interaction::<LoginButton>,
                    update_nine_slice_textures,
                ).run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnExit(AppState::MainMenu), cleanup_menu);
    }
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Main menu container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.28, 0.32)),
            MenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("TD MMO"),
                TextFont { font_size: 48.0, ..default() },
                TextColor(Color::srgb(0.92, 0.96, 0.96)),
                Node { margin: UiRect::bottom(Val::Px(5.0)), ..default() },
            ));

            // Buttons
            spawn_nine_slice_button(parent, &asset_server, ButtonStyle::SmallBlueRound, "PLAY", PlayButton);
            spawn_nine_slice_button(parent, &asset_server, ButtonStyle::SmallBlueRound, "SETTINGS", SettingsButton);
            spawn_nine_slice_button(parent, &asset_server, ButtonStyle::SmallRedRound, "QUIT", QuitButton);
        });

    // Login button in top-right corner
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            },
            GlobalZIndex(10),
            MenuUI,
        ))
        .with_children(|parent| {
            spawn_nine_slice_button(parent, &asset_server, ButtonStyle::SmallBlueRound, "LOGIN", LoginButton);
        });
}

pub fn spawn_nine_slice_button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    style: ButtonStyle,
    label: &str,
    marker: M,
) {
    let (width, height) = style.default_size();
    let corner = style.corner_display_size();
    let (tile_size, gap) = style.grid_params();
    let texture = asset_server.load(style.regular_texture());
    let font_size = style.default_font_size();

    parent
        .spawn((
            Button,
            Node {
                width,
                height,
                display: Display::Grid,
                grid_template_columns: vec![
                    GridTrack::px(corner),
                    GridTrack::fr(1.0),
                    GridTrack::px(corner),
                ],
                grid_template_rows: vec![
                    GridTrack::px(corner),
                    GridTrack::fr(1.0),
                    GridTrack::px(corner),
                ],
                ..default()
            },
            UIButton { style },
            marker,
        ))
        .with_children(|btn| {
            spawn_grid_tile(btn, texture.clone(), tile_rect(0, tile_size, gap));
            spawn_grid_tile(btn, texture.clone(), tile_rect(1, tile_size, gap));
            spawn_grid_tile(btn, texture.clone(), tile_rect(2, tile_size, gap));

            spawn_grid_tile(btn, texture.clone(), tile_rect(3, tile_size, gap));

            btn.spawn((
                ImageNode {
                    image: texture.clone(),
                    rect: Some(tile_rect(4, tile_size, gap)),
                    ..default()
                },
                Node {
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                NineSlicePart,
            )).with_children(|center| {
                center.spawn((
                    Text::new(label.to_string()),
                    TextFont { font_size, ..default() },
                    TextColor(Color::WHITE),
                ));
            });

            spawn_grid_tile(btn, texture.clone(), tile_rect(5, tile_size, gap));

            spawn_grid_tile(btn, texture.clone(), tile_rect(6, tile_size, gap));
            spawn_grid_tile(btn, texture.clone(), tile_rect(7, tile_size, gap));
            spawn_grid_tile(btn, texture.clone(), tile_rect(8, tile_size, gap));
        });
}

fn spawn_grid_tile(
    parent: &mut ChildSpawnerCommands,
    texture: Handle<Image>,
    rect: Rect,
) {
    parent.spawn((
        ImageNode {
            image: texture,
            rect: Some(rect),
            ..default()
        },
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        NineSlicePart,
    ));
}

fn button_interaction<M: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<M>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    mut login_event: EventWriter<LoginRequestEvent>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            if std::any::type_name::<M>().contains("PlayButton") {
                next_state.set(AppState::ColorSelect);
            } else if std::any::type_name::<M>().contains("QuitButton") {
                exit.write(AppExit::Success);
            } else if std::any::type_name::<M>().contains("SettingsButton") {
                info!("Settings pressed");
            } else if std::any::type_name::<M>().contains("LoginButton") {
                login_event.write(LoginRequestEvent);
            }
        }
    }
}

/// Event to trigger login from menu
#[derive(Message)]
pub struct LoginRequestEvent;

fn update_nine_slice_textures(
    asset_server: Res<AssetServer>,
    button_query: Query<(&Interaction, &UIButton, &Children), Changed<Interaction>>,
    children_query: Query<&Children>,
    mut image_query: Query<&mut ImageNode, With<NineSlicePart>>,
) {
    for (interaction, ui_button, children) in &button_query {
        let path = match interaction {
            Interaction::Pressed => ui_button.style.pressed_texture(),
            _ => ui_button.style.regular_texture(),
        };
        let texture = asset_server.load(path);
        update_children_textures(children, &children_query, &mut image_query, &texture);
    }
}

fn update_children_textures(
    children: &Children,
    children_query: &Query<&Children>,
    image_query: &mut Query<&mut ImageNode, With<NineSlicePart>>,
    texture: &Handle<Image>,
) {
    for child in children.iter() {
        if let Ok(mut img) = image_query.get_mut(child) {
            img.image = texture.clone();
        }
        if let Ok(grandchildren) = children_query.get(child) {
            update_children_textures(grandchildren, children_query, image_query, texture);
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
