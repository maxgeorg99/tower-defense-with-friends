// =============================================================================
// Game State Table
// =============================================================================
//
// Global game state (resources, lives, score, etc.)
// This is a singleton table (always id = 0).
//
// =============================================================================

use spacetimedb::SpacetimeType;

/// Game status
#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameStatus {
    #[default]
    WaitingForPlayers,
    PreWave,      // Between waves, countdown active
    WaveActive,   // Enemies spawning/active
    Victory,      // All waves completed
    GameOver,     // Lives reached 0
}

/// Global game state (singleton)
#[spacetimedb::table(name = game_state, public)]
pub struct GameState {
    #[primary_key]
    pub id: u64,  // Always 0

    /// Current game status
    pub status: GameStatus,

    /// Remaining lives (base health)
    pub lives: i32,

    /// Current gold
    pub gold: i32,

    /// Current wood
    pub wood: i32,

    /// Current meat
    pub meat: i32,

    /// Total score
    pub score: i32,

    /// Total enemies killed
    pub enemies_killed: i32,

    /// Total waves completed
    pub waves_completed: i32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            id: 0,
            status: GameStatus::WaitingForPlayers,
            lives: 20,
            gold: 200,
            wood: 50,
            meat: 0,
            score: 0,
            enemies_killed: 0,
            waves_completed: 0,
        }
    }
}
