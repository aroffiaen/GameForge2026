use bevy::prelude::*;
use super::components::{Mob, Health};
use crate::player::Player;

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

    // collecter positions : pour calculer les evitements
    let mob_positions: Vec<(Entity, Vec3)> = query_mobs
        .iter()
        .map(|(entity, _, transform)| (entity, transform.translation))
        .collect();

    // iterer mobs : boucle sur chaque ennemi mobile
    for (entity, mob, mut transform) in query_mobs.iter_mut() {
        let position = transform.translation;
        let mut direction = target_pos - position;

        // force de separation : eviter de se marcher dessus
        let mut separation = Vec3::ZERO;
        for (other_entity, other_pos) in &mob_positions {
            if entity == *other_entity { continue; }
            
            let diff = position - *other_pos;
            let distance = diff.length();
            if distance < 45.0 && distance > 0.0 {
                // force exponentielle : plus ils sont proches, plus le rejet est violent
                let strength = (55.0 - distance).powi(2) / 55.0;
                separation += diff.normalize() * strength;
            }
        }

        // combiner mouvement : vers le joueur + evitement
        if direction.length() > 0.0 {
            direction = direction.normalize();
        }
        
        let final_move = direction + separation * 0.8; // coefficient augmenté pour éviter de coller
        
        if final_move.length() > 0.0 {
            let velocity = final_move.normalize() * mob.speed * time.delta_secs();
            transform.translation += velocity;
        }
    }
}

pub fn health_system(query: Query<&Health>) {
    for health in query.iter() {
        info!("Mob HP: {}", health.hp);
    }
}