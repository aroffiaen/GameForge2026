//! Le bestiaire : faune réelle du jardin, à l'échelle mini (GDD §8).

use bevy::prelude::*;
use rand::prelude::*;

use crate::common::*;
use crate::player::Iframes;

// ---------------------------------------------------------------------------
// Définitions
// ---------------------------------------------------------------------------

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyKind {
    Puceron,
    Fourmi,
    Araignee,
    Moustique,
    Guepe,
    Scarabee,
    Escargot,
    Limace,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AiKind {
    /// Fonce sur le joueur.
    Chase,
    /// Charge en pointe quand il est proche (araignée).
    Lunge,
    /// Garde ses distances et tire.
    Ranged { min: f32, max: f32, shoot_cd: f32 },
}

pub struct EnemyDef {
    #[allow(dead_code)] // affichage futur (bestiaire, codex…)
    pub name: &'static str,
    pub hp: f32,
    pub speed: f32,
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
        EnemyKind::Puceron => EnemyDef {
            name: "Puceron",
            hp: 8.0,
            speed: 115.0,
            dmg: 4.0,
            radius: 8.0,
            pattes: 1,
            color: Color::srgb(0.55, 0.85, 0.45),
            ai: AiKind::Chase,
            poison_vuln: 1.0,
        },
        EnemyKind::Fourmi => EnemyDef {
            name: "Fourmi",
            hp: 14.0,
            speed: 150.0,
            dmg: 5.0,
            radius: 9.0,
            pattes: 1,
            color: Color::srgb(0.45, 0.25, 0.15),
            ai: AiKind::Chase,
            poison_vuln: 1.0,
        },
        EnemyKind::Araignee => EnemyDef {
            name: "Araignée",
            hp: 26.0,
            speed: 170.0,
            dmg: 8.0,
            radius: 11.0,
            pattes: 3,
            color: Color::srgb(0.25, 0.22, 0.3),
            ai: AiKind::Lunge,
            poison_vuln: 1.0,
        },
        EnemyKind::Moustique => EnemyDef {
            name: "Moustique",
            hp: 10.0,
            speed: 175.0,
            dmg: 4.0,
            radius: 8.0,
            pattes: 2,
            color: Color::srgb(0.6, 0.6, 0.7),
            ai: AiKind::Ranged { min: 150.0, max: 240.0, shoot_cd: 2.0 },
            poison_vuln: 1.0,
        },
        EnemyKind::Guepe => EnemyDef {
            name: "Guêpe",
            hp: 22.0,
            speed: 145.0,
            dmg: 7.0,
            radius: 11.0,
            pattes: 3,
            color: Color::srgb(0.95, 0.8, 0.1),
            ai: AiKind::Ranged { min: 170.0, max: 260.0, shoot_cd: 1.6 },
            poison_vuln: 1.0,
        },
        EnemyKind::Scarabee => EnemyDef {
            name: "Scarabée",
            hp: 48.0,
            speed: 85.0,
            dmg: 9.0,
            radius: 14.0,
            pattes: 3,
            color: Color::srgb(0.3, 0.4, 0.5),
            ai: AiKind::Chase,
            poison_vuln: 1.0,
        },
        EnemyKind::Escargot => EnemyDef {
            name: "Escargot",
            hp: 85.0,
            speed: 38.0,
            dmg: 12.0,
            radius: 17.0,
            pattes: 4,
            color: Color::srgb(0.65, 0.55, 0.4),
            ai: AiKind::Chase,
            poison_vuln: 1.0,
        },
        EnemyKind::Limace => EnemyDef {
            name: "Limace",
            hp: 38.0,
            speed: 48.0,
            dmg: 6.0,
            radius: 13.0,
            pattes: 2,
            color: Color::srgb(0.8, 0.6, 0.2),
            ai: AiKind::Chase,
            poison_vuln: 1.6,
        },
    }
}

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

#[derive(Component)]
pub struct LungeState {
    pub cd: Timer,
    pub active: Timer,
    pub dir: Vec2,
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
    let (hp, dmg, radius, pattes) = if elite {
        (d.hp * hp_scale * 3.0, d.dmg * dmg_scale * 1.5, d.radius * 1.5, d.pattes * 6)
    } else {
        (d.hp * hp_scale, d.dmg * dmg_scale, d.radius, d.pattes)
    };
    let color = if elite {
        d.color.mix(&Color::srgb(0.9, 0.2, 0.6), 0.4)
    } else {
        d.color
    };
    let mut rng = rand::rng();
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
        ContactDmg(dmg),
        ContactCd(Timer::from_seconds(rng.random_range(0.0..0.3), TimerMode::Once)),
        PattesDrop(pattes),
        AiSpeed(d.speed * if elite { 1.15 } else { 1.0 }),
    ));
    match d.ai {
        AiKind::Lunge => {
            e.insert(LungeState {
                cd: Timer::from_seconds(rng.random_range(0.6..1.8), TimerMode::Once),
                active: Timer::from_seconds(0.35, TimerMode::Once),
                dir: Vec2::X,
            });
        }
        AiKind::Ranged { shoot_cd, .. } => {
            e.insert(ShootCd(Timer::from_seconds(
                shoot_cd * rng.random_range(0.5..1.2),
                TimerMode::Once,
            )));
        }
        AiKind::Chase => {}
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
            (ai_movement, enemy_shoot, separation, poison_tint)
                .in_set(GameSet::Ai)
                .run_if(combat_active),
        )
        .add_systems(
            Update,
            (enemy_projectiles, hazard_puddles)
                .in_set(GameSet::Combat)
                .run_if(combat_active),
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

    for (kind, speed, tf, mut vel, lunge, slowed) in &mut enemies {
        let pos = tf.translation.truncate();
        let to_player = player_pos - pos;
        let dist = to_player.length();
        let dir = to_player.normalize_or_zero();
        let mut max = speed.0;
        if slowed {
            max *= 0.45;
        }

        let target = match def(*kind).ai {
            AiKind::Chase => dir * max,
            AiKind::Lunge => {
                if let Some(mut lunge) = lunge {
                    lunge.cd.tick(time.delta());
                    lunge.active.tick(time.delta());
                    if !lunge.active.is_finished() {
                        lunge.dir * max * 2.6
                    } else if lunge.cd.is_finished() && dist < 220.0 {
                        lunge.dir = dir;
                        lunge.active.reset();
                        lunge.cd = Timer::from_seconds(1.8, TimerMode::Once);
                        dir * max * 2.6
                    } else {
                        dir * max
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
        vel.0 = vel.0.move_towards(target, 700.0 * dt);
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
            spawn_enemy_projectile(&mut commands, pos, dir * 250.0, contact.0, def(*kind).color);
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
        Sprite::from_color(color.mix(&Color::WHITE, 0.3), Vec2::splat(7.0)),
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

/// Teinte verte sur les ennemis empoisonnés (lisibilité).
fn poison_tint(
    mut enemies: Query<
        (&mut Sprite, &BaseColor, Has<Poisoned>),
        (With<Enemy>, Without<HitFlash>),
    >,
) {
    for (mut sprite, base, poisoned) in &mut enemies {
        let target = if poisoned {
            base.0.mix(&Color::srgb(0.3, 1.0, 0.2), 0.45)
        } else {
            base.0
        };
        sprite.color = target;
    }
}
