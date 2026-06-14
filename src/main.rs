//! Little Fast Gardener (LFG) — roguelite d'action top-down sur le thème de la vitesse.
//! « Plus le personnage va vite, plus il inflige de dégâts. »
//!
//! Voir le README et docs/GDD.md pour la conception complète.

mod augments;
mod biomes;
mod boss;
mod cabanon;
mod common;
mod enemies;
mod healthbar;
mod menu;
mod meta;
mod player;
mod rooms;
mod stats;
mod terrasse;
mod ui;
mod weapons;

use bevy::prelude::*;

/// Police embarquée dans le binaire. La police par défaut de Bevy
/// (FiraMono-subset) n'a ni accents ni symboles → tout le texte accentué et
/// les « × ◆ — » s'affichaient en carrés. DejaVu Sans (libre) couvre tout.
const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/DejaVuSans.ttf");

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
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Little Fast Gardener — le jardin d'en bas".into(),
                    resolution: (w, h).into(),
                    ..default()
                }),
                ..default()
            })
            // Pixel art : filtrage au plus proche (pas de flou).
            .set(ImagePlugin::default_nearest()),
    );

    // GF_FPS=1 : affiche le FPS dans la console (diagnostic perf).
    if std::env::var("GF_FPS").is_ok() {
        app.add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
        ));
    }

    // Sprites chargés dès la construction (avant tout système / OnEnter), pour
    // que la ressource existe quand le cabanon se construit au démarrage.
    let asset_server = app.world().resource::<AssetServer>().clone();
    app.insert_resource(common::GameSprites {
        // Ordre [R+L, L+R] pour le cycle de course (RL → rien → LR → rien).
        legs_walk: [
            asset_server.load("sprites/jardinier/R+L.png"),
            asset_server.load("sprites/jardinier/L+R.png"),
        ],
        body_idle: [
            asset_server.load("sprites/jardinier/premier-sprite-player.png"),
            asset_server.load("sprites/jardinier/second-sprite-player.png"),
        ],
        body_damage: asset_server.load("sprites/jardinier/damage.png"),
        body_dash: asset_server.load("sprites/jardinier/full-dash.png"),
        bousier: asset_server.load("sprites/bousier.png"),
        zones: common::ZoneTextures {
            sol_jardin_potager: asset_server.load("sprites/zones/Sol_Jardin_Potager.png"),
            sol_gravier: asset_server.load("sprites/zones/Sol_Gravier.png"),
            sol_boue: asset_server.load("sprites/zones/Sol_boue.png"),
            sol_seche: asset_server.load("sprites/zones/Sol_Seche.png"),
            mur_jardin_boue_gravier: asset_server
                .load("sprites/zones/Mur_Jardin_Boue_Gravier.png"),
            mur_potager: asset_server.load("sprites/zones/Mur_Potager.png"),
            mur_seche: asset_server.load("sprites/zones/Mur_Seche.png"),
            terrasse: asset_server.load("sprites/zones/Terrasse.png"),
        },
    });

    app.insert_resource(ClearColor(Color::srgb(0.10, 0.08, 0.06)))
        .add_plugins((
            common::CorePlugin,
            meta::MetaPlugin,
            menu::MenuPlugin,
            stats::StatsPlugin,
            player::PlayerPlugin,
            weapons::WeaponsPlugin,
            enemies::EnemiesPlugin,
            healthbar::HealthBarPlugin,
            boss::BossPlugin,
            rooms::RoomsPlugin,
            augments::AugmentsPlugin,
            cabanon::CabanonPlugin,
            terrasse::TerrassePlugin,
            ui::UiPlugin,
        ))
        .add_systems(Startup, (setup_camera, override_default_font));

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Remplace la police par défaut de Bevy par DejaVu Sans (accents + symboles).
fn override_default_font(mut fonts: ResMut<Assets<Font>>) {
    match Font::try_from_bytes(FONT_BYTES.to_vec()) {
        Ok(font) => {
            // La police par défaut est stockée derrière l'AssetId par défaut ;
            // on écrase cet asset pour que tout `TextFont::default()` l'utilise.
            let _ = fonts.insert(bevy::asset::AssetId::<Font>::default(), font);
        }
        Err(err) => error!("Police DejaVu illisible: {err:?}"),
    }
}
