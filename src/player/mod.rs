mod dash;

use crate::{
    player::dash::{DashConfig, DashPlugin},
    speed::Speed,
};
use bevy::{prelude::*, sprite::Anchor, window::PrimaryWindow};

const PLAYER_SPEED: f32 = 200.;
const PLAYER_SIZE: Vec2 = Vec2::new(128., 128.);
const DEFAULT_BOX: Color = Color::srgb(1., 0., 0.);

const SPRITE_HAT1: &str = "player/hat1.png";
const SPRITE_HAT2: &str = "player/hat2.png";
const SPRITE_LEGS_LEFT: &str = "player/legs_left.png";
const SPRITE_LEGS_RIGHT: &str = "player/legs_right.png";
const SPRITE_LEGS_BOTH: &str = "player/legs_right.png";
const SPRITE_ARMS: &str = "player/arms.png";
const SPRITE_SHOVEL: &str = "player/shovel.png";

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DashPlugin);
        app.add_systems(Startup, (spawn_camera2d, spawn_player));
        app.add_systems(Update, (update_hat, (rotation, movements).chain()));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Iframes(pub Timer);

#[derive(Component)]
pub struct State(u8);

#[derive(Component)]
pub struct Hat;

#[derive(Component)]
pub struct Arms;

#[derive(Component)]
pub struct Legs;

#[derive(Component)]
pub struct ToggleTimer(Timer);

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let legs = commands
        .spawn((
            State(1),
            Legs,
            Sprite {
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
        ))
        .id();
    let arms = commands
        .spawn((
            State(1),
            Arms,
            Sprite {
                image: asset_server.load(SPRITE_ARMS),
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
        ))
        .id();
    let hat = commands
        .spawn((
            State(1),
            Hat,
            ToggleTimer(Timer::from_seconds(0.7, TimerMode::Once)),
            Sprite {
                image: asset_server.load(SPRITE_HAT1),
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Player,
            DashConfig::default().build(),
            Speed::new(PLAYER_SPEED),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::common::Health { hp: 100 },
            crate::common::Radius(48.0), // Rayon augmenté pour coller aux assets 128x128
            Iframes(Timer::from_seconds(0.0, TimerMode::Once)),
        ))
        .add_children(&[legs, arms, hat]);
}

fn spawn_camera2d(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_hat(
    mut player_query: Query<(&mut State, &mut Sprite, &mut ToggleTimer), With<Hat>>,
    asset_server: Res<AssetServer>,
) {
    for (mut state, mut sprite, mut timer) in &mut player_query {
        if timer.0.is_finished() {
            match state.0 {
                1 => {
                    sprite.image = asset_server.load(SPRITE_HAT2);
                    state.0 = 2;
                }
                _ => {
                    sprite.image = asset_server.load(SPRITE_HAT1);
                    state.0 = 1;
                }
            }
            timer.0.reset();
        }
    }
}

fn movements(
    mut player_query: Query<(&mut Transform, &Speed), With<Player>>,
    mut legs_query: Query<(&mut State, &mut Sprite), With<Legs>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    if let Ok((mut transform, speed)) = player_query.single_mut()
        && let Ok((mut state, mut sprite)) = legs_query.single_mut()
    {
        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * speed.get() * time.delta_secs();
        match state.0 {
            1 => {
                sprite.image = asset_server.load(SPRITE_LEGS_RIGHT);
                state.0 = 2;
            }
            _ => {
                sprite.image = asset_server.load(SPRITE_LEGS_LEFT);
                state.0 = 1;
            }
        }
    }
}

fn rotation(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut arms_query: Query<(&GlobalTransform, &mut Transform), With<Arms>>,
) {
    if let Ok(window) = windows.single()
        && let Ok((camera, camera_transform)) = camera_query.single()
        && let Some(cursor_position) = window.cursor_position()
        && let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        && let Ok((arms_global, mut arms_transform)) = arms_query.single_mut()
    {
        // Use GlobalTransform to get world position
        let direction = Vec2::new(
            world_position.x - arms_global.translation().x,
            world_position.y - arms_global.translation().y,
        );

        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
        arms_transform.rotation = Quat::from_rotation_z(angle);
    }
}
