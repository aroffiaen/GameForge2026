//! Le bestiaire : faune réelle du jardin, à l'échelle mini (GDD §8).
//!
//! Refonte v0.3 (§18.D) : **6 mobs = 3 archétypes × 2 types**, et **plus de
//! dégâts de collision** — chaque archétype a sa propre attaque :
//!   • Chase (Fourmi, Escargot)  → mêlée **télégraphée** (windup → coup).
//!   • Lunge (Araignée, Criquet) → dégâts **pendant la ruée**.
//!   • Range (Guêpe, Cigale)     → **boule rouge** (projectile).

use bevy::prelude::*;
use rand::prelude::*;

use crate::common::*;
use crate::player::Iframes;

// ---------------------------------------------------------------------------
// Définitions
// ---------------------------------------------------------------------------

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyKind {
    // Chase (poursuite, mêlée)
    Fourmi,
    Escargot,
    // Lunge (ruée)
    Araignee,
    Criquet,
    // Range (tir)
    Guepe,
    Cigale,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AiKind {
    /// Fonce sur le joueur et frappe au corps-à-corps (mêlée télégraphée).
    Chase { reach: f32, windup: f32, recover: f32, cd: f32 },
    /// Tient ses distances, se ramasse (windup télégraphié) puis bondit ;
    /// inflige des dégâts pendant le bond.
    Lunge { trigger: f32, windup: f32, speed_mult: f32, active: f32, cd: f32 },
    /// Garde ses distances et tire une boule rouge.
    Ranged { min: f32, max: f32, shoot_cd: f32 },
}

pub struct EnemyDef {
    #[allow(dead_code)] // affichage futur (bestiaire, nametags, codex…)
    pub name: &'static str,
    pub hp: f32,
    pub speed: f32,
    /// Dégâts d'attaque (mêlée / ruée / projectile) — plus de dégâts de contact.
    pub dmg: f32,
    pub radius: f32,
    pub pattes: u32,
    pub color: Color,
    pub ai: AiKind,
    /// Multiplicateur des dégâts de poison subis (les « mous », GDD §8.1).
    pub poison_vuln: f32,
}

pub fn def(kind: EnemyKind) -> EnemyDef {
    match kind {
        // ---- Chase : mêlée télégraphée ----
        EnemyKind::Fourmi => EnemyDef {
            name: "Fourmi",
            hp: 16.0,
            speed: 150.0,
            dmg: 6.0,
            radius: 9.0,
            pattes: 1,
            color: Color::srgb(0.45, 0.25, 0.15),
            ai: AiKind::Chase { reach: 24.0, windup: 0.35, recover: 0.18, cd: 0.7 },
            poison_vuln: 1.0,
        },
        EnemyKind::Escargot => EnemyDef {
            name: "Escargot",
            hp: 95.0,
            speed: 42.0,
            dmg: 12.0,
            radius: 17.0,
            pattes: 4,
            color: Color::srgb(0.65, 0.55, 0.4),
            ai: AiKind::Chase { reach: 30.0, windup: 0.6, recover: 0.4, cd: 1.2 },
            poison_vuln: 1.5, // « mou » : encaisse plus de poison
        },
        // ---- Lunge : dégâts pendant la ruée ----
        EnemyKind::Araignee => EnemyDef {
            name: "Araignée",
            hp: 28.0,
            speed: 165.0,
            dmg: 9.0,
            radius: 11.0,
            pattes: 3,
            color: Color::srgb(0.25, 0.22, 0.3),
            ai: AiKind::Lunge {
                trigger: 240.0,
                windup: 0.25,
                speed_mult: 3.0,
                active: 0.42,
                cd: 1.8,
            },
            poison_vuln: 1.0,
        },
        EnemyKind::Criquet => EnemyDef {
            name: "Criquet",
            hp: 20.0,
            speed: 140.0,
            dmg: 7.0,
            radius: 10.0,
            pattes: 2,
            color: Color::srgb(0.45, 0.6, 0.25),
            ai: AiKind::Lunge {
                trigger: 260.0,
                windup: 0.16,
                speed_mult: 3.6,
                active: 0.34,
                cd: 1.4,
            },
            poison_vuln: 1.0,
        },
        // ---- Range : boule rouge ----
        EnemyKind::Guepe => EnemyDef {
            name: "Guêpe",
            hp: 22.0,
            speed: 145.0,
            dmg: 7.0,
            radius: 11.0,
            pattes: 3,
            color: Color::srgb(0.95, 0.8, 0.1),
            ai: AiKind::Ranged { min: 150.0, max: 240.0, shoot_cd: 1.4 },
            poison_vuln: 1.0,
        },
        EnemyKind::Cigale => EnemyDef {
            name: "Cigale",
            hp: 34.0,
            speed: 110.0,
            dmg: 10.0,
            radius: 13.0,
            pattes: 3,
            color: Color::srgb(0.4, 0.5, 0.55),
            ai: AiKind::Ranged { min: 180.0, max: 290.0, shoot_cd: 2.4 },
            poison_vuln: 1.0,
        },
    }
}

/// Couleur des projectiles ennemis : une « boule rouge » lisible (GDD §8).
const ENEMY_BALL: Color = Color::srgb(0.95, 0.16, 0.12);

// ---------------------------------------------------------------------------
// Composants
// ---------------------------------------------------------------------------

/// Pattes lâchées à la mort.
#[derive(Component)]
pub struct PattesDrop(pub u32);

/// Vitesse de déplacement effective de cet ennemi.
#[derive(Component)]
pub struct AiSpeed(pub f32);

#[derive(Component)]
pub struct Elite;

/// Phases de l'attaque de mêlée télégraphée des « chase » (GDD §8).
#[derive(Clone, Copy, PartialEq)]
pub enum MeleeState {
    /// Au repos / poursuite ; `timer` sert de cooldown entre deux coups.
    Idle,
    /// Charge le coup (tell visuel) ; l'ennemi est enraciné.
    Windup,
    /// Le coup porte (bref) ; flash visuel.
    Strike,
    /// Récupération avant de pouvoir repoursuivre.
    Recover,
}

#[derive(Component)]
pub struct MeleeAttack {
    pub state: MeleeState,
    pub timer: Timer,
    pub reach: f32,
    pub windup: f32,
    pub recover: f32,
    pub cd: f32,
}

/// Phases du bond des « lunge ».
#[derive(Clone, Copy, PartialEq)]
pub enum LungePhase {
    /// Tient ses distances en attendant le cooldown.
    Ready,
    /// Se ramasse avant de bondir (tell visuel, enraciné).
    Windup,
    /// Bond : déplacement rapide, inflige des dégâts au contact.
    Burst,
}

#[derive(Component)]
pub struct LungeState {
    pub phase: LungePhase,
    /// Timer de la phase courante (Windup puis Burst).
    pub timer: Timer,
    /// Cooldown entre deux bonds (avance en phase Ready).
    pub cd: Timer,
    pub dir: Vec2,
    pub speed_mult: f32,
    pub trigger: f32,
    pub windup: f32,
    pub active: f32,
    pub base_cd: f32,
    /// Le coup de ce bond a-t-il déjà porté ? (un seul hit par bond)
    pub hit_done: bool,
}

#[derive(Component)]
pub struct ShootCd(pub Timer);

/// Projectile tiré par un ennemi.
#[derive(Component)]
pub struct EnemyProjectile {
    pub dmg: f32,
}

/// Flaque dangereuse pour le joueur (crachats du crapaud…).
#[derive(Component)]
pub struct HazardPuddle {
    pub dps: f32,
    pub radius: f32,
    pub life: Timer,
    pub tick: Timer,
}

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Fait apparaître un ennemi. Le nettoyage (DespawnOnExit…) est ajouté par
/// l'appelant selon l'état courant.
pub fn spawn_enemy(
    commands: &mut Commands,
    kind: EnemyKind,
    pos: Vec2,
    hp_scale: f32,
    dmg_scale: f32,
    elite: bool,
) -> Entity {
    let d = def(kind);
    let mut rng = rand::rng();

    // Stats de base, puis bonus d'élite : plus gros + UNE stat boostée au
    // hasard (PV / vitesse / dégâts), GDD §8.
    let mut hp = d.hp * hp_scale;
    let mut dmg = d.dmg * dmg_scale;
    let mut speed = d.speed;
    let mut radius = d.radius;
    let mut pattes = d.pattes;
    if elite {
        radius *= 1.5;
        pattes *= 5;
        hp *= 1.8; // socle : un élite encaisse toujours un peu plus
        match rng.random_range(0..3) {
            0 => hp *= 2.0,   // élite « tank »
            1 => speed *= 1.6, // élite « rapide »
            _ => dmg *= 2.2,   // élite « cogneur »
        }
    }
    let color = if elite {
        d.color.mix(&Color::srgb(0.7, 0.2, 0.9), 0.4)
    } else {
        d.color
    };

    let mut e = commands.spawn((
        Enemy,
        kind,
        Sprite::from_color(color, Vec2::splat(radius * 1.9)),
        BaseColor(color),
        Transform::from_translation(pos.extend(8.0)).with_rotation(Quat::from_rotation_z(
            rng.random_range(-0.4..0.4_f32),
        )),
        Velocity::default(),
        Knockback::default(),
        Health::new(hp),
        Radius(radius),
        ClampToArena,
        // Plus de dégâts de contact (GDD §18.D) : `ContactDmg` porte désormais
        // les dégâts d'ATTAQUE (mêlée / ruée / projectile) du mob.
        ContactDmg(dmg),
        PattesDrop(pattes),
        AiSpeed(speed),
    ));
    match d.ai {
        AiKind::Chase { reach, windup, recover, cd } => {
            e.insert(MeleeAttack {
                state: MeleeState::Idle,
                timer: Timer::from_seconds(cd * rng.random_range(0.3..1.0), TimerMode::Once),
                reach,
                windup,
                recover,
                cd,
            });
        }
        AiKind::Lunge { trigger, windup, speed_mult, active, cd } => {
            e.insert(LungeState {
                phase: LungePhase::Ready,
                timer: Timer::from_seconds(windup, TimerMode::Once),
                cd: Timer::from_seconds(cd * rng.random_range(0.5..1.3), TimerMode::Once),
                dir: Vec2::X,
                speed_mult,
                trigger,
                windup,
                active,
                base_cd: cd,
                hit_done: true, // aucun bond en cours au spawn
            });
        }
        AiKind::Ranged { shoot_cd, .. } => {
            e.insert(ShootCd(Timer::from_seconds(
                shoot_cd * rng.random_range(0.5..1.2),
                TimerMode::Once,
            )));
        }
    }
    if elite {
        e.insert(Elite);
    }
    let id = e.id();
    drop(e); // libère l'emprunt de `commands` avant de greffer la barre

    // Barre de vie flottante au-dessus du mob (apparaît dès qu'il est blessé).
    let bar_w = (radius * 2.0).clamp(22.0, 64.0);
    crate::healthbar::spawn_health_bar(commands, id, radius + 12.0, bar_w, false, None);
    id
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (ai_movement, enemy_shoot, separation, enemy_tint)
                .in_set(GameSet::Ai)
                .run_if(combat_active),
        )
        .add_systems(
            Update,
            (melee_attacks, lunge_damage, enemy_projectiles, hazard_puddles)
                .in_set(GameSet::Combat)
                .run_if(combat_active),
        )
        // Le joueur bloque les mobs : on de-chevauche après le combat pour que
        // les attaques de contact (ruée) aient pu s'enregistrer.
        .add_systems(
            Update,
            player_repulsion.in_set(GameSet::Post).run_if(combat_active),
        );
    }
}

