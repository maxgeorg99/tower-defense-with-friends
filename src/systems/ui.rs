use bevy::prelude::*;

use crate::components::{AnimationTimer, Enemy, GameUI, HealthBar, HealthBarFill};
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
    health_bars: Query<(&HealthBar, &Children)>,
    mut fills: Query<(&mut Transform, &mut Sprite, &HealthBarFill)>,
) {
    for (enemy, enemy_children) in enemies.iter() {
        for child in enemy_children.iter() {
            if let Ok((health_bar, bar_children)) = health_bars.get(child) {
                let health_percent = (enemy.health / health_bar.max_health).max(0.0).min(1.0);

                for fill_child in bar_children.iter() {
                    if let Ok((mut fill_transform, mut fill_sprite, health_bar_fill)) = fills.get_mut(fill_child) {
                        if let Some(ref mut size) = fill_sprite.custom_size {
                            size.x = health_bar_fill.max_width * health_percent;

                            let offset = (health_bar_fill.max_width - size.x) / 2.0;
                            fill_transform.translation.x = -offset;
                        }

                        fill_sprite.color = if health_percent > 0.6 {
                            Color::srgb(0.0, 1.0, 0.0)
                        } else if health_percent > 0.3 {
                            Color::srgb(1.0, 1.0, 0.0)
                        } else {
                            Color::srgb(1.0, 0.0, 0.0)
                        };
                    }
                }
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
