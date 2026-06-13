use bevy::prelude::*;
use self::components::WaveManager;

pub mod components;
pub mod ai;
pub mod spawn;

pub struct MobsPlugin;

impl Plugin for MobsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveManager>()
            .add_systems(Update, (
                ai::mob_ai,
                spawn::spawn_wave_system,
            ));
    }
}