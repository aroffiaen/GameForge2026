use bevy::prelude::*;
use crate::common::{Health, HealthBar, HealthBarFill};

pub fn update_health_bar(
    query_health: Query<&Health, Changed<Health>>,
    mut query_fill: Query<(&mut Transform, &ChildOf), With<HealthBarFill>>,
) {
    for (mut tf, child_of) in &mut query_fill {
        if let Ok(health) = query_health.get(child_of.parent()) {
            let ratio = (health.hp as f32 / health.max_hp as f32).clamp(0.0, 1.0);
            tf.scale.x = ratio;
            // On décale un peu pour que la barre se vide vers la gauche
            tf.translation.x = (ratio - 1.0) * 15.0; 
        }
    }
}

// Système pour afficher la barre de vie quand on prend des dégâts
pub fn update_health_bar_visibility(
    query_health: Query<&Health, Changed<Health>>,
    mut query_bar: Query<(&mut Visibility, &ChildOf), With<HealthBar>>,
) {
    for (mut vis, child_of) in &mut query_bar {
        if let Ok(health) = query_health.get(child_of.parent()) {
            if health.hp < health.max_hp {
                *vis = Visibility::Inherited;
            }
        }
    }
}

pub fn spawn_health_bar(commands: &mut Commands, parent_entity: Entity, y_offset: f32, always_visible: bool) {
    // Fond de la barre (noir/gris)
    let bar_id = commands.spawn((
        HealthBar,
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(Vec2::new(32.0, 6.0)),
            ..default()
        },
        Transform::from_xyz(0.0, y_offset, 10.0),
        if always_visible { Visibility::Inherited } else { Visibility::Hidden },
    )).id();


    // Remplissage (vert)
    let fill_id = commands.spawn((
        HealthBarFill,
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.2),
            custom_size: Some(Vec2::new(30.0, 4.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    )).id();

    // Hiérarchie
    commands.entity(bar_id).add_child(fill_id);
    commands.entity(parent_entity).add_child(bar_id);
}