// ---------------------------------------------------------------------------
// Systèmes
// ---------------------------------------------------------------------------

fn ai_movement(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<
        (
            &EnemyKind,
            &AiSpeed,
            &Transform,
            &mut Velocity,
            Option<&mut LungeState>,
            Option<&MeleeAttack>,
            Has<Slowed>,
        ),
        (With<Enemy>, Without<Player>, Without<crate::boss::BossAiTag>),
    >,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_secs();

    for (kind, speed, tf, mut vel, lunge, melee, slowed) in &mut enemies {
        let pos = tf.translation.truncate();
        let to_player = player_pos - pos;
        let dist = to_player.length();
        let dir = to_player.normalize_or_zero();
        let mut max = speed.0;
        if slowed {
            max *= 0.45;
        }

        // Vrai pour le bond du lunge : la vitesse est imposée d'un coup (sinon
        // le lissage l'empêche de franchir l'écart et le bond avorte).
        let mut snap = false;
        let target = match def(*kind).ai {
            AiKind::Chase { .. } => {
                // Enraciné dès que le coup se prépare (windup/strike/recover).
                if melee.is_some_and(|m| m.state != MeleeState::Idle) {
                    Vec2::ZERO
                } else {
                    dir * max
                }
            }
            AiKind::Lunge { .. } => {
                if let Some(mut lunge) = lunge {
                    lunge.timer.tick(time.delta());
                    lunge.cd.tick(time.delta());
                    match lunge.phase {
                        LungePhase::Ready => {
                            // Tient ses distances : ne reste pas collé au joueur.
                            let mv = if dist < 80.0 {
                                -dir * max
                            } else if dist > lunge.trigger {
                                dir * max
                            } else {
                                Vec2::new(-dir.y, dir.x) * max * 0.5
                            };
                            // Déclenche un bond depuis une distance moyenne.
                            if lunge.cd.is_finished() && (80.0..=lunge.trigger).contains(&dist) {
                                lunge.phase = LungePhase::Windup;
                                lunge.timer = Timer::from_seconds(lunge.windup, TimerMode::Once);
                                lunge.dir = dir;
                            }
                            mv
                        }
                        LungePhase::Windup => {
                            // Ramassé, enraciné (le tell laisse le temps d'esquiver).
                            if lunge.timer.is_finished() {
                                lunge.phase = LungePhase::Burst;
                                lunge.timer = Timer::from_seconds(lunge.active, TimerMode::Once);
                                lunge.hit_done = false;
                            }
                            Vec2::ZERO
                        }
                        LungePhase::Burst => {
                            // Bond engagé dans la direction figée au windup,
                            // vitesse imposée pour franchir l'écart d'un trait.
                            if lunge.timer.is_finished() {
                                lunge.phase = LungePhase::Ready;
                                lunge.cd = Timer::from_seconds(lunge.base_cd, TimerMode::Once);
                            }
                            snap = true;
                            lunge.dir * max * lunge.speed_mult
                        }
                    }
                } else {
                    dir * max
                }
            }
            AiKind::Ranged { min, max: band_max, .. } => {
                if dist < min {
                    -dir * max
                } else if dist > band_max {
                    dir * max
                } else {
                    // Strafe perpendiculaire pour rester vivant.
                    Vec2::new(-dir.y, dir.x) * max * 0.6
                }
            }
        };
        if snap {
            vel.0 = target;
        } else {
            vel.0 = vel.0.move_towards(target, 700.0 * dt);
        }
    }
}

