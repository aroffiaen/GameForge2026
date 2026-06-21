mod collectible;
mod common;
mod entities;
mod mobs;
mod modifier;
mod player;
mod speed;
mod ui;

use crate::common::{Arena, DamageMsg, GameState, RoomState};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ui::main_menu::MenuPlugin)
        .init_state::<GameState>()
        .init_state::<RoomState>()
        .add_plugins(player::PlayerPlugin)
        .add_plugins(mobs::MobsPlugin)
        .add_plugins(entities::EntitiesPlugin)
        .insert_resource(Arena {
            half: Vec2::new(600.0, 400.0),
        })
        .add_message::<DamageMsg>()
        .add_systems(Update, (common::move_velocity, common::update_lifetime))
        .run();
}
