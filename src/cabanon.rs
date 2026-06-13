//! Le hub : le cabanon, le bousier et sa boule de pattes (GDD §11.1).

use bevy::prelude::*;
use rand::prelude::*;

use crate::common::*;
use crate::meta::{save_meta, tool_condition, tool_price, MetaSave};
use crate::player::{spawn_player, PlayerStats};
use crate::weapons::{def as weapon_def, Loadout, WeaponKind, ALL_WEAPONS};

const QUIPS: &[&str] = &[
    "Tes outils ? Quels outils ? Hé hé.",
    "Encore des pattes ! Ma boule sera MAGNIFIQUE.",
    "Rétréci ? Moi je te trouve très bien comme ça.",
    "Reviens avec des pattes. Beaucoup. De. Pattes.",
    "Un jour, ma boule de pattes terrifiera tout le jardin.",
    "Je ne suis pas un voleur, je suis un collectionneur.",
];

#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum HubOverlay {
    #[default]
    None,
    Shop,
    Loadout,
}

#[derive(Component)]
struct OverlayUi;

#[derive(Component)]
struct HintText;

#[derive(Component)]
struct HubPattesText;

#[derive(Clone, Copy, PartialEq, Eq)]
enum InteractKind {
    Bousier,
    Etabli,
    PorteJardin,
    PorteTerrasse,
}

#[derive(Component)]
struct Interactable {
    kind: InteractKind,
    radius: f32,
}

// ---------------------------------------------------------------------------
// Boutique : items à acheter
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Upgrade {
    Hp,
    Speed,
    Dash,
    Pattes,
}

#[derive(Clone)]
enum ShopAction {
    BuyTool(WeaponKind),
    BuyUpgrade(Upgrade),
}

struct ShopItem {
    label: String,
    price: u64,
    action: ShopAction,
    affordable: bool,
    blocked: Option<String>,
}

fn upgrade_info(u: Upgrade, meta: &MetaSave) -> (u8, &'static [u64], String) {
    match u {
        Upgrade::Hp => (meta.up_hp, &[40, 90, 160], "PV max +8".into()),
        Upgrade::Speed => (meta.up_speed, &[50, 110, 190], "Vitesse +5 %".into()),
        Upgrade::Dash => (meta.up_dash, &[80, 160], "Recharge du dash -10 %".into()),
        Upgrade::Pattes => (meta.up_pattes, &[60, 130], "Gain de pattes +15 %".into()),
    }
}

fn shop_items(meta: &MetaSave) -> Vec<ShopItem> {
    let mut items = Vec::new();
    // Outils récupérables (accomplissement validé → rachat, GDD §11.3).
    for w in ALL_WEAPONS {
        if meta.is_claimable(*w) {
            let price = tool_price(*w);
            let blocked = if meta.tools_bought_this_cycle >= 2 {
                Some("max 2 outils par run — reviens après une run".to_string())
            } else {
                None
            };
            items.push(ShopItem {
                label: format!("Récupérer : {}", weapon_def(*w).name),
                price,
                affordable: meta.pattes >= price,
                action: ShopAction::BuyTool(*w),
                blocked,
            });
        }
    }
    // Upgrades permanents — modérés, glass cannon oblige (GDD §11.4).
    for u in [Upgrade::Hp, Upgrade::Speed, Upgrade::Dash, Upgrade::Pattes] {
        let (rank, prices, label) = upgrade_info(u, meta);
        if (rank as usize) < prices.len() {
            let price = prices[rank as usize];
            items.push(ShopItem {
                label: format!("{} (rang {}/{})", label, rank + 1, prices.len()),
                price,
                affordable: meta.pattes >= price,
                action: ShopAction::BuyUpgrade(u),
                blocked: None,
            });
        }
    }
    items
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CabanonPlugin;

impl Plugin for CabanonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HubOverlay>()
            .add_systems(OnEnter(AppState::Cabanon), enter_cabanon)
            .add_systems(
                Update,
                (interact_system, refresh_overlay, overlay_input, update_hub_ui)
                    .run_if(in_state(AppState::Cabanon)),
            );
    }
}

