use bevy::prelude::*;
use super::components::Mob;
use crate::player::Player;
use crate::entities::ennemies::def;

pub fn mob_ai(
    time: Res<Time>, 
    mut query_mobs: Query<(Entity, &Mob, &mut Transform), Without<Player>>,
    query_player: Query<&Transform, With<Player>>,
) {
    // position cible : joueur si present, sinon centre (0,0)
    let mut target_pos = Vec3::ZERO;

    for player_transform in query_player.iter() {
        target_pos = player_transform.translation;
    }

    // calculer les evitements
    let mob_data: Vec<(Entity, Vec3, f32)> = query_mobs
        .iter()
        .map(|(entity, mob, transform)| {
            (entity, transform.translation, def(mob.kind).radius)
        })
        .collect();

    // iterer mobs : boucle sur chaque ennemi mobile
    for (entity, mob, mut transform) in query_mobs.iter_mut() {
        let stats = def(mob.kind);
        let position = transform.translation;
        let mut direction = target_pos - position;

        // force de separation : eviter de se marcher dessus
        let mut separation = Vec3::ZERO;

        // eviter les autres mobs
        for (_other_entity, other_pos, _other_radius) in &mob_data {
            if entity == *_other_entity { continue; }

            let diff = position - *other_pos;
            let distance = diff.length();
            let safe_dist = 45.0; // Valeur fixe de test initial

            if distance < safe_dist && distance > 0.0 {
                let strength = (safe_dist - distance).powi(2) / safe_dist;
                separation += diff.normalize() * strength;
            }
        }

        // eviter le joueur (collision physique)
        let diff_player = position - target_pos;
        let dist_player = diff_player.length();
        let player_safe_dist = 45.0; // Valeur fixe de ton test initial

        if dist_player < player_safe_dist && dist_player > 0.0 {
            let strength = (player_safe_dist - dist_player).powi(2) / player_safe_dist;
            separation += diff_player.normalize() * strength * 2.0; // repulsion plus forte
        }


        // combiner mouvement : vers le joueur + evitement
        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        let final_move = direction + separation * 0.8;

        if final_move.length() > 0.0 {
            let velocity = final_move.normalize() * stats.speed * time.delta_secs();
            transform.translation += velocity;
        }
    }
}
