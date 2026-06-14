//! Système audio : effets sonores ponctuels (SFX).
//!
//! N'importe quel système peut demander un son en écrivant un message
//! [`PlaySfx`] (pas besoin d'`AssetServer`). Les handles sont préchargés au
//! démarrage ([`SfxBank`]) pour éviter un à-coup au premier déclenchement.
//!
//! Priorité du lot actuel : le cri d'apparition de chaque boss + la défaite du
//! boss + la mort du joueur. Les hooks réagissent aux messages de gameplay déjà
//! émis ([`EnemyDied`], [`PlayerDied`]), donc aucun autre système n'est touché.

use bevy::audio::{AudioSource, Volume};
use bevy::prelude::*;

use crate::boss::BossKind;
use crate::common::{EnemyDied, PlayerDied};

// ---------------------------------------------------------------------------
// Catalogue des sons
// ---------------------------------------------------------------------------

/// Un effet sonore du jeu. `path()` donne le fichier sous `assets/`.
#[derive(Clone, Copy)]
pub enum Sfx {
    // Cris d'apparition des boss (assets/sfx/Boss/).
    BossAraignee,
    BossScorpion,
    BossGromp,
    BossLimace,
    BossMillePattes,
    // Évènements globaux (assets/sfx/petitsfx/).
    BossDefeated,
    PlayerDeath,
}

impl Sfx {
    /// Tous les sons, pour le préchargement.
    const ALL: [Sfx; 7] = [
        Sfx::BossAraignee,
        Sfx::BossScorpion,
        Sfx::BossGromp,
        Sfx::BossLimace,
        Sfx::BossMillePattes,
        Sfx::BossDefeated,
        Sfx::PlayerDeath,
    ];

    fn path(self) -> &'static str {
        match self {
            Sfx::BossAraignee => "sfx/Boss/araignee.wav",
            Sfx::BossScorpion => "sfx/Boss/scorpion.wav",
            Sfx::BossGromp => "sfx/Boss/crapaud.wav",
            Sfx::BossLimace => "sfx/Boss/limace.wav",
            Sfx::BossMillePattes => "sfx/Boss/mille-pattes.wav",
            Sfx::BossDefeated => "sfx/petitsfx/Finish boss room.wav",
            Sfx::PlayerDeath => "sfx/petitsfx/Game over.wav",
        }
    }

    /// Volume linéaire (1.0 = niveau d'origine). Les cris de boss sont mis un
    /// peu en avant ; le reste légèrement en retrait.
    fn volume(self) -> f32 {
        match self {
            Sfx::PlayerDeath => 0.8,
            _ => 0.9,
        }
    }

    /// Cri d'apparition associé à un boss.
    pub fn boss(kind: BossKind) -> Self {
        match kind {
            BossKind::Araignee => Sfx::BossAraignee,
            BossKind::Scorpion => Sfx::BossScorpion,
            BossKind::Gromp => Sfx::BossGromp,
            BossKind::MegaLimace => Sfx::BossLimace,
            BossKind::MillePattes => Sfx::BossMillePattes,
        }
    }
}

/// « Joue ce SFX une fois. »
#[derive(Message)]
pub struct PlaySfx(pub Sfx);

/// Handles forts gardés vivants pour que les assets restent résidents.
#[derive(Resource, Default)]
struct SfxBank(Vec<Handle<AudioSource>>);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySfx>()
            .init_resource::<SfxBank>()
            .add_systems(PreStartup, preload_sfx)
            // `react_to_events` écrit des PlaySfx ; on l'enchaîne avant
            // `play_sfx` pour jouer le son dans la même frame.
            .add_systems(Update, (react_to_events, play_sfx).chain());
    }
}

/// Précharge tous les sons (garde des handles forts dans le bank).
fn preload_sfx(asset_server: Res<AssetServer>, mut bank: ResMut<SfxBank>) {
    bank.0 = Sfx::ALL.iter().map(|s| asset_server.load(s.path())).collect();
}

/// Joue chaque [`PlaySfx`] : spawn d'une entité audio one-shot auto-despawn.
fn play_sfx(
    mut commands: Commands,
    mut msgs: MessageReader<PlaySfx>,
    asset_server: Res<AssetServer>,
) {
    for msg in msgs.read() {
        let sfx = msg.0;
        commands.spawn((
            AudioPlayer::new(asset_server.load(sfx.path())),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(sfx.volume())),
        ));
    }
}

/// Traduit les messages de gameplay existants en sons.
fn react_to_events(
    mut deaths: MessageReader<EnemyDied>,
    mut player_died: MessageReader<PlayerDied>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    for ev in deaths.read() {
        if ev.was_boss {
            sfx.write(PlaySfx(Sfx::BossDefeated));
        }
    }
    // PlayerDied est émis une fois ; on déclenche le jingle de game over.
    if player_died.read().next().is_some() {
        sfx.write(PlaySfx(Sfx::PlayerDeath));
    }
}
