use bevy::prelude::*;

use crate::components::FogTile;
use crate::resources::FogOfWar;

pub fn update_fog_visibility(fog: Res<FogOfWar>, mut fog_tiles: Query<(&FogTile, &mut Visibility)>) {
    if !fog.is_changed() {
        return;
    }

    for (fog_tile, mut visibility) in fog_tiles.iter_mut() {
        let is_explored = fog.is_explored(fog_tile.tile_x, fog_tile.tile_y);
        *visibility = if is_explored {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}
