//! Le jardinier rétréci : déplacement, dash, vitesse → dégâts (GDD §3).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::*;

use crate::augments::{Augment, Augments};
use crate::common::*;
use crate::meta::MetaSave;
use crate::stats::Stats;
use crate::weapons::DamageMsgExt;

// Dash court et net : une esquive de distance fixe (~100 px), pas un long
// burst de vitesse. La sur-vitesse résiduelle se résorbe vite (cf. movement).
pub const DASH_SPEED: f32 = 640.0;
pub const DASH_DURATION: f32 = 0.16;

// ---------------------------------------------------------------------------
// Ressources
// ---------------------------------------------------------------------------

/// Stats effectives du joueur, recalculées en continu depuis méta + augments.
#[derive(Resource, Clone)]
pub struct PlayerStats {
    pub max_speed: f32,
    pub accel: f32,
    pub max_hp: f32,
    pub dash_charges: u32,
    pub dash_cd: f32,
    pub dash_iframes: f32,
    pub dmg_mult: f32,
    pub aoe_mult: f32,
    pub poison_mult: f32,
    pub rake_mult: f32,
    pub kb_mult: f32,
    pub pattes_mult: f32,
    // --- Refonte v0.3 : effets des stats-up (GDD §3.3) ---
    /// Multiplicateur d'intervalle d'attaque (= 100/AS%). Plus petit = plus rapide.
    pub attack_cd_mult: f32,
    /// Régénération passive en HP/s (= 1.0 × Régén%/100).
    pub regen_hps: f32,
    /// Multiplicateur des dégâts subis (= 100/Rési%). Plus petit = plus résistant.
    pub incoming_mult: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self::compute(&MetaSave::default(), &Augments::default(), &Stats::default())
    }
}

impl PlayerStats {
    /// Calcule les stats effectives. Les 7 stats-up (GDD §3.3) se **multiplient**
    /// par-dessus la base méta+augments (neutres à 100 %), de sorte que la
    /// méta-progression existante reste valable (GDD §9).
    pub fn compute(meta: &MetaSave, augments: &Augments, stats: &Stats) -> Self {
        let speed_stacks = augments.count(Augment::JambesDeCriquet) as f32;
        Self {
            // Vitesse de pointe : 250 px/s à 100 %, ×MoveSpeed%/100. La vitesse
            // ne donne plus de dégâts par défaut (refonte v0.3, voir Élan).
            max_speed: 250.0
                * (1.0 + 0.05 * meta.up_speed as f32)
                * (1.0 + 0.15 * speed_stacks)
                * stats.move_mult(),
            // Montée en régime progressive (~0.3 s pour atteindre la pointe) :
            // il faut s'engager dans le mouvement pour gagner sa vitesse.
            accel: 850.0 * if augments.has(Augment::Cafeine) { 1.4 } else { 1.0 },
            max_hp: (50.0
                + 8.0 * meta.up_hp as f32
                + 15.0 * augments.count(Augment::Carapace) as f32)
                * stats.pv_mult(),
            dash_charges: 1 + augments.has(Augment::DoubleDetente) as u32,
            dash_cd: 1.25 * (1.0 - 0.10 * meta.up_dash as f32) * stats.dash_cd_mult(),
            dash_iframes: DASH_DURATION
                + 0.02
                + if augments.has(Augment::EsquiveFeline) { 0.15 } else { 0.0 },
            dmg_mult: (1.0 + 0.20 * augments.count(Augment::Aiguillon) as f32) * stats.dmg_mult(),
            aoe_mult: if augments.has(Augment::PelleElargie) { 1.35 } else { 1.0 },
            poison_mult: if augments.has(Augment::PesticideConcentre) { 1.6 } else { 1.0 },
            rake_mult: if augments.has(Augment::RateauAimante) { 1.4 } else { 1.0 },
            kb_mult: if augments.has(Augment::BuseHautePression) { 1.6 } else { 1.0 },
            pattes_mult: 1.0 + 0.15 * meta.up_pattes as f32,
            attack_cd_mult: stats.attack_cd_mult(),
            regen_hps: stats.regen_hps(),
            incoming_mult: stats.incoming_mult(),
        }
    }
}

