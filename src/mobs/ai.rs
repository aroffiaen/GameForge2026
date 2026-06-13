use bevy::prelude::*;
use super::components::{Mob, Health};
<<<<<<< HEAD
use crate::player::Player;

pub fn mob_ai(
    time: Res<Time>, 
    mut query_mobs: Query<(&Mob, &mut Transform), Without<Player>>,
    query_player: Query<&Transform, With<Player>>,
) {
    // position cible : joueur si present, sinon centre (0,0)
    let mut target_pos = Vec3::ZERO;
    for player_transform in query_player.iter() {
        target_pos = player_transform.translation;
    }

    // iterer mobs : boucle sur chaque ennemi mobile
    for (mob, mut transform) in query_mobs.iter_mut() {
        let position = transform.translation;
        let direction = target_pos - position;

        let distance = direction.length();
        if distance > 10.0 { 
            let move_direction = direction.normalize();

            // appliquer mouvement : avance vers la cible
            transform.translation += move_direction * mob.speed * time.delta().as_secs_f32();
        }
=======

pub fn mob_ai(
    time: Res<Time>, 
    mut query: Query<(&Mob, &mut Transform)>
) {
    // iterer mobs : boucle sur chaque ennemi mobile
    for (mob, mut transform) in query.iter_mut() {
        // appliquer mouvement : multiplie vitesse par le temps ecoule
        transform.translation.x += mob.speed * time.delta().as_secs_f32();
        
        // afficher logs : verifie la coordonnee x en console
        info!("Mob X: {:.2}, speed: {}", transform.translation.x, mob.speed);
    }
}

pub fn health_system(query: Query<&Health>) {
    for health in query.iter() {
        info!("Mob HP: {}", health.hp);
>>>>>>> 6467fad (🏗️ feat: update mob AI and health system; replace Position with Transform)
    }
}

pub fn health_system(query: Query<&Health>) {
    for health in query.iter() {
        info!("Mob HP: {}", health.hp);
    }
}