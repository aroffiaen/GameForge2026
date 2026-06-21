//! Main menu module for Bevy 0.18 using the observer-based event system

use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin, egui};

// ========== CONFIGURATION ==========

const BACKGROUND_IMAGE: &str = "ui/background.png";
const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 60.0;
const BUTTON_SPACING: f32 = 25.0;

// ========== STATES ==========

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Options,
}

// ========== EVENTS ==========

#[derive(Event, Clone, Debug)]
pub enum MenuEvent {
    Play,
    Options,
    Quit,
}

// ========== PLUGIN ==========

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_state::<GameState>()
            .add_observer(on_menu_event)
            .add_systems(OnEnter(GameState::Menu), setup_menu_background)
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(Update, draw_menu_ui.run_if(in_state(GameState::Menu)));
    }
}

// ========== COMPONENTS ==========

#[derive(Component)]
struct MenuBackground;

// ========== SETUP ==========

fn setup_menu_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        MenuBackground,
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(800.0, 600.0)),
            image: asset_server.load(BACKGROUND_IMAGE),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}

// ========== UI RENDERING ==========

fn draw_menu_ui(mut egui_context: Single<&mut EguiContext>, mut commands: Commands) {
    let ctx = egui_context.get_mut();

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                // Play button
                if ui
                    .button("Play")
                    .min_size(egui::vec2(BUTTON_WIDTH, BUTTON_HEIGHT))
                    .clicked()
                {
                    commands.trigger(MenuEvent::Play);
                }

                ui.add_space(BUTTON_SPACING);

                // Options button
                if ui
                    .button("Options")
                    .min_size(egui::vec2(BUTTON_WIDTH, BUTTON_HEIGHT))
                    .clicked()
                {
                    commands.trigger(MenuEvent::Options);
                }

                ui.add_space(BUTTON_SPACING);

                // Quit button
                if ui
                    .button("Quit")
                    .min_size(egui::vec2(BUTTON_WIDTH, BUTTON_HEIGHT))
                    .clicked()
                {
                    commands.trigger(MenuEvent::Quit);
                }
            });
        });
}

// ========== EVENT HANDLING (Observer Pattern) ==========

fn on_menu_event(On(menu_event): On<MenuEvent>, mut state: ResMut<NextState<GameState>>) {
    match menu_event {
        MenuEvent::Play => state.set(GameState::Playing),
        MenuEvent::Options => state.set(GameState::Options),
        MenuEvent::Quit => std::process::exit(0),
    }
}

// ========== CLEANUP ==========

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuBackground>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
