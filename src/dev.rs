//! Bac à sable de dev (§ aide au test/dev).
//!
//! Depuis le **menu titre**, le **cabanon** ou l'écran de **mort**, la touche
//! **F3** ouvre une arène vide (une vraie `AppState::EnRun` avec
//! `RunState.sandbox = true`) où l'on fait apparaître chaque mob/boss, où l'on
//! modifie ses stats et ses augments à la volée — sans chrono, sans portes.
//!
//! Tout est piloté au clavier (cf. le panneau d'aide affiché en jeu).

use bevy::prelude::*;
use rand::prelude::*;

use crate::augments::{roll_offer, Augments};
use crate::boss::{spawn_boss, BossKind};
use crate::common::*;
use crate::enemies::{spawn_enemy, EnemyKind};
use crate::stats::{Stat, Stats};

/// Demande d'entrer en bac à sable au prochain démarrage de run (consommée par
/// `start_run` dans `rooms.rs`).
#[derive(Resource, Default)]
pub struct PendingSandbox(pub bool);

/// Mode invincible du bac à sable.
#[derive(Resource, Default)]
struct Godmode(bool);

/// Stat actuellement sélectionnée pour l'édition (index dans `Stat::ALL`).
#[derive(Resource, Default)]
struct StatCursor(usize);

/// Racine du panneau d'aide / d'état du bac à sable.
#[derive(Component)]
struct SandboxUi;

/// Le texte d'état (stats, godmode, augments) mis à jour chaque frame.
#[derive(Component)]
struct SandboxStatusText;

/// Les 6 mobs dans l'ordre des touches 1..6.
const MOB_KEYS: [EnemyKind; 6] = [
    EnemyKind::Fourmi,
    EnemyKind::Escargot,
    EnemyKind::Araignee,
    EnemyKind::Criquet,
    EnemyKind::Guepe,
    EnemyKind::Cigale,
];

/// Les 5 boss, cyclés par la touche 8.
const BOSS_CYCLE: [BossKind; 5] = [
    BossKind::Araignee,
    BossKind::Scorpion,
    BossKind::Gromp,
    BossKind::MegaLimace,
    BossKind::MillePattes,
];

pub struct DevPlugin;

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingSandbox>()
            .init_resource::<Godmode>()
            .init_resource::<StatCursor>()
            .add_systems(Update, enter_sandbox)
            .add_systems(
                Update,
                (sandbox_ui, sandbox_controls, sandbox_godmode, update_status)
                    .run_if(in_state(AppState::EnRun)),
            );
    }
}

/// F3 depuis n'importe quel état hors run : ouvre le bac à sable.
fn enter_sandbox(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut pending: ResMut<PendingSandbox>,
    mut next: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::F3) && *state.get() != AppState::EnRun {
        pending.0 = true;
        next.set(AppState::EnRun);
    }
}

/// Affiche le panneau d'aide une fois que la run de bac à sable est en place.
fn sandbox_ui(
    mut commands: Commands,
    run: Res<crate::rooms::RunState>,
    existing: Query<(), With<SandboxUi>>,
) {
    if !run.sandbox || !existing.is_empty() {
        return;
    }
    commands
        .spawn((
            SandboxUi,
            DespawnOnExit(AppState::EnRun),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(8.0),
                top: Val::Px(70.0),
                width: Val::Px(310.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.66)),
            GlobalZIndex(15),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("— BAC À SABLE (dev) —"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.4)),
            ));
            p.spawn((
                Text::new(HELP),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.82, 0.82, 0.78)),
            ));
            p.spawn((
                SandboxStatusText,
                Text::new(""),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.6, 1.0, 0.6)),
            ));
        });
}

const HELP: &str = "\
1-6 : spawn mob (Fourmi/Escargot/Araignée/Criquet/Guêpe/Cigale)
7 : élite (aléatoire)   8 : boss (cycle)
9 : vider les ennemis   0 : soin complet
↑/↓ : choisir une stat   ←/→ : -/+10 %
R : reset stats   G : godmode
V : +1 augment   C : vider les augments
Retour arrière : sortir vers le cabanon";

/// Position d'apparition : au-dessus du joueur, étalée un peu au hasard.
fn spawn_pos(player: &Query<&Transform, With<Player>>, arena: &Arena) -> Vec2 {
    let mut rng = rand::rng();
    let base = player
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);
    let p = base + Vec2::new(rng.random_range(-120.0..120.0), 150.0);
    p.clamp(-arena.half + Vec2::splat(20.0), arena.half - Vec2::splat(20.0))
}