/// Mêlée télégraphée des « chase » : poursuite → windup (tell) → coup → récup.
fn melee_attacks(
    time: Res<Time>,
    mut dmg: MessageWriter<DamageMsg>,
    mut player: Query<
        (Entity, &Transform, &Radius, &mut Knockback, &mut Iframes),
        With<Player>,
    >,
    mut enemies: Query<
        (&Transform, &Radius, &ContactDmg, &mut MeleeAttack),
        (With<Enemy>, Without<crate::boss::BossAiTag>, Without<Player>),
    >,
) {
    let Ok((player_e, player_tf, player_r, mut player_kb, mut iframes)) = player.single_mut()
    else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (tf, radius, atk_dmg, mut atk) in &mut enemies {
        atk.timer.tick(time.delta());
        let pos = tf.translation.truncate();
        let dist = pos.distance(player_pos);
        let in_reach = dist <= atk.reach + radius.0 + player_r.0;
        match atk.state {
            MeleeState::Idle => {
                if atk.timer.is_finished() && in_reach {
                    atk.state = MeleeState::Windup;
                    atk.timer = Timer::from_seconds(atk.windup, TimerMode::Once);
                }
            }
            MeleeState::Windup => {
                if atk.timer.is_finished() {
                    atk.state = MeleeState::Strike;
                    atk.timer = Timer::from_seconds(0.12, TimerMode::Once);
                    // Le coup porte si le joueur est encore à portée (+ marge).
                    if dist <= atk.reach + radius.0 + player_r.0 + 8.0
                        && iframes.0.is_finished()
                    {
                        dmg.write(DamageMsg {
                            target: player_e,
                            amount: atk_dmg.0,
                            kind: DamageKind::Hit,
                        });
                        iframes.0 = Timer::from_seconds(0.6, TimerMode::Once);
                        let kb = (player_pos - pos).normalize_or(Vec2::Y);
                        player_kb.0 += kb * 200.0;
                    }
                }
            }
            MeleeState::Strike => {
                if atk.timer.is_finished() {
                    atk.state = MeleeState::Recover;
                    atk.timer = Timer::from_seconds(atk.recover, TimerMode::Once);
                }
            }
            MeleeState::Recover => {
                if atk.timer.is_finished() {
                    atk.state = MeleeState::Idle;
                    atk.timer = Timer::from_seconds(atk.cd, TimerMode::Once);
                }
            }
        }
    }
}

