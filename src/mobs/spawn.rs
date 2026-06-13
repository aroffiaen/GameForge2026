use bevy::prelude::*;
use super::components::{Mob, Boss, WaveManager, Biome};
use crate::entities::ennemies::{def, EnemyKind};
use crate::common::*;
use rand::prelude::*;

pub fn spawn_wave_system(
    mut commands: Commands,
    mut wave_manager: ResMut<WaveManager>,
    query_mobs: Query<Entity, (With<Mob>, Without<Boss>)>,
) {
    let mob_count_remaining = query_mobs.iter().count();

    // declencher vague : si presque plus de sbires et vague <= 3
    if mob_count_remaining <= 3 && wave_manager.current_wave <= 3 {
        let mut rng = rand::rng();

        // determiner liste ennemis : selon le biome actuel
        let possible_enemies = match wave_manager.current_biome {
            Biome::Potager | Biome::Fraise => vec![EnemyKind::Puceron, EnemyKind::Limace, EnemyKind::Escargot],
            Biome::TerreSeche | Biome::Gravier => vec![EnemyKind::Fourmi, EnemyKind::Scarabee, EnemyKind::Araignee],
            Biome::Boue => vec![EnemyKind::Moustique, EnemyKind::Limace, EnemyKind::Escargot],
            Biome::Dalles | Biome::Terrasse => vec![EnemyKind::Araignee, EnemyKind::Guepe, EnemyKind::Fourmi],
        };

        // choisir type : un seul type d'ennemi pour toute la vague
        let kind = possible_enemies.choose(&mut rng).copied().unwrap_or(EnemyKind::Puceron);
        let stats = def(kind);

        // spawn sbires : commun a toutes les vagues
        let spawn_count = rng.random_range(5..10);
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
        // spawn boss : seulement a la vague 3
        if wave_manager.current_wave == 3 {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = 500.0;
            let x = angle.cos() * distance;
            let y = angle.sin() * distance;

            // definir boss : ennemi le plus imposant du biome
            let boss_kind = match wave_manager.current_biome {
                Biome::Potager | Biome::Fraise | Biome::Boue => EnemyKind::Escargot,
                Biome::TerreSeche | Biome::Gravier => EnemyKind::Scarabee,
                Biome::Dalles | Biome::Terrasse => EnemyKind::Guepe,
            };
            let boss_stats = def(boss_kind);

            commands.spawn((
                Sprite {
                    color: boss_stats.color,
                    custom_size: Some(Vec2::new(boss_stats.radius * 4.0, boss_stats.radius * 4.0)), // taille double
                    ..Default::default()
                },
                Transform::from_xyz(x, y, 0.0),
                Mob { kind: boss_kind },
                Health { hp: (boss_stats.hp * 2.0) as i32 },
                Boss,
                Enemy,
                ContactDmg(boss_stats.dmg * 2.0),
                BaseColor(boss_stats.color),
                crate::mobs::components::AiState::Idle,
            ));
            info!("VAGUE 3 [{:?}] : LE BOSS {:?} APPARAIT !", wave_manager.current_biome, boss_kind);
        } else {
            info!("VAGUE {} [{:?}] LANCEE (Type: {:?})", wave_manager.current_wave, wave_manager.current_biome, kind);
        }

        // passer a la vague suivante
        wave_manager.current_wave += 1;
    }
}
