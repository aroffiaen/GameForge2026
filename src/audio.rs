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
use crate::common::{
    AppState, DamageKind, DamageMsg, EnemyDied, Player, PlayerDied, RoomResultMsg,
};

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
    // Attaques de boss.
    GrompLick,
    PlopSlug,
    // Tirs des mobs à distance.
    GuepeShoot,
    CigaleShoot,
    // Combat (joueur).
    Hit,
    PlayerHurt,
    EnemyExplode,
    Dash,
    // Méta / récompenses.
    Pickup,
    StatUp,
    WeaponBought,
    BousierTalk,
    // Interface.
    Click,
    // Évènements globaux.
    BossDefeated,
    PlayerDeath,
    // Voix / jingles spéciaux.
    CarsOpening,
    WorldPremiere,
    Dave1,
    Dave2,
}

impl Sfx {
    /// Tous les sons, pour le préchargement.
    const ALL: [Sfx; 24] = [
        Sfx::BossAraignee,
        Sfx::BossScorpion,
        Sfx::BossGromp,
        Sfx::BossLimace,
        Sfx::BossMillePattes,
        Sfx::GrompLick,
        Sfx::PlopSlug,
        Sfx::GuepeShoot,
        Sfx::CigaleShoot,
        Sfx::Hit,
        Sfx::PlayerHurt,
        Sfx::EnemyExplode,
        Sfx::Dash,
        Sfx::Pickup,
        Sfx::StatUp,
        Sfx::WeaponBought,
        Sfx::BousierTalk,
        Sfx::Click,
        Sfx::BossDefeated,
        Sfx::PlayerDeath,
        Sfx::CarsOpening,
        Sfx::WorldPremiere,
        Sfx::Dave1,
        Sfx::Dave2,
    ];

    fn path(self) -> &'static str {
        match self {
            Sfx::BossAraignee => "sfx/Boss/araignee.wav",
            Sfx::BossScorpion => "sfx/Boss/scorpion.wav",
            Sfx::BossGromp => "sfx/Boss/crapaud.wav",
            Sfx::BossLimace => "sfx/Boss/limace.wav",
            Sfx::BossMillePattes => "sfx/Boss/mille-pattes.wav",
            Sfx::GrompLick => "sfx/Boss/lick.wav",
            Sfx::PlopSlug => "sfx/petitsfx/Plop - slug.wav",
            Sfx::GuepeShoot => "sfx/guèpe.wav",
            Sfx::CigaleShoot => "sfx/cigales.wav",
            Sfx::Hit => "sfx/petitsfx/Hit.wav",
            Sfx::PlayerHurt => "sfx/petitsfx/Damage.wav",
            Sfx::EnemyExplode => "sfx/petitsfx/Small explosion.wav",
            Sfx::Dash => "sfx/petitsfx/Dash.wav",
            Sfx::Pickup => "sfx/petitsfx/Collected!.wav",
            Sfx::StatUp => "sfx/petitsfx/Stat up.wav",
            Sfx::WeaponBought => "sfx/petitsfx/Weapon bought.wav",
            Sfx::BousierTalk => "sfx/bousier.wav",
            Sfx::Click => "sfx/petitsfx/Click.wav",
            Sfx::BossDefeated => "sfx/petitsfx/Finish boss room.wav",
            Sfx::PlayerDeath => "sfx/petitsfx/Game over.wav",
            Sfx::CarsOpening => "sfx/cars_opening.wav",
            Sfx::WorldPremiere => "sfx/world_premiere.wav",
            Sfx::Dave1 => "sfx/dave1.wav",
            Sfx::Dave2 => "sfx/dave2.wav",
        }
    }

    /// Volume linéaire (1.0 = niveau d'origine). Les cris de boss sont mis un
    /// peu en avant ; les sons fréquents (coups, tirs, clic) sont en retrait
    /// pour éviter la fatigue auditive.
    fn volume(self) -> f32 {
        match self {
            Sfx::Hit => 0.4,
            Sfx::GuepeShoot | Sfx::CigaleShoot | Sfx::EnemyExplode => 0.5,
            Sfx::Click | Sfx::Pickup => 0.55,
            Sfx::Dash | Sfx::PlayerHurt => 0.7,
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

    /// Vrai pour les cris de boss : ils sont longs, on doit pouvoir les couper
    /// si le joueur meurt avant la fin.
    fn is_boss_cry(self) -> bool {
        matches!(
            self,
            Sfx::BossAraignee
                | Sfx::BossScorpion
                | Sfx::BossGromp
                | Sfx::BossLimace
                | Sfx::BossMillePattes
        )
    }
}

/// Marque une entité audio jouant un cri de boss, pour pouvoir l'interrompre.
#[derive(Component)]
struct BossCry;

/// « Joue ce SFX une fois. »
#[derive(Message)]
pub struct PlaySfx(pub Sfx);

/// Handles forts gardés vivants pour que les assets restent résidents.
#[derive(Resource, Default)]
struct SfxBank(Vec<Handle<AudioSource>>);

/// Minuteur des répliques aléatoires du perso (dave1/dave2).
#[derive(Resource)]
struct ChatterTimer(Timer);

impl Default for ChatterTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(22.0, TimerMode::Repeating))
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySfx>()
            .init_resource::<SfxBank>()
            .init_resource::<ChatterTimer>()
            .add_systems(PreStartup, preload_sfx)
            .add_systems(Startup, play_opening)
            // `react_to_events` écrit des PlaySfx ; on l'enchaîne avant
            // `play_sfx` pour jouer le son dans la même frame. `stop_boss_cry`
            // coupe ensuite le cri du boss si le joueur vient de mourir.
            .add_systems(
                Update,
                (react_to_events, play_sfx, stop_boss_cry_on_death).chain(),
            )
            .add_systems(Update, character_chatter);
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
    cries: Query<Entity, With<BossCry>>,
) {
    for msg in msgs.read() {
        let sfx = msg.0;
        // Un nouveau cri de boss coupe le précédent : on ne superpose jamais
        // deux musiques de boss (apparition d'un nouveau boss).
        if sfx.is_boss_cry() {
            for e in &cries {
                commands.entity(e).despawn();
            }
        }
        let mut e = commands.spawn((
            AudioPlayer::new(asset_server.load(sfx.path())),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(sfx.volume())),
        ));
        if sfx.is_boss_cry() {
            e.insert(BossCry);
        }
    }
}

