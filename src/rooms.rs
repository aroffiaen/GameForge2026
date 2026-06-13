//! Structure d'une run : biomes → salles → boss → choix (GDD §6).

use bevy::prelude::*;
use rand::prelude::*;

use crate::augments::{AfterAugment, Augment, Augments};
use crate::biomes::Biome;
use crate::common::*;
use crate::enemies::{spawn_enemy, EnemyKind};
use crate::meta::MetaSave;
use crate::player::{spawn_player, PlayerStats};
use crate::stats::{Stat, Stats};
use crate::weapons::PoisonPuddle;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomKind {
    Combat,
    Elite,
    Boss,
}

/// L'état de la run en cours.
#[derive(Resource)]
pub struct RunState {
    pub biome: Biome,
    /// 0..5 — une run traverse 5 biomes (GDD §6.1).
    pub biome_index: u32,
    /// Salles intermédiaires déjà jouées dans ce biome.
    pub room_index: u32,
    /// Nombre de salles intermédiaires de ce biome (1 à 3).
    pub rooms_in_biome: u32,
    pub room_kind: RoomKind,
    /// Vague du gauntlet de boss : 1-3 = vagues, 4 = boss en jeu.
    pub gauntlet: Option<u8>,
    /// La terrasse a-t-elle été atteinte par une vraie run ?
    pub came_from_run: bool,
    // --- Stats-Up chronométré (GDD §3.1, refonte v0.3 partie B) ---
    /// Stat misée à la porte précédente, transmise à la prochaine salle.
    pub pending_bet: Option<Stat>,
    /// Mise active dans la salle courante (résolue à la fin de la salle).
    pub bet: Option<Stat>,
    /// La salle courante est-elle chronométrée ? (faux = salle 1 / boss).
    pub chrono_active: bool,
    /// Temps cible et temps écoulé (s) de la salle chronométrée courante.
    pub chrono_target: f32,
    pub chrono_elapsed: f32,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            biome: Biome::Plaine,
            biome_index: 0,
            room_index: 0,
            rooms_in_biome: 2,
            room_kind: RoomKind::Combat,
            gauntlet: None,
            came_from_run: false,
            pending_bet: None,
            bet: None,
            chrono_active: false,
            chrono_target: 0.0,
            chrono_elapsed: 0.0,
        }
    }
}

impl RunState {
    pub fn hp_scale(&self) -> f32 {
        1.0 + 0.22 * self.biome_index as f32
    }
    pub fn dmg_scale(&self) -> f32 {
        1.0 + 0.15 * self.biome_index as f32
    }
}

/// Tout ce qui appartient à la salle courante (décor, portes, piédestaux).
#[derive(Component)]
pub struct RoomEntity;

#[derive(Component)]
pub struct Door;

/// Porte-stat : la stat misée si le joueur passe par cette porte (GDD §3.1).
/// Absente sur la porte simple d'entrée du boss (pas de mise).
#[derive(Component)]
pub struct DoorChoice(pub Stat);

#[derive(Message)]
pub struct BuildRoom;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>()
            .add_message::<BuildRoom>()
            .add_systems(OnEnter(AppState::EnRun), start_run)
            .add_systems(
                Update,
                build_room.run_if(in_state(AppState::EnRun)),
            )
            .add_systems(
                Update,
                (check_room_clear, tick_chrono, crate::common::tick_run_time)
                    .run_if(combat_active),
            )
            .add_systems(OnEnter(RunPhase::DoorOpen), spawn_door)
            .add_systems(
                Update,
                door_interact.run_if(in_state(RunPhase::DoorOpen)),
            )
            .add_systems(OnEnter(RunPhase::BiomeChoice), open_biome_ui)
            .add_systems(
                Update,
                pick_biome.run_if(in_state(RunPhase::BiomeChoice)),
            );
    }
}

// ---------------------------------------------------------------------------
// Démarrage de run
// ---------------------------------------------------------------------------

