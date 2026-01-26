use bevy::prelude::*;
use std::collections::HashSet;

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

/// Create the set of blocked tiles (road path + castle)
pub fn create_blocked_tiles() -> (HashSet<(i32, i32)>, HashSet<(i32, i32)>) {
    let mut road_tiles = HashSet::new();

    // Road path segments (same waypoints as path, but we fill in all tiles between them)
    let waypoints: Vec<(i32, i32)> = vec![
        (0, 10),
        (5, 10),
        (5, 4),
        (11, 4),
        (11, 16),
        (17, 16),
        (17, 4),
        (23, 4),
        (23, 11),
        (26, 11),
    ];

    // Fill in road tiles between waypoints
    for i in 0..waypoints.len() - 1 {
        let (x1, y1) = waypoints[i];
        let (x2, y2) = waypoints[i + 1];

        // Add tiles along the path (horizontal or vertical segments)
        if x1 == x2 {
            // Vertical segment
            let (min_y, max_y) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
            for y in min_y..=max_y {
                road_tiles.insert((x1, y));
                // Road is ~2 tiles wide
                road_tiles.insert((x1 - 1, y));
                road_tiles.insert((x1 + 1, y));
            }
        } else {
            // Horizontal segment
            let (min_x, max_x) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
            for x in min_x..=max_x {
                road_tiles.insert((x, y1));
                // Road is ~2 tiles wide
                road_tiles.insert((x, y1 - 1));
                road_tiles.insert((x, y1 + 1));
            }
        }
    }

    // Castle tiles (castle is at world pos 400, 0 which is roughly tile 27, 10)
    // Castle is 4 tiles wide/tall
    let mut castle_tiles = HashSet::new();
    let castle_center_x = 27;
    let castle_center_y = 10;
    for dx in -2..=2 {
        for dy in -2..=2 {
            castle_tiles.insert((castle_center_x + dx, castle_center_y + dy));
        }
    }

    // Add castle tiles to blocked tiles too
    let all_blocked: HashSet<(i32, i32)> = road_tiles.union(&castle_tiles).cloned().collect();

    (all_blocked, castle_tiles)
}
