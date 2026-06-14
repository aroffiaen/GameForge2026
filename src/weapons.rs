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

// Refonte v0.3 §18.F : roster strict des **10 armes** (GDD §5), **plus aucun
// knockback** (retrait global). Profils : Frappe (Strike), Maintien (Hold,
// hold-to-shoot), et Utility (Frappe à système dédié : Râteau aspire, Hache
// se lance).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum WeaponKind {
    Pesticide,
    Pelle,
    Rateau,
    Karcher,
    Tronconneuse,
    Pioche,
    Faux,
    Hache,
    Serpe,
    PicDeVigne,
}

pub const ALL_WEAPONS: &[WeaponKind] = &[
    WeaponKind::Pesticide,
    WeaponKind::Pelle,
    WeaponKind::Rateau,
    WeaponKind::Karcher,
    WeaponKind::Tronconneuse,
    WeaponKind::Pioche,
    WeaponKind::Faux,
    WeaponKind::Hache,
    WeaponKind::Serpe,
    WeaponKind::PicDeVigne,
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
    /// Rayon d'effet externe (px) pour les AoE centrées / la traînée.
    pub radius: f32,
    /// Rayon interne (px) : trou central des AoE en anneau (0 = disque plein).
    pub inner: f32,
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
            dmg: 10.0, // DPS du poison (nerf : était 15)
            cd: 0.09,  // intervalle de dépôt
            range: 0.0,
            radius: 26.0,
            inner: 0.0,
            cone: 0.0,
            color: Color::srgb(0.35, 0.65, 0.9),
            size: Vec2::new(14.0, 10.0),
        },
        WeaponKind::Pelle => WeaponDef {
            name: "Pelle",
            desc: "Frappe : coup de zone en anneau autour de toi (Q de Darius).",
            profile: Profile::Strike,
            dmg: 18.0, // ~15 DPS (18/1.2)
            cd: 1.2,
            range: 0.0,    // centré sur le joueur
            radius: 102.0, // rayon externe de l'anneau (mid-range)
            inner: 44.0,   // trou central : ne touche pas au corps-à-corps
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
            inner: 0.0,
            cone: 90.0, // cône frontal
            color: Color::srgb(0.7, 0.5, 0.3),
            size: Vec2::new(20.0, 6.0),
        },
        WeaponKind::Karcher => WeaponDef {
            name: "Karcher",
            desc: "Maintien : jet en éventail 60°, dégâts soutenus.",
            profile: Profile::Hold,
            dmg: 15.0, // DPS du jet (cible ~15)
            cd: 0.07,  // intervalle de tick
            range: 235.0,
            radius: 16.0,
            inner: 0.0,
            cone: 60.0,
            color: Color::srgb(0.95, 0.85, 0.2),
            size: Vec2::new(18.0, 9.0),
        },
        WeaponKind::Tronconneuse => WeaponDef {
            name: "Tronçonneuse",
            desc: "Maintien : ligne soutenue. Te ralentit et bloque l'arme 2.",
            profile: Profile::Hold,
            dmg: 22.0, // DPS de la lame (relevé : la contrainte mérite un vrai punch)
            cd: 0.10,  // intervalle de tick (moins de ticks/s, mais plus gros chacun)
            range: 185.0, // longueur de la ligne (allongée)
            radius: 22.0, // demi-largeur
            inner: 0.0,
            cone: 0.0,
            color: Color::srgb(0.85, 0.2, 0.15),
            size: Vec2::new(22.0, 10.0),
        },
        WeaponKind::Pioche => WeaponDef {
            name: "Pioche",
            desc: "Frappe : impact de zone à distance moyenne.",
            profile: Profile::Strike,
            dmg: 18.0, // ~15 DPS (18/1.2)
            cd: 1.2,
            range: 130.0, // distance de l'impact
            radius: 46.0, // rayon de l'impact
            inner: 0.0,
            cone: 0.0,
            color: Color::srgb(0.5, 0.55, 0.6),
            size: Vec2::new(20.0, 10.0),
        },
        WeaponKind::Faux => WeaponDef {
            name: "Faux",
            desc: "Frappe : grand balayage en cône 50°, longue portée.",
            profile: Profile::Strike,
            dmg: 14.0, // ~15 DPS (14/0.9)
            cd: 0.9,
            range: 175.0, // allonge du cône
            radius: 0.0,
            inner: 0.0,
            cone: 50.0,
            color: Color::srgb(0.7, 0.75, 0.78),
            size: Vec2::new(26.0, 8.0),
        },
        WeaponKind::Hache => WeaponDef {
            name: "Hache",
            desc: "Frappe : hache lancée jusqu'au mur. Gros dégâts, long CD.",
            profile: Profile::Utility,
            dmg: 48.0, // ~30 DPS burst (48/1.6), high risk/high reward
            cd: 1.6,
            range: 720.0, // vitesse du projectile
            radius: 16.0, // rayon de touche du projectile
            inner: 0.0,
            cone: 0.0,
            color: Color::srgb(0.6, 0.45, 0.3),
            size: Vec2::new(22.0, 12.0),
        },
        WeaponKind::Serpe => WeaponDef {
            name: "Serpe",
            desc: "Frappe : balaie presque tout autour (300°), rapide.",
            profile: Profile::Strike,
            dmg: 7.0, // ~15 DPS (7/0.5), frappe légère mais très rapide
            cd: 0.5,
            range: 78.0, // allonge
            radius: 0.0,
            inner: 0.0,
            cone: 300.0,
            color: Color::srgb(0.55, 0.7, 0.4),
            size: Vec2::new(18.0, 8.0),
        },
        WeaponKind::PicDeVigne => WeaponDef {
            name: "Pic de vigne",
            desc: "Frappe : estoc qui s'allonge, du corps-à-corps à longue portée.",
            profile: Profile::Strike,
            dmg: 13.0, // ~15 DPS (13/0.85)
            cd: 0.85,
            range: 205.0, // allonge de l'estoc
            radius: 0.0,
            inner: 0.0,
            cone: 16.0, // fin (comme une lance)
            color: Color::srgb(0.4, 0.6, 0.35),
            size: Vec2::new(28.0, 5.0),
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

/// Vrai tant qu'une tronçonneuse est tenue ce frame : ralentit le joueur
/// (lu par `movement`) et bloque l'autre slot.
#[derive(Resource, Default)]
pub struct ChainsawActive(pub bool);

/// Hache lancée : traverse jusqu'au mur, touche chaque ennemi une fois.
#[derive(Component)]
pub struct ThrownAxe {
    pub dmg: f32,
    pub radius: f32,
    pub hit: Vec<Entity>,
}

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

/// `slot` est-il bloqué parce que l'AUTRE slot tient une tronçonneuse active ?
/// (GDD §5 : « tant qu'active, arme 2 inutilisable ».)
fn slot_blocked_by_chainsaw(
    loadout: &Loadout,
    buttons: &ButtonInput<MouseButton>,
    slot: usize,
) -> bool {
    let other = 1 - slot;
    loadout.0[other] == Some(WeaponKind::Tronconneuse) && slot_pressed(buttons, other, false)
}

/// FX de cône **net** (Faux, Serpe, Pic de vigne) : l'arc à la portée + les deux
/// bords, pour qu'on lise clairement la zone touchée.
fn spawn_cone_fx(
    commands: &mut Commands,
    origin: Vec2,
    dir: Vec2,
    reach: f32,
    cone_deg: f32,
    color: Color,
) {
    let base = dir.to_angle();
    let half = cone_deg.to_radians() * 0.5;
    // Arc à la portée.
    let n = (cone_deg / 12.0).clamp(4.0, 30.0) as u32;
    for i in 0..=n {
        let a = base - half + (i as f32 / n as f32) * (2.0 * half);
        let d = Vec2::from_angle(a);
        commands.spawn((
            Sprite::from_color(color.with_alpha(0.85), Vec2::splat(6.0)),
            Transform::from_translation((origin + d * reach).extend(9.0)),
            Lifetime::secs(0.2),
        ));
    }
    // Les deux bords du cône.
    for edge in [-half, half] {
        let d = Vec2::from_angle(base + edge);
        for k in 1..=7 {
            let r = reach * k as f32 / 7.0;
            commands.spawn((
                Sprite::from_color(color.with_alpha(0.7), Vec2::splat(5.0)),
                Transform::from_translation((origin + d * r).extend(9.0)),
                Lifetime::secs(0.18),
            ));
        }
    }
}

/// FX d'anneau/disque **net** (Pelle, Pioche) : contour externe (+ contour
/// interne si trou central) + un remplissage qui balaie la bande.
fn spawn_ring_fx(commands: &mut Commands, center: Vec2, inner: f32, outer: f32, color: Color) {
    let n_out = 34;
    for i in 0..n_out {
        let d = Vec2::from_angle(i as f32 / n_out as f32 * std::f32::consts::TAU);
        commands.spawn((
            Sprite::from_color(color.with_alpha(0.9), Vec2::splat(7.0)),
            Transform::from_translation((center + d * outer).extend(9.0)),
            Lifetime::secs(0.22),
        ));
    }
    if inner > 1.0 {
        let n_in = 24;
        for i in 0..n_in {
            let d = Vec2::from_angle(i as f32 / n_in as f32 * std::f32::consts::TAU);
            commands.spawn((
                Sprite::from_color(color.with_alpha(0.55), Vec2::splat(5.0)),
                Transform::from_translation((center + d * inner).extend(9.0)),
                Lifetime::secs(0.2),
            ));
        }
    }
    let start = inner.max(8.0);
    for i in 0..18 {
        let d = Vec2::from_angle(i as f32 / 18.0 * std::f32::consts::TAU);
        commands.spawn((
            Sprite::from_color(color.with_alpha(0.6), Vec2::splat(5.0)),
            Transform::from_translation((center + d * start).extend(9.0)),
            Velocity(d * (outer - start + 10.0) * 4.0),
            Lifetime::secs(0.16),
        ));
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
            .init_resource::<ChainsawActive>()
            .add_systems(
                Update,
                (
                    tick_cooldowns,
                    strike_system,
                    rake_system,
                    pesticide_system,
                    karcher_system,
                    chainsaw_system,
                    axe_system,
                    thrown_axe_system,
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
            )
            // Évite que le ralentissement « tronçonneuse » reste collé si on
            // quitte le combat en la tenant (mort, fin de run…).
            .add_systems(OnExit(AppState::EnRun), reset_chainsaw)
            .add_systems(OnExit(AppState::Terrasse), reset_chainsaw);
    }
}

fn reset_chainsaw(mut chainsaw: ResMut<ChainsawActive>) {
    chainsaw.0 = false;
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
        if slot_blocked_by_chainsaw(&loadout, &buttons, slot) {
            continue;
        }
        if !slot_pressed(&buttons, slot, true) || cds.0[slot] > 0.0 {
            continue;
        }
        cds.0[slot] = weapon.cd * stats.attack_cd_mult;
        swings.0[slot] = 0.14;

        let burst = augments.has(Augment::DashOffensif) && !dash.burst.is_finished();
        let amount = player_damage(weapon.dmg, &speed, &stats, momentum.0, burst);
        // (Plus aucun knockback, GDD §5.)
        if weapon.cone > 0.0 {
            // Cône depuis le joueur : balayage (Faux, Serpe), estoc (Pic de vigne).
            let reach = weapon.range;
            let cos_half = (weapon.cone.to_radians() * 0.5).cos();
            for (e, etf, er) in &enemies {
                let to = etf.translation.truncate() - player_pos;
                if to.length() <= reach + er.0 && to.normalize_or_zero().dot(aim.dir) >= cos_half {
                    dmg.write_hit(e, amount);
                }
            }
            spawn_cone_fx(&mut commands, player_pos, aim.dir, reach, weapon.cone, weapon.color);
        } else {
            // AoE en bande [inner, externe] : centrée sur le joueur (anneau de
            // la Pelle, range 0) ou au point visé (impact de la Pioche, range>0).
            // `inner > 0` creuse un trou central → ne touche pas au contact.
            let center = player_pos + aim.dir * weapon.range;
            let outer = weapon.radius * stats.aoe_mult;
            let inner = weapon.inner;
            for (e, etf, er) in &enemies {
                let dist = etf.translation.truncate().distance(center);
                if dist <= outer + er.0 && dist + er.0 >= inner {
                    dmg.write_hit(e, amount);
                }
            }
            spawn_ring_fx(&mut commands, center, inner, outer, weapon.color);
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
        if slot_blocked_by_chainsaw(&loadout, &buttons, slot) {
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
        if slot_blocked_by_chainsaw(&loadout, &buttons, slot) {
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
        if slot_blocked_by_chainsaw(&loadout, &buttons, slot) {
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

        // Gouttelettes du jet : flashs répartis DANS le cône, sans vélocité —
        // ils ne « traversent » plus l'écran (ne sont pas des projectiles).
        for _ in 0..6 {
            let spread = rng.random_range(-half..half);
            let dir = Vec2::from_angle(aim.dir.to_angle() + spread);
            let dist = rng.random_range(20.0..weapon.range);
            commands.spawn((
                Sprite::from_color(Color::srgb(0.5, 0.8, 1.0).with_alpha(0.8), Vec2::splat(5.0)),
                Transform::from_translation((player_pos + dir * dist).extend(8.0)),
                Lifetime::secs(0.12),
            ));
        }
    }
}

/// Tronçonneuse : Maintien → ligne soutenue devant soi (dégâts en ticks).
/// Tant qu'elle tourne : ralentit le joueur (`ChainsawActive`, lu par `movement`)
/// et bloque l'autre slot (`slot_blocked_by_chainsaw`). GDD §5.
fn chainsaw_system(
    buttons: Res<ButtonInput<MouseButton>>,
    loadout: Res<Loadout>,
    mut cds: ResMut<WeaponCds>,
    speed: Res<SpeedInfo>,
    stats: Res<PlayerStats>,
    augments: Res<Augments>,
    aim: Res<Aim>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    mut chainsaw: ResMut<ChainsawActive>,
    player: Query<(&Transform, &Dash, &Momentum), With<Player>>,
    enemies: Query<(Entity, &Transform, &Radius), With<Enemy>>,
) {
    chainsaw.0 = false;
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
        if kind != WeaponKind::Tronconneuse {
            continue;
        }
        let weapon = def(kind);
        if !slot_pressed(&buttons, slot, false) {
            continue;
        }
        // Elle tourne : ralentit le joueur, même quand le tick est en cooldown.
        chainsaw.0 = true;
        if cds.0[slot] > 0.0 {
            continue;
        }
        cds.0[slot] = weapon.cd * stats.attack_cd_mult;

        let burst = augments.has(Augment::DashOffensif) && !dash.burst.is_finished();
        let tick_dmg = player_damage(weapon.dmg, &speed, &stats, momentum.0, burst) * weapon.cd;
        for (e, etf, er) in &enemies {
            let to = etf.translation.truncate() - player_pos;
            let along = to.dot(aim.dir);
            if along < 0.0 || along > weapon.range {
                continue;
            }
            let perp = (to - aim.dir * along).length();
            if perp <= weapon.radius + er.0 {
                dmg.write_hit(e, tick_dmg);
            }
        }
        // Étincelles le long de la lame.
        for _ in 0..2 {
            let t = rng.random_range(0.3..1.0);
            let jitter = Vec2::new(rng.random_range(-6.0..6.0), rng.random_range(-6.0..6.0));
            commands.spawn((
                Sprite::from_color(weapon.color.with_alpha(0.85), Vec2::splat(5.0)),
                Transform::from_translation(
                    (player_pos + aim.dir * weapon.range * t + jitter).extend(9.0),
                ),
                Lifetime::secs(0.1),
            ));
        }
    }
}

/// Hache : Frappe → lance un projectile qui file jusqu'au mur (GDD §5).
fn axe_system(
    buttons: Res<ButtonInput<MouseButton>>,
    loadout: Res<Loadout>,
    mut cds: ResMut<WeaponCds>,
    speed: Res<SpeedInfo>,
    stats: Res<PlayerStats>,
    augments: Res<Augments>,
    aim: Res<Aim>,
    mut commands: Commands,
    player: Query<(&Transform, &Dash, &Momentum), With<Player>>,
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
        if kind != WeaponKind::Hache {
            continue;
        }
        if slot_blocked_by_chainsaw(&loadout, &buttons, slot) {
            continue;
        }
        let weapon = def(kind);
        if !slot_pressed(&buttons, slot, true) || cds.0[slot] > 0.0 {
            continue;
        }
        cds.0[slot] = weapon.cd * stats.attack_cd_mult;

        let burst = augments.has(Augment::DashOffensif) && !dash.burst.is_finished();
        let amount = player_damage(weapon.dmg, &speed, &stats, momentum.0, burst);
        commands.spawn((
            ThrownAxe { dmg: amount, radius: weapon.radius, hit: Vec::new() },
            Sprite::from_color(weapon.color, weapon.size),
            Transform::from_translation((player_pos + aim.dir * 18.0).extend(9.0))
                .with_rotation(Quat::from_rotation_z(aim.dir.to_angle())),
            Velocity(aim.dir * weapon.range), // `range` = vitesse du projectile
            Lifetime::secs(2.5),
        ));
    }
}

/// Déplace les haches lancées (via `Velocity`), inflige les dégâts (un hit par
/// ennemi) et les despawn au mur.
fn thrown_axe_system(
    mut commands: Commands,
    arena: Res<Arena>,
    mut dmg: MessageWriter<DamageMsg>,
    mut axes: Query<(Entity, &Transform, &mut ThrownAxe)>,
    enemies: Query<(Entity, &Transform, &Radius), With<Enemy>>,
) {
    for (ae, atf, mut axe) in &mut axes {
        let pos = atf.translation.truncate();
        if pos.x.abs() > arena.half.x || pos.y.abs() > arena.half.y {
            commands.entity(ae).despawn();
            continue;
        }
        for (e, etf, er) in &enemies {
            if !axe.hit.contains(&e)
                && etf.translation.truncate().distance(pos) <= er.0 + axe.radius
            {
                dmg.write_hit(e, axe.dmg);
                axe.hit.push(e);
            }
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
