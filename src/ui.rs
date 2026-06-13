//! HUD & écrans : minimaliste, lisible en plein chaos (GDD §13).

use bevy::prelude::*;

use crate::augments::{Augment, Augments};
use crate::common::*;
use crate::meta::MetaSave;
use crate::player::{Dash, SpeedInfo};
use crate::rooms::{RoomKind, RunState};
use crate::stats::{Stat, Stats};
use crate::terrasse::TerrasseState;
use crate::weapons::{def as weapon_def, Loadout, WeaponCds};

// ---------------------------------------------------------------------------
// Marqueurs
// ---------------------------------------------------------------------------

#[derive(Component)]
struct HpFill;
#[derive(Component)]
struct HpLabel;
#[derive(Component)]
struct PattesLabel;
#[derive(Component)]
struct WeaponsLabel;
#[derive(Component)]
struct SpeedLabel;
#[derive(Component)]
struct CenterLabel;
#[derive(Component)]
struct BossBarRoot;
#[derive(Component)]
struct BossBarFill;
#[derive(Component)]
struct BossBarName;
#[derive(Component)]
struct StatsPanel;
#[derive(Component)]
struct ChronoLabel;
#[derive(Component)]
struct PauseUi;
#[derive(Component)]
struct Toast {
    timer: Timer,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::EnRun), build_hud_run)
            .add_systems(OnEnter(AppState::Terrasse), build_hud_terrasse)
            .add_systems(OnEnter(AppState::GameOver), (build_game_over, reset_pause))
            .add_systems(
                Update,
                (
                    update_hp,
                    update_pattes,
                    update_weapons,
                    update_speed,
                    update_center,
                    update_boss_bar,
                    update_stats_panel,
                    update_chrono,
                )
                    .run_if(in_state(AppState::EnRun).or(in_state(AppState::Terrasse))),
            )
            .add_systems(
                Update,
                pause_system.run_if(in_state(AppState::EnRun).or(in_state(AppState::Terrasse))),
            )
            .add_systems(Update, (spawn_toasts, update_toasts))
            .add_systems(
                Update,
                game_over_input.run_if(in_state(AppState::GameOver)),
            );
    }
}

// ---------------------------------------------------------------------------
// Construction du HUD
// ---------------------------------------------------------------------------

fn build_hud_run(mut commands: Commands) {
    build_hud(&mut commands, AppState::EnRun);
}

fn build_hud_terrasse(mut commands: Commands) {
    build_hud(&mut commands, AppState::Terrasse);
}

fn build_hud(commands: &mut Commands, state: AppState) {
    // Bloc haut-gauche : PV, pattes, armes.
    commands
        .spawn((
            DespawnOnExit(state),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            GlobalZIndex(10),
        ))
        .with_children(|p| {
            // Barre de PV.
            p.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(18.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.85)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    HpFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.85, 0.25, 0.3)),
                ));
            });
            p.spawn((
                HpLabel,
                Text::new(""),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
            p.spawn((
                PattesLabel,
                Text::new(""),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.95, 0.85, 0.5)),
            ));
            p.spawn((
                WeaponsLabel,
                Text::new(""),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.85, 0.9)),
            ));
        });

    // Haut-droite : LE multiplicateur de vitesse (la mécanique signature).
    commands
        .spawn((
            DespawnOnExit(state),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            GlobalZIndex(10),
        ))
        .with_children(|p| {
            p.spawn((
                SpeedLabel,
                Text::new("×1.0"),
                TextFont { font_size: 34.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });

    // Haut-centre : salle / chrono + barre de boss.
    commands
        .spawn((
            DespawnOnExit(state),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },
            GlobalZIndex(10),
        ))
        .with_children(|p| {
            p.spawn((
                CenterLabel,
                Text::new(""),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.8)),
            ));
            p.spawn((
                ChronoLabel,
                Text::new(""),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::srgb(0.6, 1.0, 0.6)),
            ));
            p.spawn((
                BossBarRoot,
                Node {
                    width: Val::Px(420.0),
                    height: Val::Px(14.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.85)),
                Visibility::Hidden,
            ))
            .with_children(|bar| {
                bar.spawn((
                    BossBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8, 0.2, 0.5)),
                ));
            });
            p.spawn((
                BossBarName,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.9, 0.6, 0.8)),
            ));
        });

    // Bas-gauche : panneau des 7 stats-up (GDD §3.4).
    commands
        .spawn((
            DespawnOnExit(state),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(12.0),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.6)),
            GlobalZIndex(10),
        ))
        .with_children(|p| {
            p.spawn((
                StatsPanel,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.8, 0.9, 0.85)),
            ));
        });
}

// ---------------------------------------------------------------------------
// Mises à jour du HUD
// ---------------------------------------------------------------------------

