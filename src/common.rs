//! Types, composants, messages et systèmes partagés par tout le jeu.

use bevy::prelude::*;
use rand::prelude::*;

use crate::augments::{Augment, Augments};
use crate::meta::MetaSave;
use crate::player::PlayerStats;

// ---------------------------------------------------------------------------
// Constantes de design (GDD §3)
// ---------------------------------------------------------------------------

/// Modèle « flat » (GDD §3.1) : la vitesse réelle (px/s) divisée par cette
/// valeur donne le multiplicateur de dégâts. 100 px/s = ×1.0, 250 px/s = ×2.5.
/// Comme le calcul part de la vitesse absolue, chaque bonus de vitesse relève
/// AUSSI le plafond de dégâts atteignable.
pub const SPEED_PER_MULT: f32 = 100.0;
/// Plancher du multiplicateur quand le joueur est (quasi) immobile.
pub const DMG_MULT_MIN: f32 = 0.4;
pub const PLAYER_RADIUS: f32 = 12.0;

// ---------------------------------------------------------------------------
// États
// ---------------------------------------------------------------------------

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Le hub : cabanon, bousier, établi (GDD §11.1).
    #[default]
    Cabanon,
    /// Une run dans le jardin (GDD §6).
    EnRun,
    /// La finale : survie chronométrée (GDD §10).
    Terrasse,
    /// Mort — retour au cabanon avec une excuse bidon (GDD §2.3).
    GameOver,
}

/// Phase interne d'une run (n'a de sens qu'en `AppState::EnRun`).
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunPhase {
    #[default]
    None,
    /// Combat en cours dans la salle.
    Fighting,
    /// Salle nettoyée, la porte est ouverte.
    DoorOpen,
    /// Choix d'augment (3 → 1, GDD §5.1).
    Augment,
    /// Choix du biome suivant (2 options, GDD §6.5).
    BiomeChoice,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Ai,
    Move,
    Combat,
    Post,
}

#[derive(Resource, Default)]
pub struct Paused(pub bool);

/// Condition : le joueur peut bouger (hub compris).
pub fn player_active(
    state: Res<State<AppState>>,
    phase: Res<State<RunPhase>>,
    paused: Res<Paused>,
) -> bool {
    if paused.0 {
        return false;
    }
    match state.get() {
        AppState::Cabanon | AppState::Terrasse => true,
        AppState::EnRun => matches!(phase.get(), RunPhase::Fighting | RunPhase::DoorOpen),
        AppState::GameOver => false,
    }
}