fn enter_cabanon(
    mut commands: Commands,
    meta: Res<MetaSave>,
    sprites: Res<GameSprites>,
    mut loadout: ResMut<Loadout>,
    mut overlay: ResMut<HubOverlay>,
    mut arena: ResMut<Arena>,
    mut clear_color: ResMut<ClearColor>,
    mut augments: ResMut<crate::augments::Augments>,
    mut statup: ResMut<crate::stats::Stats>,
    mut run: ResMut<crate::rooms::RunState>,
) {
    *overlay = HubOverlay::None;
    augments.0.clear();
    statup.reset(); // pas de stat-up dans le hub (accès direct Terrasse = à nu)
    run.came_from_run = false;
    arena.half = Vec2::new(280.0, 160.0);
    clear_color.0 = Color::srgb(0.10, 0.08, 0.06);

    // Le loadout ne garde que des armes encore débloquées.
    for slot in loadout.0.iter_mut() {
        if let Some(w) = slot {
            if !meta.is_unlocked(*w) {
                *slot = None;
            }
        }
    }
    if loadout.0[0].is_none() && loadout.0[1].is_none() {
        loadout.0[0] = Some(WeaponKind::Poings);
    }

    let stats = PlayerStats::compute(&meta, &crate::augments::Augments::default(), &statup);
    let player = spawn_player(&mut commands, &sprites, &stats, Vec2::new(-60.0, -60.0));
    commands
        .entity(player)
        .insert(DespawnOnExit(AppState::Cabanon));

    // --- Décor du cabanon ---
    let spawn_scene = |commands: &mut Commands| {
        // Plancher.
        commands.spawn((
            DespawnOnExit(AppState::Cabanon),
            Sprite::from_color(Color::srgb(0.32, 0.22, 0.13), arena.half * 2.0 + Vec2::splat(8.0)),
            Transform::from_xyz(0.0, 0.0, -10.0),
        ));
        // Murs en planches.
        let wall = Color::srgb(0.2, 0.13, 0.08);
        let t = 14.0;
        for (pos, size) in [
            (Vec2::new(0.0, arena.half.y + t / 2.0), Vec2::new(arena.half.x * 2.0 + t * 2.0, t)),
            (Vec2::new(0.0, -arena.half.y - t / 2.0), Vec2::new(arena.half.x * 2.0 + t * 2.0, t)),
            (Vec2::new(arena.half.x + t / 2.0, 0.0), Vec2::new(t, arena.half.y * 2.0)),
            (Vec2::new(-arena.half.x - t / 2.0, 0.0), Vec2::new(t, arena.half.y * 2.0)),
        ] {
            commands.spawn((
                DespawnOnExit(AppState::Cabanon),
                Sprite::from_color(wall, size),
                Transform::from_translation(pos.extend(5.0)),
            ));
        }
    };
    spawn_scene(&mut commands);

    let mut rng = rand::rng();
    let quip = QUIPS.choose(&mut rng).copied().unwrap_or(QUIPS[0]);

    // Le bousier (sprite : scarabée poussant sa boule de pattes).
    commands
        .spawn((
            DespawnOnExit(AppState::Cabanon),
            Interactable { kind: InteractKind::Bousier, radius: 60.0 },
            Sprite {
                image: sprites.bousier.clone(),
                custom_size: Some(Vec2::splat(52.0)),
                ..default()
            },
            Transform::from_xyz(150.0, 30.0, 4.0),
        ))
        .with_children(|p| {
            p.spawn((
                Text2d::new(format!("Le Bousier\n« {quip} »")),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.9, 0.8, 0.6)),
                Transform::from_xyz(0.0, 38.0, 1.0),
            ));
        });

    // L'établi (choix des 2 armes).
    commands
        .spawn((
            DespawnOnExit(AppState::Cabanon),
            Interactable { kind: InteractKind::Etabli, radius: 60.0 },
            Sprite::from_color(Color::srgb(0.5, 0.35, 0.2), Vec2::new(60.0, 24.0)),
            Transform::from_xyz(-180.0, 40.0, 4.0),
        ))
        .with_children(|p| {
            p.spawn((
                Text2d::new("Établi"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.9, 0.85, 0.7)),
                Transform::from_xyz(0.0, 24.0, 1.0),
            ));
        });

    // Porte vers le jardin.
    commands
        .spawn((
            DespawnOnExit(AppState::Cabanon),
            Interactable { kind: InteractKind::PorteJardin, radius: 60.0 },
            Sprite::from_color(Color::srgb(0.45, 0.6, 0.3), Vec2::new(70.0, 16.0)),
            Transform::from_xyz(0.0, arena.half.y - 4.0, 6.0),
        ))
        .with_children(|p| {
            p.spawn((
                Text2d::new("LE JARDIN"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 1.0, 0.6)),
                Transform::from_xyz(0.0, -22.0, 1.0),
            ));
        });

    // Porte vers la terrasse (verrouillée tant qu'on ne l'a pas atteinte).
    let terrasse_color = if meta.terrasse_unlocked {
        Color::srgb(0.7, 0.6, 0.8)
    } else {
        Color::srgb(0.3, 0.28, 0.32)
    };
    let terrasse_label = if meta.terrasse_unlocked {
        "LA TERRASSE"
    } else {
        "Terrasse (verrouillée)"
    };
    commands
        .spawn((
            DespawnOnExit(AppState::Cabanon),
            Interactable { kind: InteractKind::PorteTerrasse, radius: 60.0 },
            Sprite::from_color(terrasse_color, Vec2::new(16.0, 60.0)),
            Transform::from_xyz(arena.half.x - 4.0, -80.0, 6.0),
        ))
        .with_children(|p| {
            p.spawn((
                Text2d::new(terrasse_label),
                TextFont { font_size: 12.0, ..default() },
                TextColor(terrasse_color.mix(&Color::WHITE, 0.4)),
                Transform::from_xyz(-50.0, 0.0, 1.0),
            ));
        });

    // --- UI du hub ---
    commands
        .spawn((
            DespawnOnExit(AppState::Cabanon),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                HubPattesText,
                Text::new(""),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.95, 0.85, 0.5)),
            ));
            let best = if meta.best_terrasse > 0.0 {
                format!("Record terrasse : {:.1} s", meta.best_terrasse)
            } else {
                "Record terrasse : —".to_string()
            };
            p.spawn((
                Text::new(format!(
                    "{best}   Runs : {}   Réveils au cabanon : {}",
                    meta.runs, meta.deaths
                )),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });

    commands.spawn((
        DespawnOnExit(AppState::Cabanon),
        HintText,
        Text::new("ZQSD/WASD bouger · Espace dash · E interagir"),
        TextFont { font_size: 15.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(14.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// ---------------------------------------------------------------------------
// Interactions
// ---------------------------------------------------------------------------

fn interact_system(
    keys: Res<ButtonInput<KeyCode>>,
    meta: Res<MetaSave>,
    mut overlay: ResMut<HubOverlay>,
    mut toasts: MessageWriter<ToastMsg>,
    mut next: ResMut<NextState<AppState>>,
    interactables: Query<(&Transform, &Interactable)>,
    player: Query<&Transform, With<Player>>,
    mut hint: Query<&mut Text, With<HintText>>,
) {
    if *overlay != HubOverlay::None {
        return;
    }
    let Ok(player_tf) = player.single() else { return };
    let player_pos = player_tf.translation.truncate();

    let mut nearest: Option<InteractKind> = None;
    for (tf, inter) in &interactables {
        if tf.translation.truncate().distance(player_pos) <= inter.radius {
            nearest = Some(inter.kind);
            break;
        }
    }

    if let Ok(mut text) = hint.single_mut() {
        text.0 = match nearest {
            Some(InteractKind::Bousier) => "E — parler au bousier (boutique)".into(),
            Some(InteractKind::Etabli) => "E — choisir tes 2 outils".into(),
            Some(InteractKind::PorteJardin) => "E — partir en run dans le jardin".into(),
            Some(InteractKind::PorteTerrasse) => {
                if meta.terrasse_unlocked {
                    "E — défier la Terrasse (survie)".into()
                } else {
                    "Atteins-la d'abord par une run…".into()
                }
            }
            None => "ZQSD/WASD bouger · Espace dash · E interagir · Clic = armes".into(),
        };
    }

    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    match nearest {
        Some(InteractKind::Bousier) => *overlay = HubOverlay::Shop,
        Some(InteractKind::Etabli) => *overlay = HubOverlay::Loadout,
        Some(InteractKind::PorteJardin) => next.set(AppState::EnRun),
        Some(InteractKind::PorteTerrasse) => {
            if meta.terrasse_unlocked {
                next.set(AppState::Terrasse);
            } else {
                toasts.write(ToastMsg(
                    "Le bousier ricane : « La terrasse ? Faut la MÉRITER. »".into(),
                ));
            }
        }
        None => {}
    }
}

fn update_hub_ui(meta: Res<MetaSave>, mut q: Query<&mut Text, With<HubPattesText>>) {
    if let Ok(mut text) = q.single_mut() {
        text.0 = format!("Pattes : {}", meta.pattes);
    }
}

// ---------------------------------------------------------------------------
// Overlays (boutique / établi)
// ---------------------------------------------------------------------------

fn refresh_overlay(
    overlay: Res<HubOverlay>,
    meta: Res<MetaSave>,
    loadout: Res<Loadout>,
    mut commands: Commands,
    existing: Query<Entity, With<OverlayUi>>,
) {
    if !overlay.is_changed() {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    match *overlay {
        HubOverlay::None => {}
        HubOverlay::Shop => build_shop_ui(&mut commands, &meta),
        HubOverlay::Loadout => build_loadout_ui(&mut commands, &meta, &loadout),
    }
}

fn build_shop_ui(commands: &mut Commands, meta: &MetaSave) {
    let items = shop_items(meta);
    commands
        .spawn((
            OverlayUi,
            DespawnOnExit(AppState::Cabanon),
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            GlobalZIndex(20),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("— LE BOUSIER —"),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::srgb(0.95, 0.8, 0.5)),
            ));
            parent.spawn((
                Text::new(format!(
                    "Tes pattes : {}   ·   Outils récupérés ce cycle : {}/2",
                    meta.pattes, meta.tools_bought_this_cycle
                )),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.95, 0.85, 0.5)),
            ));
            if items.is_empty() {
                parent.spawn((
                    Text::new("« Rien à te vendre. Reviens avec des exploits ! »"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
            }
            for (i, item) in items.iter().enumerate() {
                let (color, note) = if let Some(reason) = &item.blocked {
                    (Color::srgb(0.5, 0.5, 0.5), format!(" — {reason}"))
                } else if !item.affordable {
                    (Color::srgb(0.6, 0.45, 0.45), " — pas assez de pattes".to_string())
                } else {
                    (Color::srgb(0.8, 1.0, 0.7), String::new())
                };
                parent.spawn((
                    Text::new(format!(
                        "[{}]  {} — {} pattes{}",
                        i + 1,
                        item.label,
                        item.price,
                        note
                    )),
                    TextFont { font_size: 17.0, ..default() },
                    TextColor(color),
                ));
            }
            // Conditions des outils encore verrouillés.
            let locked: Vec<String> = ALL_WEAPONS
                .iter()
                .filter(|w| !meta.is_unlocked(**w) && !meta.is_claimable(**w))
                .map(|w| format!("{} : {}", weapon_def(*w).name, tool_condition(*w)))
                .collect();
            if !locked.is_empty() {
                parent.spawn((
                    Text::new(format!("Encore en otage :\n{}", locked.join("\n"))),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgb(0.55, 0.55, 0.6)),
                ));
            }
            parent.spawn((
                Text::new("1-9 acheter · Échap fermer"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

fn build_loadout_ui(commands: &mut Commands, meta: &MetaSave, loadout: &Loadout) {
    commands
        .spawn((
            OverlayUi,
            DespawnOnExit(AppState::Cabanon),
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            GlobalZIndex(20),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("— L'ÉTABLI : 2 outils max —"),
                TextFont { font_size: 26.0, ..default() },
                TextColor(Color::srgb(0.7, 0.9, 1.0)),
            ));
            let slot_name = |s: Option<WeaponKind>| {
                s.map(|w| weapon_def(w).name).unwrap_or("—")
            };
            parent.spawn((
                Text::new(format!(
                    "Clic gauche : {}    ·    Clic droit : {}",
                    slot_name(loadout.0[0]),
                    slot_name(loadout.0[1])
                )),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.95, 0.95, 0.8)),
            ));
            let unlocked: Vec<WeaponKind> = ALL_WEAPONS
                .iter()
                .copied()
                .filter(|w| meta.is_unlocked(*w))
                .collect();
            for (i, w) in unlocked.iter().enumerate() {
                let d = weapon_def(*w);
                let equipped = loadout.0.contains(&Some(*w));
                let marker = if equipped { "■" } else { "□" };
                let color = if equipped {
                    Color::srgb(0.7, 1.0, 0.6)
                } else {
                    Color::srgb(0.8, 0.8, 0.8)
                };
                parent.spawn((
                    Text::new(format!("[{}] {} {} — {}", i + 1, marker, d.name, d.desc)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(color),
                ));
            }
            parent.spawn((
                Text::new("1-9 équiper/retirer · Échap fermer"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

const DIGITS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

fn overlay_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<HubOverlay>,
    mut meta: ResMut<MetaSave>,
    mut loadout: ResMut<Loadout>,
    mut toasts: MessageWriter<ToastMsg>,
) {
    if *overlay == HubOverlay::None {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        // On ne sort jamais sans rien : les poings reviennent d'office.
        if loadout.0[0].is_none() && loadout.0[1].is_none() {
            loadout.0[0] = Some(WeaponKind::Poings);
        }
        *overlay = HubOverlay::None;
        return;
    }
    let pressed = DIGITS.iter().position(|k| keys.just_pressed(*k));
    let Some(index) = pressed else { return };

    match *overlay {
        HubOverlay::Shop => {
            let items = shop_items(&meta);
            let Some(item) = items.get(index) else { return };
            if let Some(reason) = &item.blocked {
                toasts.write(ToastMsg(format!("Le bousier refuse : {reason}.")));
                return;
            }
            if !item.affordable {
                toasts.write(ToastMsg("Pas assez de pattes. Le bousier soupire.".into()));
                return;
            }
            meta.pattes -= item.price;
            match item.action {
                ShopAction::BuyTool(w) => {
                    meta.claimable.retain(|x| *x != w);
                    meta.unlocked.push(w);
                    meta.tools_bought_this_cycle += 1;
                    toasts.write(ToastMsg(format!(
                        "{} récupéré ! Passe à l'établi pour l'équiper.",
                        weapon_def(w).name
                    )));
                }
                ShopAction::BuyUpgrade(u) => {
                    match u {
                        Upgrade::Hp => meta.up_hp += 1,
                        Upgrade::Speed => meta.up_speed += 1,
                        Upgrade::Dash => meta.up_dash += 1,
                        Upgrade::Pattes => meta.up_pattes += 1,
                    }
                    toasts.write(ToastMsg("Upgrade acheté. Le bousier compte ses pattes.".into()));
                }
            }
            save_meta(&meta);
            // Forcer le rebuild de l'UI.
            *overlay = HubOverlay::Shop;
        }
        HubOverlay::Loadout => {
            let unlocked: Vec<WeaponKind> = ALL_WEAPONS
                .iter()
                .copied()
                .filter(|w| meta.is_unlocked(*w))
                .collect();
            let Some(w) = unlocked.get(index).copied() else { return };
            if let Some(slot) = loadout.0.iter_mut().find(|s| **s == Some(w)) {
                *slot = None;
            } else if let Some(slot) = loadout.0.iter_mut().find(|s| s.is_none()) {
                *slot = Some(w);
            } else {
                loadout.0[1] = Some(w);
            }
            // Forcer le rebuild de l'UI.
            *overlay = HubOverlay::Loadout;
        }
        HubOverlay::None => {}
    }
}