#[allow(clippy::too_many_arguments)]
fn sandbox_controls(
    keys: Res<ButtonInput<KeyCode>>,
    run: Res<crate::rooms::RunState>,
    paused: Res<Paused>,
    arena: Res<Arena>,
    mut commands: Commands,
    mut statup: ResMut<Stats>,
    mut augments: ResMut<Augments>,
    mut godmode: ResMut<Godmode>,
    mut cursor: ResMut<StatCursor>,
    mut boss_idx: Local<usize>,
    mut toasts: MessageWriter<ToastMsg>,
    mut sfx: MessageWriter<crate::audio::PlaySfx>,
    mut next_state: ResMut<NextState<AppState>>,
    player: Query<&Transform, With<Player>>,
    mut player_hp: Query<&mut Health, With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
) {
    if !run.sandbox || paused.0 {
        return;
    }

    // --- Sortie vers le cabanon ---
    if keys.just_pressed(KeyCode::Backspace) {
        next_state.set(AppState::Cabanon);
        return;
    }

    // --- Spawn de mobs (1..6) ---
    for (i, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ]
    .into_iter()
    .enumerate()
    {
        if keys.just_pressed(key) {
            let kind = MOB_KEYS[i];
            spawn_enemy(&mut commands, kind, spawn_pos(&player, &arena), 1.0, 1.0, false);
        }
    }

    // --- Élite aléatoire ---
    if keys.just_pressed(KeyCode::Digit7) {
        let kind = *MOB_KEYS.choose(&mut rand::rng()).unwrap();
        spawn_enemy(&mut commands, kind, spawn_pos(&player, &arena), 1.0, 1.0, true);
        toasts.write(ToastMsg("Élite invoquée".into()));
    }

    // --- Boss (cycle) ---
    if keys.just_pressed(KeyCode::Digit8) {
        let boss = BOSS_CYCLE[*boss_idx % BOSS_CYCLE.len()];
        *boss_idx += 1;
        spawn_boss(&mut commands, boss, spawn_pos(&player, &arena), 1.0);
        sfx.write(crate::audio::PlaySfx(crate::audio::Sfx::boss(boss)));
        toasts.write(ToastMsg(format!("Boss : {}", boss.name())));
    }

    // --- Vider les ennemis ---
    if keys.just_pressed(KeyCode::Digit9) {
        for e in &enemies {
            commands.entity(e).despawn();
        }
        toasts.write(ToastMsg("Ennemis vidés".into()));
    }

    // --- Soin complet ---
    if keys.just_pressed(KeyCode::Digit0) {
        if let Ok(mut h) = player_hp.single_mut() {
            h.hp = h.max;
        }
    }

    // --- Sélection / édition de stat (flèches, libérées du déplacement) ---
    if keys.just_pressed(KeyCode::ArrowUp) {
        cursor.0 = (cursor.0 + Stat::ALL.len() - 1) % Stat::ALL.len();
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        cursor.0 = (cursor.0 + 1) % Stat::ALL.len();
    }
    let stat = Stat::ALL[cursor.0];
    if keys.just_pressed(KeyCode::ArrowRight) {
        statup.add(stat, 10.0);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        statup.add(stat, -10.0);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        statup.reset();
        toasts.write(ToastMsg("Stats remises à 100 %".into()));
    }

    // --- Godmode ---
    if keys.just_pressed(KeyCode::KeyG) {
        godmode.0 = !godmode.0;
        toasts.write(ToastMsg(format!(
            "Godmode {}",
            if godmode.0 { "ON" } else { "OFF" }
        )));
    }

    // --- Augments ---
    if keys.just_pressed(KeyCode::KeyV) {
        let offer = roll_offer(&augments, 1);
        if let Some(a) = offer.first().copied() {
            augments.0.push(a);
            toasts.write(ToastMsg(format!("Augment : {}", a.name())));
        } else {
            toasts.write(ToastMsg("Tous les augments sont déjà pris".into()));
        }
    }
    if keys.just_pressed(KeyCode::KeyC) {
        augments.0.clear();
        toasts.write(ToastMsg("Augments vidés".into()));
    }
}

/// Maintient le joueur à PV max tant que le godmode est actif.
fn sandbox_godmode(
    run: Res<crate::rooms::RunState>,
    godmode: Res<Godmode>,
    mut player: Query<&mut Health, With<Player>>,
) {
    if !run.sandbox || !godmode.0 {
        return;
    }
    if let Ok(mut h) = player.single_mut() {
        h.hp = h.max;
    }
}

/// Met à jour le texte d'état (stats courantes, stat sélectionnée, godmode…).
fn update_status(
    run: Res<crate::rooms::RunState>,
    statup: Res<Stats>,
    augments: Res<Augments>,
    godmode: Res<Godmode>,
    cursor: Res<StatCursor>,
    mut text: Query<&mut Text, With<SandboxStatusText>>,
) {
    if !run.sandbox {
        return;
    }
    let Ok(mut text) = text.single_mut() else { return };
    let mut s = String::new();
    for (i, stat) in Stat::ALL.iter().enumerate() {
        let marker = if i == cursor.0 { '>' } else { ' ' };
        s.push_str(&format!(
            "{} {:<8} {:>3.0}%\n",
            marker,
            stat.label(),
            statup.get(*stat)
        ));
    }
    s.push_str(&format!(
        "Godmode: {}   Augments: {}",
        if godmode.0 { "ON" } else { "OFF" },
        augments.0.len()
    ));
    text.0 = s;
}