/// Dégâts infligés pendant la ruée des « lunge » (un seul hit par ruée).
fn lunge_damage(
    mut dmg: MessageWriter<DamageMsg>,
    mut player: Query<
        (Entity, &Transform, &Radius, &mut Knockback, &mut Iframes),
        With<Player>,
    >,
    mut enemies: Query<
        (&Transform, &Radius, &ContactDmg, &mut LungeState),
        (With<Enemy>, Without<crate::boss::BossAiTag>, Without<Player>),
    >,
) {
    let Ok((player_e, player_tf, player_r, mut player_kb, mut iframes)) = player.single_mut()
    else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (tf, radius, atk_dmg, mut lunge) in &mut enemies {
        if lunge.phase != LungePhase::Burst || lunge.hit_done {
            continue;
        }
        let pos = tf.translation.truncate();
        if pos.distance(player_pos) <= radius.0 + player_r.0 + 2.0 && iframes.0.is_finished() {
            dmg.write(DamageMsg {
                target: player_e,
                amount: atk_dmg.0,
                kind: DamageKind::Hit,
            });
            iframes.0 = Timer::from_seconds(0.5, TimerMode::Once);
            let kb = (player_pos - pos).normalize_or(Vec2::Y);
            player_kb.0 += kb * 240.0;
            lunge.hit_done = true;
        }
    }
}

