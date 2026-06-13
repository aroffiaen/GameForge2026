use bevy::prelude::*;

trait Chain {
    type Item;
    fn chain(items: &[Self::Item]) -> Option<Self::Item>;
}

#[derive(Component, Clone, Copy)]
pub struct Modifier(f32);

impl Modifier {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

impl Default for Modifier {
    fn default() -> Self {
        Self(1.)
    }
}

impl Chain for Modifier {
    type Item = Modifier;
    fn chain(items: &[Self::Item]) -> Option<Self::Item> {
        items.iter().copied().reduce(|a, b| Modifier(a.0 * b.0))
    }
}
