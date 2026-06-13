use bevy::prelude::*;

pub mod ennemies;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            ennemies::enemy_shoot,
            ennemies::poison_tint,
            ennemies::enemy_projectiles,
            ennemies::hazard_puddles,
            ennemies::contact_damage,
            ennemies::handle_damage,
            ennemies::death_system,
            ennemies::debug_logger_system,
            ennemies::debug_kill_mobs_system,
        ));
    }
}
