mod player;
mod mobs;

use bevy::prelude::*;
use crate::mobs::MobsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(MobsPlugin)
        .run();
}