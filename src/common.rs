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
