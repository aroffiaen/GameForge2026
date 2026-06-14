//! Le matériel de jardin : 2 slots, visée manuelle, profils Frappe / Maintien /
//! Utilitaire (GDD §4).

use bevy::prelude::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::augments::{Augment, Augments};
use crate::common::*;
use crate::player::{Aim, Dash, Momentum, PlayerStats, SpeedInfo};

// ---------------------------------------------------------------------------
// Définition des armes
// ---------------------------------------------------------------------------

// Refonte v0.3 §18.F — Lot 1 : roster strict (Poings/Petite pelle retirés,
// Arrosoir → Pesticide), **plus aucun knockback** (retrait global, GDD §5).
// Les 6 armes restantes (Tronçonneuse, Pioche, Faux, Hache, Serpe, Pic de vigne)
// arrivent en Lot 2.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum WeaponKind {
    Pesticide,
    Pelle,
    Rateau,
    Karcher,
}

pub const ALL_WEAPONS: &[WeaponKind] = &[
    WeaponKind::Pesticide,
    WeaponKind::Pelle,
    WeaponKind::Rateau,
    WeaponKind::Karcher,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// Frappe : un clic = un coup.
    Strike,
    /// Maintien : gâchette tenue = effet continu (hold-to-shoot, sans coût).
    Hold,
    /// Frappe à effet de contrôle sur cooldown (le râteau aspire).
    Utility,
}

pub struct WeaponDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub profile: Profile,
    pub dmg: f32,
    pub cd: f32,
    /// Portée (px). 0 = AoE centrée sur le joueur (anneau).
    pub range: f32,
    /// Rayon d'effet (px) pour les AoE centrées / la traînée.
    pub radius: f32,
    /// Angle total du cône (degrés) pour les armes coniques. 0 = sans objet.
    pub cone: f32,
    pub color: Color,
    pub size: Vec2,
}

pub fn def(kind: WeaponKind) -> WeaponDef {
    match kind {
        WeaponKind::Pesticide => WeaponDef {
            name: "Pesticide",
            desc: "Maintien : pose une traînée de poison au sol (DoT).",
            profile: Profile::Hold,
            dmg: 14.0, // DPS du poison
            cd: 0.09,  // intervalle de dépôt
            range: 0.0,
            radius: 26.0,
            cone: 0.0,
            color: Color::srgb(0.35, 0.65, 0.9),
            size: Vec2::new(14.0, 10.0),
        },
        WeaponKind::Pelle => WeaponDef {
            name: "Pelle",
            desc: "Frappe : coup de zone en anneau autour de toi (Q de Darius).",
            profile: Profile::Strike,
            dmg: 26.0,
            cd: 0.7,
            range: 0.0,   // centré sur le joueur
            radius: 78.0, // rayon de l'anneau
            cone: 0.0,
            color: Color::srgb(0.6, 0.6, 0.68),
            size: Vec2::new(24.0, 8.0),
        },
        WeaponKind::Rateau => WeaponDef {
            name: "Râteau",
            desc: "Frappe : attire les ennemis devant toi (cône). Pilier de synergie.",
            profile: Profile::Utility,
            dmg: 5.0,
            cd: 1.8,
            range: 230.0, // allonge de l'aspiration
            radius: 0.0,
            cone: 110.0, // cône frontal
            color: Color::srgb(0.7, 0.5, 0.3),
            size: Vec2::new(20.0, 6.0),
        },
        WeaponKind::Karcher => WeaponDef {
            name: "Karcher",
            desc: "Maintien : jet en éventail 60°, dégâts soutenus.",
            profile: Profile::Hold,
            dmg: 34.0, // DPS du jet
            cd: 0.07,  // intervalle de tick
            range: 235.0,
            radius: 16.0,
            cone: 60.0,
            color: Color::srgb(0.95, 0.85, 0.2),
            size: Vec2::new(18.0, 9.0),
        },
    }
}

// ---------------------------------------------------------------------------
// Ressources & composants
// ---------------------------------------------------------------------------

/// Les 2 slots d'armes équipées (clic gauche / clic droit).
#[derive(Resource)]
pub struct Loadout(pub [Option<WeaponKind>; 2]);

impl Default for Loadout {
    fn default() -> Self {
        Self([Some(WeaponKind::Pelle), None])
    }
}

