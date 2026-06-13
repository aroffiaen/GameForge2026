use bevy::{prelude::*, sprite::Anchor, window::PrimaryWindow};

const PLAYER_SPEED: f32 = 200.;

const SPRITE_IDLE1: &str = "player/idle1.png";
const SPRITE_IDLE2: &str = "player/idel2.png";
const SPRITE_WALK1: &str = "player/walk1.png";
const SPRITE_WALK2: &str = "player/walk2.png";
const SPRITE_DASH: &str = "player/dash.png";

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera2d, spawn_player));
        app.add_systems(Update, (rotation, dash, movements).chain());
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
struct PlayerSpeed(f32);

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        Sprite {
            image: asset_server.load(SPRITE_IDLE1),
            custom_size: Some(Vec2 { x: 32., y: 32. }),
            ..default()
        },
        Anchor::CENTER,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

pub fn spawn_camera2d(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn movements(
    mut player_query: Query<&mut Transform, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for mut transform in &mut player_query {
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

        transform.translation += direction * PLAYER_SPEED * time.delta_secs();
    }
}

pub fn dash() {}

fn rotation(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(window) = windows.single()
        && let Ok((camera, camera_transform)) = camera_query.single()
        && let Some(cursor_position) = window.cursor_position()
        && let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        for mut player_transform in &mut player_query {
            let direction = Vec2::new(
                world_position.x - player_transform.translation.x,
                world_position.y - player_transform.translation.y,
            );

            // Calculate angle in radians
            // atan2(dy, dx) gives angle from +X axis
            // Bevy sprites face +Y (up) by default, so subtract π/2 to convert
            // This makes: up=0, right=-π/2, down=π, left=π/2
            let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

            player_transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}
