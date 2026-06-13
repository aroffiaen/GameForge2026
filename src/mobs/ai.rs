use bevy::prelude::*;
use super::components::{Mob, Health};

pub fn mob_ai(
    time: Res<Time>, 
    mut query: Query<(&Mob, &mut Transform)>
) {
    // iterer mobs : boucle sur chaque ennemi mobile
    for (mob, mut transform) in query.iter_mut() {
        // calculer direction : vecteur vers le centre (0,0,0)
        let target = Vec3::ZERO;
        let position = transform.translation;
        let direction = target - position;

        // normaliser direction : evite les deplacements infinis ou nuls
        let distance = direction.length();
        if distance > 80.0 { // s'arreter un peu avant le centre exact
            let move_direction = direction.normalize();

            // appliquer mouvement : avance vers le centre selon la vitesse
            transform.translation += move_direction * mob.speed * time.delta().as_secs_f32();
        }

        // afficher logs : verifie la distance au centre
        // info!("Mob distance au centre: {:.2}", distance);
    }
}

pub fn health_system(query: Query<&Health>) {
    for health in query.iter() {
        info!("Mob HP: {}", health.hp);
    }
}