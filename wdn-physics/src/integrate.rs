use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_time::prelude::*;
use bevy_transform::prelude::*;

use crate::{
    PhysicsSystems,
    collision::{Collision, Collisions},
};

pub struct IntegratePlugin;

#[derive(Clone, Copy, Component, Default, Debug)]
#[require(Transform)]
pub struct Velocity {
    linear: Vec2,
    angular: f32,
}

impl Plugin for IntegratePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            integrate_velocity
                .in_set(PhysicsSystems::Integrate)
                .after(PhysicsSystems::ResolveCollisions),
        );
    }
}

pub fn integrate_velocity(
    mut query: Query<(&mut Transform, &mut Velocity, Option<&Collisions>)>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();

    query
        .par_iter_mut()
        .for_each(|(mut transform, mut velocity, collisions)| {
            if velocity.is_zero() {
                return;
            }

            if velocity.linear != Vec2::ZERO {
                if let Some(collisions) = collisions {
                    for collision in collisions.active() {
                        if collision.solid {
                            velocity.collide(collision);
                        }
                    }

                    if velocity.linear != Vec2::ZERO {
                        if let Some(collision) = collisions.next() {
                            transform.translation.x = collision.position.x;
                            transform.translation.y = collision.position.y;

                            velocity.collide(collision);
                        } else {
                            transform.translation.x += velocity.linear.x * delta_secs;
                            transform.translation.y += velocity.linear.y * delta_secs;
                        }
                    }
                } else {
                    transform.translation.x += velocity.linear.x * delta_secs;
                    transform.translation.y += velocity.linear.y * delta_secs;
                }
            }

            if velocity.angular != 0.0 {
                transform.rotate_z(velocity.angular * delta_secs);
            }
        });
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use approx::assert_relative_eq;
    use bevy_app::prelude::*;
    use bevy_ecs::prelude::*;
    use bevy_math::prelude::*;
    use bevy_time::{TimePlugin, TimeUpdateStrategy, prelude::*};
    use bevy_transform::prelude::*;

    use crate::{
        collision::{Collision, CollisionTarget, Collisions},
        integrate::{IntegratePlugin, Velocity},
    };

    #[test]
    fn integrate() {
        let mut app = make_app();

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::new(Vec2::new(5.0, 3.0)),
            ))
            .id();

        app.update();

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation.xy(), Vec2::new(5.0, 3.0));
    }

    #[test]
    fn integrate_zero_velocity() {
        let mut app = make_app();

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(10.0, 20.0, 0.0),
                Velocity::new(Vec2::ZERO),
            ))
            .id();

        let change_tick = app
            .world()
            .entity(entity)
            .get_ref::<Transform>()
            .unwrap()
            .last_changed();

        app.update();

        let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
        assert_relative_eq!(transform.translation.xy(), Vec2::new(10.0, 20.0));
        assert_eq!(transform.last_changed(), change_tick);
    }

    #[test]
    fn integrate_angular_velocity() {
        let mut app = make_app();

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::new(Vec2::ZERO).with_angular(1.0),
            ))
            .id();

        app.update();

        let transform = app.world().get::<Transform>(entity).unwrap();
        let expected = Quat::from_rotation_z(1.0);
        assert_relative_eq!(transform.rotation.x, expected.x, epsilon = 1e-6);
        assert_relative_eq!(transform.rotation.y, expected.y, epsilon = 1e-6);
        assert_relative_eq!(transform.rotation.z, expected.z, epsilon = 1e-6);
        assert_relative_eq!(transform.rotation.w, expected.w, epsilon = 1e-6);
    }

    #[test]
    fn integrate_wall_collision_active() {
        let mut app = make_app();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision {
                position: Default::default(),
                normal: Dir2::X,
                target: CollisionTarget::Tile {
                    id: None,
                    position: Default::default(),
                },
                solid: true,
            },
            0.0,
        );

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::new(Vec2::new(-5.0, 3.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.linear(), Vec2::new(0.0, 3.0));

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation.xy(), Vec2::new(0.0, 3.0));
    }

    #[test]
    fn integrate_wall_collision() {
        let mut app = make_app();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision {
                position: Vec2::new(2.0, -1.0),
                normal: Dir2::Y,
                target: CollisionTarget::Tile {
                    id: None,
                    position: Default::default(),
                },
                solid: true,
            },
            0.5,
        );

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::new(Vec2::new(4.0, -2.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.linear(), Vec2::new(4.0, 0.0));

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation.xy(), Vec2::new(2.0, -1.0));
    }

    #[test]
    fn integrate_multiple_collisions() {
        let mut app = make_app();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision {
                position: Default::default(),
                normal: Dir2::X,
                target: CollisionTarget::Tile {
                    id: None,
                    position: Default::default(),
                },
                solid: true,
            },
            0.0,
        );
        collisions.insert(
            Collision {
                position: Default::default(),
                normal: Dir2::Y,
                target: CollisionTarget::Tile {
                    id: None,
                    position: Default::default(),
                },
                solid: true,
            },
            0.0,
        );

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::new(Vec2::new(-3.0, -4.0)),
                collisions,
            ))
            .id();

        let change_tick = app
            .world()
            .entity(entity)
            .get_ref::<Transform>()
            .unwrap()
            .last_changed();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.linear(), Vec2::ZERO);

        let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
        assert_relative_eq!(transform.translation.xy(), Vec2::ZERO);
        assert_eq!(transform.last_changed(), change_tick);
    }

    #[test]
    fn integrate_wall_collision_receding() {
        let mut app = make_app();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision {
                position: Default::default(),
                normal: Dir2::X,
                target: CollisionTarget::Tile {
                    id: None,
                    position: Default::default(),
                },
                solid: true,
            },
            0.0,
        );

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::new(Vec2::new(5.0, 3.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.linear(), Vec2::new(5.0, 3.0));

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation.xy(), Vec2::new(5.0, 3.0));
    }

    #[test]
    fn integrate_collider_collision() {
        let mut app = make_app();

        let other_entity = app.world_mut().spawn_empty().id();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision {
                position: Vec2::new(-2.0, 1.0),
                normal: Dir2::new(Vec2::new(1.0, 1.0)).unwrap(),
                target: CollisionTarget::Collider {
                    id: other_entity,
                    position: Vec2::new(5.0, 5.0),
                },
                solid: true,
            },
            0.5,
        );

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::new(Vec2::new(-4.0, -2.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.linear(), Vec2::new(-1.0, 1.0), epsilon = 1e-4);

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(
            transform.translation.xy(),
            Vec2::new(-2.0, 1.0),
            epsilon = 1e-4
        );
    }

    #[test]
    fn integrate_non_solid_collision_active() {
        let mut app = make_app();

        let other_entity = app.world_mut().spawn_empty().id();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision {
                position: Vec2::new(2.0, 1.0),
                normal: Dir2::X,
                target: CollisionTarget::Collider {
                    id: other_entity,
                    position: Vec2::new(3.0, 1.0),
                },
                solid: false,
            },
            0.5,
        );

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::new(Vec2::new(4.0, 2.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.linear(), Vec2::new(4.0, 2.0));

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation.xy(), Vec2::new(4.0, 2.0));
    }

    #[test]
    fn integrate_non_solid_wall_collision_active() {
        let mut app = make_app();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision {
                position: Vec2::new(2.0, 1.0),
                normal: Dir2::Y,
                target: CollisionTarget::Tile {
                    id: None,
                    position: Default::default(),
                },
                solid: false,
            },
            0.0,
        );

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::new(Vec2::new(3.0, -5.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.linear(), Vec2::new(3.0, -5.0));

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation.xy(), Vec2::new(3.0, -5.0));
    }

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), TimePlugin, IntegratePlugin));

        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs(1)));
        app.insert_resource(Time::<Virtual>::from_max_delta(Duration::MAX));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));

        app.world_mut()
            .resource_mut::<Time<Real>>()
            .update_with_duration(Duration::ZERO);

        app
    }
}
