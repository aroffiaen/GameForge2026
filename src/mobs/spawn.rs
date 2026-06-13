use bevy::prelude::*;
use super::components::{Mob, WaveManager, Biome};
use crate::entities::ennemies::{def, EnemyKind};
use crate::common::{Health, Enemy, ContactDmg, BaseColor, AiKind, ShootCd, RoomState, GameState, Velocity, LungeState};
use rand::prelude::*;

// Fonction générique pour spawn un ennemi
pub fn spawn_enemy(
    commands: &mut Commands,
    kind: EnemyKind,
    pos: Vec2,
    hp_mult: f32,
    dmg_mult: f32,
) -> Entity {
    let stats = def(kind);
    let entity_id = commands.spawn((
        Sprite {
            color: stats.color,
            custom_size: Some(Vec2::new(stats.radius * 2.0, stats.radius * 2.0)),
            ..Default::default()
        },
        Transform::from_translation(pos.extend(0.0)),
        Mob { kind },
        Health { 
            hp: (stats.hp * hp_mult) as i32,
            max_hp: (stats.hp * hp_mult) as i32,
        },
        Enemy,
        Velocity(Vec2::ZERO),
        ContactDmg(stats.dmg * dmg_mult),
        crate::common::Radius(stats.radius),
        BaseColor(stats.color),
        crate::mobs::components::AiState::Idle,
    )).id();
    
    crate::entities::ui::spawn_health_bar(commands, entity_id, 40.0);

    if let AiKind::Ranged { shoot_cd, .. } = stats.ai {
        commands.entity(entity_id).insert(ShootCd(Timer::from_seconds(shoot_cd, TimerMode::Once)));
    }
    if let AiKind::Lunge = stats.ai {
        let mut active_timer = Timer::from_seconds(0.3, TimerMode::Once);
        active_timer.tick(std::time::Duration::from_secs(1)); 
        commands.entity(entity_id).insert(LungeState {
            cd: Timer::from_seconds(1.8, TimerMode::Once),
            active: active_timer,
            dir: Vec2::ZERO,
        });
    }

    entity_id
}

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
    let spawn_count = rng.random_range(5..=15); // entre 5 et 15 sbires
    
    info!("STATEUP : Salle de Combat (Vague {}) - Spawn de {} {:?}", wave_manager.current_wave, spawn_count, kind);

    for _ in 0..spawn_count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(300.0..800.0);
        let pos = Vec2::new(angle.cos() * distance, angle.sin() * distance);
        spawn_enemy(&mut commands, kind, pos, 1.0, 1.0);
    }
}

// Spawn du boss
pub fn setup_boss_room(
    mut commands: Commands,
    wave_manager: Res<WaveManager>,
) {
    let mut rng = rand::rng();

    // Déterminer quel boss spécialisé spawn selon le biome
    let boss_kind = match wave_manager.current_biome {
        Biome::Potager | Biome::Fraise | Biome::Boue => super::bosses::BossKind::Gromp,
        Biome::TerreSeche | Biome::Gravier => super::bosses::BossKind::Scorpion,
        Biome::Dalles | Biome::Terrasse => super::bosses::BossKind::Araignee,
    };
    
    info!("STATEUP : Salle de Boss [{:?}] - {} APPARAIT !", wave_manager.current_biome, boss_kind.name());

    // Spawn du Boss spécialisé
    super::bosses::spawn_boss_specialized(&mut commands, boss_kind, Vec2::new(0.0, 500.0), 1.0);

    // Spawn de quelques sbires d'accompagnement
    let possible_enemies = match wave_manager.current_biome {
        Biome::Potager | Biome::Fraise => vec![EnemyKind::Puceron, EnemyKind::Limace, EnemyKind::Escargot],
        Biome::TerreSeche | Biome::Gravier => vec![EnemyKind::Fourmi, EnemyKind::Scarabee, EnemyKind::Araignee],
        Biome::Boue => vec![EnemyKind::Moustique, EnemyKind::Limace, EnemyKind::Escargot],
        Biome::Dalles | Biome::Terrasse => vec![EnemyKind::Araignee, EnemyKind::Guepe, EnemyKind::Fourmi],
    };
    let minion_kind = possible_enemies.choose(&mut rng).copied().unwrap_or(EnemyKind::Puceron);
    let spawn_count = rng.random_range(3..=5); // quelques sbires

    for _ in 0..spawn_count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(300.0..600.0);
        let pos = Vec2::new(angle.cos() * distance, angle.sin() * distance);
        spawn_enemy(&mut commands, minion_kind, pos, 1.0, 1.0);
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

