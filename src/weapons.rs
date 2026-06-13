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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum WeaponKind {
    Poings,
    PetitePelle,
    Pelle,
    Rateau,
    Arrosoir,
    Karcher,
}

pub const ALL_WEAPONS: &[WeaponKind] = &[
    WeaponKind::Poings,
    WeaponKind::PetitePelle,
    WeaponKind::Pelle,
    WeaponKind::Rateau,
    WeaponKind::Arrosoir,
    WeaponKind::Karcher,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// Un clic = un coup visé.
    Strike,
    /// Gâchette tenue = effet continu.
    Hold,
    /// Effet de contrôle sur cooldown.
    Utility,
}

pub struct WeaponDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub profile: Profile,
    pub dmg: f32,
    pub cd: f32,
    pub range: f32,
    pub radius: f32,
    pub kb: f32,
    pub color: Color,
    pub size: Vec2,
}

pub fn def(kind: WeaponKind) -> WeaponDef {
    match kind {
        WeaponKind::Poings => WeaponDef {
            name: "Poings",
            desc: "L'arme de base. Rapide, courte portée.",
            profile: Profile::Strike,
            dmg: 6.0,
            cd: 0.25,
            range: 28.0,
            radius: 24.0,
            kb: 70.0,
            color: Color::srgb(0.9, 0.75, 0.6),
            size: Vec2::new(8.0, 8.0),
        },
        WeaponKind::PetitePelle => WeaponDef {
            name: "Petite pelle",
            desc: "Mêlée rapide, DPS régulier.",
            profile: Profile::Strike,
            dmg: 9.0,
            cd: 0.28,
            range: 40.0,
            radius: 28.0,
            kb: 100.0,
            color: Color::srgb(0.75, 0.75, 0.8),
            size: Vec2::new(16.0, 6.0),
        },
        WeaponKind::Pelle => WeaponDef {
            name: "Pelle",
            desc: "Lourde et lente : gros coup, gros recul.",
            profile: Profile::Strike,
            dmg: 24.0,
            cd: 0.75,
            range: 48.0,
            radius: 38.0,
            kb: 300.0,
            color: Color::srgb(0.6, 0.6, 0.68),
            size: Vec2::new(24.0, 8.0),
        },
        WeaponKind::Rateau => WeaponDef {
            name: "Râteau",
            desc: "Attire les ennemis vers toi. Pilier de synergie.",
            profile: Profile::Utility,
            dmg: 4.0,
            cd: 2.2,
            range: 0.0,
            radius: 230.0,
            kb: 0.0,
            color: Color::srgb(0.7, 0.5, 0.3),
            size: Vec2::new(20.0, 6.0),
        },
        WeaponKind::Arrosoir => WeaponDef {
            name: "Arrosoir",
            desc: "Pose une traînée de pesticide qui empoisonne.",
            profile: Profile::Hold,
            dmg: 14.0, // DPS du poison
            cd: 0.09,  // intervalle de dépôt
            range: 0.0,
            radius: 26.0,
            kb: 0.0,
            color: Color::srgb(0.35, 0.65, 0.9),
            size: Vec2::new(14.0, 10.0),
        },
        WeaponKind::Karcher => WeaponDef {
            name: "Karcher",
            desc: "Jet haute pression : dégâts soutenus + recul continu.",
            profile: Profile::Hold,
            dmg: 32.0, // DPS du jet
            cd: 0.07,  // intervalle de tick
            range: 235.0,
            radius: 16.0, // demi-largeur du jet
            kb: 60.0,
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
        Self([Some(WeaponKind::Poings), None])
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
                    arrosoir_system,
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
    mut enemies: Query<(Entity, &Transform, &Radius, &mut Knockback), With<Enemy>>,
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
        cds.0[slot] = weapon.cd;
        swings.0[slot] = 0.14;

        let burst = augments.has(Augment::DashOffensif) && !dash.burst.is_finished();
        let amount = player_damage(weapon.dmg, &speed, &stats, momentum.0, burst);
        let center = player_pos + aim.dir * weapon.range;
        let radius = weapon.radius * stats.aoe_mult;

        for (e, etf, er, mut kb) in &mut enemies {
            let epos = etf.translation.truncate();
            if epos.distance(center) <= radius + er.0 {
                dmg.write_hit(e, amount);
                let push = (epos - player_pos).normalize_or(aim.dir);
                kb.0 += push * weapon.kb * stats.kb_mult;
            }
        }

        // Visuel du coup : un arc qui s'efface aussitôt.
        let angle = aim.dir.y.atan2(aim.dir.x);
        commands.spawn((
            Sprite::from_color(weapon.color.with_alpha(0.5), Vec2::new(radius * 1.6, 10.0)),
            Transform::from_translation(center.extend(9.0))
                .with_rotation(Quat::from_rotation_z(angle + std::f32::consts::FRAC_PI_2)),
            Lifetime::secs(0.12),
        ));
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
        cds.0[slot] = weapon.cd;
        swings.0[slot] = 0.2;

        let radius = weapon.radius * stats.rake_mult;
        let burst = augments.has(Augment::DashOffensif) && !dash.burst.is_finished();
        let amount = player_damage(weapon.dmg, &speed, &stats, momentum.0, burst);

        for (e, etf, mut kb) in &mut enemies {
            let epos = etf.translation.truncate();
            let dist = epos.distance(player_pos);
            if dist <= radius {
                let pull = (player_pos - epos).normalize_or_zero();
                // On tire fort, mais pas au-delà du joueur.
                kb.0 += pull * (260.0 + dist * 1.2);
                dmg.write_hit(e, amount);
                if augments.has(Augment::RateauAimante) {
                    commands
                        .entity(e)
                        .insert(Slowed(Timer::from_seconds(1.5, TimerMode::Once)));
                }
            }
        }

        // Visuel : un anneau de dents de râteau qui converge.
        for i in 0..18 {
            let dir = Vec2::from_angle(i as f32 / 18.0 * std::f32::consts::TAU);
            commands.spawn((
                Sprite::from_color(weapon.color.with_alpha(0.8), Vec2::splat(5.0)),
                Transform::from_translation((player_pos + dir * radius).extend(8.0)),
                Velocity(-dir * radius * 2.4),
                Lifetime::secs(0.3),
            ));
        }
    }
}

/// Arrosoir : gâchette tenue → traînée de pesticide le long du trajet
/// (le « Q de Singed », GDD §4.2).
fn arrosoir_system(
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
        if kind != WeaponKind::Arrosoir {
            continue;
        }
        let weapon = def(kind);
        if !slot_pressed(&buttons, slot, false) || cds.0[slot] > 0.0 {
            continue;
        }
        cds.0[slot] = weapon.cd;
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

/// Karcher : jet continu, dégâts en ticks + recul (GDD §4.2).
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
    mut enemies: Query<(Entity, &Transform, &Radius, &mut Knockback), With<Enemy>>,
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
        cds.0[slot] = weapon.cd;

        let burst = augments.has(Augment::DashOffensif) && !dash.burst.is_finished();
        let tick_dmg =
            player_damage(weapon.dmg, &speed, &stats, momentum.0, burst) * weapon.cd;
        let extra = if augments.has(Augment::BuseHautePression) { 1.25 } else { 1.0 };

        for (e, etf, er, mut kb) in &mut enemies {
            let to_enemy = etf.translation.truncate() - player_pos;
            let along = to_enemy.dot(aim.dir);
            if along < 0.0 || along > weapon.range {
                continue;
            }
            let perp = (to_enemy - aim.dir * along).length();
            if perp <= weapon.radius + er.0 {
                dmg.write_hit(e, tick_dmg * extra);
                kb.0 += aim.dir * weapon.kb * stats.kb_mult;
            }
        }

        // Gouttelettes du jet.
        for _ in 0..3 {
            let spread = rng.random_range(-0.12..0.12);
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
    mut q: Query<(&WeaponSprite, &mut Sprite, &mut Transform)>,
) {
    for (slot_marker, mut sprite, mut tf) in &mut q {
        let slot = slot_marker.0;
        match loadout.0[slot] {
            None => {
                sprite.color = Color::NONE;
            }
            Some(kind) => {
                let weapon = def(kind);
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
