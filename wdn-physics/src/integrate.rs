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
pub struct Velocity(Vec2);

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

            if let Some(collisions) = collisions {
                for collision in collisions.active() {
                    if collision.solid {
                        velocity.collide(collision);
                    }
                }

                if velocity.is_zero() {
                    return;
                }

                if let Some(collision) = collisions.next() {
                    transform.translation.x = collision.position.x;
                    transform.translation.y = collision.position.y;

                    velocity.collide(collision);
                    return;
                }
            }

            transform.translation.x += velocity.0.x * delta_secs;
            transform.translation.y += velocity.0.y * delta_secs;
        });
}

impl Velocity {
    pub fn new(velocity: Vec2) -> Self {
        Velocity(velocity)
    }

    pub fn get(&self) -> Vec2 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == Vec2::ZERO
    }

    pub fn decelerate(&mut self, decel: f32) {
        if self.is_zero() {
            return;
        }

        let speed = self.0.length();
        let new_speed = speed - decel;

        if new_speed <= 0.0 {
            self.0 = Vec2::ZERO;
        } else {
            let scale = new_speed / speed;
            self.0 *= scale;
        }
    }

    pub fn accelerate(&mut self, target: Vec2, accel: f32) {
        self.0 += (target - self.0).clamp_length_max(accel);
    }

    pub fn collide(&mut self, collision: Collision) {
        let normal = collision.normal;
        let projected = self.0.dot(*normal);
        if projected < 0.0 {
            self.0 -= projected * normal;
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
                Velocity(Vec2::new(5.0, 3.0)),
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
            .spawn((Transform::from_xyz(10.0, 20.0, 0.0), Velocity(Vec2::ZERO)))
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
                Velocity(Vec2::new(-5.0, 3.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.0, Vec2::new(0.0, 3.0));

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
                Velocity(Vec2::new(4.0, -2.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.0, Vec2::new(4.0, 0.0));

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
                Velocity(Vec2::new(-3.0, -4.0)),
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
        assert_relative_eq!(velocity.0, Vec2::ZERO);

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
                Velocity(Vec2::new(5.0, 3.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.0, Vec2::new(5.0, 3.0));

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
                Velocity(Vec2::new(-4.0, -2.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.0, Vec2::new(-1.0, 1.0), epsilon = 1e-4);

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
                Velocity(Vec2::new(4.0, 2.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.0, Vec2::new(4.0, 2.0));

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
                Velocity(Vec2::new(3.0, -5.0)),
                collisions,
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity>(entity).unwrap();
        assert_relative_eq!(velocity.0, Vec2::new(3.0, -5.0));

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
