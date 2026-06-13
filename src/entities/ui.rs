use bevy::prelude::*;
use crate::common::{Health, HealthBar, HealthBarFill};
use crate::common::GameState;
use crate::player::Player;

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

pub fn spawn_health_bar(commands: &mut Commands, parent_entity: Entity, y_offset: f32, always_visible: bool, optional_name: Option<&str>) {
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
    )).with_children(|parent| {
        // Remplissage (vert)
        parent.spawn((
            HealthBarFill,
            Sprite {
                color: Color::srgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(30.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
        
        // Nom (Nametag) si fourni
        if let Some(name) = optional_name {
            parent.spawn((
                Text::new(name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 12.0, 2.0), // Un peu plus haut que la barre
                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
            ));
        }
    }).id();

    // Hiérarchie
    commands.entity(parent_entity).add_child(bar_id);
}


// Tag pour retrouver l'UI du Game Over facilement
#[derive(Component)]
pub struct GameOverScreen;

// afficher ui : se lance UNIQUEMENT quand on entre dans l'état GameOver
pub fn spawn_game_over_ui(mut commands: Commands) {
    commands.spawn((
        GameOverScreen,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0), // Espace entre les textes
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)), // Fond noir plus opaque
    )).with_children(|parent| {
        // Titre Principal
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font_size: 80.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.1, 0.1)), // Rouge vif
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        ));

        // Sous-titre / Instruction
        parent.spawn((
            Text::new("Le jardin a eu raison de vous..."),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)), // Gris clair
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        ));

        // Bouton de restart
        parent.spawn((
            Text::new("Appuyez sur [ R ] pour recommencer"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)), // Blanc
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        ));
    });
}

            // redemarrer jeu : ecoute la touche R et reinitialise tout
            pub fn restart_game(
            keyboard_input: Res<ButtonInput<KeyCode>>,
            mut next_state: ResMut<NextState<GameState>>,
            mut player_query: Query<(&mut Health, &mut Transform), With<Player>>,
            mobs_query: Query<Entity, With<crate::common::Enemy>>,
            ui_query: Query<Entity, With<GameOverScreen>>,
            mut commands: Commands,
            ) {
            if keyboard_input.just_pressed(KeyCode::KeyR) {
            
            // restaurer le joueur
            if let Ok((mut health, mut transform)) = player_query.single_mut() {

            health.hp = health.max_hp;
            transform.translation = Vec3::ZERO; // Le ramener au centre
        }

        // nettoyer la carte des ennemis
        for mob in mobs_query.iter() {
            commands.entity(mob).despawn();
        }

        // supprimer l'écran de Game Over
        for ui in ui_query.iter() {
            commands.entity(ui).despawn();
        }

        // 4. Relancer le jeu
        next_state.set(GameState::InGame);
        info!("🔄 Jeu redémarré !");
        }
        }