/// Condition : le combat est actif (armes, ennemis, dégâts).
pub fn combat_active(
    state: Res<State<AppState>>,
    phase: Res<State<RunPhase>>,
    paused: Res<Paused>,
) -> bool {
    if paused.0 {
        return false;
    }
    match state.get() {
        AppState::Terrasse => true,
        AppState::EnRun => matches!(phase.get(), RunPhase::Fighting | RunPhase::DoorOpen),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Composants génériques
// ---------------------------------------------------------------------------

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

/// Impulsion de recul, appliquée puis amortie chaque frame.
#[derive(Component, Default)]
pub struct Knockback(pub Vec2);

#[derive(Component)]
pub struct Health {
    pub hp: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { hp: max, max }
    }
    pub fn ratio(&self) -> f32 {
        (self.hp / self.max).clamp(0.0, 1.0)
    }
}

/// Rayon de collision (cercles partout, simple et suffisant).
#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct BossTag;

/// Dégâts de contact infligés au joueur.
#[derive(Component)]
pub struct ContactDmg(pub f32);

/// Anti-spam des dégâts de contact.
#[derive(Component)]
pub struct ContactCd(pub Timer);

/// L'entité ignore les dégâts de contact (taupe enfouie, crapaud en l'air…).
#[derive(Component)]
pub struct Untouchable;

/// Les entités marquées sont contraintes aux bords de l'arène.
#[derive(Component)]
pub struct ClampToArena;

/// Couleur de base du sprite (pour les flashs de dégâts / teintes de statut).
#[derive(Component)]
pub struct BaseColor(pub Color);

/// Despawn automatique + fondu du sprite sur la fin de vie.
#[derive(Component)]
pub struct Lifetime(pub Timer);

impl Lifetime {
    pub fn secs(s: f32) -> Self {
        Self(Timer::from_seconds(s, TimerMode::Once))
    }
}

/// Texte flottant (dégâts, pattes…).
#[derive(Component)]
pub struct Floaty {
    pub timer: Timer,
    pub vel: Vec2,
}

/// Patte d'insecte à ramasser (la monnaie, GDD §11.2).
#[derive(Component)]
pub struct PattePickup(pub u32);

/// Empoisonné par le pesticide (GDD §4.3) — le timer se rafraîchit au contact.
#[derive(Component)]
pub struct Poisoned {
    pub timer: Timer,
    pub tick: Timer,
    pub dps: f32,
}

/// Ralenti (râteau aimanté, etc.).
#[derive(Component)]
pub struct Slowed(pub Timer);

/// Flash blanc bref quand une entité encaisse un coup.
#[derive(Component)]
pub struct HitFlash(pub Timer);

// ---------------------------------------------------------------------------
// Ressources
// ---------------------------------------------------------------------------

/// Handles des sprites chargés une fois au démarrage.
#[derive(Resource)]
pub struct GameSprites {
    /// Jambes : 2 frames de marche (alternées).
    pub legs_walk: [Handle<Image>; 2],
    /// Jambes : pose de dash.
    pub legs_dash: Handle<Image>,
    /// Bras (couche du milieu, orientée vers la visée).
    pub arms: Handle<Image>,
    /// Chapeau (couche du haut, teintée).
    pub body: Handle<Image>,
    /// Le bousier (PNJ du cabanon).
    pub bousier: Handle<Image>,
    /// Sprite de la pelle (arme).
    pub pelle: Handle<Image>,
}

/// Demi-dimensions de l'arène courante.
#[derive(Resource)]
pub struct Arena {
    pub half: Vec2,
}

/// Statistiques de la run en cours.
#[derive(Resource, Default)]
pub struct RunStats {
    pub kills: u32,
    pub pattes: u64,
    pub bosses: u32,
    pub time: f32,
}

/// Infos affichées sur l'écran de mort.
#[derive(Resource, Default)]
pub struct DeathInfo {
    pub excuse: String,
    pub kills: u32,
    pub pattes: u64,
    pub time: f32,
    pub terrasse_time: Option<f32>,
    pub new_best: bool,
}

/// Excuses bidon de retour au cabanon (GDD §2.3).
pub const EXCUSES: &[&str] = &[
    "Un ruissellement d'eau de pluie t'a charrié jusqu'au cabanon.",
    "Un insecte super sympa t'a ramené sur son dos.",
    "Tu t'es réveillé. C'était un rêve. Enfin… presque.",
    "Une bourrasque t'a déposé pile devant la porte. Pratique.",
    "Le bousier jure qu'il n'y est pour rien. Il rigole, pourtant.",
    "Tu as glissé sur une limace. Longtemps. Très longtemps.",
    "Un escargot livreur t'a raccompagné. Délai non garanti.",
    "Tu t'es évanoui d'avoir trop couru. L'ironie ne t'échappe pas.",
    "Une fourmi t'a confondu avec une miette et t'a rapporté ici.",
    "Le tuyau d'arrosage a eu un hoquet. Te voilà rincé, et rentré.",
    "Tu as pris un pétale dans la figure. Un GROS pétale.",
    "Quelqu'un a crié « apéro » et tes jambes ont décidé seules.",
    "Une taupe t'a poliment montré la sortie. Par en dessous.",
    "Le jardin a demandé une pause. Toi aussi, apparemment.",
];

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DamageKind {
    Hit,
    Poison,
}

#[derive(Message)]
pub struct DamageMsg {
    pub target: Entity,
    pub amount: f32,
    pub kind: DamageKind,
}

#[derive(Message)]
#[allow(dead_code)] // pos/pattes : utiles aux futurs consommateurs (audio, FX…)
pub struct EnemyDied {
    pub pos: Vec2,
    pub pattes: u32,
    pub was_boss: bool,
    pub was_poisoned: bool,
}

#[derive(Message)]
pub struct PlayerDied;

/// Petit message d'annonce affiché en haut de l'écran.
#[derive(Message)]
pub struct ToastMsg(pub String);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<RunPhase>()
            .init_resource::<Paused>()
            .init_resource::<RunStats>()
            .init_resource::<DeathInfo>()
            .insert_resource(Arena {
                half: Vec2::new(550.0, 310.0),
            })
            .add_message::<DamageMsg>()
            .add_message::<EnemyDied>()
            .add_message::<PlayerDied>()
            .add_message::<ToastMsg>()
            .configure_sets(
                Update,
                (
                    GameSet::Input,
                    GameSet::Ai,
                    GameSet::Move,
                    GameSet::Combat,
                    GameSet::Post,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (apply_velocity, apply_knockback)
                    .in_set(GameSet::Move)
                    .run_if(player_active),
            )
            .add_systems(
                Update,
                (poison_tick, apply_damage, contact_damage)
                    .in_set(GameSet::Combat)
                    .run_if(combat_active),
            )
            .add_systems(
                Update,
                (check_deaths, handle_player_death)
                    .chain()
                    .in_set(GameSet::Post)
                    .run_if(combat_active),
            )
            .add_systems(
                Update,
                (
                    update_lifetimes,
                    update_floaties,
                    update_hit_flash,
                    update_slowed,
                    collect_pattes,
                )
                    .in_set(GameSet::Post)
                    .run_if(player_active),
            )
            .add_systems(OnExit(AppState::EnRun), cleanup_combat_entities)
            .add_systems(OnExit(AppState::Terrasse), cleanup_combat_entities);
    }
}

