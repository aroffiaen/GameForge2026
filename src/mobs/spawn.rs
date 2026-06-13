use bevy::prelude::*;
use super::components::{Mob, Boss, WaveManager, Biome};
use crate::entities::ennemies::{def, EnemyKind};
use crate::common::{Health, Enemy, ContactDmg, BaseColor, AiKind, ShootCd, RoomState};
use rand::prelude::*;

// Spawn d'une vague standard lors de l'entrée dans une salle de combat
pub fn setup_combat_room(
    mut commands: Commands,
    wave_manager: Res<WaveManager>,
) {
    let mut rng = rand::rng();

    let possible_enemies = match wave_manager.current_biome {
        Biome::Potager | Biome::Fraise => vec![EnemyKind::Puceron, EnemyKind::Limace, EnemyKind::Escargot],
        Biome::TerreSeche | Biome::Gravier => vec![EnemyKind::Fourmi, EnemyKind::Scarabee, EnemyKind::Araignee],
        Biome::Boue => vec![EnemyKind::Moustique, EnemyKind::Limace, EnemyKind::Escargot],
        Biome::Dalles | Biome::Terrasse => vec![EnemyKind::Araignee, EnemyKind::Guepe, EnemyKind::Fourmi],
    };

    let kind = possible_enemies.choose(&mut rng).copied().unwrap_or(EnemyKind::Puceron);
    let stats = def(kind);

    let spawn_count = rng.random_range(5..10) + wave_manager.current_wave; // augmente un peu avec les vagues
    
    info!("STATEUP : Salle de Combat (Vague {}) - Spawn de {} {:?}", wave_manager.current_wave, spawn_count, kind);

    for _ in 0..spawn_count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(300.0..800.0);
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;

        let mut entity_cmd = commands.spawn((
            Sprite {
                color: stats.color,
                custom_size: Some(Vec2::new(stats.radius * 2.0, stats.radius * 2.0)),
                ..Default::default()
            },
            Transform::from_xyz(x, y, 0.0),
            Mob { kind },
            Health { hp: stats.hp as i32 },
            Enemy,
            ContactDmg(stats.dmg),
            BaseColor(stats.color),
            crate::mobs::components::AiState::Idle,
        ));

        if let AiKind::Ranged { shoot_cd, .. } = stats.ai {
            entity_cmd.insert(ShootCd(Timer::from_seconds(shoot_cd, TimerMode::Once)));
        }
    }
}

// Spawn du boss
pub fn setup_boss_room(
    mut commands: Commands,
    wave_manager: Res<WaveManager>,
) {
    let boss_kind = match wave_manager.current_biome {
        Biome::Potager | Biome::Fraise | Biome::Boue => EnemyKind::Escargot,
        Biome::TerreSeche | Biome::Gravier => EnemyKind::Scarabee,
        Biome::Dalles | Biome::Terrasse => EnemyKind::Guepe,
    };
    let boss_stats = def(boss_kind);
    
    info!("STATEUP : Salle de Boss [{:?}] - {:?} APPARAIT !", wave_manager.current_biome, boss_kind);

    commands.spawn((
        Sprite {
            color: boss_stats.color,
            custom_size: Some(Vec2::new(boss_stats.radius * 4.0, boss_stats.radius * 4.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 500.0, 0.0),
        Mob { kind: boss_kind },
        Health { hp: (boss_stats.hp * 3.0) as i32 }, // Boss bien tanky
        Boss,
        Enemy,
        ContactDmg(boss_stats.dmg * 2.0),
        BaseColor(boss_stats.color),
        crate::mobs::components::AiState::Idle,
    ));
}

// Vérifie si la salle est terminée pour passer à la suivante
pub fn check_room_clear(
    mut wave_manager: ResMut<WaveManager>,
    query_mobs: Query<Entity, With<Enemy>>,
    current_state: Res<State<RoomState>>,
    mut next_state: ResMut<NextState<RoomState>>,
) {
    if query_mobs.is_empty() {
        if *current_state.get() == RoomState::Combat {
            wave_manager.current_wave += 1;
            if wave_manager.current_wave >= 5 {
                next_state.set(RoomState::Boss);
            } else {
                next_state.set(RoomState::Transition);
            }
        } else if *current_state.get() == RoomState::Boss {
            info!("Boss vaincu ! Salle au trésor...");
            next_state.set(RoomState::Treasure);
        }
    }
}

pub fn transition_delay(
    mut next_state: ResMut<NextState<RoomState>>,
) {
    next_state.set(RoomState::Combat);
}

// Logique temporaire de la salle au trésor / Choix de biome
pub fn setup_treasure_room(
    mut wave_manager: ResMut<WaveManager>,
    mut next_state: ResMut<NextState<RoomState>>,
) {
    let mut rng = rand::rng();
    
    // 1. Reset des vagues pour le prochain niveau
    wave_manager.current_wave = 1;

    // 2. Choix d'un nouveau biome au hasard (plus tard, le joueur choisira)
    let biomes = [
        Biome::Terrasse, Biome::Gravier, Biome::Boue, 
        Biome::TerreSeche, Biome::Potager, Biome::Fraise, Biome::Dalles
    ];
    let new_biome = *biomes.choose(&mut rng).unwrap();
    wave_manager.current_biome = new_biome;

    info!("Salle au trésor récupérée ! Nouveau biome sélectionné : {:?}", new_biome);

    // 3. Départ direct vers la prochaine salle (boucle de run)
    next_state.set(RoomState::Transition);
}

