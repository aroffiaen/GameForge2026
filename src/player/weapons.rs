use bevy::prelude::*;
use bevy::prelude::MessageWriter as BevyMessageWriter;
use bevy::math::primitives::CircularSector;
use crate::common::{Enemy, Pulled, Slowed, DamageMsg, DamageKind, Lifetime};
use crate::player::Player;
use super::Arms;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (rake_attack, tick_pulled, tick_rake_slow));
    }
}

// --- Paramètres (inspirés de l'E "Apprehend" de Darius) ---
const RAKE_RANGE: f32 = 420.0;
const RAKE_HALF_ANGLE: f32 = 75.0; // demi-angle en degrés → cône de 150° total
const RAKE_PULL_FORCE: f32 = 700.0;
const RAKE_PULL_DURATION: f32 = 0.35;
const RAKE_SLOW_DURATION: f32 = 1.0; // slow 1s après la fin du pull
const RAKE_COOLDOWN: f32 = 1.5;
const RAKE_DMG: f32 = 18.0;

#[derive(Component)]
pub struct RakeWeapon {
    pub cooldown: Timer,
}

impl Default for RakeWeapon {
    fn default() -> Self {
        let mut t = Timer::from_seconds(RAKE_COOLDOWN, TimerMode::Once);
        t.finish();
        Self { cooldown: t }
    }
}

#[derive(Component)]
struct RakeSlowTimer(Timer);

fn rake_attack(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_q: Query<(&Transform, &mut RakeWeapon), With<Player>>,
    arms_q: Query<&GlobalTransform, With<Arms>>,
    enemies_q: Query<(Entity, &Transform), With<Enemy>>,
    mut dmg: BevyMessageWriter<DamageMsg>,
) {
    let Ok((player_tf, mut rake)) = player_q.single_mut() else { return; };
    rake.cooldown.tick(time.delta());
    if !mouse.just_pressed(MouseButton::Left) || !rake.cooldown.is_finished() { return; }
    rake.cooldown.reset();

    let player_pos = player_tf.translation.truncate();
    let dir = arms_q.single().ok()
        .map(|g| {
            let (_, rot, _) = g.to_scale_rotation_translation();
            (rot * Vec3::Y).truncate().normalize_or_zero()
        })
        .unwrap_or(Vec2::Y);
    if dir == Vec2::ZERO { return; }

    let half_rad = RAKE_HALF_ANGLE.to_radians();
    let cos_half = half_rad.cos();

    // Cône visuel (CircularSector — même orientation que le sprite rectangle : bisectrice = +Y)
    let visual_angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
    commands.spawn((
        Mesh2d(meshes.add(CircularSector::new(RAKE_RANGE, half_rad * 2.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(0.85, 0.2, 0.05, 0.55)))),
        Transform {
            translation: player_pos.extend(5.0),
            rotation: Quat::from_rotation_z(visual_angle),
            ..default()
        },
        Lifetime::secs(0.20),
    ));

    // Détection : cône (sans atan2 — cos(angle) = along/dist >= cos(half_rad))
    for (enemy_e, enemy_tf) in &enemies_q {
        let offset = enemy_tf.translation.truncate() - player_pos;
        let dist = offset.length();
        if dist > RAKE_RANGE || dist < 1.0 { continue; }

        let along = offset.dot(dir);
        if along < dist * cos_half { continue; }

        commands.entity(enemy_e).insert(Pulled {
            force: RAKE_PULL_FORCE,
            timer: Timer::from_seconds(RAKE_PULL_DURATION, TimerMode::Once),
        });
        dmg.write(DamageMsg {
            target: enemy_e,
            amount: RAKE_DMG,
            kind: DamageKind::Hit,
        });
    }
}

fn tick_pulled(
    time: Res<Time>,
    mut commands: Commands,
    mut pulled_q: Query<(Entity, &mut Pulled)>,
) {
    for (e, mut pulled) in &mut pulled_q {
        pulled.timer.tick(time.delta());
        if pulled.timer.is_finished() {
            commands.entity(e)
                .remove::<Pulled>()
                .insert((
                    Slowed,
                    RakeSlowTimer(Timer::from_seconds(RAKE_SLOW_DURATION, TimerMode::Once)),
                ));
        }
    }
}

fn tick_rake_slow(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut RakeSlowTimer)>,
) {
    for (e, mut t) in &mut q {
        t.0.tick(time.delta());
        if t.0.is_finished() {
            commands.entity(e).remove::<(Slowed, RakeSlowTimer)>();
        }
    }
}
