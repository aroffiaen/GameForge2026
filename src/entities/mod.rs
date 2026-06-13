use bevy::prelude::*;

pub mod ennemies;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            ennemies::enemy_shoot,
            ennemies::poison_tint,
        ));
    }
}