/// Lecture de la vitesse instantanée. `ratio` (0..1) = vitesse/vitesse max
/// (feedback visuel + augment Élan). `mult` = multiplicateur de dégâts lié à la
/// vitesse : ×1.0 par défaut (refonte v0.3), ×0.8→×1.5 avec l'augment « Élan ».
#[derive(Resource, Default)]
pub struct SpeedInfo {
    pub ratio: f32,
    pub mult: f32,
}

/// Direction et position de visée (souris, GDD §3.4).
#[derive(Resource, Default)]
pub struct Aim {
    pub world: Vec2,
    pub dir: Vec2,
}

// ---------------------------------------------------------------------------
// Composants
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Dash {
    pub dir: Vec2,
    pub active: Timer,
    pub recharge: Timer,
    pub charges: u32,
    /// Fenêtre de burst post-dash (augment « dash offensif »).
    pub burst: Timer,
}

impl Dash {
    pub fn dashing(&self) -> bool {
        !self.active.is_finished()
    }
}

#[derive(Component)]
pub struct Iframes(pub Timer);

/// Bonus de dégâts qui monte tant qu'on bouge (augment « Momentum »).
#[derive(Component, Default)]
pub struct Momentum(pub f32);

/// Sprite fantôme de la traînée de vitesse.
#[derive(Component)]
struct TrailGhost;

/// Timer interne d'émission de la traînée.
#[derive(Resource)]
struct TrailTimer(Timer);

/// Couche du bas : les jambes, orientées vers le déplacement, animées.
#[derive(Component)]
struct PlayerLegs {
    anim: Timer,
    frame: usize,
}

/// Couche du milieu : les bras, qui tiennent les armes (orientés vers la visée).
#[derive(Component)]
struct PlayerArms;

/// Couche du haut : le chapeau, teinté (i-frames dash → bleu, dégâts → rouge).
#[derive(Component)]
struct PlayerBody;

/// Marqueur : cette couche s'oriente vers la visée (souris).
#[derive(Component)]
struct AimOriented;

