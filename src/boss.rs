//! Les boss : prédateurs du jardin, un par biome, avec patterns propres
//! (GDD §8.2). Chacun arrive après un gauntlet de 3 vagues (GDD §6.3).

use bevy::prelude::*;
use rand::prelude::*;

use crate::common::*;
use crate::enemies::{spawn_enemy_projectile, AiSpeed, HazardPuddle, PattesDrop};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossKind {
    Taupe,
    Frelon,
    Crapaud,
}

impl BossKind {
    pub fn name(self) -> &'static str {
        match self {
            BossKind::Taupe => "Gérard la Taupe",
            BossKind::Frelon => "Baron Frelon",
            BossKind::Crapaud => "Maître Crapaud",
        }
    }
}

/// Marqueur : cette entité est pilotée par une IA de boss, pas l'IA générique.
#[derive(Component)]
pub struct BossAiTag;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

pub fn spawn_boss(commands: &mut Commands, kind: BossKind, pos: Vec2, scale: f32) -> Entity {
    let (hp, radius, color, contact) = match kind {
        BossKind::Taupe => (380.0, 26.0, Color::srgb(0.4, 0.3, 0.25), 14.0),
        BossKind::Frelon => (330.0, 22.0, Color::srgb(0.9, 0.65, 0.1), 15.0),
        BossKind::Crapaud => (470.0, 30.0, Color::srgb(0.35, 0.55, 0.3), 15.0),
    };
    let color = color.mix(&Color::srgb(0.8, 0.1, 0.1), 0.15);
    let mut e = commands.spawn((
        (Enemy, BossTag, BossAiTag),
        Sprite::from_color(color, Vec2::splat(radius * 2.1)),
        BaseColor(color),
        Transform::from_translation(pos.extend(8.5)),
        Velocity::default(),
        Knockback::default(),
        Health::new(hp * scale),
        Radius(radius),
        ClampToArena,
        ContactDmg(contact * scale.sqrt()),
        ContactCd(Timer::from_seconds(0.4, TimerMode::Once)),
        PattesDrop(20),
        AiSpeed(80.0),
        crate::enemies::EnemyKind::Scarabee, // type « gros » par défaut pour les systèmes génériques
    ));
    match kind {
        BossKind::Taupe => {
            e.insert(Taupe {
                state: TaupeState::Chase,
                timer: Timer::from_seconds(2.2, TimerMode::Once),
            });
        }
        BossKind::Frelon => {
            e.insert(Frelon {
                state: FrelonState::Strafe,
                timer: Timer::from_seconds(1.4, TimerMode::Once),
                charge_dir: Vec2::X,
                volley_left: 0,
            });
        }
        BossKind::Crapaud => {
            e.insert(Crapaud {
                state: CrapaudState::Chase,
                timer: Timer::from_seconds(1.2, TimerMode::Once),
                leap_from: pos,
                leap_to: pos,
            });
        }
    }
    e.id()
}

// ---------------------------------------------------------------------------
// La Taupe (Plaine) : s'enfouit, fonce sous terre, jaillit en AoE.
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Taupe {
    state: TaupeState,
    timer: Timer,
}

enum TaupeState {
    Chase,
    Burrow,
    Emerge,
}

