use bevy::prelude::*;
use self::components::WaveManager;
use crate::common::{GameState, RoomState};

pub mod components;
pub mod ai;
pub mod spawn;

pub struct MobsPlugin;

impl Plugin for MobsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveManager>()
            .add_systems(OnEnter(RoomState::Combat), spawn::setup_combat_room)
            .add_systems(OnEnter(RoomState::Boss), spawn::setup_boss_room)
            .add_systems(OnEnter(RoomState::Treasure), spawn::setup_treasure_room)
            .add_systems(OnEnter(RoomState::Transition), spawn::transition_delay)
            .add_systems(
                Update,
                (
                    ai::mob_ai,
                    spawn::check_room_clear,
                )
                    .run_if(in_state(GameState::InGame))
                    .run_if(
                        in_state(RoomState::Combat).or(in_state(RoomState::Boss)),
                    ),
            );
    }
}