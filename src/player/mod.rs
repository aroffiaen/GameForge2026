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
        app.add_systems(Update, (
            update_hat, 
            (rotation, movements, player_attack).chain()
        ));
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
pub struct ArmLeft;

#[derive(Component)]
pub struct ArmRight;

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
    
    // Bras Gauche
    let arm_left = commands
        .spawn((
            ArmLeft,
            Sprite {
                image: asset_server.load(SPRITE_ARMS),
                ..default()
            },
            Transform::from_xyz(-20.0, 0.0, 0.1),
        ))
        .id();

    // Bras Droit
    let arm_right = commands
        .spawn((
            ArmRight,
            Sprite {
                image: asset_server.load(SPRITE_ARMS),
                ..default()
            },
            Transform::from_xyz(20.0, 0.0, 0.1),
        ))
        .id();

    let hat = commands
        .spawn((
            State(1),
            Hat,
            ToggleTimer(Timer::from_seconds(0.7, TimerMode::Once)),
            Sprite {
                image: asset_server.load(SPRITE_HAT1),
                ..default()
            },
        ))
        .id();

    let player_id = commands
        .spawn((
            Player,
            DashConfig::default().build(),
            Speed::new(PLAYER_SPEED),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::common::Health { hp: 100, max_hp: 100 },
            crate::common::Radius(48.0), // Rayon augmenté pour coller aux assets 128x128
            Iframes(Timer::from_seconds(0.0, TimerMode::Once)),
            crate::common::Velocity(Vec2::ZERO),
        ))
        .add_children(&[legs, arm_left, arm_right, hat])
        .id();

    crate::entities::ui::spawn_health_bar(&mut commands, player_id, 70.0, true);
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

// Détection des clics de souris pour les attaques
fn player_attack(
    mouse_input: Res<ButtonInput<MouseButton>>,
    query_player: Query<&GlobalTransform, With<Player>>,
    mut query_mobs: Query<(Entity, &GlobalTransform, &crate::common::Radius), With<crate::common::Enemy>>,
    mut dmg_writer: MessageWriter<crate::common::DamageMsg>,
) {
    let Ok(player_gtf) = query_player.single() else { return };
    let player_pos = player_gtf.translation().truncate();
    let player_dir = player_gtf.compute_transform().rotation.mul_vec3(Vec3::Y).truncate();
    
    // Détection clic gauche (Arme Gauche)
    if mouse_input.just_pressed(MouseButton::Left) {
        info!("Attaque GAUCHE !");
        perform_melee_attack(player_pos, player_dir, 10.0, &mut query_mobs, &mut dmg_writer);
    }
    
    // Détection clic droit (Arme Droite)
    if mouse_input.just_pressed(MouseButton::Right) {
        info!("Attaque DROITE !");
        perform_melee_attack(player_pos, player_dir, 15.0, &mut query_mobs, &mut dmg_writer);
    }
}

// Logique d'attaque de mêlée simple
fn perform_melee_attack(
    origin: Vec2,
    direction: Vec2,
    damage: f32,
    query_mobs: &mut Query<(Entity, &GlobalTransform, &crate::common::Radius), With<crate::common::Enemy>>,
    dmg_writer: &mut MessageWriter<crate::common::DamageMsg>,
) {
    let attack_range = 100.0;
    
    for (mob_e, mob_tf, mob_r) in query_mobs.iter() {
        let mob_pos = mob_tf.translation().truncate();
        let to_mob = mob_pos - origin;
        let dist = to_mob.length();
        
        if dist < attack_range + mob_r.0 {
            // Un cône d'attaque assez large pour le confort
            let dot = to_mob.normalize_or_zero().dot(direction);
            if dot > 0.5 { // environ 60°
                dmg_writer.write(crate::common::DamageMsg {
                    target: mob_e,
                    amount: damage,
                    kind: crate::common::DamageKind::Hit,
                });
            }
        }
    }
}

fn rotation(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut arm_left_query: Query<(&GlobalTransform, &mut Transform), (With<ArmLeft>, Without<ArmRight>)>,
    mut arm_right_query: Query<(&GlobalTransform, &mut Transform), (With<ArmRight>, Without<ArmLeft>)>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Some(cursor_position) = window.cursor_position() else { return };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else { return };

    // Rotation Bras Gauche
    if let Ok((arms_global, mut arms_transform)) = arm_left_query.single_mut() {
        let direction = Vec2::new(
            world_position.x - arms_global.translation().x,
            world_position.y - arms_global.translation().y,
        );
        
        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
        arms_transform.rotation = Quat::from_rotation_z(angle);
    }

    // Rotation Bras Droit
    if let Ok((arms_global, mut arms_transform)) = arm_right_query.single_mut() {
        let direction = Vec2::new(
            world_position.x - arms_global.translation().x,
            world_position.y - arms_global.translation().y,
        );
        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
        arms_transform.rotation = Quat::from_rotation_z(angle);
    }
}
