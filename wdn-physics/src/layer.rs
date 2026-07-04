use bevy_ecs::prelude::*;

#[derive(Copy, Clone, Component, Debug, Default)]
pub struct Layer {
    height: i32,
}

#[derive(Component, Default)]
pub struct LayerStack {}

impl Layer {
    pub fn new(height: i32) -> Self {
        Self { height }
    }

    pub fn height(&self) -> i32 {
        self.height
    }
}
