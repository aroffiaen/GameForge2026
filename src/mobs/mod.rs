use bevy::prelude::*;
use self::components::WaveManager;

pub mod components;
pub mod ai;
pub mod spawn;

pub struct MobsPlugin;

impl Plugin for MobsPlugin {
    fn build(&self, app: &mut App) {
<<<<<<< HEAD
        app.init_resource::<WaveManager>()
            .add_systems(Update, (
                ai::mob_ai,
                ai::health_system,
                spawn::spawn_wave_system,
            ));
=======
        app.add_systems(Update, ai::mob_ai)
            .add_systems(Update, ai::health_system)
            .add_systems(Startup, spawn::spawn_mobs);
>>>>>>> 6467fad (🏗️ feat: update mob AI and health system; replace Position with Transform)
    }
}