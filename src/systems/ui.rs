use bevy::prelude::*;

use crate::components::{AnimationTimer, Enemy, GameUI, HealthBar};
use crate::constants::SCALED_TILE_SIZE;
use crate::resources::GameState;
use crate::systems::AnimationInfo;

pub fn update_ui(game_state: Res<GameState>, mut ui_query: Query<&mut Text, With<GameUI>>) {
    for mut text in ui_query.iter_mut() {
        **text = format!(
            "Lives: {} | Gold: {} | Wave: {} | Score: {}",
            game_state.lives, game_state.gold, game_state.wave, game_state.score
        );
    }
}

pub fn update_health_bars(
    enemies: Query<(&Enemy, &Children)>,
    mut health_bars: Query<(&mut Sprite, &HealthBar)>,
) {
    for (enemy, children) in enemies.iter() {
        for child in children.iter() {
            if let Ok((mut sprite, health_bar)) = health_bars.get_mut(child) {
                // Update health bar width based on current health
                let health_percent = (enemy.health / health_bar.max_health).clamp(0.0, 1.0);
                sprite.custom_size = Some(Vec2::new(SCALED_TILE_SIZE * health_percent, 4.0));

                // Update color based on health percentage
                sprite.color = if health_percent > 0.6 {
                    Color::srgb(0.0, 1.0, 0.0) // Green
                } else if health_percent > 0.3 {
                    Color::srgb(1.0, 0.6, 0.0) // Orange
                } else {
                    Color::srgb(1.0, 0.0, 0.0) // Red
                };
            }
        }
    }
}
pub fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationInfo)>,
) {
    for (mut timer, mut sprite, anim_info) in query.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                // Cycle through frames based on the unit's frame_count
                atlas.index = (atlas.index + 1) % anim_info.frame_count;
            }
        }
    }
}
