use bevy::prelude::*;
use super::components::{Mob, Boss, WaveManager};
use crate::entities::ennemies::{def, EnemyKind};
use crate::common::*;
use rand::prelude::*;

pub fn spawn_wave_system(
    mut commands: Commands,
    mut wave_manager: ResMut<WaveManager>,
    query_mobs: Query<Entity, (With<Mob>, Without<Boss>)>,
) {
    // compter sbires : on verifie combien il en reste
    let mob_count_remaining = query_mobs.iter().count();

    // declencher vague : si presque plus de sbires (<= 3) et qu'on a pas depassé la vague 3
    if mob_count_remaining <= 3 && wave_manager.current_wave <= 3 {
        let mut rng = rand::rng();

        // choix du type d'ennemi pour TOUTE la vague
        let kind = if rng.random_bool(0.5) { EnemyKind::Fourmi } else { EnemyKind::Puceron };
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

            let kind = EnemyKind::Scarabee;
            let stats = def(kind);

            commands.spawn((
                Sprite {
                    color: stats.color,
                    custom_size: Some(Vec2::new(stats.radius * 4.0, stats.radius * 4.0)), // 2x plus grand (diamètre standard * 2)
                    ..Default::default()
                },
                Transform::from_xyz(x, y, 0.0),
                Mob { kind },
                Health { hp: (stats.hp * 2.0) as i32 },
                Boss,
                Enemy,
                ContactDmg(stats.dmg * 2.0),
                BaseColor(stats.color),
            ));
            info!("VAGUE 3 : LE BOSS {:?} APPARAIT !", kind);
        } else {
            info!("VAGUE {} LANCEE", wave_manager.current_wave);
        }

        // passer a la vague suivante
        wave_manager.current_wave += 1;
    }
}
