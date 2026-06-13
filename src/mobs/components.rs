use bevy::prelude::*;

#[derive(Component)]
#[allow(dead_code)]
pub struct Mob {
    pub speed: f32,
}

#[derive(Component)]
pub struct Health {
    pub hp: i32, // entier en 32bits
}

#[derive(Component)]
pub struct Boss;

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