fn update_hp(
    player: Query<&Health, With<Player>>,
    mut fill: Query<&mut Node, With<HpFill>>,
    mut label: Query<&mut Text, With<HpLabel>>,
) {
    let Ok(health) = player.single() else { return };
    if let Ok(mut node) = fill.single_mut() {
        node.width = Val::Percent(health.ratio() * 100.0);
    }
    if let Ok(mut text) = label.single_mut() {
        text.0 = format!("PV {:.0}/{:.0}", health.hp.max(0.0), health.max);
    }
}

fn update_pattes(meta: Res<MetaSave>, stats: Res<RunStats>, mut q: Query<&mut Text, With<PattesLabel>>) {
    if let Ok(mut text) = q.single_mut() {
        text.0 = format!("Pattes : {}  (+{} cette run)", meta.pattes, stats.pattes);
    }
}

fn update_weapons(
    loadout: Res<Loadout>,
    cds: Res<WeaponCds>,
    dash: Query<&Dash, With<Player>>,
    mut q: Query<&mut Text, With<WeaponsLabel>>,
) {
    let Ok(mut text) = q.single_mut() else { return };
    let slot = |i: usize| match loadout.0[i] {
        Some(w) => {
            let d = weapon_def(w);
            if cds.0[i] > 0.05 {
                format!("{} ({:.1}s)", d.name, cds.0[i])
            } else {
                d.name.to_string()
            }
        }
        None => "—".to_string(),
    };
    let dash_txt = dash
        .single()
        .map(|d| "◆".repeat(d.charges as usize))
        .unwrap_or_default();
    text.0 = format!("G: {}   D: {}   Dash: {}", slot(0), slot(1), dash_txt);
}

fn update_speed(
    info: Res<SpeedInfo>,
    augments: Res<Augments>,
    mut q: Query<(&mut Text, &mut TextColor), With<SpeedLabel>>,
) {
    let Ok((mut text, mut color)) = q.single_mut() else { return };
    // Sans l'augment « Élan », la vitesse ne donne plus de dégâts → rien à montrer.
    if augments.has(Augment::Elan) {
        text.0 = format!("Élan ×{:.1}", info.mult);
        let cold = Color::srgb(0.75, 0.75, 0.75);
        let hot = Color::srgb(1.0, 0.45, 0.1);
        color.0 = cold.mix(&hot, info.ratio);
    } else {
        text.0.clear();
    }
}

fn update_center(
    state: Res<State<AppState>>,
    run: Res<RunState>,
    terrasse: Res<TerrasseState>,
    stats: Res<RunStats>,
    mut q: Query<&mut Text, With<CenterLabel>>,
) {
    let Ok(mut text) = q.single_mut() else { return };
    text.0 = match state.get() {
        AppState::Terrasse => format!("TERRASSE — {:.1} s — {} kills", terrasse.time, stats.kills),
        AppState::EnRun => {
            let room = match run.room_kind {
                RoomKind::Boss => match run.gauntlet {
                    Some(4) => "BOSS".to_string(),
                    Some(w) => format!("vague {}/3", w),
                    None => "boss".to_string(),
                },
                RoomKind::Elite => "élite".to_string(),
                RoomKind::Combat => format!("salle {}/{}", run.room_index + 1, run.rooms_in_biome),
            };
            format!(
                "Biome {}/5 : {} — {}",
                run.biome_index + 1,
                run.biome.name(),
                room
            )
        }
        _ => String::new(),
    };
}

fn update_boss_bar(
    run: Res<RunState>,
    state: Res<State<AppState>>,
    boss: Query<&Health, With<BossTag>>,
    mut root: Query<&mut Visibility, With<BossBarRoot>>,
    mut fill: Query<&mut Node, With<BossBarFill>>,
    mut name: Query<&mut Text, With<BossBarName>>,
) {
    let Ok(mut visibility) = root.single_mut() else { return };
    match boss.single() {
        Ok(health) => {
            *visibility = Visibility::Visible;
            if let Ok(mut node) = fill.single_mut() {
                node.width = Val::Percent(health.ratio() * 100.0);
            }
            if let Ok(mut text) = name.single_mut() {
                let kind = if *state.get() == AppState::EnRun {
                    run.biome.boss().name()
                } else {
                    "Un boss"
                };
                text.0 = kind.to_string();
            }
        }
        Err(_) => {
            *visibility = Visibility::Hidden;
            if let Ok(mut text) = name.single_mut() {
                text.0 = String::new();
            }
        }
    }
}

/// Chrono de la salle courante (GDD §3.4) : vert tant qu'on est dans les temps,
/// rouge si on dépasse la cible. Vide hors salle chronométrée.
fn update_chrono(run: Res<RunState>, mut q: Query<(&mut Text, &mut TextColor), With<ChronoLabel>>) {
    let Ok((mut text, mut color)) = q.single_mut() else { return };
    if run.chrono_active {
        let bet = run.bet.map(|s| s.label()).unwrap_or("");
        text.0 = format!("⏱ {:.1}s / {:.1}s  ({bet})", run.chrono_elapsed, run.chrono_target);
        color.0 = if run.chrono_elapsed <= run.chrono_target {
            Color::srgb(0.5, 1.0, 0.5)
        } else {
            Color::srgb(1.0, 0.4, 0.3)
        };
    } else {
        text.0.clear();
    }
}