fn start_run(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    mut run: ResMut<RunState>,
    mut stats: ResMut<RunStats>,
    mut statup: ResMut<Stats>,
    mut augments: ResMut<Augments>,
    mut meta: ResMut<MetaSave>,
    mut arena: ResMut<Arena>,
    mut build: MessageWriter<BuildRoom>,
) {
    let mut rng = rand::rng();
    *stats = RunStats::default();
    statup.reset(); // chaque run repart à 100 % sur les 7 stats (GDD §3.3)
    augments.0.clear();
    meta.tools_bought_this_cycle = 0;
    meta.runs += 1;
    crate::meta::save_meta(&meta);

    *run = RunState {
        biome: Biome::Plaine,
        biome_index: 0,
        room_index: 0,
        rooms_in_biome: rng.random_range(1..=3),
        room_kind: RoomKind::Combat,
        gauntlet: None,
        came_from_run: false,
        ..default()
    };
    arena.half = Vec2::new(550.0, 310.0);

    let stats_now = PlayerStats::compute(&meta, &Augments::default(), &statup);
    let player = spawn_player(&mut commands, &sprites, &stats_now, Vec2::new(0.0, -240.0));
    commands.entity(player).insert(DespawnOnExit(AppState::EnRun));

    build.write(BuildRoom);
}