/// Échelle de rendu (unités monde par pixel source). Un seul réglage pour
/// toutes les couches → elles restent à la même échelle et alignées.
const PLAYER_SCALE: f32 = 0.8;
/// Jambes et bras sont dessinés sur un canvas 54×54.
const LIMB_SIZE: f32 = 54.0 * PLAYER_SCALE;
/// Le chapeau (jardinier.png) est un canvas 32×32, centré.
const HAT_SIZE: f32 = 32.0 * PLAYER_SCALE;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerStats>()
            .init_resource::<SpeedInfo>()
            .init_resource::<Aim>()
            .insert_resource(TrailTimer(Timer::from_seconds(0.045, TimerMode::Repeating)))
            .add_systems(
                Update,
                recompute_stats.run_if(|p: Res<Paused>| !p.0),
            )
            .add_systems(
                Update,
                (update_aim, movement, dash_system, update_speed_info)
                    .chain()
                    .in_set(GameSet::Input)
                    .run_if(player_active),
            )
            .add_systems(
                Update,
                (
                    speed_feedback,
                    tick_iframes,
                    animate_legs,
                    orient_body,
                    tint_body,
                    momentum_system,
                    photosynthese,
                    regen_health,
                )
                    .in_set(GameSet::Post)
                    .run_if(player_active),
            );
    }
}

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Fait apparaître le jardinier en 3 couches superposées (GDD §13) :
/// 1. jambes (bas) — orientées vers le déplacement, animées ;
/// 2. bras (milieu) — orientés vers la visée, tiennent les armes ;
/// 3. sprites d'armes — un par slot, séparés (GDD §4.1) ;
/// 4. chapeau (haut) — orienté vers la visée, teinté.
pub fn spawn_player(
    commands: &mut Commands,
    sprites: &GameSprites,
    stats: &PlayerStats,
    pos: Vec2,
) -> Entity {
    let limb = Vec2::splat(LIMB_SIZE);
    let hat = Vec2::splat(HAT_SIZE);
    commands
        .spawn((
            Player,
            Transform::from_translation(pos.extend(10.0)),
            Visibility::Visible,
            Velocity::default(),
            Knockback::default(),
            Health::new(stats.max_hp),
            Radius(PLAYER_RADIUS),
            ClampToArena,
            Iframes(Timer::from_seconds(0.0, TimerMode::Once)),
            Momentum::default(),
            Dash {
                dir: Vec2::X,
                active: timer_done(DASH_DURATION),
                recharge: Timer::from_seconds(stats.dash_cd, TimerMode::Once),
                charges: stats.dash_charges,
                burst: timer_done(0.8),
            },
        ))
        .with_children(|parent| {
            // Couche 1 — jambes (sous tout le reste), orientées vers la marche.
            parent.spawn((
                PlayerLegs {
                    anim: Timer::from_seconds(0.15, TimerMode::Repeating),
                    frame: 0,
                },
                Sprite {
                    image: sprites.legs_walk[0].clone(),
                    custom_size: Some(limb),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -1.0),
            ));
            // Couche 2 — bras (centrés), orientés vers la visée.
            parent.spawn((
                PlayerArms,
                AimOriented,
                Sprite {
                    image: sprites.arms.clone(),
                    custom_size: Some(limb),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            // Couche 3 — slots d'armes (formes colorées), tenus par les bras.
            parent.spawn((
                crate::weapons::WeaponSprite(0),
                Sprite::from_color(Color::NONE, Vec2::new(4.0, 4.0)),
                Transform::from_xyz(0.0, 0.0, 0.5),
            ));
            parent.spawn((
                crate::weapons::WeaponSprite(1),
                Sprite::from_color(Color::NONE, Vec2::new(4.0, 4.0)),
                Transform::from_xyz(0.0, 0.0, 0.5),
            ));
            // Couche 4 — chapeau (au-dessus), orienté vers la visée, teinté.
            parent.spawn((
                PlayerBody,
                AimOriented,
                Sprite {
                    image: sprites.body.clone(),
                    custom_size: Some(hat),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 1.0),
            ));
        })
        .id()
}

fn timer_done(secs: f32) -> Timer {
    let mut t = Timer::from_seconds(secs, TimerMode::Once);
    t.tick(std::time::Duration::from_secs_f32(secs + 1.0));
    t
}

// ---------------------------------------------------------------------------
// Systèmes
// ---------------------------------------------------------------------------

fn recompute_stats(
    meta: Res<MetaSave>,
    augments: Res<Augments>,
    statup: Res<Stats>,
    mut stats: ResMut<PlayerStats>,
    mut player: Query<&mut Health, With<Player>>,
) {
    let new = PlayerStats::compute(&meta, &augments, &statup);
    // Si les PV max montent (Carapace, achat…), on crédite la différence.
    if let Ok(mut health) = player.single_mut() {
        let delta = new.max_hp - health.max;
        if delta.abs() > 0.01 {
            health.max = new.max_hp;
            if delta > 0.0 {
                health.hp = (health.hp + delta).min(health.max);
            } else {
                health.hp = health.hp.min(health.max);
            }
        }
    }
    *stats = new;
}

fn update_aim(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    player: Query<&Transform, With<Player>>,
    mut aim: ResMut<Aim>,
) {
    let Ok(window) = window.single() else { return };
    let Ok((camera, cam_tf)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world) = camera.viewport_to_world_2d(cam_tf, cursor) else { return };
    aim.world = world;
    if let Ok(tf) = player.single() {
        aim.dir = (world - tf.translation.truncate()).normalize_or(Vec2::X);
    }
}

fn movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    stats: Res<PlayerStats>,
    augments: Res<Augments>,
    mut player: Query<(&mut Velocity, &Health, &Dash), With<Player>>,
) {
    let Ok((mut vel, health, dash)) = player.single_mut() else {
        return;
    };
    if dash.dashing() {
        vel.0 = dash.dir * DASH_SPEED;
        return;
    }
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    let mut max = stats.max_speed;
    // Adrénaline : à PV bas, on court (donc on frappe) plus fort.
    if augments.has(Augment::Adrenaline) && health.ratio() < 0.3 {
        max *= 1.25;
    }
    let dt = time.delta_secs();
    let dir = dir.normalize_or_zero();
    let cur_speed = vel.0.length();

    if dir == Vec2::ZERO {
        // Pas d'input : on freine en conservant la direction. Le freinage est
        // plus vif que l'accélération → s'arrêter coûte de la vitesse (donc des
        // dégâts), ce qui pousse à rester en mouvement.
        let new_speed = (cur_speed - stats.accel * 2.0 * dt).max(0.0);
        vel.0 = if cur_speed > 0.001 {
            vel.0 / cur_speed * new_speed
        } else {
            Vec2::ZERO
        };
    } else {
        // Avec input : on sépare la MAGNITUDE (qui monte vers `max`, ou
        // redescend si on sort d'un dash en sur-vitesse) du CAP de direction
        // (qui pivote vers l'input). Conséquence clé : tourner ne fait (presque)
        // pas perdre de vitesse — on garde son élan en changeant d'angle.
        let target_speed = if cur_speed > max {
            (cur_speed - stats.accel * 2.5 * dt).max(max) // résorbe la sur-vitesse du dash
        } else {
            (cur_speed + stats.accel * dt).min(max)
        };
        let cur_dir = if cur_speed > 0.001 {
            vel.0 / cur_speed
        } else {
            dir
        };
        // Pivot rapide mais continu vers la direction visée.
        let turn = (14.0 * dt).min(1.0);
        let new_dir = cur_dir.lerp(dir, turn).normalize_or(dir);
        vel.0 = new_dir * target_speed;
    }
}

fn dash_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    stats: Res<PlayerStats>,
    augments: Res<Augments>,
    speed: Res<SpeedInfo>,
    aim: Res<Aim>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    mut player: Query<(&mut Dash, &mut Iframes, &Velocity, &Transform), With<Player>>,
    enemies: Query<(Entity, &Transform, &Radius), With<Enemy>>,
) {
    let Ok((mut dash, mut iframes, vel, tf)) = player.single_mut() else {
        return;
    };
    let was_dashing = dash.dashing();
    dash.active.tick(time.delta());
    dash.burst.tick(time.delta());

    // Recharge des charges de dash.
    if dash.charges < stats.dash_charges {
        dash.recharge.tick(time.delta());
        if dash.recharge.is_finished() {
            dash.charges += 1;
            dash.recharge = Timer::from_seconds(stats.dash_cd, TimerMode::Once);
        }
    }

    // Fin de dash : fenêtre de burst + explosion éventuelle (augments).
    if was_dashing && !dash.dashing() {
        if augments.has(Augment::DashOffensif) {
            dash.burst = Timer::from_seconds(0.8, TimerMode::Once);
        }
        if augments.has(Augment::SortieExplosive) {
            let pos = tf.translation.truncate();
            let amount = 18.0 * speed.mult * stats.dmg_mult;
            for (e, etf, er) in &enemies {
                if etf.translation.truncate().distance(pos) < 95.0 + er.0 {
                    dmg.write_hit(e, amount);
                }
            }
            // Onde de choc visuelle.
            for i in 0..14 {
                let dir = Vec2::from_angle(i as f32 / 14.0 * std::f32::consts::TAU);
                commands.spawn((
                    Sprite::from_color(Color::srgb(1.0, 0.8, 0.3), Vec2::splat(6.0)),
                    Transform::from_translation((pos + dir * 20.0).extend(8.0)),
                    Velocity(dir * 320.0),
                    Lifetime::secs(0.25),
                ));
            }
        }
    }

    let pressed = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::ShiftLeft)
        || keys.just_pressed(KeyCode::ShiftRight);
    if pressed && dash.charges > 0 && !dash.dashing() {
        dash.charges -= 1;
        let dir = if vel.0.length_squared() > 100.0 {
            vel.0.normalize()
        } else {
            aim.dir
        };
        dash.dir = dir;
        dash.active = Timer::from_seconds(DASH_DURATION, TimerMode::Once);
        iframes.0 = Timer::from_seconds(stats.dash_iframes, TimerMode::Once);
    }
}