/// À la sortie d'un état de combat, on purge tout ce qui a été créé en jeu
/// (les entités à `DespawnOnExit` sont gérées par Bevy, ceci couvre le reste).
fn cleanup_combat_entities(
    mut commands: Commands,
    q: Query<
        Entity,
        Or<(
            With<Enemy>,
            With<crate::rooms::RoomEntity>,
            With<crate::weapons::PoisonPuddle>,
            With<crate::enemies::HazardPuddle>,
            With<crate::enemies::EnemyProjectile>,
            With<PattePickup>,
            With<Floaty>,
            With<Lifetime>,
        )>,
    >,
) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------------------------------------------------------------------
// Systèmes
// ---------------------------------------------------------------------------

fn apply_velocity(
    time: Res<Time>,
    arena: Res<Arena>,
    mut q: Query<(&mut Transform, &Velocity, Option<&Radius>, Has<ClampToArena>)>,
) {
    let dt = time.delta_secs();
    for (mut tf, vel, radius, clamp) in &mut q {
        tf.translation.x += vel.0.x * dt;
        tf.translation.y += vel.0.y * dt;
        if clamp {
            let r = radius.map(|r| r.0).unwrap_or(10.0);
            let lim = arena.half - Vec2::splat(r);
            tf.translation.x = tf.translation.x.clamp(-lim.x, lim.x);
            tf.translation.y = tf.translation.y.clamp(-lim.y, lim.y);
        }
    }
}

fn apply_knockback(time: Res<Time>, mut q: Query<(&mut Transform, &mut Knockback)>) {
    let dt = time.delta_secs();
    for (mut tf, mut kb) in &mut q {
        if kb.0.length_squared() < 1.0 {
            kb.0 = Vec2::ZERO;
            continue;
        }
        tf.translation.x += kb.0.x * dt;
        tf.translation.y += kb.0.y * dt;
        let decay = (1.0 - 6.0 * dt).max(0.0);
        kb.0 *= decay;
    }
}