/// Cooldowns restants des deux slots (en secondes).
#[derive(Resource, Default)]
pub struct WeaponCds(pub [f32; 2]);

/// Sprite d'arme attaché au perso (séparé du corps, GDD §4.1).
#[derive(Component)]
pub struct WeaponSprite(pub usize);

/// Animation de coup en cours sur un slot.
#[derive(Resource, Default)]
pub struct SwingAnims(pub [f32; 2]);

/// Flaque de pesticide posée par le joueur (GDD §4.3).
#[derive(Component)]
pub struct PoisonPuddle {
    pub dps: f32,
    pub radius: f32,
    pub life: Timer,
}

/// Raccourci pour écrire des dégâts de type coup.
pub trait DamageMsgExt {
    fn write_hit(&mut self, target: Entity, amount: f32);
}

impl DamageMsgExt for MessageWriter<'_, DamageMsg> {
    fn write_hit(&mut self, target: Entity, amount: f32) {
        self.write(DamageMsg {
            target,
            amount,
            kind: DamageKind::Hit,
        });
    }
}

/// Dégâts du joueur : base × mult. vitesse × augments × momentum × burst.
fn player_damage(
    base: f32,
    speed: &SpeedInfo,
    stats: &PlayerStats,
    momentum: f32,
    burst: bool,
) -> f32 {
    let mut d = base * speed.mult * stats.dmg_mult * (1.0 + momentum);
    if burst {
        d *= 1.5;
    }
    d
}

fn can_attack(dash: &Dash, augments: &Augments) -> bool {
    !dash.dashing() || augments.has(Augment::DashOffensif)
}

fn slot_pressed(buttons: &ButtonInput<MouseButton>, slot: usize, just: bool) -> bool {
    let button = if slot == 0 { MouseButton::Left } else { MouseButton::Right };
    if just {
        buttons.just_pressed(button)
    } else {
        buttons.pressed(button)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Loadout>()
            .init_resource::<WeaponCds>()
            .init_resource::<SwingAnims>()
            .add_systems(
                Update,
                (
                    tick_cooldowns,
                    strike_system,
                    rake_system,
                    pesticide_system,
                    karcher_system,
                    dash_trail_system,
                )
                    .in_set(GameSet::Combat)
                    .run_if(combat_active),
            )
            .add_systems(
                Update,
                (puddle_system,)
                    .in_set(GameSet::Combat)
                    .run_if(combat_active),
            )
            .add_systems(
                Update,
                sync_weapon_sprites.in_set(GameSet::Post).run_if(player_active),
            );
    }
}

// ---------------------------------------------------------------------------
// Systèmes
// ---------------------------------------------------------------------------

fn tick_cooldowns(time: Res<Time>, mut cds: ResMut<WeaponCds>, mut swings: ResMut<SwingAnims>) {
    for cd in cds.0.iter_mut() {
        *cd = (*cd - time.delta_secs()).max(0.0);
    }
    for s in swings.0.iter_mut() {
        *s = (*s - time.delta_secs()).max(0.0);
    }
}

