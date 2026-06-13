use bevy::prelude::*;
use bevy::prelude::MessageWriter as BevyMessageWriter;
use crate::common::{Enemy, Pulled, DamageMsg, DamageKind, Lifetime};
use crate::player::Player;
use super::Arms;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (rake_attack, tick_pulled));
    }
}

const RAKE_LENGTH: f32 = 380.0;
const RAKE_WIDTH: f32 = 90.0;
const RAKE_PULL_FORCE: f32 = 520.0;
const RAKE_PULL_DURATION: f32 = 0.35;
const RAKE_COOLDOWN: f32 = 1.2;
const RAKE_DMG: f32 = 10.0;

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

fn rake_attack(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut player_q: Query<(&Transform, &mut RakeWeapon), With<Player>>,
    arms_q: Query<&GlobalTransform, With<Arms>>,
    enemies_q: Query<(Entity, &Transform), With<Enemy>>,
    mut dmg: BevyMessageWriter<DamageMsg>,
) {
    let Ok((player_tf, mut rake)) = player_q.single_mut() else { return; };
    rake.cooldown.tick(time.delta());

    if !mouse.just_pressed(MouseButton::Left) || !rake.cooldown.is_finished() {
        return;
    }
    rake.cooldown.reset();

    let player_pos = player_tf.translation.truncate();

    // Direction calculée depuis la rotation des bras (pointe vers la souris)
    let dir = arms_q.single().ok()
        .map(|g| {
            let (_, rot, _) = g.to_scale_rotation_translation();
            (rot * Vec3::Y).truncate().normalize_or_zero()
        })
        .unwrap_or(Vec2::Y);

    if dir == Vec2::ZERO { return; }
    let perp = Vec2::new(-dir.y, dir.x);

    // Rectangle visuel semi-transparent devant le joueur
    let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
    commands.spawn((
        Sprite {
            color: Color::srgba(0.7, 0.45, 0.1, 0.45),
            custom_size: Some(Vec2::new(RAKE_WIDTH, RAKE_LENGTH)),
            ..default()
        },
        Transform {
            translation: (player_pos + dir * RAKE_LENGTH * 0.5).extend(5.0),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        Lifetime::secs(0.18),
    ));

    // Détection et attraction des ennemis dans le rectangle
    for (enemy_e, enemy_tf) in &enemies_q {
        let enemy_pos = enemy_tf.translation.truncate();
        let offset = enemy_pos - player_pos;
        let along = offset.dot(dir);
        let side = offset.dot(perp).abs();

        if along >= 0.0 && along <= RAKE_LENGTH && side <= RAKE_WIDTH * 0.5 {
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
}

fn tick_pulled(
    time: Res<Time>,
    mut commands: Commands,
    mut pulled_q: Query<(Entity, &mut Pulled)>,
) {
    for (e, mut pulled) in &mut pulled_q {
        pulled.timer.tick(time.delta());
        if pulled.timer.is_finished() {
            commands.entity(e).remove::<Pulled>();
        }
    }
}
