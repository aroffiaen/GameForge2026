mod collectible;
mod modifier;
mod player;
mod mobs;
mod entities;
mod common;
mod speed;

use bevy::prelude::*;
use crate::common::{Arena, DamageMsg, RoomState, GameState};
use crate::entities::ui;

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
        .add_systems(Update, (common::move_velocity, common::update_lifetime).run_if(in_state(GameState::InGame)))
        
        // Affiche l'écran uniquement en entrant dans l'état GameOver
        .add_systems(OnEnter(GameState::GameOver), ui::spawn_game_over_ui)

        // Permet de restart uniquement si on est dans l'état GameOver
        .add_systems(Update, ui::restart_game.run_if(in_state(GameState::GameOver)))
        .run();
}