/// Armes de frappe : un clic = un coup en arc devant le joueur.
fn strike_system(
    buttons: Res<ButtonInput<MouseButton>>,
    loadout: Res<Loadout>,
    mut cds: ResMut<WeaponCds>,
    mut swings: ResMut<SwingAnims>,
    speed: Res<SpeedInfo>,
    stats: Res<PlayerStats>,
    augments: Res<Augments>,
    aim: Res<Aim>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    player: Query<(&Transform, &Dash, &Momentum), With<Player>>,
    enemies: Query<(Entity, &Transform, &Radius), With<Enemy>>,
) {
    let Ok((player_tf, dash, momentum)) = player.single() else {
        return;
    };
    if !can_attack(dash, &augments) {
        return;
    }
    let player_pos = player_tf.translation.truncate();
    for slot in 0..2 {
        let Some(kind) = loadout.0[slot] else { continue };
        let weapon = def(kind);
        if weapon.profile != Profile::Strike {
            continue;
        }
        if !slot_pressed(&buttons, slot, true) || cds.0[slot] > 0.0 {
            continue;
        }
        cds.0[slot] = weapon.cd * stats.attack_cd_mult;
        swings.0[slot] = 0.14;

        let burst = augments.has(Augment::DashOffensif) && !dash.burst.is_finished();
        let amount = player_damage(weapon.dmg, &speed, &stats, momentum.0, burst);
        // `range == 0` → AoE centrée sur le joueur (anneau de la Pelle) ;
        // sinon coup en avant. (Plus aucun knockback, GDD §5.)
        let center = player_pos + aim.dir * weapon.range;
        let radius = weapon.radius * stats.aoe_mult;

        for (e, etf, er) in &enemies {
            if etf.translation.truncate().distance(center) <= radius + er.0 {
                dmg.write_hit(e, amount);
            }
        }

        // Visuel : anneau qui s'étend (AoE centrée) ou arc devant soi.
        if weapon.range == 0.0 {
            for i in 0..20 {
                let dir = Vec2::from_angle(i as f32 / 20.0 * std::f32::consts::TAU);
                commands.spawn((
                    Sprite::from_color(weapon.color.with_alpha(0.7), Vec2::splat(6.0)),
                    Transform::from_translation((player_pos + dir * 12.0).extend(9.0)),
                    Velocity(dir * radius * 6.0),
                    Lifetime::secs(0.16),
                ));
            }
        } else {
            let angle = aim.dir.y.atan2(aim.dir.x);
            commands.spawn((
                Sprite::from_color(weapon.color.with_alpha(0.5), Vec2::new(radius * 1.6, 10.0)),
                Transform::from_translation(center.extend(9.0))
                    .with_rotation(Quat::from_rotation_z(angle + std::f32::consts::FRAC_PI_2)),
                Lifetime::secs(0.12),
            ));
        }
    }
}

/// Râteau : attire tous les ennemis proches vers le joueur (GDD §4.3).
fn rake_system(
    buttons: Res<ButtonInput<MouseButton>>,
    loadout: Res<Loadout>,
    mut cds: ResMut<WeaponCds>,
    mut swings: ResMut<SwingAnims>,
    speed: Res<SpeedInfo>,
    stats: Res<PlayerStats>,
    augments: Res<Augments>,
    aim: Res<Aim>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    player: Query<(&Transform, &Dash, &Momentum), With<Player>>,
    mut enemies: Query<(Entity, &Transform, &mut Knockback), With<Enemy>>,
) {
    let Ok((player_tf, dash, momentum)) = player.single() else {
        return;
    };
    if !can_attack(dash, &augments) {
        return;
    }
    let player_pos = player_tf.translation.truncate();
    for slot in 0..2 {
        let Some(kind) = loadout.0[slot] else { continue };
        if kind != WeaponKind::Rateau {
            continue;
        }
        let weapon = def(kind);
        if !slot_pressed(&buttons, slot, true) || cds.0[slot] > 0.0 {
            continue;
        }
        cds.0[slot] = weapon.cd * stats.attack_cd_mult;
        swings.0[slot] = 0.2;

        let reach = weapon.range * stats.rake_mult;
        // Cône frontal : on n'aspire que ce qui est DEVANT (GDD §5).
        let cos_half = (weapon.cone.to_radians() * 0.5).cos();
        let burst = augments.has(Augment::DashOffensif) && !dash.burst.is_finished();
        let amount = player_damage(weapon.dmg, &speed, &stats, momentum.0, burst);

        for (e, etf, mut kb) in &mut enemies {
            let epos = etf.translation.truncate();
            let to_enemy = epos - player_pos;
            let dist = to_enemy.length();
            if dist <= reach && to_enemy.normalize_or_zero().dot(aim.dir) >= cos_half {
                // Aspiration vers le joueur (un PULL, pas un knockback).
                let pull = (player_pos - epos).normalize_or_zero();
                kb.0 += pull * (260.0 + dist * 1.2);
                dmg.write_hit(e, amount);
                if augments.has(Augment::RateauAimante) {
                    commands
                        .entity(e)
                        .insert(Slowed(Timer::from_seconds(1.5, TimerMode::Once)));
                }
            }
        }

        // Visuel : des dents qui balaient le cône frontal vers le joueur.
        let base = aim.dir.to_angle();
        for i in 0..11 {
            let spread = (i as f32 / 10.0 - 0.5) * weapon.cone.to_radians();
            let dir = Vec2::from_angle(base + spread);
            commands.spawn((
                Sprite::from_color(weapon.color.with_alpha(0.8), Vec2::splat(5.0)),
                Transform::from_translation((player_pos + dir * reach).extend(8.0)),
                Velocity(-dir * reach * 2.4),
                Lifetime::secs(0.3),
            ));
        }
    }
}

