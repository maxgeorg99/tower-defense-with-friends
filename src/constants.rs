use bevy::prelude::*;

// Tile and map scaling
pub const TILE_SIZE: f32 = 16.0;
pub const MAP_SCALE: f32 = 2.0;
pub const SCALED_TILE_SIZE: f32 = TILE_SIZE * MAP_SCALE; // 32 pixels

// Asset dimensions
pub const TOWER_SIZE: Vec2 = Vec2::new(128.0, 256.0);
pub const ARROW_SIZE: Vec2 = Vec2::new(64.0, 64.0);
pub const CASTLE_SIZE: Vec2 = Vec2::new(320.0, 256.0);

// Map dimensions
pub const MAP_WIDTH: i32 = 30;
pub const MAP_HEIGHT: i32 = 20;

// Fog of war
pub const EXPLORE_COST: i32 = 50;
pub const EXPLORE_RADIUS: i32 = 4;
