//! Menu titre au lancement : Jouer / Options (remapping des touches) / Quitter.
//! L'utilisateur peut aussi y revenir depuis le cabanon (Échap) ou la pause.

use bevy::prelude::*;

use crate::common::*;
use crate::meta::{save_meta, KeybindsSave, MetaSave};

// ---------------------------------------------------------------------------
// Ressources & composants
// ---------------------------------------------------------------------------

/// Vue courante du menu titre.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuView {
    #[default]
    Title,
    Settings,
    Credits,
}

/// Action en cours de remappage (capture de la prochaine touche).
#[derive(Resource, Default)]
pub struct Rebinding(pub Option<Action>);

/// Racine de l'UI du menu (despawn au rebuild / à la sortie de l'état).
#[derive(Component)]
struct MenuRoot;

#[derive(Component, Clone, Copy)]
enum MenuButton {
    Play,
    Settings,
    Credits,
    Quit,
    Back,
    Rebind(Action),
}

const BTN_NORMAL: Color = Color::srgb(0.15, 0.20, 0.30);
const BTN_HOVER: Color = Color::srgb(0.24, 0.34, 0.50);
const BTN_PRESS: Color = Color::srgb(0.32, 0.52, 0.72);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuView>()
            .init_resource::<Rebinding>()
            .add_systems(OnEnter(AppState::Title), enter_title)
            .add_systems(
                Update,
                (menu_buttons, rebind_capture, button_hover).run_if(in_state(AppState::Title)),
            );
    }
}

fn enter_title(
    mut commands: Commands,
    mut view: ResMut<MenuView>,
    mut rebinding: ResMut<Rebinding>,
    kb: Res<Keybinds>,
) {
    *view = MenuView::Title;
    rebinding.0 = None;
    build_menu(&mut commands, MenuView::Title, &kb, None);
}

// ---------------------------------------------------------------------------
// Construction de l'UI
// ---------------------------------------------------------------------------

fn build_menu(commands: &mut Commands, view: MenuView, kb: &Keybinds, rebinding: Option<Action>) {
    commands
        .spawn((
            MenuRoot,
            DespawnOnExit(AppState::Title),
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
            GlobalZIndex(100),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("GameForge2026"),
                TextFont { font_size: 48.0, ..default() },
                TextColor(Color::srgb(0.7, 0.95, 0.6)),
            ));
            p.spawn((
                Text::new("le jardin d'en bas"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.6, 0.7, 0.5)),
                Node { margin: UiRect::bottom(Val::Px(18.0)), ..default() },
            ));

            // Petit fabricant de bouton sous forme de macro pour éviter les borrow conflicts
            macro_rules! btn {
                ($builder:expr, $kind:expr, $label:expr, $width:expr) => {
                    $builder.spawn((
                        $kind,
                        Button,
                        Node {
                            width: Val::Px($width),
                            padding: UiRect::all(Val::Px(12.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(BTN_NORMAL),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new($label.to_string()),
                            TextFont { font_size: 20.0, ..default() },
                            TextColor(Color::srgb(0.95, 0.95, 0.9)),
                        ));
                    });
                };
            }

            match view {
                MenuView::Title => {
                    btn!(p, MenuButton::Play, "JOUER", 300.0);
                    btn!(p, MenuButton::Settings, "OPTIONS", 300.0);
                    btn!(p, MenuButton::Credits, "CRÉDITS", 300.0);
                    btn!(p, MenuButton::Quit, "QUITTER", 300.0);
                }
                MenuView::Settings => {
                    btn!(p, MenuButton::Back, "← RETOUR", 300.0);
                    for action in Action::ALL {
                        let label = if rebinding == Some(action) {
                            format!("{} :  < presse une touche… >", action.label())
                        } else {
                            format!("{} :  {}", action.label(), key_label(action.get(kb)))
                        };
                        btn!(p, MenuButton::Rebind(action), label, 420.0);
                    }
                }
                MenuView::Credits => {
                    p.spawn((
                        Text::new("GameForge2026 — Équipe Reims"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.95, 0.6)),
                        Node { margin: UiRect::bottom(Val::Px(24.0)), ..default() },
                    ));
                    p.spawn((
                        Text::new("Game Design & Help Dev : Esteban\nGraphismes : Tangui, Astride\nDéveloppement : Abel, Gwenhaël\n\nCréé en 2026"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        Node { margin: UiRect::bottom(Val::Px(32.0)), ..default() },
                    ));
                    btn!(p, MenuButton::Back, "← RETOUR", 300.0);
                }
            }

            if view == MenuView::Settings {
                p.spawn((
                    Text::new("Clique une ligne puis presse la nouvelle touche · Échap annule"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    Node { margin: UiRect::top(Val::Px(10.0)), ..default() },
                ));
            }
        });
}

/// Despawn l'UI actuelle et la reconstruit (après un changement de vue / bind).
fn refresh(
    commands: &mut Commands,
    roots: &Query<Entity, With<MenuRoot>>,
    view: MenuView,
    kb: &Keybinds,
    rebinding: Option<Action>,
) {
    for e in roots {
        commands.entity(e).despawn();
    }
    build_menu(commands, view, kb, rebinding);
}

// ---------------------------------------------------------------------------
// Systèmes
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn menu_buttons(
    interactions: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    roots: Query<Entity, With<MenuRoot>>,
    mut commands: Commands,
    mut view: ResMut<MenuView>,
    mut rebinding: ResMut<Rebinding>,
    kb: Res<Keybinds>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *button {
            MenuButton::Play => next.set(AppState::Cabanon),
            MenuButton::Quit => {
                exit.write(AppExit::Success);
            }
            MenuButton::Settings => {
                *view = MenuView::Settings;
                rebinding.0 = None;
                refresh(&mut commands, &roots, *view, &kb, None);
            }
            MenuButton::Credits => {
                *view = MenuView::Credits;
                rebinding.0 = None;
                refresh(&mut commands, &roots, *view, &kb, None);
            }
            MenuButton::Back => {
                *view = MenuView::Title;
                rebinding.0 = None;
                refresh(&mut commands, &roots, *view, &kb, None);
            }
            MenuButton::Rebind(action) => {
                rebinding.0 = Some(action);
                refresh(&mut commands, &roots, *view, &kb, rebinding.0);
            }
        }
    }
}

/// En mode remappage : la prochaine touche bindable devient le nouveau bind.
fn rebind_capture(
    keys: Res<ButtonInput<KeyCode>>,
    roots: Query<Entity, With<MenuRoot>>,
    mut commands: Commands,
    mut rebinding: ResMut<Rebinding>,
    mut kb: ResMut<Keybinds>,
    mut meta: ResMut<MetaSave>,
    view: Res<MenuView>,
) {
    let Some(action) = rebinding.0 else { return };

    if keys.just_pressed(KeyCode::Escape) {
        rebinding.0 = None;
        refresh(&mut commands, &roots, *view, &kb, None);
        return;
    }
    for (key, _) in BINDABLE {
        if keys.just_pressed(*key) {
            action.set(&mut kb, *key);
            meta.keybinds = KeybindsSave::from_binds(&kb);
            save_meta(&meta);
            rebinding.0 = None;
            refresh(&mut commands, &roots, *view, &kb, None);
            return;
        }
    }
}

fn button_hover(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<MenuButton>)>,
) {
    for (interaction, mut bg) in &mut q {
        bg.0 = match interaction {
            Interaction::Pressed => BTN_PRESS,
            Interaction::Hovered => BTN_HOVER,
            Interaction::None => BTN_NORMAL,
        };
    }
}
