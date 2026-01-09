#[cfg(test)]
mod tests;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_time::prelude::*;
use bevy_transform::prelude::*;

use crate::{
    PhysicsSystems,
    collision::{Collision, Collisions},
    layer::LayerVelocity,
};

pub struct KinematicsPlugin;

#[derive(Clone, Copy, Component, Default, Debug)]
#[require(Transform, LayerVelocity)]
pub struct Velocity {
    linear: Vec2,
    angular: f32,
}

pub fn update_kinematics(
    mut query: Query<(&mut Transform, &mut Velocity, Option<&Collisions>)>,
    time: Res<Time>,
) {
    query
        .par_iter_mut()
        .for_each(|(mut transform, mut velocity, collisions)| {
            if velocity.is_zero() {
                return;
            }

            let mut delta_secs = time.delta_secs();

            if velocity.linear != Vec2::ZERO {
                if let Some(collisions) = collisions {
                    for collision in collisions.active() {
                        if collision.solid {
                            velocity.collide(collision);
                        }
                    }

                    if let Some((collision, t)) = collisions.next() {
                        transform.translation.x = collision.position.x;
                        transform.translation.y = collision.position.y;
                        delta_secs -= t;

                        velocity.collide(collision);
                    }
                }

                if velocity.linear != Vec2::ZERO {
                    transform.translation.x += velocity.linear.x * delta_secs;
                    transform.translation.y += velocity.linear.y * delta_secs;
                }
            }

            if velocity.angular != 0.0 {
                transform.rotate_z(velocity.angular * delta_secs);
            }
        });
}

impl Plugin for KinematicsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            PhysicsSystems::Kinematics
                .after(PhysicsSystems::Sync)
                .after(PhysicsSystems::Collisions),
        );

        app.add_systems(
            FixedUpdate,
            update_kinematics.in_set(PhysicsSystems::Kinematics),
        );
    }
}

impl Velocity {
    pub fn new(linear: Vec2) -> Self {
        Velocity {
            linear,
            angular: 0.0,
        }
    }

    pub fn linear(&self) -> Vec2 {
        self.linear
    }

    pub fn angular(&self) -> f32 {
        self.angular
    }

    pub fn set_angular(&mut self, angular: f32) {
        self.angular = angular;
    }

    pub fn with_angular(mut self, angular: f32) -> Self {
        self.angular = angular;
        self
    }

    pub fn is_zero(&self) -> bool {
        self.linear == Vec2::ZERO && self.angular == 0.0
    }

    pub fn decelerate(&mut self, decel: f32) {
        if self.linear == Vec2::ZERO {
            return;
        }

        let speed = self.linear.length();
        let new_speed = speed - decel;

        if new_speed <= 0.0 {
            self.linear = Vec2::ZERO;
        } else {
            let scale = new_speed / speed;
            self.linear *= scale;
        }
    }

    pub fn accelerate(&mut self, target: Vec2, accel: f32) {
        self.linear += (target - self.linear).clamp_length_max(accel);
    }

    pub fn collide(&mut self, collision: Collision) {
        let normal = collision.normal;
        let projected = self.linear.dot(*normal);
        if projected < 0.0 {
            self.linear -= projected * normal;
        }
    }
}
