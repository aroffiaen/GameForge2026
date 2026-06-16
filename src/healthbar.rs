//! Barres de vie flottantes (+ nametags) au-dessus des ennemis et des boss.
//!
//! Porté depuis `feat/player` (équipe), mais réécrit pour l'archi de `test` :
//! - `Health { hp, max: f32 }` (et non `i32`),
//! - texte monde via `Text2d` (et non `Text` UI),
//! - barres **non parentées** qui *suivent* leur propriétaire : les mobs de
//!   `test` portent un léger tilt aléatoire ; parenter la barre lui ferait
//!   hériter de cette rotation (barre de travers). On la garde donc à plat et
//!   on recopie la position chaque frame. Bonus : quand le propriétaire meurt
//!   (ou est purgé en sortie de salle), la barre devient orpheline et se
//!   despawn toute seule — pas de fuite d'entités.

use bevy::prelude::*;

use crate::common::Health;

/// Racine d'une barre flottante (le fond sombre). Référence son propriétaire.
#[derive(Component)]
pub struct FloatingBar {
    owner: Entity,
    y_offset: f32,
    /// Largeur du fond (le remplissage fait `width - 2`).
    width: f32,
    /// `true` pour les boss (toujours visible) ; `false` pour les mobs
    /// (visible uniquement quand ils ont encaissé des dégâts).
    always: bool,
}

/// Le remplissage vert (enfant de la racine), mis à l'échelle selon les PV.
#[derive(Component)]
pub struct HealthBarFill;

const BAR_HEIGHT: f32 = 5.0;

/// Crée une barre de vie flottante pour `owner`.
///
/// `y_offset` : hauteur de la barre au-dessus du centre du propriétaire.
/// `width` : largeur du fond. `always` : visible en permanence (boss).
/// `name` : si fourni, un nametag est affiché juste au-dessus de la barre.
pub fn spawn_health_bar(
    commands: &mut Commands,
    owner: Entity,
    y_offset: f32,
    width: f32,
    always: bool,
    name: Option<&str>,
) {
    commands
        .spawn((
            FloatingBar {
                owner,
                y_offset,
                width,
                always,
            },
            Sprite {
                color: Color::srgb(0.08, 0.08, 0.08),
                custom_size: Some(Vec2::new(width, BAR_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 20.0),
            if always {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        ))
        .with_children(|root| {
            root.spawn((
                HealthBarFill,
                Sprite {
                    color: Color::srgb(0.2, 0.8, 0.2),
                    custom_size: Some(Vec2::new(width - 2.0, BAR_HEIGHT - 2.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
            if let Some(name) = name {
                root.spawn((
                    Text2d::new(name.to_string()),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 10.0, 0.2),
                ));
            }
        });
}

/// Suit le propriétaire, met à jour le remplissage et la visibilité ; despawn
/// la barre dès que le propriétaire n'existe plus.
fn update_floating_bars(
    mut commands: Commands,
    mut roots: Query<
        (Entity, &FloatingBar, &mut Transform, &mut Visibility, &Children),
        Without<HealthBarFill>,
    >,
    owners: Query<(&Transform, &Health), (Without<FloatingBar>, Without<HealthBarFill>)>,
    mut fills: Query<&mut Transform, (With<HealthBarFill>, Without<FloatingBar>)>,
) {
    for (root_e, bar, mut tf, mut vis, children) in &mut roots {
        let Ok((owner_tf, health)) = owners.get(bar.owner) else {
            // Propriétaire mort ou purgé : la barre n'a plus de raison d'être.
            commands.entity(root_e).despawn();
            continue;
        };

        // Position à plat, juste au-dessus du propriétaire (pas de tilt hérité).
        let pos = owner_tf.translation.truncate() + Vec2::new(0.0, bar.y_offset);
        tf.translation = pos.extend(20.0);
        tf.rotation = Quat::IDENTITY;

        let ratio = health.ratio();
        *vis = if bar.always || health.hp < health.max {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        // Le remplissage se vide vers la gauche (ancré à gauche).
        let fill_w = bar.width - 2.0;
        for child in children {
            if let Ok(mut fill_tf) = fills.get_mut(*child) {
                fill_tf.scale.x = ratio.max(0.0001);
                fill_tf.translation.x = (ratio - 1.0) * fill_w * 0.5;
            }
        }
    }
}

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        // Volontairement sans `run_if` : le système doit aussi tourner hors
        // combat pour nettoyer les barres orphelines (sortie de salle, etc.).
        app.add_systems(Update, update_floating_bars.in_set(crate::common::GameSet::Post));
    }
}