fn poison_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut dmg: MessageWriter<DamageMsg>,
    mut q: Query<(Entity, &mut Poisoned, Option<&crate::enemies::EnemyKind>)>,
) {
    for (e, mut poison, kind) in &mut q {
        poison.timer.tick(time.delta());
        poison.tick.tick(time.delta());
        if poison.tick.just_finished() {
            // Les « mous » (limaces…) fondent plus vite au pesticide (GDD §8.1).
            let vuln = kind.map(|k| crate::enemies::def(*k).poison_vuln).unwrap_or(1.0);
            dmg.write(DamageMsg {
                target: e,
                amount: poison.dps * vuln * poison.tick.duration().as_secs_f32(),
                kind: DamageKind::Poison,
            });
        }
        if poison.timer.is_finished() {
            commands.entity(e).remove::<Poisoned>();
        }
    }
}

fn apply_damage(
    mut commands: Commands,
    mut msgs: MessageReader<DamageMsg>,
    mut q: Query<(&mut Health, &Transform)>,
) {
    let mut rng = rand::rng();
    for msg in msgs.read() {
        let Ok((mut health, tf)) = q.get_mut(msg.target) else {
            continue;
        };
        health.hp -= msg.amount;
        commands
            .entity(msg.target)
            .insert(HitFlash(Timer::from_seconds(0.08, TimerMode::Once)));
        // Texte flottant de dégâts.
        let color = match msg.kind {
            DamageKind::Hit => Color::srgb(1.0, 0.9, 0.3),
            DamageKind::Poison => Color::srgb(0.5, 1.0, 0.3),
        };
        let jitter = Vec2::new(rng.random_range(-8.0..8.0), rng.random_range(0.0..8.0));
        spawn_floaty(
            &mut commands,
            tf.translation.truncate() + jitter + Vec2::Y * 16.0,
            format!("{}", msg.amount.round().max(1.0) as i32),
            color,
            13.0,
        );
    }
}

pub fn spawn_floaty(commands: &mut Commands, pos: Vec2, text: String, color: Color, size: f32) {
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(pos.extend(50.0)),
        Floaty {
            timer: Timer::from_seconds(0.7, TimerMode::Once),
            vel: Vec2::new(0.0, 45.0),
        },
    ));
}

fn update_floaties(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Floaty, &mut Transform, &mut TextColor)>,
) {
    for (e, mut floaty, mut tf, mut color) in &mut q {
        floaty.timer.tick(time.delta());
        tf.translation.x += floaty.vel.x * time.delta_secs();
        tf.translation.y += floaty.vel.y * time.delta_secs();
        let left = 1.0 - floaty.timer.fraction();
        color.0 = color.0.with_alpha(left.min(1.0));
        if floaty.timer.is_finished() {
            commands.entity(e).despawn();
        }
    }
}

fn update_lifetimes(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Lifetime, Option<&mut Sprite>)>,
) {
    for (e, mut life, sprite) in &mut q {
        life.0.tick(time.delta());
        if let Some(mut sprite) = sprite {
            let left = 1.0 - life.0.fraction();
            let alpha = sprite.color.alpha().min(left * 2.0);
            sprite.color = sprite.color.with_alpha(alpha);
        }
        if life.0.is_finished() {
            commands.entity(e).despawn();
        }
    }
}

fn update_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut HitFlash, &mut Sprite, Option<&BaseColor>)>,
) {
    for (e, mut flash, mut sprite, base) in &mut q {
        flash.0.tick(time.delta());
        if flash.0.is_finished() {
            if let Some(base) = base {
                sprite.color = base.0;
            }
            commands.entity(e).remove::<HitFlash>();
        } else {
            sprite.color = Color::WHITE;
        }
    }
}