// ---------------------------------------------------------------------------
// Construction de salle
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn build_room(
    mut msgs: MessageReader<BuildRoom>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    arena: Res<Arena>,
    mut clear_color: ResMut<ClearColor>,
    mut toasts: MessageWriter<ToastMsg>,
    mut next_phase: ResMut<NextState<RunPhase>>,
    mut player: Query<(&mut Transform, &mut Velocity), With<Player>>,
    cleanup: Query<
        Entity,
        Or<(
            With<RoomEntity>,
            With<PoisonPuddle>,
            With<crate::enemies::HazardPuddle>,
            With<crate::enemies::EnemyProjectile>,
            With<PattePickup>,
            With<Enemy>,
        )>,
    >,
) {
    if msgs.read().next().is_none() {
        return;
    }
    let mut rng = rand::rng();

    // Nettoyage de la salle précédente.
    for e in &cleanup {
        commands.entity(e).despawn();
    }

    let biome = run.biome;
    clear_color.0 = biome.clear_color();

    // Sol + murs.
    commands.spawn((
        RoomEntity,
        Sprite::from_color(biome.ground_color(), arena.half * 2.0 + Vec2::splat(8.0)),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
    let wall_color = biome.clear_color().mix(&Color::BLACK, 0.3);
    let t = 14.0;
    for (pos, size) in [
        (Vec2::new(0.0, arena.half.y + t / 2.0), Vec2::new(arena.half.x * 2.0 + t * 2.0, t)),
        (Vec2::new(0.0, -arena.half.y - t / 2.0), Vec2::new(arena.half.x * 2.0 + t * 2.0, t)),
        (Vec2::new(arena.half.x + t / 2.0, 0.0), Vec2::new(t, arena.half.y * 2.0)),
        (Vec2::new(-arena.half.x - t / 2.0, 0.0), Vec2::new(t, arena.half.y * 2.0)),
    ] {
        commands.spawn((
            RoomEntity,
            Sprite::from_color(wall_color, size),
            Transform::from_translation(pos.extend(5.0)),
        ));
    }

    // Décor : touffes, cailloux.
    for _ in 0..26 {
        let pos = Vec2::new(
            rng.random_range(-arena.half.x + 20.0..arena.half.x - 20.0),
            rng.random_range(-arena.half.y + 20.0..arena.half.y - 20.0),
        );
        let s = rng.random_range(5.0..14.0);
        commands.spawn((
            RoomEntity,
            Sprite::from_color(
                biome.accent_color().with_alpha(rng.random_range(0.4..0.9)),
                Vec2::new(s, s * rng.random_range(0.5..1.2)),
            ),
            Transform::from_translation(pos.extend(-8.0)).with_rotation(
                Quat::from_rotation_z(rng.random_range(0.0..std::f32::consts::TAU)),
            ),
        ));
    }

    // Le joueur repart du bas de la salle.
    if let Ok((mut tf, mut vel)) = player.single_mut() {
        tf.translation.x = 0.0;
        tf.translation.y = -arena.half.y + 70.0;
        vel.0 = Vec2::ZERO;
    }

    // Contenu selon le type de salle.
    let hp_scale = run.hp_scale();
    let dmg_scale = run.dmg_scale();
    match run.room_kind {
        RoomKind::Combat => {
            run.gauntlet = None;
            let depth = run.biome_index;
            let n_small = rng.random_range(4..=6) + depth;
            let n_med = rng.random_range(1..=2) + depth / 2;
            let n_big = if depth >= 2 { rng.random_range(0..=1) } else { 0 };
            spawn_tier(&mut commands, &mut rng, biome, 0, n_small, hp_scale, dmg_scale, &arena);
            spawn_tier(&mut commands, &mut rng, biome, 1, n_med, hp_scale, dmg_scale, &arena);
            spawn_tier(&mut commands, &mut rng, biome, 2, n_big, hp_scale, dmg_scale, &arena);
            toasts.write(ToastMsg(format!(
                "{} — salle {}/{}",
                biome.name(),
                run.room_index + 1,
                run.rooms_in_biome
            )));
            setup_room_chrono(&mut run, n_small + n_med + n_big, false);
            next_phase.set(RunPhase::Fighting);
        }
        RoomKind::Elite => {
            run.gauntlet = None;
            let kinds = biome.tier(1);
            let kind = *kinds.choose(&mut rng).unwrap_or(&EnemyKind::Araignee);
            spawn_enemy(
                &mut commands,
                kind,
                Vec2::new(0.0, arena.half.y * 0.4),
                hp_scale,
                dmg_scale,
                true,
            );
            spawn_tier(&mut commands, &mut rng, biome, 0, 3, hp_scale, dmg_scale, &arena);
            toasts.write(ToastMsg("SALLE D'ÉLITE — grosse bête, grosse récompense".into()));
            setup_room_chrono(&mut run, 4, true);
            next_phase.set(RunPhase::Fighting);
        }
        RoomKind::Boss => {
            run.gauntlet = Some(1);
            setup_room_chrono(&mut run, 0, false); // jamais de chrono au boss
            let n = 7 + run.biome_index;
            spawn_tier(&mut commands, &mut rng, biome, 0, n, hp_scale, dmg_scale, &arena);
            toasts.write(ToastMsg(format!(
                "ANTRE DU BOSS — vague 1/3 ({})",
                biome.name()
            )));
            next_phase.set(RunPhase::Fighting);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_tier(
    commands: &mut Commands,
    rng: &mut impl Rng,
    biome: Biome,
    tier: u8,
    count: u32,
    hp_scale: f32,
    dmg_scale: f32,
    arena: &Arena,
) {
    let kinds = biome.tier(tier);
    let player_spawn = Vec2::new(0.0, -arena.half.y + 70.0);
    for _ in 0..count {
        let kind = *kinds.choose(rng).unwrap_or(&EnemyKind::Puceron);
        // On apparaît loin du joueur.
        let mut pos = Vec2::ZERO;
        for _ in 0..12 {
            pos = Vec2::new(
                rng.random_range(-arena.half.x + 30.0..arena.half.x - 30.0),
                rng.random_range(-arena.half.y + 30.0..arena.half.y - 30.0),
            );
            if pos.distance(player_spawn) > 220.0 {
                break;
            }
        }
        spawn_enemy(commands, kind, pos, hp_scale, dmg_scale, false);
    }
}

// ---------------------------------------------------------------------------
// Stats-Up chronométré (GDD §3.1-3.2)
// ---------------------------------------------------------------------------

/// Active (ou non) le chrono de la salle qu'on vient de construire, selon la
/// mise posée à la porte précédente (`pending_bet`).
fn setup_room_chrono(run: &mut RunState, enemy_count: u32, is_elite: bool) {
    match run.pending_bet.take() {
        Some(stat) => {
            // Seuil ∝ densité de la salle ; plus permissif en élite (GDD §3.2).
            let base = 4.0 + 1.1 * enemy_count as f32 + 0.5 * run.biome_index as f32;
            run.chrono_target = if is_elite { base * 1.6 } else { base };
            run.chrono_elapsed = 0.0;
            run.chrono_active = true;
            run.bet = Some(stat);
        }
        None => {
            run.chrono_active = false;
            run.bet = None;
        }
    }
}

/// Résout la mise de la salle chronométrée qu'on vient de nettoyer :
/// réussite (sous le temps) → +2 pts/s d'avance ; échec → −1 pt/s de retard,
/// plafonné à −15 (GDD §3.2).
fn resolve_chrono(run: &mut RunState, statup: &mut Stats, toasts: &mut MessageWriter<ToastMsg>) {
    if !run.chrono_active {
        return;
    }
    run.chrono_active = false;
    let Some(bet) = run.bet.take() else { return };
    let (target, elapsed) = (run.chrono_target, run.chrono_elapsed);
    if elapsed <= target {
        let gain = 2.0 * (target - elapsed);
        statup.add(bet, gain);
        toasts.write(ToastMsg(format!(
            "⏱ {elapsed:.1}s ≤ {target:.1}s — {} +{gain:.0}%",
            bet.label()
        )));
    } else {
        let loss = (elapsed - target).min(15.0);
        statup.add(bet, -loss);
        toasts.write(ToastMsg(format!(
            "⏱ {elapsed:.1}s > {target:.1}s — {} −{loss:.0}%",
            bet.label()
        )));
    }
}

/// Fait avancer le chrono de la salle courante (seulement en combat actif).
fn tick_chrono(time: Res<Time>, phase: Res<State<RunPhase>>, mut run: ResMut<RunState>) {
    if run.chrono_active && *phase.get() == RunPhase::Fighting {
        run.chrono_elapsed += time.delta_secs();
    }
}

// ---------------------------------------------------------------------------
// Détection de fin de salle & gauntlet de boss (GDD §6.3)
// ---------------------------------------------------------------------------

fn check_room_clear(
    state: Res<State<AppState>>,
    phase: Res<State<RunPhase>>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    arena: Res<Arena>,
    mut stats: ResMut<RunStats>,
    mut statup: ResMut<Stats>,
    augments: Res<Augments>,
    mut toasts: MessageWriter<ToastMsg>,
    mut after: ResMut<AfterAugment>,
    mut next_phase: ResMut<NextState<RunPhase>>,
    enemies: Query<Entity, With<Enemy>>,
    mut player: Query<&mut Health, With<Player>>,
) {
    if *state.get() != AppState::EnRun || *phase.get() != RunPhase::Fighting {
        return;
    }
    if !enemies.is_empty() {
        return;
    }
    let mut rng = rand::rng();

    match run.room_kind {
        RoomKind::Boss => {
            let wave = run.gauntlet.unwrap_or(1);
            let biome = run.biome;
            let hp_scale = run.hp_scale();
            let dmg_scale = run.dmg_scale();
            match wave {
                1 => {
                    run.gauntlet = Some(2);
                    let n = 4 + run.biome_index / 2;
                    spawn_tier(&mut commands, &mut rng, biome, 1, n, hp_scale, dmg_scale, &arena);
                    toasts.write(ToastMsg("Vague 2/3 — ça grossit…".into()));
                }
                2 => {
                    run.gauntlet = Some(3);
                    let n = 2 + run.biome_index / 2;
                    spawn_tier(&mut commands, &mut rng, biome, 2, n, hp_scale, dmg_scale, &arena);
                    toasts.write(ToastMsg("Vague 3/3 — les gros calibres.".into()));
                }
                3 => {
                    run.gauntlet = Some(4);
                    let boss = biome.boss();
                    crate::boss::spawn_boss(
                        &mut commands,
                        boss,
                        Vec2::new(0.0, arena.half.y * 0.5),
                        hp_scale,
                    );
                    toasts.write(ToastMsg(format!("{} entre en scène !", boss.name())));
                }
                _ => {
                    // Boss vaincu !
                    stats.bosses += 1;
                    run.gauntlet = None;
                    if augments.has(Augment::Rosee) {
                        if let Ok(mut health) = player.single_mut() {
                            health.hp = (health.hp + 12.0).min(health.max);
                        }
                        toasts.write(ToastMsg("Rosée du matin : +12 PV.".into()));
                    }
                    *after = AfterAugment::PostBoss;
                    next_phase.set(RunPhase::Augment);
                }
            }
        }
        RoomKind::Elite => {
            // La salle d'élite était chronométrée (permissive) : on résout la mise.
            resolve_chrono(&mut run, &mut statup, &mut toasts);
            // Grosse récompense : un augment (GDD §6.4).
            *after = AfterAugment::Door;
            next_phase.set(RunPhase::Augment);
        }
        _ => {
            // Fin d'une salle (éventuellement chronométrée) : on résout la mise,
            // puis on ouvre les portes-stat suivantes.
            resolve_chrono(&mut run, &mut statup, &mut toasts);
            next_phase.set(RunPhase::DoorOpen);
        }
    }
}

// ---------------------------------------------------------------------------
// Porte vers la salle suivante
// ---------------------------------------------------------------------------

fn spawn_door(
    state: Res<State<AppState>>,
    arena: Res<Arena>,
    run: Res<RunState>,
    mut commands: Commands,
    doors: Query<(), With<Door>>,
) {
    if *state.get() != AppState::EnRun || !doors.is_empty() {
        return;
    }
    // Juste avant le boss : porte simple (pas de mise). Sinon : 3 portes-stat.
    if run.room_index + 1 >= run.rooms_in_biome {
        spawn_door_inner(&mut commands, &arena);
    } else {
        spawn_stat_doors(&mut commands, &arena);
    }
}

/// Les 3 portes-stat (GDD §3.1) : 3 stats distinctes tirées au hasard, une par
/// porte. Passer par une porte = miser cette stat (la salle d'après est chrono).
fn spawn_stat_doors(commands: &mut Commands, arena: &Arena) {
    let mut rng = rand::rng();
    let mut pool = Stat::ALL.to_vec();
    pool.shuffle(&mut rng);
    let xs = [-220.0_f32, 0.0, 220.0];
    for (i, stat) in pool[0..3].iter().enumerate() {
        commands
            .spawn((
                RoomEntity,
                Door,
                DoorChoice(*stat),
                Sprite::from_color(Color::srgb(0.8, 0.7, 0.4), Vec2::new(84.0, 18.0)),
                Transform::from_translation(Vec2::new(xs[i], arena.half.y - 4.0).extend(6.0)),
            ))
            .with_children(|p| {
                p.spawn((
                    Text2d::new(format!("[{}] {}", i + 1, stat.label())),
                    TextFont { font_size: 15.0, ..default() },
                    TextColor(Color::srgb(0.85, 0.95, 1.0)),
                    Transform::from_xyz(0.0, 16.0, 1.0),
                ));
                p.spawn((
                    Text2d::new("miser (E / 1-2-3)"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.85, 0.6)),
                    Transform::from_xyz(0.0, -20.0, 1.0),
                ));
            });
    }
}

fn spawn_door_inner(commands: &mut Commands, arena: &Arena) {
    commands
        .spawn((
            RoomEntity,
            Door,
            Sprite::from_color(Color::srgb(0.8, 0.7, 0.4), Vec2::new(80.0, 18.0)),
            Transform::from_translation(Vec2::new(0.0, arena.half.y - 4.0).extend(6.0)),
        ))
        .with_children(|p| {
            p.spawn((
                Text2d::new("E — salle suivante"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(1.0, 0.95, 0.7)),
                Transform::from_xyz(0.0, -22.0, 1.0),
            ));
        });
}

fn door_interact(
    keys: Res<ButtonInput<KeyCode>>,
    mut run: ResMut<RunState>,
    mut build: MessageWriter<BuildRoom>,
    doors: Query<(&Transform, Option<&DoorChoice>), With<Door>>,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(player_tf) = player.single() else { return };
    let ppos = player_tf.translation.truncate();

    // Portes-stat triées par x (gauche→droite) pour la sélection au clavier.
    let mut stat_doors: Vec<(f32, Stat)> = doors
        .iter()
        .filter_map(|(tf, dc)| dc.map(|d| (tf.translation.x, d.0)))
        .collect();
    stat_doors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Choix : Some(Some(stat)) = porte-stat (mise), Some(None) = porte simple (boss).
    let mut chosen: Option<Option<Stat>> = None;

    // 1/2/3 sélectionnent la porte-stat correspondante.
    let key_idx = if keys.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else {
        None
    };
    if let Some(i) = key_idx {
        if let Some((_, stat)) = stat_doors.get(i) {
            chosen = Some(Some(*stat));
        }
    }

    // Sinon, E valide la porte la plus proche du joueur.
    if chosen.is_none() && keys.just_pressed(KeyCode::KeyE) {
        let mut best: Option<(f32, Option<Stat>)> = None;
        for (tf, dc) in &doors {
            let d = tf.translation.truncate().distance(ppos);
            if d <= 80.0 && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, dc.map(|x| x.0)));
            }
        }
        if let Some((_, stat)) = best {
            chosen = Some(stat);
        }
    }

    let Some(bet) = chosen else { return };
    run.pending_bet = bet;

    let mut rng = rand::rng();
    run.room_index += 1;
    if run.room_index >= run.rooms_in_biome {
        run.room_kind = RoomKind::Boss;
    } else {
        // Combat majoritaire, parfois élite. Les salles trésor sont retirées :
        // le système de portes-stat remplace ces respirations (GDD §6).
        run.room_kind = if rng.random_bool(0.8) {
            RoomKind::Combat
        } else {
            RoomKind::Elite
        };
    }
    build.write(BuildRoom);
}

// ---------------------------------------------------------------------------
// Choix du biome suivant (GDD §6.5)
// ---------------------------------------------------------------------------

fn open_biome_ui(mut commands: Commands, run: Res<RunState>) {
    let choices = run.biome.next_choices();
    commands
        .spawn((
            DespawnOnExit(RunPhase::BiomeChoice),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
            GlobalZIndex(20),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!(
                    "— BIOME {}/5 TERMINÉ — Où va-t-on ?",
                    run.biome_index + 1
                )),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::srgb(0.95, 0.9, 0.6)),
            ));
            for (i, biome) in choices.iter().enumerate() {
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            width: Val::Px(440.0),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.15, 0.2, 0.95)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(format!("[{}]  {}", i + 1, biome.name())),
                            TextFont { font_size: 20.0, ..default() },
                            TextColor(Color::srgb(0.6, 0.9, 1.0)),
                        ));
                        card.spawn((
                            Text::new(biome.desc()),
                            TextFont { font_size: 15.0, ..default() },
                            TextColor(Color::srgb(0.85, 0.85, 0.8)),
                        ));
                    });
            }
            parent.spawn((
                Text::new("Choisis avec 1 ou 2"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn pick_biome(
    keys: Res<ButtonInput<KeyCode>>,
    mut run: ResMut<RunState>,
    mut build: MessageWriter<BuildRoom>,
) {
    let choices = run.biome.next_choices();
    let picked = if keys.just_pressed(KeyCode::Digit1) {
        Some(choices[0])
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(choices[1])
    } else {
        None
    };
    let Some(biome) = picked else { return };
    let mut rng = rand::rng();
    run.biome = biome;
    run.biome_index += 1;
    run.room_index = 0;
    run.rooms_in_biome = rng.random_range(1..=3);
    run.room_kind = RoomKind::Combat;
    build.write(BuildRoom);
}