fn taupe_ai(
    time: Res<Time>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    player: Query<(Entity, &Transform, &Radius, &crate::player::Iframes), With<Player>>,
    mut bosses: Query<(Entity, &Transform, &mut Velocity, &mut Sprite, &BaseColor, &mut Taupe)>,
) {
    let Ok((player_e, player_tf, player_r, iframes)) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (e, tf, mut vel, mut sprite, base, mut taupe) in &mut bosses {
        taupe.timer.tick(time.delta());
        let pos = tf.translation.truncate();
        let dir = (player_pos - pos).normalize_or_zero();
        match taupe.state {
            TaupeState::Chase => {
                vel.0 = vel.0.move_towards(dir * 80.0, 500.0 * time.delta_secs());
                if taupe.timer.is_finished() {
                    taupe.state = TaupeState::Burrow;
                    taupe.timer = Timer::from_seconds(1.5, TimerMode::Once);
                    sprite.color = base.0.with_alpha(0.35);
                    commands.entity(e).insert(Untouchable);
                }
            }
            TaupeState::Burrow => {
                // Une bosse de terre qui file vers le joueur.
                vel.0 = vel.0.move_towards(dir * 290.0, 900.0 * time.delta_secs());
                if taupe.timer.is_finished() {
                    taupe.state = TaupeState::Emerge;
                    taupe.timer = Timer::from_seconds(0.4, TimerMode::Once);
                    vel.0 = Vec2::ZERO;
                    // Télégraphe du jaillissement.
                    commands.spawn((
                        Sprite::from_color(
                            Color::srgb(0.9, 0.3, 0.1).with_alpha(0.25),
                            Vec2::splat(220.0),
                        ),
                        Transform::from_translation(pos.extend(-2.0))
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                        Lifetime::secs(0.4),
                    ));
                }
            }
            TaupeState::Emerge => {
                vel.0 = Vec2::ZERO;
                if taupe.timer.is_finished() {
                    sprite.color = base.0;
                    commands.entity(e).remove::<Untouchable>();
                    // AoE de jaillissement.
                    if pos.distance(player_pos) < 110.0 + player_r.0 && iframes.0.is_finished() {
                        dmg.write(DamageMsg {
                            target: player_e,
                            amount: 16.0,
                            kind: DamageKind::Hit,
                        });
                    }
                    // Anneau de mottes de terre.
                    for i in 0..10 {
                        let d = Vec2::from_angle(i as f32 / 10.0 * std::f32::consts::TAU);
                        spawn_enemy_projectile(
                            &mut commands,
                            pos + d * 20.0,
                            d * 190.0,
                            8.0,
                            Color::srgb(0.5, 0.35, 0.2),
                        );
                    }
                    // Effet visuel d'éclat.
                    for i in 0..12 {
                        let d = Vec2::from_angle(i as f32 / 12.0 * std::f32::consts::TAU);
                        commands.spawn((
                            Sprite::from_color(Color::srgb(0.55, 0.4, 0.25), Vec2::splat(7.0)),
                            Transform::from_translation((pos + d * 15.0).extend(9.0)),
                            Velocity(d * 260.0),
                            Lifetime::secs(0.3),
                        ));
                    }
                    taupe.state = TaupeState::Chase;
                    taupe.timer = Timer::from_seconds(2.2, TimerMode::Once);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Le Frelon (Savane) : strafe, charge télégraphée, rafales d'aiguillons.
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Frelon {
    state: FrelonState,
    timer: Timer,
    charge_dir: Vec2,
    volley_left: u32,
}

enum FrelonState {
    Strafe,
    Telegraph,
    Charge,
    Volley,
}

fn frelon_ai(
    time: Res<Time>,
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut bosses: Query<(&Transform, &mut Velocity, &mut Sprite, &BaseColor, &mut Frelon)>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let mut rng = rand::rng();
    for (tf, mut vel, mut sprite, base, mut frelon) in &mut bosses {
        frelon.timer.tick(time.delta());
        let pos = tf.translation.truncate();
        let to_player = player_pos - pos;
        let dir = to_player.normalize_or(Vec2::X);
        match frelon.state {
            FrelonState::Strafe => {
                // Orbite autour du joueur à ~240 de distance.
                let dist = to_player.length();
                let orbit = Vec2::new(-dir.y, dir.x) * 160.0;
                let approach = if dist > 260.0 {
                    dir * 140.0
                } else if dist < 200.0 {
                    -dir * 140.0
                } else {
                    Vec2::ZERO
                };
                vel.0 = vel.0.move_towards(orbit + approach, 800.0 * time.delta_secs());
                if frelon.timer.is_finished() {
                    if rng.random_bool(0.55) {
                        frelon.state = FrelonState::Telegraph;
                        frelon.timer = Timer::from_seconds(0.45, TimerMode::Once);
                    } else {
                        frelon.state = FrelonState::Volley;
                        frelon.timer = Timer::from_seconds(0.25, TimerMode::Once);
                        frelon.volley_left = 3;
                    }
                }
            }
            FrelonState::Telegraph => {
                vel.0 = vel.0.move_towards(Vec2::ZERO, 1200.0 * time.delta_secs());
                sprite.color = base.0.mix(&Color::srgb(1.0, 0.1, 0.1), 0.6);
                frelon.charge_dir = dir;
                if frelon.timer.is_finished() {
                    sprite.color = base.0;
                    frelon.state = FrelonState::Charge;
                    frelon.timer = Timer::from_seconds(0.55, TimerMode::Once);
                }
            }
            FrelonState::Charge => {
                vel.0 = frelon.charge_dir * 620.0;
                if frelon.timer.is_finished() {
                    frelon.state = FrelonState::Strafe;
                    frelon.timer = Timer::from_seconds(1.4, TimerMode::Once);
                }
            }
            FrelonState::Volley => {
                vel.0 = vel.0.move_towards(Vec2::ZERO, 1000.0 * time.delta_secs());
                if frelon.timer.is_finished() {
                    frelon.volley_left = frelon.volley_left.saturating_sub(1);
                    for spread in [-0.22, 0.0, 0.22] {
                        let d = Vec2::from_angle(dir.to_angle() + spread);
                        spawn_enemy_projectile(
                            &mut commands,
                            pos + d * 24.0,
                            d * 310.0,
                            7.0,
                            Color::srgb(1.0, 0.9, 0.3),
                        );
                    }
                    if frelon.volley_left == 0 {
                        frelon.state = FrelonState::Strafe;
                        frelon.timer = Timer::from_seconds(1.4, TimerMode::Once);
                    } else {
                        frelon.timer = Timer::from_seconds(0.3, TimerMode::Once);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Le Crapaud (Jungle) : bonds en AoE, langue en ligne, crachats toxiques.
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Crapaud {
    state: CrapaudState,
    timer: Timer,
    leap_from: Vec2,
    leap_to: Vec2,
}

enum CrapaudState {
    Chase,
    LeapTelegraph,
    Leap,
    TongueTelegraph { dir: Vec2 },
}

fn crapaud_ai(
    time: Res<Time>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    player: Query<(Entity, &Transform, &Radius, &crate::player::Iframes), With<Player>>,
    mut bosses: Query<(Entity, &mut Transform, &mut Velocity, &mut Crapaud), Without<Player>>,
) {
    let Ok((player_e, player_tf, player_r, iframes)) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let mut rng = rand::rng();
    for (e, mut tf, mut vel, mut crapaud) in &mut bosses {
        crapaud.timer.tick(time.delta());
        let pos = tf.translation.truncate();
        let dir = (player_pos - pos).normalize_or(Vec2::X);
        match crapaud.state {
            CrapaudState::Chase => {
                vel.0 = vel.0.move_towards(dir * 60.0, 400.0 * time.delta_secs());
                if crapaud.timer.is_finished() {
                    if rng.random_bool(0.6) {
                        crapaud.state = CrapaudState::LeapTelegraph;
                        crapaud.timer = Timer::from_seconds(0.45, TimerMode::Once);
                        crapaud.leap_from = pos;
                        crapaud.leap_to = player_pos;
                        commands.spawn((
                            Sprite::from_color(
                                Color::srgb(0.9, 0.3, 0.1).with_alpha(0.25),
                                Vec2::splat(200.0),
                            ),
                            Transform::from_translation(player_pos.extend(-2.0))
                                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                            Lifetime::secs(1.0),
                        ));
                    } else {
                        crapaud.state = CrapaudState::TongueTelegraph { dir };
                        crapaud.timer = Timer::from_seconds(0.35, TimerMode::Once);
                        let angle = dir.to_angle();
                        commands.spawn((
                            Sprite::from_color(
                                Color::srgb(1.0, 0.4, 0.5).with_alpha(0.3),
                                Vec2::new(270.0, 22.0),
                            ),
                            Transform::from_translation((pos + dir * 135.0).extend(-2.0))
                                .with_rotation(Quat::from_rotation_z(angle)),
                            Lifetime::secs(0.35),
                        ));
                    }
                }
            }
            CrapaudState::LeapTelegraph => {
                vel.0 = Vec2::ZERO;
                if crapaud.timer.is_finished() {
                    crapaud.state = CrapaudState::Leap;
                    crapaud.timer = Timer::from_seconds(0.55, TimerMode::Once);
                    commands.entity(e).insert(Untouchable);
                }
            }
            CrapaudState::Leap => {
                vel.0 = Vec2::ZERO;
                let t = crapaud.timer.fraction();
                let arc = (t * std::f32::consts::PI).sin() * 60.0;
                let xy = crapaud.leap_from.lerp(crapaud.leap_to, t);
                tf.translation.x = xy.x;
                tf.translation.y = xy.y + arc;
                if crapaud.timer.is_finished() {
                    commands.entity(e).remove::<Untouchable>();
                    tf.translation.x = crapaud.leap_to.x;
                    tf.translation.y = crapaud.leap_to.y;
                    let land = crapaud.leap_to;
                    if land.distance(player_pos) < 100.0 + player_r.0 && iframes.0.is_finished() {
                        dmg.write(DamageMsg {
                            target: player_e,
                            amount: 15.0,
                            kind: DamageKind::Hit,
                        });
                    }
                    // Crachats toxiques qui laissent des flaques.
                    for _ in 0..4 {
                        let d = Vec2::from_angle(rng.random_range(0.0..std::f32::consts::TAU));
                        let target = land + d * rng.random_range(70.0..160.0);
                        commands.spawn((
                            Sprite::from_color(Color::srgb(0.6, 0.3, 0.7), Vec2::splat(9.0)),
                            Transform::from_translation(land.extend(9.0)),
                            Velocity((target - land) / 0.6),
                            Glob {
                                timer: Timer::from_seconds(0.6, TimerMode::Once),
                            },
                        ));
                    }
                    crapaud.state = CrapaudState::Chase;
                    crapaud.timer = Timer::from_seconds(1.1, TimerMode::Once);
                }
            }
            CrapaudState::TongueTelegraph { dir } => {
                vel.0 = Vec2::ZERO;
                if crapaud.timer.is_finished() {
                    // Coup de langue : dégâts en ligne.
                    let to_player = player_pos - pos;
                    let along = to_player.dot(dir);
                    let perp = (to_player - dir * along).length();
                    if (0.0..=270.0).contains(&along)
                        && perp < 22.0 + player_r.0
                        && iframes.0.is_finished()
                    {
                        dmg.write(DamageMsg {
                            target: player_e,
                            amount: 12.0,
                            kind: DamageKind::Hit,
                        });
                    }
                    let angle = dir.to_angle();
                    commands.spawn((
                        Sprite::from_color(Color::srgb(1.0, 0.5, 0.6), Vec2::new(270.0, 14.0)),
                        Transform::from_translation((pos + dir * 135.0).extend(9.0))
                            .with_rotation(Quat::from_rotation_z(angle)),
                        Lifetime::secs(0.15),
                    ));
                    crapaud.state = CrapaudState::Chase;
                    crapaud.timer = Timer::from_seconds(1.0, TimerMode::Once);
                }
            }
        }
    }
}

/// Crachat en vol : à l'atterrissage, devient une flaque dangereuse.
#[derive(Component)]
pub struct Glob {
    timer: Timer,
}

fn glob_system(
    time: Res<Time>,
    mut commands: Commands,
    mut globs: Query<(Entity, &Transform, &mut Glob)>,
) {
    for (e, tf, mut glob) in &mut globs {
        glob.timer.tick(time.delta());
        if glob.timer.is_finished() {
            let pos = tf.translation.truncate();
            commands.entity(e).despawn();
            commands.spawn((
                Sprite::from_color(Color::srgb(0.6, 0.3, 0.7).with_alpha(0.45), Vec2::splat(60.0)),
                Transform::from_translation(pos.extend(-3.0))
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                HazardPuddle {
                    dps: 9.0,
                    radius: 32.0,
                    life: Timer::from_seconds(3.5, TimerMode::Once),
                    tick: Timer::from_seconds(0.4, TimerMode::Repeating),
                },
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (taupe_ai, frelon_ai, crapaud_ai, glob_system)
                .in_set(GameSet::Ai)
                .run_if(combat_active),
        );
    }
}
