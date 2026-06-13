use bevy::prelude::*;
use super::components::{Mob, Health, Boss, WaveManager};
use rand::prelude::*;

const SPEED: f32 = 40.0;
const HP: i32 = 1;
const BOSS_HP: i32 = 50;

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
        
        // calculer vitesse : augmente a chaque vague (+20.0 par vague)
        let wave_speed = SPEED + (wave_manager.current_wave as f32 - 1.0) * 20.0;
        
        // spawn sbires : commun a toutes les vagues
        let spawn_count = rng.random_range(5..10);
        for _ in 0..spawn_count {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = rng.random_range(300.0..800.0);
            let x = angle.cos() * distance;
            let y = angle.sin() * distance;

            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0), // Rouge
                    custom_size: Some(Vec2::new(32.0, 32.0)),
                    ..Default::default()
                },
                Transform::from_xyz(x, y, 0.0),
                Mob { speed: wave_speed },
                Health { hp: HP },
            ));
        }

        // spawn boss : seulement a la vague 3
        if wave_manager.current_wave == 3 {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = 500.0;
            let x = angle.cos() * distance;
            let y = angle.sin() * distance;

            commands.spawn((
                Sprite {
                    color: Color::srgb(0.0, 0.0, 1.0), // Bleu
                    custom_size: Some(Vec2::new(64.0, 64.0)),
                    ..Default::default()
                },
                Transform::from_xyz(x, y, 0.0),
                Mob { speed: wave_speed * 0.5 }, // Le boss profite aussi de l'augmentation
                Health { hp: BOSS_HP },
                Boss,
            ));
            info!("VAGUE 3 : LE BOSS APPARAIT ! (vitesse sbires: {:.1})", wave_speed);
        } else {
            info!("VAGUE {} LANCEE (vitesse sbires: {:.1})", wave_manager.current_wave, wave_speed);
        }

        // passer a la vague suivante
        wave_manager.current_wave += 1;
    }
}