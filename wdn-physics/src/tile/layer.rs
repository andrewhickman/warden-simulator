use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_transform::prelude::*;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Transform)]
pub struct TileLayer {}

#[derive(Copy, Clone, Component, Debug, Default)]
pub struct LayerPosition {
    isometry: Isometry2d,
}

#[derive(Copy, Clone, Component, Debug, Default)]
pub struct LayerVelocity {
    linear: Vec2,
    angular: f32,
}

impl LayerPosition {
    pub fn new(isometry: Isometry2d) -> Self {
        Self { isometry }
    }

    pub fn position(&self) -> Vec2 {
        self.isometry.translation
    }

    pub fn rotation(&self) -> Rot2 {
        self.isometry.rotation
    }
}

impl LayerVelocity {
    pub fn new(linear: Vec2, angular: f32) -> Self {
        Self { linear, angular }
    }

    pub fn linear(&self) -> Vec2 {
        self.linear
    }

    pub fn angular(&self) -> f32 {
        self.angular
    }
}
