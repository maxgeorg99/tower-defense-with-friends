use bevy::prelude::*;

use crate::constants::{MAP_HEIGHT, SCALED_TILE_SIZE};

pub fn tile_to_world(tile_x: i32, tile_y: i32) -> Vec2 {
    // Convert tile coordinates to world space
    // Origin of tilemap is at -480, -320 with scale 2.0
    let world_x = -480.0 + (tile_x as f32 * SCALED_TILE_SIZE) + (SCALED_TILE_SIZE / 2.0);
    let world_y =
        -320.0 + ((MAP_HEIGHT - 1 - tile_y) as f32 * SCALED_TILE_SIZE) + (SCALED_TILE_SIZE / 2.0);
    Vec2::new(world_x, world_y)
}

pub fn world_to_tile(world_pos: Vec2) -> (i32, i32) {
    // Convert world coordinates to tile coordinates
    let tile_x = ((world_pos.x + 480.0) / SCALED_TILE_SIZE).floor() as i32;
    let tile_y = MAP_HEIGHT - 1 - ((world_pos.y + 320.0) / SCALED_TILE_SIZE).floor() as i32;
    (tile_x, tile_y)
}

pub fn create_path_waypoints() -> Vec<Vec2> {
    // Based on your tilemap, manually define the path waypoints
    // Starting from left, following the road tiles
    // Converting tile coordinates to world coordinates
    let waypoints = vec![
        (0, 10),  // Start left side, row 10
        (5, 10),  // Move right
        (5, 4),   // Turn up
        (11, 4),  // Move right
        (11, 16), // Move down
        (17, 16), // Move right
        (17, 4),  // Move up
        (23, 4),  // Move right
        (23, 11), // Move down
        (26, 11), // End at castle (right side)
    ];

    // Convert tile coordinates to world positions
    waypoints
        .iter()
        .map(|(x, y)| tile_to_world(*x, *y))
        .collect()
}
