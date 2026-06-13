use bevy::prelude::*;

const LEG_SIZE: Vec2 = Vec2::new(8., 8.);
const LEG_IMAGE: &str = "leg.png";

#[derive(Component)]
pub struct LegCollectible;

fn spaw_leg(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        LegCollectible,
        Sprite {
            image: asset_server.load(LEG_IMAGE),
            custom_size: Some(LEG_SIZE),
            ..default()
        },
        Transform::from_xyz(100.0, 50.0, 0.0),
    ));
}
