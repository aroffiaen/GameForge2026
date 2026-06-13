use crate::player::{Legs, PLAYER_SPEED, Player, SPRITE_LEGS_BOTH};
use bevy::{prelude::*, window::PrimaryWindow};

pub struct DashPlugin;

impl Plugin for DashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (dash_system, dash_movement).chain());
    }
}

pub struct DashConfig {
    multiplier: f32,
    duration: f32,
    cooldown: f32,
}

impl DashConfig {
    pub fn speed_multiplier(self, multiplier: f32) -> Self {
        Self { multiplier, ..self }
    }

    pub fn duration(self, duration: f32) -> Self {
        Self { duration, ..self }
    }

    pub fn cooldown(self, cooldown: f32) -> Self {
        Self { cooldown, ..self }
    }

    pub fn build(self) -> Dash {
        let mut cooldown = Timer::from_seconds(self.cooldown, TimerMode::Once);
        let mut timer = Timer::from_seconds(self.duration, TimerMode::Once);
        cooldown.finish();
        timer.finish();
        Dash {
            cooldown,
            timer,
            multiplier: self.multiplier,
            direction: Vec2::ZERO,
        }
    }
}

impl Default for DashConfig {
    fn default() -> Self {
        Self {
            multiplier: 5.0,
            duration: 0.3,
            cooldown: 3.0,
        }
    }
}

#[derive(Component)]
pub struct Dash {
    cooldown: Timer,
    timer: Timer,
    multiplier: f32,
    direction: Vec2,
}

fn dash_movement(time: Res<Time>, mut player_query: Query<(&mut Transform, &Dash), With<Player>>) {
    for (mut transform, dash) in &mut player_query {
        if !dash.timer.is_finished() {
            let direction = Vec3::new(dash.direction.x, dash.direction.y, 0.0);
            let speed = PLAYER_SPEED * dash.multiplier;
            transform.translation += direction * speed * time.delta_secs();
        }
    }
}

fn dash_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(&Transform, &mut Dash), With<Player>>,
    mut legs_query: Query<&mut Sprite, With<Legs>>,
) {
    if let Ok(window) = windows.single()
        && let Ok((camera, camera_transform)) = camera_q.single()
    {
        for (player_transform, mut dash) in &mut player_query {
            if dash.cooldown.is_finished() && keyboard_input.just_pressed(KeyCode::Space) {
                if let Some(cursor_position) = window.cursor_position()
                    && let Ok(world_position) =
                        camera.viewport_to_world_2d(camera_transform, cursor_position)
                {
                    dash.direction = Vec2::new(
                        world_position.x - player_transform.translation.x,
                        world_position.y - player_transform.translation.y,
                    )
                    .normalize_or_zero();
                }

                dash.timer.reset();
                dash.cooldown.reset();
                if let Ok(mut sprite) = legs_query.single_mut() {
                    sprite.image = asset_server.load(SPRITE_LEGS_BOTH);
                }
            }
            let delta = time.delta();
            dash.timer.tick(delta);
            dash.cooldown.tick(delta);
        }
    }
}