/// Pesticide : Maintien → traînée de poison le long du trajet (« Q de Singed »,
/// GDD §5). Hold-to-shoot : aucun coût, l'intervalle de dépôt fait office de cd.
fn pesticide_system(
    buttons: Res<ButtonInput<MouseButton>>,
    loadout: Res<Loadout>,
    mut cds: ResMut<WeaponCds>,
    stats: Res<PlayerStats>,
    augments: Res<Augments>,
    mut commands: Commands,
    player: Query<(&Transform, &Dash), With<Player>>,
) {
    let Ok((player_tf, dash)) = player.single() else {
        return;
    };
    if !can_attack(dash, &augments) {
        return;
    }
    for slot in 0..2 {
        let Some(kind) = loadout.0[slot] else { continue };
        if kind != WeaponKind::Pesticide {
            continue;
        }
        let weapon = def(kind);
        if !slot_pressed(&buttons, slot, false) || cds.0[slot] > 0.0 {
            continue;
        }
        cds.0[slot] = weapon.cd * stats.attack_cd_mult;
        spawn_puddle(
            &mut commands,
            player_tf.translation.truncate(),
            weapon.dmg * stats.poison_mult,
            weapon.radius,
        );
    }
}

pub fn spawn_puddle(commands: &mut Commands, pos: Vec2, dps: f32, radius: f32) {
    let mut rng = rand::rng();
    let jitter = Vec2::new(rng.random_range(-6.0..6.0), rng.random_range(-6.0..6.0));
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.55, 0.85, 0.25).with_alpha(0.4),
            Vec2::splat(radius * 1.8),
        ),
        Transform::from_translation((pos + jitter).extend(-3.0)).with_rotation(
            Quat::from_rotation_z(rng.random_range(0.0..std::f32::consts::TAU)),
        ),
        PoisonPuddle {
            dps,
            radius,
            life: Timer::from_seconds(3.2, TimerMode::Once),
        },
    ));
}

/// Keystone « Traînée toxique » : le dash laisse du pesticide (GDD §5.2).
fn dash_trail_system(
    augments: Res<Augments>,
    stats: Res<PlayerStats>,
    mut commands: Commands,
    mut last: Local<Vec2>,
    player: Query<(&Transform, &Dash), With<Player>>,
) {
    if !augments.has(Augment::TraineeToxique) {
        return;
    }
    let Ok((tf, dash)) = player.single() else {
        return;
    };
    if !dash.dashing() {
        return;
    }
    let pos = tf.translation.truncate();
    if pos.distance(*last) > 24.0 {
        *last = pos;
        spawn_puddle(&mut commands, pos, 11.0 * stats.poison_mult, 24.0);
    }
}

/// Karcher : Maintien → jet en éventail 60°, dégâts soutenus (plus de recul,
/// GDD §5). Hold-to-shoot : l'intervalle de tick fait office de cd.
fn karcher_system(
    buttons: Res<ButtonInput<MouseButton>>,
    loadout: Res<Loadout>,
    mut cds: ResMut<WeaponCds>,
    speed: Res<SpeedInfo>,
    stats: Res<PlayerStats>,
    augments: Res<Augments>,
    aim: Res<Aim>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    player: Query<(&Transform, &Dash, &Momentum), With<Player>>,
    enemies: Query<(Entity, &Transform, &Radius), With<Enemy>>,
) {
    let Ok((player_tf, dash, momentum)) = player.single() else {
        return;
    };
    if !can_attack(dash, &augments) {
        return;
    }
    let player_pos = player_tf.translation.truncate();
    let mut rng = rand::rng();
    for slot in 0..2 {
        let Some(kind) = loadout.0[slot] else { continue };
        if kind != WeaponKind::Karcher {
            continue;
        }
        let weapon = def(kind);
        if !slot_pressed(&buttons, slot, false) || cds.0[slot] > 0.0 {
            continue;
        }
        cds.0[slot] = weapon.cd * stats.attack_cd_mult;

        let burst = augments.has(Augment::DashOffensif) && !dash.burst.is_finished();
        let tick_dmg =
            player_damage(weapon.dmg, &speed, &stats, momentum.0, burst) * weapon.cd;
        let extra = if augments.has(Augment::BuseHautePression) { 1.25 } else { 1.0 };
        let half = weapon.cone.to_radians() * 0.5;
        let cos_half = half.cos();

        for (e, etf, er) in &enemies {
            let to_enemy = etf.translation.truncate() - player_pos;
            let dist = to_enemy.length();
            // Éventail : dans la portée ET dans l'angle du cône.
            if dist <= weapon.range + er.0 && to_enemy.normalize_or_zero().dot(aim.dir) >= cos_half {
                dmg.write_hit(e, tick_dmg * extra);
            }
        }

        // Gouttelettes du jet, réparties sur tout l'éventail.
        for _ in 0..4 {
            let spread = rng.random_range(-half..half);
            let dir = Vec2::from_angle(aim.dir.to_angle() + spread);
            commands.spawn((
                Sprite::from_color(Color::srgb(0.5, 0.8, 1.0).with_alpha(0.8), Vec2::splat(5.0)),
                Transform::from_translation((player_pos + dir * 20.0).extend(8.0)),
                Velocity(dir * rng.random_range(500.0..650.0)),
                Lifetime::secs(weapon.range / 580.0),
            ));
        }
    }
}