fn update_speed_info(
    stats: Res<PlayerStats>,
    augments: Res<Augments>,
    mut info: ResMut<SpeedInfo>,
    player: Query<&Velocity, With<Player>>,
) {
    let Ok(vel) = player.single() else {
        return;
    };
    let speed_len = vel.0.length();
    // `ratio` (0..1) sert au feedback visuel (teinte/traînée/HUD) et à l'augment
    // « Élan ». On le plafonne à 1 pour que le dash (sur-vitesse) ne le fasse pas
    // exploser.
    info.ratio = (speed_len / stats.max_speed).clamp(0.0, 1.0);
    // Refonte v0.3 (GDD §4.x) : la vitesse ne donne PLUS de dégâts par défaut.
    // Le multiplicateur reste neutre (×1.0), sauf si l'augment « Élan » est pris,
    // qui réintroduit une courbe douce ×0.8 (arrêt) → ×1.5 (vitesse max).
    info.mult = if augments.has(Augment::Elan) {
        0.8 + 0.7 * info.ratio
    } else {
        1.0
    };
}

/// Feedback visuel de la vitesse : traînée de fantômes du chapeau quand on file.
/// « Indispensable pour que le joueur ressente sa puissance » (GDD §3.1).
fn speed_feedback(
    time: Res<Time>,
    info: Res<SpeedInfo>,
    sprites: Res<GameSprites>,
    mut trail: ResMut<TrailTimer>,
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(tf) = player.single() else {
        return;
    };
    trail.0.tick(time.delta());
    if info.ratio > 0.55 && trail.0.just_finished() {
        let alpha = 0.15 + 0.3 * info.ratio;
        commands.spawn((
            TrailGhost,
            Sprite {
                image: sprites.body.clone(),
                custom_size: Some(Vec2::splat(HAT_SIZE)),
                color: Color::srgb(1.0, 0.6, 0.25).with_alpha(alpha),
                ..default()
            },
            Transform::from_translation(tf.translation - Vec3::Z * 2.0),
            Lifetime::secs(0.28),
        ));
    }
}

