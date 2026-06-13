use bevy::prelude::*;
use crate::entities::ennemies::EnemyKind;

#[derive(Component)]
pub struct Mob {
    pub kind: EnemyKind,
}

#[derive(Component)]
pub struct Boss;

#[derive(Component)]
pub enum AiState {
    Idle,
    Charging { timer: Timer },
    Lunging { timer: Timer, direction: Vec3 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum Biome {
    Terrasse,
    Gravier,
    Boue,
    TerreSeche,
    Potager, // ou Tomate
    Fraise,
    Dalles,
}

#[derive(Resource)]
pub struct WaveManager {
    pub current_wave: u32,
    pub current_biome: Biome,
    pub biomes_cleared: u32,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_wave: 1,
            current_biome: Biome::TerreSeche, // biome de départ au hasard
            biomes_cleared: 0,
        }
    }
}