use bevy::prelude::*;
use super::components::{Mob, AiState};
use crate::player::Player;
use crate::entities::ennemies::def;
use crate::common::AiKind;

pub fn mob_ai(
    time: Res<Time>, 
    mut query_mobs: Query<(Entity, &Mob, &mut Transform, &mut AiState), Without<Player>>,
    query_player: Query<&Transform, With<Player>>,
) {
    let mut target_pos = Vec3::ZERO;

    for player_transform in query_player.iter() {
        target_pos = player_transform.translation;
    }

    let mob_data: Vec<(Entity, Vec3, f32)> = query_mobs
        .iter()
        .map(|(entity, mob, transform, _)| {
            (entity, transform.translation, def(mob.kind).radius)
        })
        .collect();

    for (entity, mob, mut transform, mut ai_state) in query_mobs.iter_mut() {
        let stats = def(mob.kind);
        let position = transform.translation;
        let diff_player = target_pos - position;
        let dist_player = diff_player.length();
        let mut direction = diff_player;

        let mut current_speed = stats.speed;
        let mut should_move = true;

        match stats.ai {
            AiKind::Chase => {
                // Comportement standard, direction pointe vers le joueur
            }
            AiKind::Ranged { min, max, .. } => {
                if dist_player < min {
                    direction = -diff_player; // Fuit
                } else if dist_player > max {
                    // Poursuit
                } else {
                    should_move = false; // Reste à distance pour tirer
                }
            }
            AiKind::Lunge => {
                let dash_dir = diff_player.normalize_or_zero();
                match *ai_state {
                    AiState::Idle => {
                        if dist_player < 150.0 {
                            *ai_state = AiState::Charging { 
                                timer: Timer::from_seconds(0.8, TimerMode::Once) 
                            };
                            should_move = false;
                        }
                    }
                    AiState::Charging { ref mut timer } => {
                        timer.tick(time.delta());
                        should_move = false;
                        if timer.just_finished() {
                            *ai_state = AiState::Lunging { 
                                timer: Timer::from_seconds(0.3, TimerMode::Once),
                                direction: dash_dir,
                            };
                        }
                    }
                    AiState::Lunging { ref mut timer, direction: lunge_dir } => {
                        timer.tick(time.delta());
                        direction = lunge_dir;
                        current_speed = stats.speed * 3.5;
                        
                        if timer.just_finished() {
                            *ai_state = AiState::Idle;
                        }
                    }
                }
            }
        }

        let mut separation = Vec3::ZERO;

        for (_other_entity, other_pos, _other_radius) in &mob_data {
            if entity == *_other_entity { continue; }

            let diff = position - *other_pos;
            let distance = diff.length();
            let safe_dist = 45.0;

            if distance < safe_dist && distance > 0.0 {
                let strength = (safe_dist - distance).powi(2) / safe_dist;
                separation += diff.normalize() * strength;
            }
        }

        if let AiState::Lunging { .. } = *ai_state {
            // Traverse partiellement le joueur en chargeant
        } else {
            // Distance de sécurité dynamique : rayon du mob + nouveau rayon du joueur (48.0) + petite marge
            let player_safe_dist = stats.radius + 48.0 + 5.0; 
            if dist_player < player_safe_dist && dist_player > 0.0 {
                let strength = (player_safe_dist - dist_player).powi(2) / player_safe_dist;
                // diff_player pointe VERS le joueur, on soustrait pour être REPOUSSÉ
                separation -= diff_player.normalize() * strength * 3.0; 
            }
        }

        if should_move {
            if direction.length() > 0.0 {
                direction = direction.normalize();
            }

            let final_move = direction + separation * 0.8;

            if final_move.length() > 0.0 {
                let velocity = final_move.normalize() * current_speed * time.delta_secs();
                transform.translation += velocity;
            }
        } else if separation.length() > 0.0 {
             let velocity = separation.normalize() * (stats.speed * 0.5) * time.delta_secs();
             transform.translation += velocity;
        }
    }
}