fn update_slowed(time: Res<Time>, mut commands: Commands, mut q: Query<(Entity, &mut Slowed)>) {
    for (e, mut slowed) in &mut q {
        slowed.0.tick(time.delta());
        if slowed.0.is_finished() {
            commands.entity(e).remove::<Slowed>();
        }
    }
}

/// Dégâts de contact ennemi → joueur, avec i-frames côté joueur.
fn contact_damage(
    time: Res<Time>,
    mut dmg: MessageWriter<DamageMsg>,
    mut enemies: Query<
        (&Transform, &Radius, &ContactDmg, &mut ContactCd, Has<Untouchable>),
        With<Enemy>,
    >,
    mut player: Query<
        (Entity, &Transform, &Radius, &mut Knockback, &mut crate::player::Iframes),
        (With<Player>, Without<Enemy>),
    >,
) {
    let Ok((player_e, player_tf, player_r, mut player_kb, mut iframes)) = player.single_mut()
    else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (tf, radius, contact, mut cd, untouchable) in &mut enemies {
        cd.0.tick(time.delta());
        if untouchable || !cd.0.is_finished() {
            continue;
        }
        let pos = tf.translation.truncate();
        if pos.distance(player_pos) <= radius.0 + player_r.0 {
            if !iframes.0.is_finished() {
                continue;
            }
            dmg.write(DamageMsg {
                target: player_e,
                amount: contact.0,
                kind: DamageKind::Hit,
            });
            // I-frames après un coup reçu + petit recul.
            iframes.0 = Timer::from_seconds(0.6, TimerMode::Once);
            let dir = (player_pos - pos).normalize_or(Vec2::Y);
            player_kb.0 += dir * 220.0;
            cd.0 = Timer::from_seconds(0.8, TimerMode::Once);
        }
    }
}

/// Détecte les morts : ennemis → pattes + gibs + message ; joueur → PlayerDied.
fn check_deaths(
    mut commands: Commands,
    mut died: MessageWriter<EnemyDied>,
    mut player_died: MessageWriter<PlayerDied>,
    mut stats: ResMut<RunStats>,
    augments: Res<Augments>,
    player_stats: Res<PlayerStats>,
    enemies: Query<
        (Entity, &Transform, &Health, &crate::enemies::PattesDrop, Has<BossTag>, Has<Poisoned>),
        With<Enemy>,
    >,
    all_enemies: Query<(Entity, &Transform), With<Enemy>>,
    player: Query<(&Health, &Transform), With<Player>>,
) {
    let mut rng = rand::rng();
    let mut poison_spreads: Vec<Vec2> = Vec::new();
    let mut dead: Vec<Entity> = Vec::new();

    for (e, tf, health, drop, is_boss, was_poisoned) in &enemies {
        if health.hp > 0.0 {
            continue;
        }
        let pos = tf.translation.truncate();
        dead.push(e);
        stats.kills += 1;

        // Drop de pattes (bonus méta « gain de pattes » appliqué ici).
        let mut count = drop.0 as f32 * player_stats.pattes_mult;
        if rng.random_bool((count.fract() as f64).clamp(0.0, 1.0)) {
            count += 1.0;
        }
        for _ in 0..count as u32 {
            let offset = Vec2::new(rng.random_range(-18.0..18.0), rng.random_range(-18.0..18.0));
            commands.spawn((
                Sprite::from_color(Color::srgb(0.95, 0.85, 0.5), Vec2::new(7.0, 3.0)),
                Transform::from_translation((pos + offset).extend(5.0))
                    .with_rotation(Quat::from_rotation_z(rng.random_range(0.0..std::f32::consts::TAU))),
                PattePickup(1),
                Lifetime::secs(12.0),
            ));
        }

        // Gibs : petits débris colorés.
        for _ in 0..5 {
            let dir = Vec2::from_angle(rng.random_range(0.0..std::f32::consts::TAU));
            commands.spawn((
                Sprite::from_color(Color::srgb(0.4, 0.5, 0.2), Vec2::splat(4.0)),
                Transform::from_translation(pos.extend(6.0)),
                Velocity(dir * rng.random_range(60.0..160.0)),
                Lifetime::secs(0.45),
            ));
        }

        if was_poisoned && augments.has(Augment::Epidemie) {
            poison_spreads.push(pos);
        }

        died.write(EnemyDied {
            pos,
            pattes: count as u32,
            was_boss: is_boss,
            was_poisoned,
        });
        commands.entity(e).despawn();
    }

    // Keystone « Épidémie » : le poison se propage à la mort (GDD §5.2).
    for pos in poison_spreads {
        for (e, tf) in &all_enemies {
            if dead.contains(&e) {
                continue;
            }
            if tf.translation.truncate().distance(pos) < 110.0 {
                commands.entity(e).insert(Poisoned {
                    timer: Timer::from_seconds(2.5, TimerMode::Once),
                    tick: Timer::from_seconds(0.4, TimerMode::Repeating),
                    dps: 12.0 * player_stats.poison_mult,
                });
            }
        }
    }

    if let Ok((health, _)) = player.single() {
        if health.hp <= 0.0 {
            player_died.write(PlayerDied);
        }
    }
}

