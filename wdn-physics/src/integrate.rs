use bevy::prelude::*;

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
            Update,
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
    let delta_seconds = time.delta_secs();

    query
        .par_iter_mut()
        .for_each(|(mut transform, mut velocity, collisions)| {
            if velocity.0 == Vec2::ZERO {
                return;
            }

            if let Some(collisions) = collisions {
                for collision in collisions.active() {
                    velocity.collide(&collision);
                }

                if velocity.0 == Vec2::ZERO {
                    return;
                }

                if let Some((collision, t)) = collisions.next() {
                    transform.translation.x += velocity.0.x * t;
                    transform.translation.y += velocity.0.y * t;

                    velocity.collide(collision);
                    return;
                }
            }

            transform.translation.x += velocity.0.x * delta_seconds;
            transform.translation.y += velocity.0.y * delta_seconds;
        });
}

impl Velocity {
    pub fn new(velocity: Vec2) -> Self {
        Velocity(velocity)
    }

    pub fn collide(&mut self, collision: &Collision) {
        let normal = collision.normal();
        let projected = self.0.dot(normal);
        if projected < 0.0 {
            self.0 -= projected * normal;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use approx::assert_relative_eq;
    use bevy::{prelude::*, time::TimeUpdateStrategy};

    use crate::{
        collision::{Collision, Collisions},
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

        let transform = app.world().entity(entity).get::<Transform>().unwrap();
        assert_relative_eq!(transform.translation.x, 5.0);
        assert_relative_eq!(transform.translation.y, 3.0);
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
        assert_relative_eq!(transform.translation.x, 10.0);
        assert_relative_eq!(transform.translation.y, 20.0);
        assert_eq!(transform.last_changed(), change_tick);
    }

    #[test]
    fn integrate_wall_collision_active() {
        let mut app = make_app();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision::Wall {
                normal: Vec2::X,
                position: Default::default(),
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

        let velocity = app.world().entity(entity).get::<Velocity>().unwrap();
        assert_relative_eq!(velocity.0.x, 0.0);
        assert_relative_eq!(velocity.0.y, 3.0);

        let transform = app.world().entity(entity).get::<Transform>().unwrap();
        assert_relative_eq!(transform.translation.x, 0.0);
        assert_relative_eq!(transform.translation.y, 3.0);
    }

    #[test]
    fn integrate_wall_collision() {
        let mut app = make_app();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision::Wall {
                normal: Vec2::Y,
                position: Default::default(),
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

        let velocity = app.world().entity(entity).get::<Velocity>().unwrap();
        assert_relative_eq!(velocity.0.x, 4.0);
        assert_relative_eq!(velocity.0.y, 0.0);

        let transform = app.world().entity(entity).get::<Transform>().unwrap();
        assert_relative_eq!(transform.translation.x, 2.0);
        assert_relative_eq!(transform.translation.y, -1.0);
    }

    #[test]
    fn integrate_multiple_collisions() {
        let mut app = make_app();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision::Wall {
                normal: Vec2::X,
                position: Default::default(),
            },
            0.0,
        );
        collisions.insert(
            Collision::Wall {
                normal: Vec2::Y,
                position: Default::default(),
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

        let velocity = app.world().entity(entity).get::<Velocity>().unwrap();
        assert_relative_eq!(velocity.0.x, 0.0);
        assert_relative_eq!(velocity.0.y, 0.0);

        let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
        assert_relative_eq!(transform.translation.x, 0.0);
        assert_relative_eq!(transform.translation.y, 0.0);
        assert_eq!(transform.last_changed(), change_tick);
    }

    #[test]
    fn integrate_wall_collision_receding() {
        let mut app = make_app();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision::Wall {
                normal: Vec2::X,
                position: Default::default(),
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

        let velocity = app.world().entity(entity).get::<Velocity>().unwrap();
        assert_relative_eq!(velocity.0.x, 5.0);
        assert_relative_eq!(velocity.0.y, 3.0);

        let transform = app.world().entity(entity).get::<Transform>().unwrap();
        assert_relative_eq!(transform.translation.x, 5.0);
        assert_relative_eq!(transform.translation.y, 3.0);
    }

    #[test]
    fn integrate_collider_collision() {
        let mut app = make_app();

        let other_entity = app.world_mut().spawn_empty().id();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision::Collider {
                normal: Vec2::new(1.0, 1.0).normalize(),
                id: other_entity,
                position: Vec2::new(5.0, 5.0),
            },
            0.0,
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

        let velocity = app.world().entity(entity).get::<Velocity>().unwrap();
        assert_relative_eq!(velocity.0.x, -1.0, epsilon = 0.0001);
        assert_relative_eq!(velocity.0.y, 1.0, epsilon = 0.0001);

        let transform = app.world().entity(entity).get::<Transform>().unwrap();
        assert_relative_eq!(transform.translation.x, -1.0, epsilon = 0.0001);
        assert_relative_eq!(transform.translation.y, 1.0, epsilon = 0.0001);
    }

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, IntegratePlugin));

        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs(1)));
        app.insert_resource(Time::<Virtual>::from_max_delta(Duration::MAX));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));

        app.world_mut()
            .resource_mut::<Time<Real>>()
            .update_with_duration(Duration::ZERO);

        app
    }
}
