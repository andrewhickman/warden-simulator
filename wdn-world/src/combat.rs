use std::time::Duration;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_time::prelude::*;

use wdn_physics::{
    PhysicsSystems,
    collision::{CollisionTarget, Collisions},
    kinematics::RelativeVelocity,
    lerp::Interpolate,
};

use crate::WorldSystems;

pub struct CombatPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Message)]
pub struct Damaged {
    pub source: Entity,
    pub target: Entity,
}

#[derive(Clone, Component, Debug)]
#[require(RelativeVelocity, Interpolate, Collisions)]
pub struct Projectile {
    pub source: Entity,
    pub damage: u32,
    pub timer: Timer,
}

pub fn apply_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Projectile, &Collisions)>,
    mut pawns: Query<&mut Health>,
    mut damaged_writer: MessageWriter<Damaged>,
    time: Res<Time>,
) {
    projectiles
        .iter_mut()
        .for_each(|(id, mut projectile, collisions)| {
            collisions.started().for_each(|collision| {
                let target = match collision.target {
                    CollisionTarget::Collider { id, .. } => id,
                    CollisionTarget::Tile { .. } => return,
                };

                if target == projectile.source {
                    return;
                }

                if let Ok(mut health) = pawns.get_mut(target) {
                    health.damage(projectile.damage);

                    if !health.is_alive() {
                        commands.entity(target).try_despawn();
                    }

                    damaged_writer.write(Damaged {
                        source: projectile.source,
                        target,
                    });
                }
            });

            if projectile.timer.tick(time.delta()).is_finished() {
                commands.entity(id).despawn();
            }
        });
}

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Damaged>();

        app.configure_sets(
            FixedUpdate,
            WorldSystems::ApplyProjectiles.after(PhysicsSystems::Collisions),
        );

        app.add_systems(
            FixedUpdate,
            apply_projectiles.in_set(WorldSystems::ApplyProjectiles),
        );
    }
}

impl Health {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn damage(&mut self, amount: u32) {
        self.current = self.current.saturating_sub(amount);
    }

    pub fn heal(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }
}

impl Projectile {
    pub fn new(source: Entity, damage: u32, duration: Duration) -> Self {
        Projectile {
            source,
            damage,
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy_app::prelude::*;
    use bevy_ecs::{message::MessageCursor, prelude::*};
    use bevy_math::prelude::*;
    use bevy_time::TimePlugin;
    use bevy_transform::prelude::*;

    use wdn_physics::{
        collision::{Collider, Collision, CollisionTarget, Collisions},
        layer::Layer,
    };

    use crate::combat::{CombatPlugin, Damaged, Health, Projectile};

    #[test]
    fn apply_projectiles() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Health::new(10),
                Collider::new(0.2, true),
                Transform::from_xyz(5.0, 5.0, 0.0),
                ChildOf(layer),
            ))
            .id();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision {
                position: Vec2::new(5.0, 5.0),
                normal: Dir2::X,
                target: CollisionTarget::Collider {
                    id: entity,
                    position: Vec2::new(5.0, 5.0),
                },
                solid: true,
            },
            0.0,
        );

        let source = app.world_mut().spawn_empty().id();
        let projectile = app
            .world_mut()
            .spawn((
                Projectile::new(source, 4, Duration::from_secs(1)),
                Collider::new(0.1, false),
                Transform::from_xyz(0.5, 0.5, 0.0),
                collisions,
            ))
            .id();

        app.world_mut().run_schedule(FixedUpdate);

        let health = app.world().get::<Health>(entity).unwrap();
        assert_eq!(health.current(), 6);

        let mut damaged_cursor = MessageCursor::default();
        let damaged_messages: Vec<_> = damaged_cursor
            .read(app.world().resource::<Messages<Damaged>>())
            .collect();
        assert_eq!(damaged_messages.len(), 1);
        assert_eq!(damaged_messages[0].source, source);
        assert_eq!(damaged_messages[0].target, entity);

        let mut collisions = app.world_mut().get_mut::<Collisions>(projectile).unwrap();
        collisions.clear();
        collisions.insert(
            Collision {
                position: Vec2::new(5.0, 5.0),
                normal: Dir2::X,
                target: CollisionTarget::Collider {
                    id: entity,
                    position: Vec2::new(5.0, 5.0),
                },
                solid: true,
            },
            0.0,
        );

        app.world_mut().run_schedule(FixedUpdate);

        let health = app.world().get::<Health>(entity).unwrap();
        assert_eq!(health.current(), 6);

        let damaged_messages: Vec<_> = damaged_cursor
            .read(app.world().resource::<Messages<Damaged>>())
            .collect();
        assert_eq!(damaged_messages.len(), 0);
    }

    #[test]
    fn projectile_ignores_source() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let source = app
            .world_mut()
            .spawn((
                Health::new(10),
                Collider::new(0.2, true),
                Transform::from_xyz(5.0, 5.0, 0.0),
                ChildOf(layer),
            ))
            .id();

        let mut collisions = Collisions::default();
        collisions.insert(
            Collision {
                position: Vec2::new(5.0, 5.0),
                normal: Dir2::X,
                target: CollisionTarget::Collider {
                    id: source,
                    position: Vec2::new(5.0, 5.0),
                },
                solid: true,
            },
            0.0,
        );

        app.world_mut().spawn((
            Projectile::new(source, 4, Duration::from_secs(1)),
            Collider::new(0.1, false),
            Transform::from_xyz(0.5, 0.5, 0.0),
            collisions,
            ChildOf(source),
        ));

        app.world_mut().run_schedule(FixedUpdate);

        let health = app.world().get::<Health>(source).unwrap();
        assert_eq!(health.current(), 10);

        let mut damaged_cursor = MessageCursor::default();
        let damaged_messages: Vec<_> = damaged_cursor
            .read(app.world().resource::<Messages<Damaged>>())
            .collect();
        assert_eq!(damaged_messages.len(), 0);
    }

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), TimePlugin, CombatPlugin));

        app
    }

    fn spawn_layer(app: &mut App) -> Entity {
        app.world_mut().spawn(Layer::default()).id()
    }
}