/// Coupe le cri du boss en cours si le joueur meurt (despawn = arrêt du son).
fn stop_boss_cry_on_death(
    mut commands: Commands,
    mut player_died: MessageReader<PlayerDied>,
    cries: Query<Entity, With<BossCry>>,
) {
    if player_died.read().next().is_some() {
        for e in &cries {
            commands.entity(e).despawn();
        }
    }
}

/// Traduit les messages de gameplay existants en sons. Pour les évènements
/// potentiellement nombreux dans la même frame (coups, mort d'ennemis), on ne
/// joue **qu'un** son par frame pour éviter l'empilement assourdissant.
fn react_to_events(
    mut deaths: MessageReader<EnemyDied>,
    mut player_died: MessageReader<PlayerDied>,
    mut damage: MessageReader<DamageMsg>,
    mut results: MessageReader<RoomResultMsg>,
    players: Query<(), With<Player>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    // Morts d'ennemis : petite explosion (mob) ou jingle de victoire (boss).
    let mut any_mob_died = false;
    for ev in deaths.read() {
        if ev.was_boss {
            sfx.write(PlaySfx(Sfx::BossDefeated));
        } else {
            any_mob_died = true;
        }
    }
    if any_mob_died {
        sfx.write(PlaySfx(Sfx::EnemyExplode));
    }

    // Dégâts : coup encaissé par le joueur (prioritaire) vs touche sur un mob.
    let mut player_hurt = false;
    let mut enemy_hit = false;
    for msg in damage.read() {
        if players.contains(msg.target) {
            player_hurt = true;
        } else if matches!(msg.kind, DamageKind::Hit) {
            // Les ticks de poison ne « claquent » pas ; seuls les coups francs.
            enemy_hit = true;
        }
    }
    if player_hurt {
        sfx.write(PlaySfx(Sfx::PlayerHurt));
    } else if enemy_hit {
        sfx.write(PlaySfx(Sfx::Hit));
    }

    // Stat-Up chronométré réussi (résultat « bon ») : petit jingle.
    for r in results.read() {
        if r.good {
            sfx.write(PlaySfx(Sfx::StatUp));
        }
    }

    // PlayerDied est émis une fois ; on déclenche le jingle de game over.
    if player_died.read().next().is_some() {
        sfx.write(PlaySfx(Sfx::PlayerDeath));
    }
}

/// Jingle d'ouverture du jeu (joué une fois au lancement).
fn play_opening(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(Sfx::CarsOpening.path())),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(Sfx::CarsOpening.volume())),
    ));
}

/// Le perso parle « comme ça » : de temps en temps, une réplique au hasard
/// (dave1/dave2) pendant qu'on joue (run / cabanon / terrasse, pas au menu ni
/// au game over).
fn character_chatter(
    time: Res<Time>,
    state: Res<State<AppState>>,
    mut timer: ResMut<ChatterTimer>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    if matches!(state.get(), AppState::Title | AppState::GameOver) {
        return;
    }
    if timer.0.tick(time.delta()).just_finished() {
        let line = if rand::random::<bool>() {
            Sfx::Dave1
        } else {
            Sfx::Dave2
        };
        sfx.write(PlaySfx(line));
    }
}
