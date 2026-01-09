use std::time::Duration;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_time::prelude::*;

use wdn_physics::{
    PhysicsSystems,
    collision::{CollisionTarget, Collisions},
    kinematics::Velocity,
    lerp::Interpolated,
};

use crate::WorldSystems;

pub struct CombatPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Clone, Component, Debug, Default)]
#[require(Velocity, Interpolated, Collisions)]
pub struct Projectile {
    pub damage: u32,
    pub timer: Timer,
}

pub fn apply_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Projectile, &Collisions)>,
    mut pawns: Query<&mut Health>,
    time: Res<Time>,
) {
    projectiles
        .iter_mut()
        .for_each(|(id, mut projectile, collisions)| {
            for collision in collisions.started() {
                let target = match collision.target {
                    CollisionTarget::Collider { id, .. } => id,
                    CollisionTarget::Tile { .. } => continue,
                };

                if let Ok(mut health) = pawns.get_mut(target) {
                    health.damage(projectile.damage);
                }
            }

            if projectile.timer.tick(time.delta()).is_finished() {
                commands.entity(id).despawn();
            }
        });
}

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
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
    pub fn new(damage: u32, duration: Duration) -> Self {
        Projectile {
            damage,
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy_app::prelude::*;
    use bevy_ecs::prelude::*;
    use bevy_math::prelude::*;
    use bevy_transform::prelude::*;

    use wdn_physics::{
        collision::{Collider, Collision, CollisionTarget, Collisions},
        layer::Layer,
    };

    use crate::combat::{CombatPlugin, Health, Projectile};

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

        let projectile = app
            .world_mut()
            .spawn((
                Projectile::new(4, Duration::from_secs(1)),
                Collider::new(0.1, false),
                Transform::from_xyz(0.5, 0.5, 0.0),
                collisions,
            ))
            .id();

        app.world_mut().run_schedule(FixedUpdate);

        let health = app.world().get::<Health>(entity).unwrap();
        assert_eq!(health.current(), 6);

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
    }

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), CombatPlugin));

        app
    }

    fn spawn_layer(app: &mut App) -> Entity {
        app.world_mut().spawn(Layer::default()).id()
    }
}
