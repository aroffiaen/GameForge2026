use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct ContactDmg(#[allow(dead_code)] pub f32);

#[derive(Component)]
pub struct ShootCd(pub Timer);

#[derive(Component)]
pub struct BaseColor(pub Color);

#[derive(Component)]
pub struct Poisoned;

#[derive(Component)]
pub struct HitFlash(#[allow(dead_code)] pub Timer);

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum AiKind {
    Chase,
    Lunge,
    Ranged { 
        min: f32, 
        max: f32, 
        shoot_cd: f32 
    },
}

#[derive(Component)]
pub struct EnemyProjectile {
    pub dmg: f32,
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Lifetime(pub Timer);

impl Lifetime {
    pub fn secs(s: f32) -> Self {
        Self(Timer::from_seconds(s, TimerMode::Once))
    }
}

#[derive(Resource)]
pub struct Arena {
    pub half: Vec2,
}

#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Component)]
pub struct HazardPuddle {
    pub radius: f32,
    pub dps: f32,
    pub life: Timer,
    pub tick: Timer,
}

pub enum DamageKind {
    Hit,
    Poison,
}

#[derive(Message)]
pub struct DamageMsg {
    pub target: Entity,
    pub amount: f32,
    pub kind: DamageKind,
}

#[derive(Component)]
pub struct Health {
    pub hp: i32,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    InGame,
    GameOver,
    Victory,
    Cabanon,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum RoomState {
    #[default]
    Combat,
    Boss,
    NextBiome,
    Transition,
}

pub fn move_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut tf, vel) in &mut query {
        tf.translation += vel.0.extend(0.0) * time.delta_secs();
    }
}

pub fn update_lifetime(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (e, mut life) in &mut query {
        life.0.tick(time.delta());
        if life.0.is_finished() {
            commands.entity(e).despawn();
        }
    }
}
