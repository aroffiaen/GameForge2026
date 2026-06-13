use bevy::prelude::*;

pub mod ennemies;
pub mod ui;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            ennemies::enemy_shoot,
            ennemies::poison_tint,
            ennemies::enemy_projectiles,
            ennemies::hazard_puddles,
            ennemies::contact_damage,
            ennemies::tick_iframes,
            ennemies::handle_damage,
            ennemies::death_system,
            ennemies::debug_logger_system,
            ennemies::debug_kill_mobs_system,
            ui::update_health_bar,
            ui::update_health_bar_visibility,
        ));
    }
}