/// Vie des flaques + application du poison (le timer se rafraîchit au contact).
fn puddle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut puddles: Query<(Entity, &Transform, &mut PoisonPuddle, &mut Sprite)>,
    mut enemies: Query<(Entity, &Transform, &Radius, Option<&mut Poisoned>), With<Enemy>>,
) {
    for (puddle_e, puddle_tf, mut puddle, mut sprite) in &mut puddles {
        puddle.life.tick(time.delta());
        let left = 1.0 - puddle.life.fraction();
        if left < 0.3 {
            sprite.color = sprite.color.with_alpha(0.4 * (left / 0.3));
        }
        if puddle.life.is_finished() {
            commands.entity(puddle_e).despawn();
            continue;
        }
        let puddle_pos = puddle_tf.translation.truncate();
        for (e, etf, er, poisoned) in &mut enemies {
            if etf.translation.truncate().distance(puddle_pos) > puddle.radius + er.0 {
                continue;
            }
            match poisoned {
                Some(mut p) => {
                    // « Rester dans le poison reset son timer » (GDD §4.3).
                    p.timer.reset();
                    p.dps = p.dps.max(puddle.dps);
                }
                None => {
                    commands.entity(e).insert(Poisoned {
                        timer: Timer::from_seconds(2.5, TimerMode::Once),
                        tick: Timer::from_seconds(0.4, TimerMode::Repeating),
                        dps: puddle.dps,
                    });
                }
            }
        }
    }
}

/// Synchronise les sprites d'armes (séparés du corps) : équipement, visée,
/// animation de coup.
fn sync_weapon_sprites(
    loadout: Res<Loadout>,
    aim: Res<Aim>,
    swings: Res<SwingAnims>,
    sprites: Res<GameSprites>,
    mut q: Query<(&WeaponSprite, &mut Sprite, &mut Transform)>,
) {
    for (slot_marker, mut sprite, mut tf) in &mut q {
        let slot = slot_marker.0;
        match loadout.0[slot] {
            None => {
                sprite.color = Color::NONE;
            }
            Some(WeaponKind::Pelle) => {
                // La pelle a son propre sprite (arme à deux mains), centrée
                // devant le perso et orientée vers la visée.
                sprite.image = sprites.pelle.clone();
                sprite.color = Color::WHITE;
                sprite.custom_size = Some(Vec2::splat(46.0));
                let swing = swings.0[slot];
                let offset = aim.dir * (10.0 + swing * 90.0);
                tf.translation = offset.extend(2.0);
                tf.rotation = Quat::from_rotation_z(aim.dir.to_angle());
            }
            Some(kind) => {
                let weapon = def(kind);
                // Forme colorée (placeholder en attendant sprite-arme-R/L).
                sprite.image = Handle::default();
                sprite.color = weapon.color;
                sprite.custom_size = Some(weapon.size);
                let side = if slot == 0 { 1.0 } else { -1.0 };
                let perp = Vec2::new(-aim.dir.y, aim.dir.x) * 7.0 * side;
                let swing = swings.0[slot];
                let reach = 14.0 + swing * 120.0;
                let offset = aim.dir * reach + perp;
                tf.translation = offset.extend(2.0);
                tf.rotation = Quat::from_rotation_z(aim.dir.to_angle());
            }
        }
    }
}
