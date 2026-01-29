#[cfg(feature = "bevy")]
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitType {
    pub id: String,
    pub name: String,
    pub sprite_path: String,
    pub avatar_path: String,
    pub base_health: f32,
    pub base_speed: f32,
    pub damage_to_base: i32,
    pub gold_reward: i32,
    pub frame_count: usize,
    pub frame_size: [u32; 2],
    #[serde(default = "default_defense_type")]
    pub defense_type: String,
}

fn default_defense_type() -> String {
    "armor".to_string()
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
    #[serde(default = "default_attack_type")]
    pub attack_type: String,
}

fn default_attack_type() -> String {
    "pierce".to_string()
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
#[cfg_attr(feature = "bevy", derive(Resource))]
pub struct UnitsConfig {
    pub units: Vec<UnitType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(Resource))]
pub struct WavesConfig {
    pub waves: Vec<Wave>,
}

impl UnitsConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(not(target_arch = "wasm32"))]
        let content = std::fs::read_to_string("units.toml")?;
        #[cfg(target_arch = "wasm32")]
        let content = include_str!("../units.toml").to_string();
        Ok(toml::from_str(&content)?)
    }
}

impl WavesConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(not(target_arch = "wasm32"))]
        let content = std::fs::read_to_string("waves.toml")?;
        #[cfg(target_arch = "wasm32")]
        let content = include_str!("../waves.toml").to_string();
        Ok(toml::from_str(&content)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowersConfig {
    pub towers: Vec<TowerType>,
}

impl TowersConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(not(target_arch = "wasm32"))]
        let content = std::fs::read_to_string("towers.toml")?;
        #[cfg(target_arch = "wasm32")]
        let content = include_str!("../towers.toml").to_string();
        Ok(toml::from_str(&content)?)
    }
}
