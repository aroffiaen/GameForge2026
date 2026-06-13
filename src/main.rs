mod collectible;
mod modifier;
mod player;
mod mobs;
mod entities;
mod common;
mod speed;

use bevy::prelude::*;
use crate::common::{Arena, DamageMsg, GameState, RoomState};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .init_state::<RoomState>()
        .add_plugins(player::PlayerPlugin)
        .add_plugins(mobs::MobsPlugin)
        .add_plugins(entities::EntitiesPlugin)
        .insert_resource(Arena { half: Vec2::new(600.0, 400.0) })
        .add_message::<DamageMsg>()
        .add_systems(Update, (common::move_velocity, common::update_lifetime))
        .run();
}