fn enemy_shoot(
    time: Res<Time>,
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(&EnemyKind, &Transform, &ContactDmg, &mut ShootCd), With<Enemy>>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (kind, tf, contact, mut cd) in &mut enemies {
        cd.0.tick(time.delta());
        let AiKind::Ranged { max, shoot_cd, .. } = def(*kind).ai else {
            continue;
        };
        let pos = tf.translation.truncate();
        if cd.0.is_finished() && pos.distance(player_pos) <= max + 40.0 {
            cd.0 = Timer::from_seconds(shoot_cd, TimerMode::Once);
            let dir = (player_pos - pos).normalize_or(Vec2::X);
            spawn_enemy_projectile(&mut commands, pos, dir * 250.0, contact.0, ENEMY_BALL);
        }
    }
}

pub fn spawn_enemy_projectile(
    commands: &mut Commands,
    pos: Vec2,
    vel: Vec2,
    dmg: f32,
    color: Color,
) {
    commands.spawn((
        EnemyProjectile { dmg },
        Sprite::from_color(color.mix(&Color::WHITE, 0.18), Vec2::splat(8.0)),
        Transform::from_translation(pos.extend(9.0)),
        Velocity(vel),
        Lifetime::secs(3.0),
    ));
}

fn enemy_projectiles(
    mut commands: Commands,
    arena: Res<Arena>,
    mut dmg: MessageWriter<DamageMsg>,
    projectiles: Query<(Entity, &Transform, &EnemyProjectile)>,
    mut player: Query<(Entity, &Transform, &Radius, &mut Iframes), With<Player>>,
) {
    let Ok((player_e, player_tf, player_r, mut iframes)) = player.single_mut() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (e, tf, proj) in &projectiles {
        let pos = tf.translation.truncate();
        if pos.x.abs() > arena.half.x + 30.0 || pos.y.abs() > arena.half.y + 30.0 {
            commands.entity(e).despawn();
            continue;
        }
        if pos.distance(player_pos) <= player_r.0 + 5.0 {
            if iframes.0.is_finished() {
                dmg.write(DamageMsg {
                    target: player_e,
                    amount: proj.dmg,
                    kind: DamageKind::Hit,
                });
                iframes.0 = Timer::from_seconds(0.5, TimerMode::Once);
            }
            commands.entity(e).despawn();
        }
    }
}

