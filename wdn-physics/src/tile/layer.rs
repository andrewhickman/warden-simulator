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

pub fn transform_to_isometry(transform: &Transform) -> Isometry2d {
    let translation = transform.translation.xy();
    let rotation = quat_to_rot(transform.rotation);
    Isometry2d::new(translation, rotation)
}

fn quat_to_rot(quat: Quat) -> Rot2 {
    let y2 = quat.y + quat.y;
    let z2 = quat.z + quat.z;
    let cx = 1.0 - (quat.y * y2 + quat.z * z2);
    let cy = quat.w * z2 - quat.x * y2;

    match Dir2::from_xy(cx, cy) {
        Ok(dir) => Rot2::from_sin_cos(dir.y, dir.x),
        Err(_) => Rot2::IDENTITY,
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use approx::assert_relative_eq;
    use bevy_math::prelude::*;

    use crate::tile::layer::quat_to_rot;

    #[test]
    fn quat_to_rot2_identity() {
        let rot = quat_to_rot(Quat::IDENTITY);
        assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
    }

    #[test]
    fn quat_to_rot2_z() {
        for angle in [
            -0.7853982, 0.0, 0.7853982, 1.0, 1.570796, 3.141593, 4.712389, 6.283185, 10.0,
        ] {
            let rot = quat_to_rot(Quat::from_rotation_z(angle));
            let expected = Rot2::radians(angle);
            assert_relative_eq!(rot, expected, epsilon = 1e-4);
        }
    }

    #[test]
    fn quat_to_rot2_x() {
        let rot = quat_to_rot(Quat::from_rotation_x(PI / 2.0));
        assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
    }

    #[test]
    fn quat_to_rot2_y() {
        let rot = quat_to_rot(Quat::from_rotation_y(PI / 2.0));
        assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
    }

    #[test]
    fn quat_to_rot2_xy() {
        let quat = Quat::from_rotation_x(1.0) * Quat::from_rotation_y(-1.5);
        let rot = quat_to_rot(quat);
        assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
    }

    #[test]
    fn quat_to_rot2_xz() {
        let angle = PI / 3.0;
        let rot = quat_to_rot(Quat::from_rotation_x(PI / 4.0) * Quat::from_rotation_z(angle));
        assert_relative_eq!(rot, Rot2::radians(angle), epsilon = 1e-5);
    }

    #[test]
    fn quat_to_rot2_yz() {
        let angle = PI / 3.0;
        let rot = quat_to_rot(Quat::from_rotation_y(PI / 3.0) * Quat::from_rotation_z(angle));
        assert_relative_eq!(rot, Rot2::radians(angle), epsilon = 1e-5);
    }

    #[test]
    fn quat_to_rot2_xyz() {
        let angle = 1.234;
        let quat = Quat::from_euler(EulerRot::XYZ, 1.5, 0.4, angle);
        let rot = quat_to_rot(quat);
        assert_relative_eq!(rot, Rot2::radians(angle), epsilon = 1e-5);
    }
}
