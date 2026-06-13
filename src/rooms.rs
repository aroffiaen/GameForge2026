//! Structure d'une run : biomes → salles → boss → choix (GDD §6).

use bevy::prelude::*;
use rand::prelude::*;

use crate::augments::{grant_random, AfterAugment, Augment, Augments};
use crate::biomes::Biome;
use crate::common::*;
use crate::enemies::{spawn_enemy, EnemyKind};
use crate::meta::MetaSave;
use crate::player::{spawn_player, PlayerStats};
use crate::stats::Stats;
use crate::weapons::PoisonPuddle;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomKind {
    Combat,
    Elite,
    Tresor,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Reward {
    Soin,
    Pattes,
    AugmentAleatoire,
}

#[derive(Component)]
pub struct Pedestal(pub Reward);

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
                (check_room_clear, crate::common::tick_run_time).run_if(combat_active),
            )
            .add_systems(OnEnter(RunPhase::DoorOpen), spawn_door)
            .add_systems(
                Update,
                (door_interact, pedestal_interact).run_if(in_state(RunPhase::DoorOpen)),
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
            next_phase.set(RunPhase::Fighting);
        }
        RoomKind::Tresor => {
            run.gauntlet = None;
            let rewards = [
                (Reward::Soin, "Soin (+20 PV)", Color::srgb(0.9, 0.3, 0.4)),
                (Reward::Pattes, "Pattes (+35)", Color::srgb(0.95, 0.85, 0.5)),
                (Reward::AugmentAleatoire, "Augment mystère", Color::srgb(0.6, 0.4, 0.9)),
            ];
            for (i, (reward, label, color)) in rewards.into_iter().enumerate() {
                let x = (i as f32 - 1.0) * 150.0;
                commands
                    .spawn((
                        RoomEntity,
                        Pedestal(reward),
                        Sprite::from_color(color, Vec2::splat(26.0)),
                        Transform::from_translation(Vec2::new(x, 30.0).extend(4.0)),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text2d::new(label),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(Color::WHITE),
                            Transform::from_xyz(0.0, 28.0, 1.0),
                        ));
                    });
            }
            toasts.write(ToastMsg("Salle au trésor — choisis UN cadeau (E)".into()));
            // La phase reste DoorOpen → pas de transition d'état, donc on pose
            // la porte à la main.
            spawn_door_inner(&mut commands, &arena);
            next_phase.set(RunPhase::DoorOpen);
        }
        RoomKind::Boss => {
            run.gauntlet = Some(1);
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
// Détection de fin de salle & gauntlet de boss (GDD §6.3)
// ---------------------------------------------------------------------------

fn check_room_clear(
    state: Res<State<AppState>>,
    phase: Res<State<RunPhase>>,
    mut commands: Commands,
    mut run: ResMut<RunState>,
    arena: Res<Arena>,
    mut stats: ResMut<RunStats>,
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
            // Grosse récompense : un augment (GDD §6.4).
            *after = AfterAugment::Door;
            next_phase.set(RunPhase::Augment);
        }
        _ => {
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
    mut commands: Commands,
    doors: Query<(), With<Door>>,
) {
    if *state.get() != AppState::EnRun || !doors.is_empty() {
        return;
    }
    spawn_door_inner(&mut commands, &arena);
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
    doors: Query<&Transform, With<Door>>,
    player: Query<&Transform, With<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(player_tf) = player.single() else { return };
    let Ok(door_tf) = doors.single() else { return };
    if player_tf
        .translation
        .truncate()
        .distance(door_tf.translation.truncate())
        > 70.0
    {
        return;
    }
    let mut rng = rand::rng();
    run.room_index += 1;
    if run.room_index >= run.rooms_in_biome {
        run.room_kind = RoomKind::Boss;
    } else {
        // Première salle d'un biome toujours en combat, ensuite ça varie.
        let roll: f32 = rng.random_range(0.0..1.0);
        run.room_kind = if roll < 0.70 {
            RoomKind::Combat
        } else if roll < 0.85 {
            RoomKind::Elite
        } else {
            RoomKind::Tresor
        };
    }
    build.write(BuildRoom);
}

fn pedestal_interact(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut augments: ResMut<Augments>,
    mut toasts: MessageWriter<ToastMsg>,
    pedestals: Query<(Entity, &Transform, &Pedestal)>,
    mut player: Query<(&Transform, &mut Health), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok((player_tf, mut health)) = player.single_mut() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let mut taken = false;
    let mut rng = rand::rng();
    for (_, tf, pedestal) in &pedestals {
        if tf.translation.truncate().distance(player_pos) > 50.0 {
            continue;
        }
        taken = true;
        match pedestal.0 {
            Reward::Soin => {
                health.hp = (health.hp + 20.0).min(health.max);
                toasts.write(ToastMsg("+20 PV. Ça repousse.".into()));
            }
            Reward::Pattes => {
                let pos = tf.translation.truncate();
                for _ in 0..35 {
                    let offset =
                        Vec2::new(rng.random_range(-60.0..60.0), rng.random_range(-40.0..40.0));
                    commands.spawn((
                        Sprite::from_color(Color::srgb(0.95, 0.85, 0.5), Vec2::new(7.0, 3.0)),
                        Transform::from_translation((pos + offset).extend(5.0)),
                        PattePickup(1),
                        Lifetime::secs(15.0),
                    ));
                }
                toasts.write(ToastMsg("Une pluie de pattes ! Le bousier serait fier.".into()));
            }
            Reward::AugmentAleatoire => {
                grant_random(&mut augments, &mut toasts);
            }
        }
        break;
    }
    if taken {
        for (e, _, _) in &pedestals {
            commands.entity(e).despawn();
        }
    }
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
