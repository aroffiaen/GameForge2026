use bevy::prelude::*;
use self::components::WaveManager;
use crate::common::{GameState, RoomState};

pub mod components;
pub mod ai;
pub mod spawn;
pub mod bosses;

pub struct MobsPlugin;

impl Plugin for MobsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveManager>()
            .add_systems(OnEnter(RoomState::Combat), spawn::setup_combat_room)
            .add_systems(OnEnter(RoomState::Boss), spawn::setup_boss_room)
            .add_systems(OnEnter(RoomState::NextBiome), spawn::setup_next_biome)
            .add_systems(OnEnter(RoomState::Transition), spawn::transition_delay)
            .add_systems(
                Update,
                (
                    ai::mob_ai,
                    spawn::check_room_clear,
                    bosses::araignee_ai,
                    bosses::scorpion_ai,
                    bosses::gromp_ai,
                    bosses::glob_system,
                )
                    .run_if(in_state(GameState::InGame))
                    .run_if(
                        in_state(RoomState::Combat).or(in_state(RoomState::Boss)),
                    ),
            );
    }
}