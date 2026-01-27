use bevy::prelude::*;

use crate::config::{TowerType, UnitType, Wave};
use crate::constants::{MAP_HEIGHT, MAP_WIDTH};

/// SpacetimeDB connection configuration (for deferred connection)
#[derive(Resource, Clone)]
pub struct StdbConfig {
    pub uri: String,
    pub module: String,
    pub token: Option<String>,
}

#[derive(Resource)]
pub struct GameState {
    pub lives: i32,
    pub gold: i32,
    pub wood: i32,
    pub meat: i32,
    pub wave: i32,
    pub score: i32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            lives: 20,
            gold: 100,
            wood: 0,
            meat: 0,
            wave: 1,
            score: 0,
        }
    }
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    /// Main Menu
    #[default]
    MainMenu,
    /// Color and username selection screen
    ColorSelect,
    /// Main game
    InGame,
    /// Game over screen
    GameOver,
}

#[derive(Resource)]
pub struct EnemySpawner {
    pub timer: Timer,
    pub enemies_this_wave: i32,
    pub enemies_spawned: i32,
}

impl EnemySpawner {
    pub fn from_wave_config(wave: &Wave) -> Self {
        let total_enemies: i32 = wave.spawns.iter().map(|s| s.count).sum();
        Self {
            timer: Timer::from_seconds(wave.spawn_interval, TimerMode::Repeating),
            enemies_this_wave: total_enemies,
            enemies_spawned: 0,
        }
    }
}

#[derive(Resource)]
pub struct PathWaypoints {
    pub points: Vec<Vec2>,
}

#[derive(Resource)]
pub struct WaveConfigs {
    pub units: Vec<UnitType>,
    pub waves: Vec<Wave>,
}

#[derive(Resource)]
pub struct TowerConfigs {
    pub towers: Vec<TowerType>,
}

#[derive(Resource)]
pub struct TowerWheelState {
    pub active: bool,
    pub position: Vec2,
}

#[derive(Resource)]
pub struct FogOfWar {
    /// 2D grid of explored tiles (true = explored, false = fog)
    pub explored: Vec<Vec<bool>>,
}

impl FogOfWar {
    pub fn new() -> Self {
        // Initialize all tiles as unexplored (fog)
        Self {
            explored: vec![vec![false; MAP_WIDTH as usize]; MAP_HEIGHT as usize],
        }
    }

    pub fn is_explored(&self, tile_x: i32, tile_y: i32) -> bool {
        if tile_x < 0 || tile_x >= MAP_WIDTH || tile_y < 0 || tile_y >= MAP_HEIGHT {
            return false;
        }
        self.explored[tile_y as usize][tile_x as usize]
    }

    pub fn set_explored(&mut self, tile_x: i32, tile_y: i32, explored: bool) {
        if tile_x >= 0 && tile_x < MAP_WIDTH && tile_y >= 0 && tile_y < MAP_HEIGHT {
            self.explored[tile_y as usize][tile_x as usize] = explored;
        }
    }

    /// Explore a rectangular area around a center tile
    pub fn explore_rect(&mut self, center_x: i32, center_y: i32, radius: i32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                self.set_explored(center_x + dx, center_y + dy, true);
            }
        }
    }
}

impl Default for FogOfWar {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource, Default)]
pub struct RecruitMenuState {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct HouseMenuState {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct TowerUpgradeMenuState {
    pub active: bool,
    pub selected_tower: Option<Entity>,
}

/// Tracks tiles where building is not allowed (roads, castle, etc.)
#[derive(Resource)]
pub struct BlockedTiles {
    /// Set of (tile_x, tile_y) coordinates that are blocked
    pub tiles: std::collections::HashSet<(i32, i32)>,
    /// Castle tile coordinates
    pub castle_tiles: std::collections::HashSet<(i32, i32)>,
}

impl BlockedTiles {
    pub fn new() -> Self {
        Self {
            tiles: std::collections::HashSet::new(),
            castle_tiles: std::collections::HashSet::new(),
        }
    }

    pub fn is_blocked(&self, tile_x: i32, tile_y: i32) -> bool {
        self.tiles.contains(&(tile_x, tile_y))
    }

    pub fn is_castle(&self, tile_x: i32, tile_y: i32) -> bool {
        self.castle_tiles.contains(&(tile_x, tile_y))
    }

    pub fn is_road(&self, tile_x: i32, tile_y: i32) -> bool {
        self.tiles.contains(&(tile_x, tile_y)) && !self.castle_tiles.contains(&(tile_x, tile_y))
    }
}

impl Default for BlockedTiles {
    fn default() -> Self {
        Self::new()
    }
}