fn hazard_puddles(
    time: Res<Time>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    mut puddles: Query<(Entity, &Transform, &mut HazardPuddle, &mut Sprite)>,
    player: Query<(Entity, &Transform, &Radius), With<Player>>,
) {
    let Ok((player_e, player_tf, player_r)) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (e, tf, mut puddle, mut sprite) in &mut puddles {
        puddle.life.tick(time.delta());
        puddle.tick.tick(time.delta());
        let left = 1.0 - puddle.life.fraction();
        if left < 0.3 {
            sprite.color = sprite.color.with_alpha(0.45 * (left / 0.3));
        }
        if puddle.life.is_finished() {
            commands.entity(e).despawn();
            continue;
        }
        if puddle.tick.just_finished()
            && tf.translation.truncate().distance(player_pos) <= puddle.radius + player_r.0
        {
            dmg.write(DamageMsg {
                target: player_e,
                amount: puddle.dps * puddle.tick.duration().as_secs_f32(),
                kind: DamageKind::Poison,
            });
        }
    }
}

/// Évite l'empilement parfait des ennemis (séparation O(n²), n reste petit).
fn separation(mut enemies: Query<(&mut Transform, &Radius), With<Enemy>>) {
    let mut pairs = enemies.iter_combinations_mut();
    while let Some([(mut tf_a, r_a), (mut tf_b, r_b)]) = pairs.fetch_next() {
        let a = tf_a.translation.truncate();
        let b = tf_b.translation.truncate();
        let min_dist = (r_a.0 + r_b.0) * 0.9;
        let delta = b - a;
        let dist = delta.length();
        if dist > 0.001 && dist < min_dist {
            let push = delta / dist * (min_dist - dist) * 0.5;
            tf_a.translation -= push.extend(0.0);
            tf_b.translation += push.extend(0.0);
        }
    }
}

/// Le joueur bloque les mobs : on les repousse hors de son cercle (le joueur,
/// lui, ne bouge pas). Le boss est exclu (il garde ses dégâts de contact).
fn player_repulsion(
    player: Query<(&Transform, &Radius), With<Player>>,
    mut enemies: Query<
        (&mut Transform, &Radius),
        (With<Enemy>, Without<Player>, Without<crate::boss::BossAiTag>),
    >,
) {
    let Ok((player_tf, player_r)) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (mut tf, radius) in &mut enemies {
        let pos = tf.translation.truncate();
        let min_dist = player_r.0 + radius.0;
        let delta = pos - player_pos;
        let dist = delta.length();
        if dist > 0.001 && dist < min_dist {
            tf.translation += (delta / dist * (min_dist - dist)).extend(0.0);
        }
    }
}

/// Teintes des ennemis : poison (vert) + tell de mêlée (windup → rouge,
/// strike → flash jaune). Le flash de dégât (HitFlash) reste prioritaire.
fn enemy_tint(
    mut enemies: Query<
        (
            &mut Sprite,
            &BaseColor,
            Has<Poisoned>,
            Option<&MeleeAttack>,
            Option<&LungeState>,
        ),
        (With<Enemy>, Without<HitFlash>),
    >,
) {
    for (mut sprite, base, poisoned, melee, lunge) in &mut enemies {
        let mut color = base.0;
        if poisoned {
            color = color.mix(&Color::srgb(0.3, 1.0, 0.2), 0.45);
        }
        if let Some(m) = melee {
            match m.state {
                MeleeState::Windup => {
                    // Vire au rouge à mesure que le coup se charge.
                    let f = m.timer.fraction();
                    color = color.mix(&Color::srgb(1.0, 0.25, 0.1), 0.35 + 0.45 * f);
                }
                MeleeState::Strike => {
                    color = color.mix(&Color::srgb(1.0, 0.95, 0.4), 0.75);
                }
                _ => {}
            }
        }
        if let Some(l) = lunge {
            match l.phase {
                LungePhase::Windup => {
                    // Se ramasse : tell rouge qui s'intensifie avant le bond.
                    let f = l.timer.fraction();
                    color = color.mix(&Color::srgb(1.0, 0.25, 0.1), 0.4 + 0.4 * f);
                }
                LungePhase::Burst => {
                    color = color.mix(&Color::srgb(1.0, 0.55, 0.1), 0.6);
                }
                LungePhase::Ready => {}
            }
        }
        sprite.color = color;
    }
}
