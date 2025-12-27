use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use wdn_physics::{
    PhysicsSystems,
    collision::{CollisionTarget, Collisions},
    integrate::Velocity,
    lerp::Interpolated,
};

use crate::WorldSystems;

pub struct CombatPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Velocity, Interpolated)]
pub struct Projectile {
    pub damage: u32,
}

pub fn apply_projectiles(
    projectiles: Query<(&Projectile, &Collisions)>,
    mut pawns: Query<&mut Health>,
) {
    projectiles.iter().for_each(|(projectile, collisions)| {
        for collision in collisions.started() {
            let target = match collision.target {
                CollisionTarget::Tile { id, .. } => id,
                CollisionTarget::Wall { .. } => continue,
            };

            if let Ok(mut health) = pawns.get_mut(target) {
                health.damage(projectile.damage);
            }
        }
    });
}

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            apply_projectiles
                .in_set(WorldSystems::ApplyProjectiles)
                .after(PhysicsSystems::ResolveCollisions),
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

    pub fn damage(&mut self, amount: u32) {
        self.current = self.current.saturating_sub(amount);
    }

    pub fn heal(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }
}