/// Fait juste avancer le timer d'i-frames (le visuel est dans `tint_body`).
fn tick_iframes(time: Res<Time>, mut player: Query<&mut Iframes, With<Player>>) {
    if let Ok(mut iframes) = player.single_mut() {
        iframes.0.tick(time.delta());
    }
}

/// Couche 1 : oriente les jambes vers le déplacement et anime la marche
/// (alternance des 2 frames), pose de dash pendant le dash.
fn animate_legs(
    time: Res<Time>,
    sprites: Res<GameSprites>,
    player: Query<(&Velocity, &Dash), With<Player>>,
    mut legs: Query<(&mut PlayerLegs, &mut Sprite, &mut Transform)>,
) {
    let Ok((vel, dash)) = player.single() else {
        return;
    };
    let Ok((mut state, mut sprite, mut tf)) = legs.single_mut() else {
        return;
    };
    let speed = vel.0.length();
    // Oriente les jambes vers la marche (les pieds pointent vers +Y dans le
    // sprite → on retranche 90°). On garde l'orientation précédente à l'arrêt.
    if speed > 5.0 {
        tf.rotation = Quat::from_rotation_z(vel.0.to_angle() - std::f32::consts::FRAC_PI_2);
    }
    if dash.dashing() {
        sprite.image = sprites.legs_dash.clone();
        return;
    }
    if speed > 10.0 {
        // Cadence de l'animation proportionnelle à la vitesse.
        let rate = (0.22 - 0.0004 * speed).clamp(0.08, 0.22);
        state.anim.set_duration(std::time::Duration::from_secs_f32(rate));
        state.anim.tick(time.delta());
        if state.anim.just_finished() {
            state.frame = (state.frame + 1) % 2;
        }
        sprite.image = sprites.legs_walk[state.frame].clone();
    } else {
        // À l'arrêt : pose neutre (1re frame).
        sprite.image = sprites.legs_walk[0].clone();
    }
}

