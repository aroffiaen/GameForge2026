//! Le mode Terrasse : survie chronométrée sans fin (GDD §10).
//! Ce n'est pas un refuge. Ça ne l'a jamais été.

use bevy::prelude::*;
use rand::prelude::*;

use crate::biomes::ALL_BIOMES;
use crate::boss::spawn_boss;
use crate::common::*;
use crate::enemies::{spawn_enemy, EnemyKind};
use crate::meta::MetaSave;
use crate::player::{spawn_player, PlayerStats};
use crate::rooms::RunState;

#[derive(Resource)]
pub struct TerrasseState {
    pub time: f32,
    pub spawn_timer: Timer,
    pub next_boss_at: f32,
}

impl Default for TerrasseState {
    fn default() -> Self {
        Self {
            time: 0.0,
            spawn_timer: Timer::from_seconds(1.6, TimerMode::Repeating),
            next_boss_at: 60.0,
        }
    }
}

const ALL_KINDS: &[EnemyKind] = &[
    EnemyKind::Fourmi,
    EnemyKind::Escargot,
    EnemyKind::Araignee,
    EnemyKind::Criquet,
    EnemyKind::Guepe,
    EnemyKind::Cigale,
];

pub struct TerrassePlugin;

impl Plugin for TerrassePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrasseState>()
            .add_systems(OnEnter(AppState::Terrasse), enter_terrasse)
            .add_systems(
                Update,
                (tick_terrasse, terrasse_spawner)
                    .run_if(in_state(AppState::Terrasse))
                    .run_if(|p: Res<Paused>| !p.0),
            );
    }
}

fn enter_terrasse(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    mut terrasse: ResMut<TerrasseState>,
    mut arena: ResMut<Arena>,
    mut clear_color: ResMut<ClearColor>,
    run: Res<RunState>,
    meta: Res<MetaSave>,
    augments: Res<crate::augments::Augments>,
    statup: Res<crate::stats::Stats>,
    mut stats: ResMut<RunStats>,
    mut toasts: MessageWriter<ToastMsg>,
) {
    *terrasse = TerrasseState::default();
    arena.half = Vec2::new(620.0, 340.0);
    clear_color.0 = Color::srgb(0.12, 0.12, 0.14);

    // En accès direct depuis le cabanon, la run repart de zéro.
    if !run.came_from_run {
        *stats = RunStats::default();
    }

    // Sol de la terrasse (texture dédiée).
    let tile = Color::srgb(0.45, 0.42, 0.4);
    commands.spawn((
        DespawnOnExit(AppState::Terrasse),
        Sprite {
            image: sprites.zones.terrasse.clone(),
            custom_size: Some(arena.half * 2.0 + Vec2::splat(8.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
    let mut rng = rand::rng();
    for _ in 0..40 {
        let pos = Vec2::new(
            rng.random_range(-arena.half.x + 20.0..arena.half.x - 20.0),
            rng.random_range(-arena.half.y + 20.0..arena.half.y - 20.0),
        );
        commands.spawn((
            DespawnOnExit(AppState::Terrasse),
            Sprite::from_color(
                tile.mix(&Color::BLACK, rng.random_range(0.05..0.25)),
                Vec2::new(rng.random_range(30.0..80.0), 3.0),
            ),
            Transform::from_translation(pos.extend(-9.0)),
        ));
    }

    // Le joueur arrive avec son build de run (ou à nu en accès direct).
    let player_stats = PlayerStats::compute(&meta, &augments, &statup);
    let player = spawn_player(&mut commands, &sprites, &player_stats, Vec2::ZERO);
    commands
        .entity(player)
        .insert(DespawnOnExit(AppState::Terrasse));

    toasts.write(ToastMsg(
        "LA TERRASSE. Ce n'était pas un refuge. Tiens bon !".into(),
    ));
}

fn tick_terrasse(time: Res<Time>, mut terrasse: ResMut<TerrasseState>) {
    terrasse.time += time.delta_secs();
}

fn terrasse_spawner(
    time: Res<Time>,
    mut commands: Commands,
    mut terrasse: ResMut<TerrasseState>,
    arena: Res<Arena>,
    mut toasts: MessageWriter<ToastMsg>,
    mut sfx: MessageWriter<crate::audio::PlaySfx>,
) {
    // Le rythme s'accélère avec le temps (GDD : « de plus en plus forts »).
    let interval = (1.6 - terrasse.time * 0.015).max(0.45);
    terrasse
        .spawn_timer
        .set_duration(std::time::Duration::from_secs_f32(interval));
    terrasse.spawn_timer.tick(time.delta());

    let mut rng = rand::rng();
    let scale = 1.0 + terrasse.time / 45.0;

    if terrasse.spawn_timer.just_finished() {
        let count = (1 + (terrasse.time / 25.0) as u32).min(4);
        for _ in 0..count {
            let kind = *ALL_KINDS.choose(&mut rng).unwrap_or(&EnemyKind::Fourmi);
            // Apparition sur les bords.
            let side = rng.random_range(0..4);
            let pos = match side {
                0 => Vec2::new(rng.random_range(-arena.half.x..arena.half.x), arena.half.y - 20.0),
                1 => Vec2::new(rng.random_range(-arena.half.x..arena.half.x), -arena.half.y + 20.0),
                2 => Vec2::new(arena.half.x - 20.0, rng.random_range(-arena.half.y..arena.half.y)),
                _ => Vec2::new(-arena.half.x + 20.0, rng.random_range(-arena.half.y..arena.half.y)),
            };
            let elite = rng.random_bool((terrasse.time as f64 / 600.0).clamp(0.0, 0.15));
            spawn_enemy(&mut commands, kind, pos, scale, scale.sqrt(), elite);
        }
    }

    // Un boss surprise de temps en temps, parce que la terrasse est cruelle.
    if terrasse.time >= terrasse.next_boss_at {
        terrasse.next_boss_at += 75.0;
        let biome = *ALL_BIOMES.choose(&mut rng).unwrap_or(&crate::biomes::Biome::Jardin);
        let boss = biome.boss();
        spawn_boss(
            &mut commands,
            boss,
            Vec2::new(0.0, arena.half.y - 60.0),
            1.0 + terrasse.time / 90.0,
        );
        sfx.write(crate::audio::PlaySfx(crate::audio::Sfx::boss(boss)));
        toasts.write(ToastMsg(format!("{} s'invite sur la terrasse !", boss.name())));
    }
}
