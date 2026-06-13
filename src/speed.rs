use bevy::prelude::*;

#[derive(Component)]
pub struct Speed(f32);

impl Speed {
    pub fn new(speed: f32) -> Self {
        Speed(speed)
    }

    pub fn get(&self) -> f32 {
        self.0
    }

    pub fn modify(&mut self, modifier: f32) {
        self.0 *= modifier;
    }
}
