mod player;
mod mobs;
mod entities;
mod common;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(mobs::MobsPlugin)
        .add_plugins(entities::EntitiesPlugin)
        .run();
}