/// Mort du joueur : excuse bidon, sauvegarde, écran de game over.
fn handle_player_death(
    mut msgs: MessageReader<PlayerDied>,
    mut death: ResMut<DeathInfo>,
    mut meta: ResMut<MetaSave>,
    stats: Res<RunStats>,
    state: Res<State<AppState>>,
    terrasse: Option<Res<crate::terrasse::TerrasseState>>,
    mut next: ResMut<NextState<AppState>>,
    mut next_phase: ResMut<NextState<RunPhase>>,
) {
    if msgs.read().next().is_none() {
        return;
    }
    let mut rng = rand::rng();
    let excuse = EXCUSES.choose(&mut rng).unwrap_or(&EXCUSES[0]);

    let terrasse_time = if *state.get() == AppState::Terrasse {
        terrasse.map(|t| t.time)
    } else {
        None
    };
    let mut new_best = false;
    if let Some(t) = terrasse_time {
        if t > meta.best_terrasse {
            meta.best_terrasse = t;
            new_best = true;
        }
    }

    *death = DeathInfo {
        excuse: excuse.to_string(),
        kills: stats.kills,
        pattes: stats.pattes,
        time: stats.time,
        terrasse_time,
        new_best,
    };
    meta.deaths += 1;
    crate::meta::save_meta(&meta);
    next_phase.set(RunPhase::None);
    next.set(AppState::GameOver);
}

/// Les pattes volent vers le joueur puis sont encaissées (monnaie persistante).
fn collect_pattes(
    time: Res<Time>,
    mut commands: Commands,
    mut meta: ResMut<MetaSave>,
    mut stats: ResMut<RunStats>,
    mut pickups: Query<(Entity, &mut Transform, &PattePickup), Without<Player>>,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (e, mut tf, pickup) in &mut pickups {
        let pos = tf.translation.truncate();
        let dist = pos.distance(player_pos);
        if dist < 140.0 {
            let dir = (player_pos - pos).normalize_or_zero();
            let speed = 380.0 * (1.0 - dist / 160.0).max(0.3);
            tf.translation += (dir * speed * time.delta_secs()).extend(0.0);
        }
        if dist < 18.0 {
            meta.pattes += pickup.0 as u64;
            stats.pattes += pickup.0 as u64;
            commands.entity(e).despawn();
        }
    }
}

/// Compte le temps de run.
pub fn tick_run_time(time: Res<Time>, mut stats: ResMut<RunStats>) {
    stats.time += time.delta_secs();
}
