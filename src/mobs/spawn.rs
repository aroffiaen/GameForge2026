use bevy::prelude::*;
use super::components::{Mob, Boss, WaveManager, Biome};
use crate::entities::ennemies::{def, EnemyKind};
use crate::common::{Health, Enemy, ContactDmg, BaseColor, AiKind, ShootCd, RoomState, GameState};
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

    let spawn_count = rng.random_range(5..=15); // entre 5 et 15 sbires
    
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
    let mut rng = rand::rng();

    let boss_kind = match wave_manager.current_biome {
        Biome::Potager | Biome::Fraise | Biome::Boue => EnemyKind::Escargot,
        Biome::TerreSeche | Biome::Gravier => EnemyKind::Scarabee,
        Biome::Dalles | Biome::Terrasse => EnemyKind::Guepe,
    };
    let boss_stats = def(boss_kind);
    
    info!("STATEUP : Salle de Boss [{:?}] - {:?} APPARAIT !", wave_manager.current_biome, boss_kind);

    // Spawn du Boss
    commands.spawn((
        Sprite {
            color: boss_stats.color,
            custom_size: Some(Vec2::new(boss_stats.radius * 4.0, boss_stats.radius * 4.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 500.0, 0.0),
        Mob { kind: boss_kind },
        Health { hp: (boss_stats.hp * 3.0) as i32 },
        Boss,
        Enemy,
        ContactDmg(boss_stats.dmg * 2.0),
        BaseColor(boss_stats.color),
        crate::mobs::components::AiState::Idle,
    ));

    // Spawn de quelques sbires d'accompagnement
    let possible_enemies = match wave_manager.current_biome {
        Biome::Potager | Biome::Fraise => vec![EnemyKind::Puceron, EnemyKind::Limace, EnemyKind::Escargot],
        Biome::TerreSeche | Biome::Gravier => vec![EnemyKind::Fourmi, EnemyKind::Scarabee, EnemyKind::Araignee],
        Biome::Boue => vec![EnemyKind::Moustique, EnemyKind::Limace, EnemyKind::Escargot],
        Biome::Dalles | Biome::Terrasse => vec![EnemyKind::Araignee, EnemyKind::Guepe, EnemyKind::Fourmi],
    };
    let minion_kind = possible_enemies.choose(&mut rng).copied().unwrap_or(EnemyKind::Puceron);
    let minion_stats = def(minion_kind);
    let spawn_count = rng.random_range(3..=5); // quelques sbires

    for _ in 0..spawn_count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(300.0..600.0);
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;

        let mut entity_cmd = commands.spawn((
            Sprite {
                color: minion_stats.color,
                custom_size: Some(Vec2::new(minion_stats.radius * 2.0, minion_stats.radius * 2.0)),
                ..Default::default()
            },
            Transform::from_xyz(x, y, 0.0),
            Mob { kind: minion_kind },
            Health { hp: minion_stats.hp as i32 },
            Enemy,
            ContactDmg(minion_stats.dmg),
            BaseColor(minion_stats.color),
            crate::mobs::components::AiState::Idle,
        ));

        if let AiKind::Ranged { shoot_cd, .. } = minion_stats.ai {
            entity_cmd.insert(ShootCd(Timer::from_seconds(shoot_cd, TimerMode::Once)));
        }
    }
}

// Vérifie si la salle est terminée pour passer à la suivante
pub fn check_room_clear(
    mut wave_manager: ResMut<WaveManager>,
    query_mobs: Query<Entity, With<Enemy>>,
    current_state: Res<State<RoomState>>,
    mut next_state: ResMut<NextState<RoomState>>,
    mut game_state: ResMut<NextState<GameState>>,
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
            wave_manager.biomes_cleared += 1;
            
            if wave_manager.biomes_cleared >= 5 {
                info!("VICTOIRE ! 5 biomes nettoyés.");
                game_state.set(GameState::Victory);
            } else {
                info!("Boss vaincu ! Passage au biome suivant...");
                next_state.set(RoomState::NextBiome);
            }
        }
    }
}

pub fn transition_delay(
    mut next_state: ResMut<NextState<RoomState>>,
) {
    next_state.set(RoomState::Combat);
}

// Logique pour passer au biome suivant
pub fn setup_next_biome(
    mut wave_manager: ResMut<WaveManager>,
    mut next_state: ResMut<NextState<RoomState>>,
) {
    let mut rng = rand::rng();
    
    // reset des vagues pour le prochain biome
    wave_manager.current_wave = 1;

    // choix d'un nouveau biome au hasard
    let biomes = [
        Biome::Terrasse, Biome::Gravier, Biome::Boue, 
        Biome::TerreSeche, Biome::Potager, Biome::Fraise, Biome::Dalles
    ];
    let new_biome = *biomes.choose(&mut rng).unwrap();
    wave_manager.current_biome = new_biome;

    info!("Progression : Biomes terminés = {}. Nouveau biome : {:?}", wave_manager.biomes_cleared, new_biome);

    // depart direct vers la prochaine salle
    next_state.set(RoomState::Transition);
}