/// Affiche les 7 stats-up (%) sur deux lignes (GDD §3.4).
fn update_stats_panel(statup: Res<Stats>, mut q: Query<&mut Text, With<StatsPanel>>) {
    let Ok(mut text) = q.single_mut() else { return };
    let cell = |s: Stat| format!("{} {:.0}%", s.label(), statup.get(s));
    text.0 = format!(
        "{}   {}   {}   {}\n{}   {}   {}",
        cell(Stat::Pv),
        cell(Stat::Regen),
        cell(Stat::Dmg),
        cell(Stat::Resistance),
        cell(Stat::MoveSpeed),
        cell(Stat::AttackSpeed),
        cell(Stat::DashCd),
    );
}

// ---------------------------------------------------------------------------
// Toasts
// ---------------------------------------------------------------------------

fn spawn_toasts(
    mut msgs: MessageReader<ToastMsg>,
    mut commands: Commands,
    existing: Query<(), With<Toast>>,
) {
    let mut offset = existing.iter().count();
    for msg in msgs.read() {
        commands.spawn((
            Toast {
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            },
            Text::new(msg.0.clone()),
            TextFont { font_size: 17.0, ..default() },
            TextColor(Color::srgb(1.0, 0.95, 0.75)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(64.0 + offset as f32 * 26.0),
                justify_self: JustifySelf::Center,
                ..default()
            },
            GlobalZIndex(30),
        ));
        offset += 1;
    }
}

fn update_toasts(
    time: Res<Time>,
    mut commands: Commands,
    mut toasts: Query<(Entity, &mut Toast, &mut TextColor)>,
) {
    for (e, mut toast, mut color) in &mut toasts {
        toast.timer.tick(time.delta());
        let left = 1.0 - toast.timer.fraction();
        color.0 = color.0.with_alpha((left * 3.0).min(1.0));
        if toast.timer.is_finished() {
            commands.entity(e).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Pause
// ---------------------------------------------------------------------------

fn pause_system(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    phase: Res<State<RunPhase>>,
    mut paused: ResMut<Paused>,
    mut commands: Commands,
    pause_ui: Query<Entity, With<PauseUi>>,
) {
    // Esc ne met en pause que pendant le combat, pas dans les menus.
    let pausable = match state.get() {
        AppState::Terrasse => true,
        AppState::EnRun => matches!(phase.get(), RunPhase::Fighting | RunPhase::DoorOpen),
        _ => false,
    };
    if !pausable || !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    paused.0 = !paused.0;
    if paused.0 {
        commands
            .spawn((
                PauseUi,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                GlobalZIndex(40),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("PAUSE"),
                    TextFont { font_size: 40.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                p.spawn((
                    Text::new("Le jardin t'attend. (Échap pour reprendre)"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
            });
    } else {
        for e in &pause_ui {
            commands.entity(e).despawn();
        }
    }
}

fn reset_pause(mut paused: ResMut<Paused>, mut commands: Commands, q: Query<Entity, With<PauseUi>>) {
    paused.0 = false;
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------------------------------------------------------------------
// Game over : l'excuse bidon (GDD §2.3)
// ---------------------------------------------------------------------------

fn build_game_over(mut commands: Commands, death: Res<DeathInfo>) {
    commands
        .spawn((
            DespawnOnExit(AppState::GameOver),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.05, 0.04)),
            GlobalZIndex(50),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("Tu te réveilles au cabanon."),
                TextFont { font_size: 34.0, ..default() },
                TextColor(Color::srgb(0.95, 0.9, 0.7)),
            ));
            p.spawn((
                Text::new(format!("« {} »", death.excuse)),
                TextFont { font_size: 19.0, ..default() },
                TextColor(Color::srgb(0.7, 0.85, 0.6)),
            ));
            let mut stats_line = format!(
                "Insectes dézingués : {}   ·   Pattes ramassées : {}   ·   Temps : {:.0} s",
                death.kills, death.pattes, death.time
            );
            if let Some(t) = death.terrasse_time {
                stats_line.push_str(&format!("\nTenue sur la terrasse : {t:.1} s"));
                if death.new_best {
                    stats_line.push_str("   — NOUVEAU RECORD !");
                }
            }
            p.spawn((
                Text::new(stats_line),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
            p.spawn((
                Text::new("ESPACE — retourner au cabanon"),
                TextFont { font_size: 15.0, ..default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));
        });
}

fn game_over_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next.set(AppState::Cabanon);
    }
}
