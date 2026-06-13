//! GameForge2026 — roguelite d'action top-down sur le thème de la vitesse.
//! « Plus le personnage va vite, plus il inflige de dégâts. »
//!
//! Voir le README et docs/GDD.md pour la conception complète.

mod augments;
mod biomes;
mod boss;
mod cabanon;
mod common;
mod enemies;
mod meta;
mod player;
mod rooms;
mod terrasse;
mod ui;
mod weapons;

use bevy::prelude::*;

fn main() {
    // Taille de fenêtre réglable via GF_WIN (ex. GF_WIN=960x540). Sous le
    // rendu logiciel de WSLg, une fenêtre plus petite = beaucoup plus de FPS.
    let (w, h) = std::env::var("GF_WIN")
        .ok()
        .and_then(|s| {
            let (a, b) = s.split_once('x')?;
            Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
        })
        .unwrap_or((1280u32, 720u32));

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "GameForge2026 — le jardin d'en bas".into(),
            resolution: (w, h).into(),
            ..default()
        }),
        ..default()
    }));

    // GF_FPS=1 : affiche le FPS dans la console (diagnostic perf).
    if std::env::var("GF_FPS").is_ok() {
        app.add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
        ));
    }

    app.insert_resource(ClearColor(Color::srgb(0.10, 0.08, 0.06)))
        .add_plugins((
            common::CorePlugin,
            meta::MetaPlugin,
            player::PlayerPlugin,
            weapons::WeaponsPlugin,
            enemies::EnemiesPlugin,
            boss::BossPlugin,
            rooms::RoomsPlugin,
            augments::AugmentsPlugin,
            cabanon::CabanonPlugin,
            terrasse::TerrassePlugin,
            ui::UiPlugin,
        ))
        .add_systems(Startup, setup_camera);

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
