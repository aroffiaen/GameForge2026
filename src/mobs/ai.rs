use bevy::prelude::*;
use super::components::Mob;
use crate::player::Player;
use crate::entities::ennemies::def;
use crate::common::{AiKind, Velocity, LungeState, Slowed};

pub fn mob_ai(
    time: Res<Time>, 
    mut query_mobs: Query<(Entity, &Mob, &Transform, &mut Velocity, Option<&mut LungeState>, Has<Slowed>), Without<Player>>,
    query_player: Query<&Transform, With<Player>>,
) {
    let Ok(player_tf) = query_player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_secs();

    // Calculer les evitements
    let mob_data: Vec<(Entity, Vec2, f32)> = query_mobs
        .iter()
        .map(|(entity, mob, transform, _, _, _)| {
            (entity, transform.translation.truncate(), def(mob.kind).radius)
        })
        .collect();

    for (entity, mob, tf, mut vel, mut lunge, slowed) in &mut query_mobs {
        let stats = def(mob.kind);
        let pos = tf.translation.truncate();
        let to_player = player_pos - pos;
        let dist = to_player.length();
        let dir = to_player.normalize_or_zero();
        
        let mut max_speed = stats.speed;
        if slowed {
            max_speed *= 0.45;
        }

        let mut is_lunging_now = false;

        let target_vel = match stats.ai {
            AiKind::Chase => dir * max_speed,
            AiKind::Lunge => {
                if let Some(mut lunge) = lunge {
                    lunge.cd.tick(time.delta());
                    lunge.active.tick(time.delta());
                    if !lunge.active.is_finished() {
                        is_lunging_now = true;
                        lunge.dir * max_speed * 2.6
                    } else if lunge.cd.is_finished() && dist < 220.0 {
                        lunge.dir = dir;
                        lunge.active.reset();
                        lunge.cd = Timer::from_seconds(1.8, TimerMode::Once);
                        is_lunging_now = true;
                        dir * max_speed * 2.6
                    } else {
                        dir * max_speed
                    }
                } else {
                    dir * max_speed
                }
            }
            AiKind::Ranged { min, max: band_max, .. } => {
                if dist < min {
                    -dir * max_speed
                } else if dist > band_max {
                    dir * max_speed
                } else {
                    // Strafe perpendiculaire pour rester vivant.
                    Vec2::new(-dir.y, dir.x) * max_speed * 0.6
                }
            }
        };

        // --- LOGIQUE DE SÉPARATION (Anti-chevauchement) ---
        let mut separation = Vec2::ZERO;

        for (_other_entity, other_pos, other_radius) in &mob_data {
            if entity == *_other_entity { continue; }

            let diff = pos - *other_pos;
            let distance_mobs = diff.length();
            let safe_dist = stats.radius + *other_radius + 10.0;

            if distance_mobs < safe_dist && distance_mobs > 0.0 {
                let strength = (safe_dist - distance_mobs).powi(2) / safe_dist;
                separation += diff.normalize() * strength;
            }
        }

        if !is_lunging_now {
            let player_safe_dist = stats.radius + 48.0 + 5.0; // Rayon du joueur à 48.0
            if dist < player_safe_dist && dist > 0.0 {
                let strength = (player_safe_dist - dist).powi(2) / player_safe_dist;
                separation -= dir * strength * 3.0; // dir pointe vers le joueur, on soustrait
            }
        }

        // Appliquer la séparation à la vélocité cible
        let final_target = target_vel + separation * 0.8;

        // Interpolation douce (smooth movement)
        vel.0 = vel.0.move_towards(final_target, 700.0 * dt);
    }
}
