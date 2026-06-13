mod player;
mod mobs;

use bevy::prelude::*;
use crate::mobs::MobsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(MobsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // fait apparaître : caméra 2D
    commands.spawn(Camera2d::default());
}