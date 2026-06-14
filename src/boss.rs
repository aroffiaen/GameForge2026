//! Les boss : prédateurs du jardin, un par biome, avec patterns propres
//! (GDD §7-8). La salle de boss ne contient que le boss (plus de vagues, §6).
//!
//! - Jardin     : Mémé Mygale (araignée) — bonds, jet de toile, araignéeaux.
//! - Gravier    : Mille-Pattes — ruée en ligne (corps dangereux), salve en
//!                éventail, traînée de bave toxique.
//! - Boue       : Grompaud (crapaud, clin d'œil au Gromp de LoL) — bonds AoE,
//!                langue en ligne, crachats toxiques.
//! - Terre Sèche: Roger le Scorpion — charges de pinces, salves de dard venimeux.
//! - Potager    : Méga-Limace — reptation baveuse, crachat radial, ponte.
//! - Dalles     : Araignée géante — même répertoire que Mémé Mygale, gros PV.

use bevy::prelude::*;
use rand::prelude::*;

use crate::common::*;
use crate::enemies::{spawn_enemy, spawn_enemy_projectile, AiSpeed, EnemyKind, HazardPuddle, PattesDrop};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossKind {
    Araignee,
    Scorpion,
    Gromp,
    /// Boss du Potager (GDD §7). Porté de feat/player (« Giga Limace »).
    MegaLimace,
    /// Boss du Gravier (GDD §7) : corps segmenté, ruée en ligne.
    MillePattes,
    /// Boss des Dalles (GDD §7) : réutilise l'IA Araignée, gros PV (§19).
    AraigneeGeante,
}

impl BossKind {
    pub fn name(self) -> &'static str {
        match self {
            BossKind::Araignee => "Mémé Mygale",
            BossKind::Scorpion => "Roger le Scorpion",
            BossKind::Gromp => "Grompaud",
            BossKind::MegaLimace => "Méga-Limace",
            BossKind::MillePattes => "Mille-Pattes",
            BossKind::AraigneeGeante => "Araignée géante",
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
        BossKind::Araignee => (360.0, 25.0, Color::srgb(0.28, 0.24, 0.34), 13.0),
        BossKind::Scorpion => (360.0, 24.0, Color::srgb(0.65, 0.45, 0.2), 15.0),
        BossKind::Gromp => (470.0, 30.0, Color::srgb(0.35, 0.55, 0.3), 15.0),
        BossKind::MegaLimace => (430.0, 34.0, Color::srgb(0.8, 0.85, 0.3), 11.0),
        BossKind::MillePattes => (460.0, 20.0, Color::srgb(0.62, 0.34, 0.20), 13.0),
        BossKind::AraigneeGeante => (900.0, 40.0, Color::srgb(0.20, 0.18, 0.26), 16.0),
    };
    let color = color.mix(&Color::srgb(0.8, 0.1, 0.1), 0.15);
    // La limace a une silhouette allongée ; les autres sont ~circulaires.
    let size = match kind {
        BossKind::MegaLimace => Vec2::new(radius * 3.4, radius * 2.2),
        _ => Vec2::splat(radius * 2.1),
    };
    let mut e = commands.spawn((
        (Enemy, BossTag, BossAiTag),
        Sprite::from_color(color, size),
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
        crate::enemies::EnemyKind::Escargot, // type « gros » par défaut pour les systèmes génériques
    ));
    match kind {
        BossKind::Araignee => {
            e.insert(Araignee {
                state: AraigneeState::Chase,
                timer: Timer::from_seconds(1.6, TimerMode::Once),
                leap_from: pos,
                leap_to: pos,
            });
        }
        BossKind::Scorpion => {
            e.insert(Scorpion {
                state: ScorpionState::Strafe,
                timer: Timer::from_seconds(1.4, TimerMode::Once),
                charge_dir: Vec2::X,
                volley_left: 0,
            });
        }
        BossKind::Gromp => {
            e.insert(Gromp {
                state: GrompState::Chase,
                timer: Timer::from_seconds(1.2, TimerMode::Once),
                leap_from: pos,
                leap_to: pos,
            });
        }
        BossKind::MegaLimace => {
            e.insert(MegaLimace {
                state: LimaceState::Crawl,
                timer: Timer::from_seconds(1.5, TimerMode::Once),
                trail: Timer::from_seconds(0.45, TimerMode::Repeating),
            });
        }
        BossKind::MillePattes => {
            let seg_count = 7usize;
            let seg_spacing = radius * 1.35;
            e.insert(MillePattes {
                state: MpState::Crawl,
                timer: Timer::from_seconds(2.2, TimerMode::Once),
                charge_dir: Vec2::X,
                heading: 0.0,
                trail_drop: Timer::from_seconds(0.45, TimerMode::Repeating),
                volley_left: 0,
                seg_count,
                seg_spacing,
            });
            // Corps : segments décroissants greffés derrière la tête (axe -X
            // local) ; ils suivent la rotation de la tête et se despawn avec elle.
            let seg_color = color.mix(&Color::BLACK, 0.15);
            e.with_children(|p| {
                for i in 0..seg_count {
                    let k = i as f32 + 1.0;
                    let r = radius * (1.0 - 0.06 * k).max(0.45);
                    p.spawn((
                        Sprite::from_color(seg_color, Vec2::splat(r * 2.0)),
                        Transform::from_xyz(-seg_spacing * k, 0.0, -0.05 * k),
                    ));
                }
            });
        }
        BossKind::AraigneeGeante => {
            // Réutilise l'IA Araignée (placeholder assumé, GDD §19) ; ses gros
            // PV / sa taille (ci-dessus) en font la version « géante ».
            e.insert(Araignee {
                state: AraigneeState::Chase,
                timer: Timer::from_seconds(1.6, TimerMode::Once),
                leap_from: pos,
                leap_to: pos,
            });
        }
    }
    let id = e.id();
    drop(e); // libère l'emprunt de `commands` avant de greffer la barre

