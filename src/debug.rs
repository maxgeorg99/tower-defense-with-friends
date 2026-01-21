use bevy::prelude::*;
use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};

use crate::config::{TowersConfig, UnitsConfig, WavesConfig};
use crate::resources::{TowerConfigs, WaveConfigs};

/// Resource for file watching (hot-reloading)
#[derive(Resource, Clone)]
pub struct FileWatcher {
    pub receiver: Arc<Mutex<Receiver<notify::Result<Event>>>>,
    pub _watcher: Arc<Mutex<Box<dyn Watcher + Send>>>,
}

/// Plugin for development tools (devtools connection, hot-reloading)
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Connect to devtools for hot-reloading
        std::thread::spawn(|| {
            dioxus_devtools::connect_subsecond();
        });

        // Setup file watching for hot-reloading
        let (tx, rx) = channel();
        let mut watcher =
            notify::recommended_watcher(tx).expect("Failed to create file watcher");

        watcher
            .watch(Path::new("units.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch units.toml");
        watcher
            .watch(Path::new("waves.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch waves.toml");
        watcher
            .watch(Path::new("towers.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch towers.toml");

        app.insert_resource(FileWatcher {
            receiver: Arc::new(Mutex::new(rx)),
            _watcher: Arc::new(Mutex::new(Box::new(watcher))),
        });
        app.add_systems(Update, watch_config_files);
    }
}

/// System that watches for config file changes and hot-reloads them
pub fn watch_config_files(
    file_watcher: Res<FileWatcher>,
    mut wave_configs: ResMut<WaveConfigs>,
    mut tower_configs: ResMut<TowerConfigs>,
) {
    // Check for file change events (non-blocking)
    let receiver = file_watcher.receiver.lock().unwrap();
    while let Ok(Ok(event)) = receiver.try_recv() {
        use notify::EventKind;

        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                // Check which file was modified
                for path in event.paths {
                    if path.ends_with("units.toml") {
                        match UnitsConfig::load() {
                            Ok(config) => {
                                wave_configs.units = config.units;
                                info!(
                                    "Hot-reloaded units.toml - {} units loaded",
                                    wave_configs.units.len()
                                );
                            }
                            Err(e) => error!("Failed to reload units.toml: {}", e),
                        }
                    } else if path.ends_with("waves.toml") {
                        match WavesConfig::load() {
                            Ok(config) => {
                                wave_configs.waves = config.waves;
                                info!(
                                    "Hot-reloaded waves.toml - {} waves loaded",
                                    wave_configs.waves.len()
                                );
                            }
                            Err(e) => error!("Failed to reload waves.toml: {}", e),
                        }
                    } else if path.ends_with("towers.toml") {
                        match TowersConfig::load() {
                            Ok(config) => {
                                tower_configs.towers = config.towers;
                                info!(
                                    "Hot-reloaded towers.toml - {} towers loaded",
                                    tower_configs.towers.len()
                                );
                            }
                            Err(e) => error!("Failed to reload towers.toml: {}", e),
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
