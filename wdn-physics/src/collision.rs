use bevy::prelude::*;

use crate::{
    PhysicsSystems,
    integrate::Velocity,
    tile::{TilePosition, index::TileIndex},
};

pub struct CollisionPlugin;

#[derive(Component, Clone, Copy)]
#[require(Transform, Collisions)]
pub struct Collider {
    radius: f32,
}

#[derive(Component, Clone, Default)]
pub struct Collisions {
    active: Vec<Collision>,
    nearest: Option<(Collision, f32)>,
}

#[derive(Clone, Copy, Debug)]
pub enum Collision {
    Collider {
        normal: Vec2,
        id: Entity,
        position: Vec2,
    },
    Wall {
        normal: Vec2,
        position: TilePosition,
    },
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            resolve_collisions
                .in_set(PhysicsSystems::ResolveCollisions)
                .after(PhysicsSystems::UpdateTile),
        );
    }
}

pub fn resolve_collisions(
    index: Res<TileIndex>,
    colliders: Query<(
        &Collider,
        &TilePosition,
        &Transform,
        &Velocity,
        &mut Collisions,
    )>,
) {
}

impl Collisions {
    pub fn iter(&self) -> impl Iterator<Item = Collision> {
        self.active()
            .chain(self.next().map(|(collision, _)| *collision))
    }

    pub(crate) fn active(&self) -> impl ExactSizeIterator<Item = Collision> {
        self.active.iter().copied()
    }

    pub(crate) fn next(&self) -> Option<&(Collision, f32)> {
        self.nearest.as_ref()
    }

    pub(crate) fn insert(&mut self, collision: Collision, t: f32) {
        if t <= 0.0 {
            self.active.push(collision);
        } else {
            match &self.nearest {
                Some((_, existing_t)) if *existing_t <= t => {}
                _ => {
                    self.nearest = Some((collision, t));
                }
            }
        }
    }
}

impl Collision {
    pub fn normal(&self) -> Vec2 {
        match *self {
            Collision::Collider { normal, .. } | Collision::Wall { normal, .. } => normal,
        }
    }
}