    // Barre de vie flottante + nametag, toujours visible (boss).
    crate::healthbar::spawn_health_bar(
        commands,
        id,
        radius + 20.0,
        (radius * 2.2).max(48.0),
        true,
        Some(kind.name()),
    );
    id
}

// ---------------------------------------------------------------------------
// Mémé Mygale (Plaine) : skitter, bond AoE, jet de toile radial, araignéeaux.
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Araignee {
    state: AraigneeState,
    timer: Timer,
    leap_from: Vec2,
    leap_to: Vec2,
}

enum AraigneeState {
    Chase,
    LeapTelegraph,
    Leap,
    WebBurst,
    Summon,
}

fn araignee_ai(
    time: Res<Time>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    player: Query<(Entity, &Transform, &Radius, &crate::player::Iframes), With<Player>>,
    mut bosses: Query<(Entity, &mut Transform, &mut Velocity, &mut Araignee), Without<Player>>,
) {
    let Ok((player_e, player_tf, player_r, iframes)) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let mut rng = rand::rng();
    for (e, mut tf, mut vel, mut spider) in &mut bosses {
        spider.timer.tick(time.delta());
        let pos = tf.translation.truncate();
        let dir = (player_pos - pos).normalize_or(Vec2::X);
        match spider.state {
            AraigneeState::Chase => {
                // Déplacement saccadé d'araignée.
                let jitter = Vec2::from_angle(rng.random_range(-0.5..0.5));
                vel.0 = vel.0.move_towards(dir.rotate(jitter) * 120.0, 600.0 * time.delta_secs());
                if spider.timer.is_finished() {
                    let roll: f32 = rng.random_range(0.0..1.0);
                    if roll < 0.5 {
                        spider.state = AraigneeState::LeapTelegraph;
                        spider.timer = Timer::from_seconds(0.4, TimerMode::Once);
                        spider.leap_from = pos;
                        spider.leap_to = player_pos;
                        commands.spawn((
                            Sprite::from_color(
                                Color::srgb(0.9, 0.3, 0.1).with_alpha(0.25),
                                Vec2::splat(180.0),
                            ),
                            Transform::from_translation(player_pos.extend(-2.0))
                                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                            Lifetime::secs(0.9),
                        ));
                    } else if roll < 0.8 {
                        spider.state = AraigneeState::WebBurst;
                        spider.timer = Timer::from_seconds(0.3, TimerMode::Once);
                        vel.0 = Vec2::ZERO;
                    } else {
                        spider.state = AraigneeState::Summon;
                        spider.timer = Timer::from_seconds(0.3, TimerMode::Once);
                        vel.0 = Vec2::ZERO;
                    }
                }
            }
            AraigneeState::LeapTelegraph => {
                vel.0 = Vec2::ZERO;
                if spider.timer.is_finished() {
                    spider.state = AraigneeState::Leap;
                    spider.timer = Timer::from_seconds(0.5, TimerMode::Once);
                    commands.entity(e).insert(Untouchable);
                }
            }
            AraigneeState::Leap => {
                vel.0 = Vec2::ZERO;
                let t = spider.timer.fraction();
                let arc = (t * std::f32::consts::PI).sin() * 55.0;
                let xy = spider.leap_from.lerp(spider.leap_to, t);
                tf.translation.x = xy.x;
                tf.translation.y = xy.y + arc;
                if spider.timer.is_finished() {
                    commands.entity(e).remove::<Untouchable>();
                    let land = spider.leap_to;
                    tf.translation.x = land.x;
                    tf.translation.y = land.y;
                    if land.distance(player_pos) < 95.0 + player_r.0 && iframes.0.is_finished() {
                        dmg.write(DamageMsg {
                            target: player_e,
                            amount: 15.0,
                            kind: DamageKind::Hit,
                        });
                    }
                    // Éclaboussure de toile autour du point de chute.
                    for i in 0..8 {
                        let d = Vec2::from_angle(i as f32 / 8.0 * std::f32::consts::TAU);
                        spawn_enemy_projectile(
                            &mut commands,
                            land + d * 18.0,
                            d * 170.0,
                            7.0,
                            Color::srgb(0.85, 0.85, 0.9),
                        );
                    }
                    spider.state = AraigneeState::Chase;
                    spider.timer = Timer::from_seconds(1.4, TimerMode::Once);
                }
            }
            AraigneeState::WebBurst => {
                vel.0 = Vec2::ZERO;
                if spider.timer.is_finished() {
                    // Jet de toile en éventail radial dense.
                    for i in 0..12 {
                        let d = Vec2::from_angle(i as f32 / 12.0 * std::f32::consts::TAU);
                        spawn_enemy_projectile(
                            &mut commands,
                            pos + d * 22.0,
                            d * 215.0,
                            6.0,
                            Color::srgb(0.85, 0.85, 0.9),
                        );
                    }
                    spider.state = AraigneeState::Chase;
                    spider.timer = Timer::from_seconds(1.2, TimerMode::Once);
                }
            }
            AraigneeState::Summon => {
                vel.0 = Vec2::ZERO;
                if spider.timer.is_finished() {
                    // Pond 2-3 araignéeaux (petites araignées).
                    for _ in 0..rng.random_range(2..=3) {
                        let off = Vec2::new(
                            rng.random_range(-40.0..40.0),
                            rng.random_range(-40.0..40.0),
                        );
                        spawn_enemy(&mut commands, EnemyKind::Araignee, pos + off, 1.0, 1.0, false);
                    }
                    spider.state = AraigneeState::Chase;
                    spider.timer = Timer::from_seconds(1.6, TimerMode::Once);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Roger le Scorpion (Savane) : strafe, charge de pinces, salves de dard.
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Scorpion {
    state: ScorpionState,
    timer: Timer,
    charge_dir: Vec2,
    volley_left: u32,
}

enum ScorpionState {
    Strafe,
    Telegraph,
    Charge,
    Volley,
}

fn scorpion_ai(
    time: Res<Time>,
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut bosses: Query<(&Transform, &mut Velocity, &mut Sprite, &BaseColor, &mut Scorpion)>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let mut rng = rand::rng();
    for (tf, mut vel, mut sprite, base, mut scorpion) in &mut bosses {
        scorpion.timer.tick(time.delta());
        let pos = tf.translation.truncate();
        let to_player = player_pos - pos;
        let dir = to_player.normalize_or(Vec2::X);
        match scorpion.state {
            ScorpionState::Strafe => {
                let dist = to_player.length();
                let orbit = Vec2::new(-dir.y, dir.x) * 150.0;
                let approach = if dist > 260.0 {
                    dir * 130.0
                } else if dist < 200.0 {
                    -dir * 130.0
                } else {
                    Vec2::ZERO
                };
                vel.0 = vel.0.move_towards(orbit + approach, 800.0 * time.delta_secs());
                if scorpion.timer.is_finished() {
                    if rng.random_bool(0.55) {
                        scorpion.state = ScorpionState::Telegraph;
                        scorpion.timer = Timer::from_seconds(0.45, TimerMode::Once);
                    } else {
                        scorpion.state = ScorpionState::Volley;
                        scorpion.timer = Timer::from_seconds(0.25, TimerMode::Once);
                        scorpion.volley_left = 3;
                    }
                }
            }
            ScorpionState::Telegraph => {
                vel.0 = vel.0.move_towards(Vec2::ZERO, 1200.0 * time.delta_secs());
                sprite.color = base.0.mix(&Color::srgb(1.0, 0.1, 0.1), 0.6);
                scorpion.charge_dir = dir;
                if scorpion.timer.is_finished() {
                    sprite.color = base.0;
                    scorpion.state = ScorpionState::Charge;
                    scorpion.timer = Timer::from_seconds(0.55, TimerMode::Once);
                }
            }
            ScorpionState::Charge => {
                // Charge de pinces.
                vel.0 = scorpion.charge_dir * 600.0;
                if scorpion.timer.is_finished() {
                    scorpion.state = ScorpionState::Strafe;
                    scorpion.timer = Timer::from_seconds(1.4, TimerMode::Once);
                }
            }
            ScorpionState::Volley => {
                vel.0 = vel.0.move_towards(Vec2::ZERO, 1000.0 * time.delta_secs());
                if scorpion.timer.is_finished() {
                    scorpion.volley_left = scorpion.volley_left.saturating_sub(1);
                    // Dard venimeux en éventail.
                    for spread in [-0.22, 0.0, 0.22] {
                        let d = Vec2::from_angle(dir.to_angle() + spread);
                        spawn_enemy_projectile(
                            &mut commands,
                            pos + d * 24.0,
                            d * 320.0,
                            7.0,
                            Color::srgb(0.6, 0.9, 0.3),
                        );
                    }
                    if scorpion.volley_left == 0 {
                        scorpion.state = ScorpionState::Strafe;
                        scorpion.timer = Timer::from_seconds(1.4, TimerMode::Once);
                    } else {
                        scorpion.timer = Timer::from_seconds(0.3, TimerMode::Once);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Grompaud (Jungle, clin d'œil au Gromp de LoL) : bonds AoE, langue, crachats.
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Gromp {
    state: GrompState,
    timer: Timer,
    leap_from: Vec2,
    leap_to: Vec2,
}

enum GrompState {
    Chase,
    LeapTelegraph,
    Leap,
    TongueTelegraph { dir: Vec2 },
}

fn gromp_ai(
    time: Res<Time>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    player: Query<(Entity, &Transform, &Radius, &crate::player::Iframes), With<Player>>,
    mut bosses: Query<(Entity, &mut Transform, &mut Velocity, &mut Gromp), Without<Player>>,
) {
    let Ok((player_e, player_tf, player_r, iframes)) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let mut rng = rand::rng();
    for (e, mut tf, mut vel, mut gromp) in &mut bosses {
        gromp.timer.tick(time.delta());
        let pos = tf.translation.truncate();
        let dir = (player_pos - pos).normalize_or(Vec2::X);
        match gromp.state {
            GrompState::Chase => {
                vel.0 = vel.0.move_towards(dir * 60.0, 400.0 * time.delta_secs());
                if gromp.timer.is_finished() {
                    if rng.random_bool(0.6) {
                        gromp.state = GrompState::LeapTelegraph;
                        gromp.timer = Timer::from_seconds(0.45, TimerMode::Once);
                        gromp.leap_from = pos;
                        gromp.leap_to = player_pos;
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
                        gromp.state = GrompState::TongueTelegraph { dir };
                        gromp.timer = Timer::from_seconds(0.35, TimerMode::Once);
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
            GrompState::LeapTelegraph => {
                vel.0 = Vec2::ZERO;
                if gromp.timer.is_finished() {
                    gromp.state = GrompState::Leap;
                    gromp.timer = Timer::from_seconds(0.55, TimerMode::Once);
                    commands.entity(e).insert(Untouchable);
                }
            }
            GrompState::Leap => {
                vel.0 = Vec2::ZERO;
                let t = gromp.timer.fraction();
                let arc = (t * std::f32::consts::PI).sin() * 60.0;
                let xy = gromp.leap_from.lerp(gromp.leap_to, t);
                tf.translation.x = xy.x;
                tf.translation.y = xy.y + arc;
                if gromp.timer.is_finished() {
                    commands.entity(e).remove::<Untouchable>();
                    tf.translation.x = gromp.leap_to.x;
                    tf.translation.y = gromp.leap_to.y;
                    let land = gromp.leap_to;
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
                    gromp.state = GrompState::Chase;
                    gromp.timer = Timer::from_seconds(1.1, TimerMode::Once);
                }
            }
            GrompState::TongueTelegraph { dir } => {
                vel.0 = Vec2::ZERO;
                if gromp.timer.is_finished() {
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
                    gromp.state = GrompState::Chase;
                    gromp.timer = Timer::from_seconds(1.0, TimerMode::Once);
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
// Méga-Limace (Potager) : reptation baveuse, crachat radial, ponte de limaces.
// Portée de feat/player (« Giga Limace ») — la reptation + traînée de bave est
// fidèle à l'original ; le crachat et la ponte la dotent des 3 patterns
// attendus d'un boss (GDD §8.2), au niveau des autres boss de test.
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct MegaLimace {
    state: LimaceState,
    timer: Timer,
    /// Cadence de dépose de la traînée de bave (en reptation).
    trail: Timer,
}

enum LimaceState {
    Crawl,
    SpitTelegraph,
    Spit,
    Summon,
}

fn mega_limace_ai(
    time: Res<Time>,
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut bosses: Query<(&Transform, &mut Velocity, &mut MegaLimace), Without<Player>>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let mut rng = rand::rng();
    let dt = time.delta_secs();

    for (tf, mut vel, mut slug) in &mut bosses {
        slug.timer.tick(time.delta());
        slug.trail.tick(time.delta());
        let pos = tf.translation.truncate();
        let dir = (player_pos - pos).normalize_or(Vec2::X);

        match slug.state {
            LimaceState::Crawl => {
                // Lente mais inexorable, en laissant une traînée de bave toxique.
                vel.0 = vel.0.move_towards(dir * 55.0, 300.0 * dt);
                if slug.trail.just_finished() {
                    commands.spawn((
                        Sprite::from_color(
                            Color::srgb(0.8, 0.9, 0.2).with_alpha(0.4),
                            Vec2::splat(45.0),
                        ),
                        Transform::from_translation(pos.extend(-1.0)),
                        HazardPuddle {
                            dps: 12.0,
                            radius: 25.0,
                            life: Timer::from_seconds(4.0, TimerMode::Once),
                            tick: Timer::from_seconds(0.5, TimerMode::Repeating),
                        },
                    ));
                }
                if slug.timer.is_finished() {
                    if rng.random_bool(0.6) {
                        slug.state = LimaceState::SpitTelegraph;
                        slug.timer = Timer::from_seconds(0.4, TimerMode::Once);
                    } else {
                        slug.state = LimaceState::Summon;
                        slug.timer = Timer::from_seconds(0.3, TimerMode::Once);
                    }
                }
            }
            LimaceState::SpitTelegraph => {
                vel.0 = vel.0.move_towards(Vec2::ZERO, 600.0 * dt);
                if slug.timer.is_finished() {
                    slug.state = LimaceState::Spit;
                    slug.timer = Timer::from_seconds(0.2, TimerMode::Once);
                }
            }
            LimaceState::Spit => {
                vel.0 = Vec2::ZERO;
                if slug.timer.is_finished() {
                    // Crachat de bave en couronne radiale.
                    for i in 0..12 {
                        let d = Vec2::from_angle(i as f32 / 12.0 * std::f32::consts::TAU);
                        spawn_enemy_projectile(
                            &mut commands,
                            pos + d * 26.0,
                            d * 200.0,
                            8.0,
                            Color::srgb(0.7, 0.9, 0.25),
                        );
                    }
                    slug.state = LimaceState::Crawl;
                    slug.timer = Timer::from_seconds(1.4, TimerMode::Once);
                }
            }
            LimaceState::Summon => {
                vel.0 = Vec2::ZERO;
                if slug.timer.is_finished() {
                    // Ponte de 2 rejetons (petits escargots baveux, allégés
                    // pour rester au niveau des anciennes limaces invoquées).
                    for _ in 0..2 {
                        let off = Vec2::new(
                            rng.random_range(-45.0..45.0),
                            rng.random_range(-45.0..45.0),
                        );
                        spawn_enemy(&mut commands, EnemyKind::Escargot, pos + off, 0.45, 0.8, false);
                    }
                    slug.state = LimaceState::Crawl;
                    slug.timer = Timer::from_seconds(1.6, TimerMode::Once);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mille-Pattes (Gravier) : corps segmenté qui suit la tête. 3 patterns —
// ruée en ligne (le corps blesse), salve en éventail, traînée de bave toxique.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum MpState {
    /// Reptation vers le joueur ; dépose une traînée de bave.
    Crawl,
    /// Se cabre avant la ruée (tell rouge, immobile).
    ChargeTelegraph,
    /// Ruée en ligne : déplacement rapide, tête + corps dangereux.
    Charge,
    /// Salve de projectiles en éventail.
    Volley,
}

#[derive(Component)]
pub struct MillePattes {
    state: MpState,
    timer: Timer,
    charge_dir: Vec2,
    /// Angle de facing de la tête (oriente le corps segmenté).
    heading: f32,
    trail_drop: Timer,
    volley_left: u32,
    seg_count: usize,
    seg_spacing: f32,
}

#[allow(clippy::too_many_arguments)]
fn mille_pattes_ai(
    time: Res<Time>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    mut player: Query<(Entity, &Transform, &Radius, &mut crate::player::Iframes), With<Player>>,
    mut bosses: Query<
        (&mut Transform, &mut Velocity, &mut Sprite, &BaseColor, &mut MillePattes),
        Without<Player>,
    >,
) {
    let Ok((player_e, player_tf, player_r, mut iframes)) = player.single_mut() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_secs();
    let mut rng = rand::rng();

    for (mut tf, mut vel, mut sprite, base, mut mp) in &mut bosses {
        mp.timer.tick(time.delta());
        mp.trail_drop.tick(time.delta());
        let pos = tf.translation.truncate();
        let dir = (player_pos - pos).normalize_or(Vec2::X);

        match mp.state {
            MpState::Crawl => {
                vel.0 = vel.0.move_towards(dir * 95.0, 500.0 * dt);
                sprite.color = base.0;
                // Traînée de bave toxique laissée derrière la tête.
                if mp.trail_drop.just_finished() {
                    commands.spawn((
                        Sprite::from_color(
                            Color::srgb(0.5, 0.55, 0.2).with_alpha(0.4),
                            Vec2::splat(46.0),
                        ),
                        Transform::from_translation(pos.extend(-3.0)),
                        HazardPuddle {
                            dps: 7.0,
                            radius: 24.0,
                            life: Timer::from_seconds(3.0, TimerMode::Once),
                            tick: Timer::from_seconds(0.4, TimerMode::Repeating),
                        },
                    ));
                }
                if mp.timer.is_finished() {
                    if rng.random_bool(0.6) {
                        mp.state = MpState::ChargeTelegraph;
                        mp.timer = Timer::from_seconds(0.55, TimerMode::Once);
                        mp.charge_dir = dir;
                    } else {
                        mp.state = MpState::Volley;
                        mp.timer = Timer::from_seconds(0.25, TimerMode::Once);
                        mp.volley_left = 3;
                    }
                }
            }
            MpState::ChargeTelegraph => {
                vel.0 = vel.0.move_towards(Vec2::ZERO, 1200.0 * dt);
                sprite.color = base.0.mix(&Color::srgb(1.0, 0.15, 0.1), 0.6);
                if mp.timer.is_finished() {
                    sprite.color = base.0;
                    mp.state = MpState::Charge;
                    mp.timer = Timer::from_seconds(0.5, TimerMode::Once);
                }
            }
            MpState::Charge => {
                vel.0 = mp.charge_dir * 540.0;
                // Le corps blesse pendant la ruée (la tête, elle, est gérée par
                // le système de contact générique du boss).
                if iframes.0.is_finished() {
                    let mut hit = false;
                    for i in 1..=mp.seg_count {
                        let seg = pos - mp.charge_dir * (mp.seg_spacing * i as f32);
                        if seg.distance(player_pos) <= 16.0 + player_r.0 {
                            hit = true;
                            break;
                        }
                    }
                    if hit {
                        dmg.write(DamageMsg {
                            target: player_e,
                            amount: 12.0,
                            kind: DamageKind::Hit,
                        });
                        iframes.0 = Timer::from_seconds(0.5, TimerMode::Once);
                    }
                }
                if mp.timer.is_finished() {
                    mp.state = MpState::Crawl;
                    mp.timer = Timer::from_seconds(1.6, TimerMode::Once);
                }
            }
            MpState::Volley => {
                vel.0 = vel.0.move_towards(Vec2::ZERO, 1000.0 * dt);
                if mp.timer.is_finished() {
                    mp.volley_left = mp.volley_left.saturating_sub(1);
                    let base_angle = dir.to_angle();
                    for spread in [-0.4, -0.2, 0.0, 0.2, 0.4] {
                        let d = Vec2::from_angle(base_angle + spread);
                        spawn_enemy_projectile(
                            &mut commands,
                            pos + d * 24.0,
                            d * 300.0,
                            8.0,
                            Color::srgb(0.95, 0.16, 0.12),
                        );
                    }
                    if mp.volley_left == 0 {
                        mp.state = MpState::Crawl;
                        mp.timer = Timer::from_seconds(1.4, TimerMode::Once);
                    } else {
                        mp.timer = Timer::from_seconds(0.3, TimerMode::Once);
                    }
                }
            }
        }

        // Oriente la tête (et donc le corps segmenté greffé en enfant) selon le
        // déplacement, ou la direction de ruée (snap immédiat).
        let snap = matches!(mp.state, MpState::Charge | MpState::ChargeTelegraph);
        let desired = if snap {
            mp.charge_dir
        } else if vel.0.length() > 10.0 {
            vel.0.normalize()
        } else {
            Vec2::from_angle(mp.heading)
        };
        let blend = if snap { 1.0 } else { (dt * 8.0).min(1.0) };
        let cur = Vec2::from_angle(mp.heading);
        let nd = cur.lerp(desired, blend).normalize_or(desired);
        mp.heading = nd.to_angle();
        tf.rotation = Quat::from_rotation_z(mp.heading);
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
            (
                araignee_ai,
                scorpion_ai,
                gromp_ai,
                mega_limace_ai,
                mille_pattes_ai,
                glob_system,
            )
                .in_set(GameSet::Ai)
                .run_if(combat_active),
        );
    }
}
