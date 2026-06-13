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

#[derive(Resource)]
pub struct WaveManager {
    pub current_wave: u32,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_wave: 1,
        }
    }
}