/// Couches bras + chapeau : orientées vers la visée (souris).
fn orient_body(aim: Res<Aim>, mut layers: Query<&mut Transform, With<AimOriented>>) {
    // L'avant des sprites pointe vers +Y → on retranche 90°.
    let rot = Quat::from_rotation_z(aim.dir.to_angle() - std::f32::consts::FRAC_PI_2);
    for mut tf in &mut layers {
        tf.rotation = rot;
    }
}

/// Couche 2 (teinte) : rouge quand on encaisse, bleu pendant les i-frames de
/// dash, sinon teinte normale (blanc = sprite tel quel).
fn tint_body(
    time: Res<Time>,
    mut commands: Commands,
    mut player: Query<(Entity, &Iframes, &Dash, Option<&mut HitFlash>), With<Player>>,
    mut body: Query<&mut Sprite, With<PlayerBody>>,
) {
    let Ok((entity, iframes, dash, hit_flash)) = player.single_mut() else {
        return;
    };
    let Ok(mut sprite) = body.single_mut() else {
        return;
    };

    // Priorité : flash de dégâts (rouge) > i-frames (bleu clignotant) > normal.
    if let Some(mut flash) = hit_flash {
        flash.0.tick(time.delta());
        if flash.0.is_finished() {
            commands.entity(entity).remove::<HitFlash>();
        } else {
            sprite.color = Color::srgb(1.0, 0.3, 0.3);
            return;
        }
    }

    if !iframes.0.is_finished() {
        // Pendant le dash : bleu franc. Hors dash (coup encaissé) : bleu clignotant.
        let blink = dash.dashing() || (iframes.0.elapsed_secs() * 30.0).sin() > 0.0;
        sprite.color = if blink {
            Color::srgb(0.55, 0.8, 1.0)
        } else {
            Color::srgb(0.85, 0.95, 1.0)
        };
    } else {
        sprite.color = Color::WHITE;
    }
}

/// Régénération passive de PV (stat « Régén », GDD §3.3). Ne ressuscite pas un
/// joueur déjà mort (hp ≤ 0 est géré par la mort).
fn regen_health(
    time: Res<Time>,
    stats: Res<PlayerStats>,
    mut player: Query<&mut Health, With<Player>>,
) {
    if stats.regen_hps <= 0.0 {
        return;
    }
    let Ok(mut health) = player.single_mut() else {
        return;
    };
    if health.hp > 0.0 && health.hp < health.max {
        health.hp = (health.hp + stats.regen_hps * time.delta_secs()).min(health.max);
    }
}

fn momentum_system(
    time: Res<Time>,
    augments: Res<Augments>,
    info: Res<SpeedInfo>,
    mut player: Query<&mut Momentum, With<Player>>,
) {
    let Ok(mut momentum) = player.single_mut() else {
        return;
    };
    if !augments.has(Augment::Momentum) {
        momentum.0 = 0.0;
        return;
    }
    if info.ratio > 0.4 {
        momentum.0 = (momentum.0 + 0.02 * time.delta_secs() * 10.0).min(0.4);
    } else {
        momentum.0 = 0.0;
    }
}

/// « Photosynthèse » : régénère au-dessus de 70 % de vitesse.
fn photosynthese(
    time: Res<Time>,
    augments: Res<Augments>,
    info: Res<SpeedInfo>,
    mut commands: Commands,
    mut player: Query<(&Transform, &mut Health), With<Player>>,
) {
    if !augments.has(Augment::Photosynthese) || info.ratio < 0.7 {
        return;
    }
    let Ok((tf, mut health)) = player.single_mut() else {
        return;
    };
    if health.hp < health.max {
        health.hp = (health.hp + 2.0 * time.delta_secs()).min(health.max);
        let mut rng = rand::rng();
        if rng.random_bool((time.delta_secs() * 6.0).clamp(0.0, 1.0) as f64) {
            let pos = tf.translation.truncate()
                + Vec2::new(rng.random_range(-12.0..12.0), rng.random_range(-12.0..12.0));
            commands.spawn((
                Sprite::from_color(Color::srgb(0.4, 1.0, 0.4).with_alpha(0.8), Vec2::splat(4.0)),
                Transform::from_translation(pos.extend(12.0)),
                Velocity(Vec2::Y * 40.0),
                Lifetime::secs(0.5),
            ));
        }
    }
}
