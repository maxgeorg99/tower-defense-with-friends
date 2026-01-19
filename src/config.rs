use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitType {
    pub id: String,
    pub name: String,
    pub sprite_path: String,
    pub base_health: f32,
    pub base_speed: f32,
    pub damage_to_base: i32,
    pub gold_reward: i32,
    pub frame_count: usize,
    pub frame_size: [u32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerType {
    pub id: String,
    pub name: String,
    pub sprite_path: String,
    pub cost: i32,
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32,
    pub projectile_sprite: String,
    pub projectile_speed: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitSpawn {
    pub unit_id: String,
    pub count: i32,
    pub health_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wave {
    pub wave_number: i32,
    pub spawn_interval: f32,
    pub spawns: Vec<UnitSpawn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitsConfig {
    pub units: Vec<UnitType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WavesConfig {
    pub waves: Vec<Wave>,
}

impl UnitsConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string("units.toml")?;
        Ok(toml::from_str(&content)?)
    }
}

impl WavesConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string("waves.toml")?;
        Ok(toml::from_str(&content)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowersConfig {
    pub towers: Vec<TowerType>,
}

impl TowersConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string("towers.toml")?;
        Ok(toml::from_str(&content)?)
    }
}
