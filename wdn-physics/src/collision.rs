use bevy::{ecs::query::QueryData, prelude::*};

use crate::{
    PhysicsSystems,
    integrate::Velocity,
    tile::{
        TilePosition,
        index::TileIndex,
        storage::{Tile, TileOccupancy, TileStorage},
    },
};

pub struct CollisionPlugin;

#[derive(Component, Clone, Copy)]
#[require(Transform, Collisions)]
pub struct Collider {
    radius: f32,
}

#[derive(QueryData)]
pub struct ColliderQuery {
    collider: &'static Collider,
    transform: &'static Transform,
    velocity: Option<&'static Velocity>,
}

#[derive(Component, Clone, Default)]
pub struct Collisions {
    active: Vec<Collision>,
    nearest: Option<(Collision, f32)>,
}

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub position: Vec2,
    pub normal: Vec2,
    pub target: CollisionTarget,
}

#[derive(Clone, Copy, Debug)]
pub enum CollisionTarget {
    Collider { id: Entity, position: Vec2 },
    Wall { position: TilePosition },
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
    storage: TileStorage,
    mut colliders: Query<(ColliderQuery, &TilePosition, &mut Collisions)>,
    candidates: Query<ColliderQuery>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();

    colliders
        .par_iter_mut()
        .for_each(|(collider, &tile_position, mut collisions)| {
            collisions.clear();

            index.get_neighborhood(tile_position).for_each(|candidate| {
                let Ok(candidate_collider) = candidates.get(candidate) else {
                    return;
                };

                collisions.check_collider(&collider, candidate, &candidate_collider, delta_secs);
            });

            if let Some(tile) = storage.get(tile_position) {
                collisions.check_tile(&collider, tile_position, tile, delta_secs);
            }
        });
}

impl ColliderQueryItem<'_, '_> {
    pub fn radius(&self) -> f32 {
        self.collider.radius
    }

    pub fn position(&self) -> Vec2 {
        self.transform.translation.xy()
    }

    pub fn position_at(&self, t: f32) -> Vec2 {
        self.position() + self.velocity() * t.max(0.0)
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity.map_or(Vec2::ZERO, |v| v.get())
    }
}

impl Collisions {
    pub fn iter(&self) -> impl Iterator<Item = Collision> {
        self.active()
            .chain(self.next().map(|(collision, _)| *collision))
    }

    pub fn active(&self) -> impl ExactSizeIterator<Item = Collision> {
        self.active.iter().copied()
    }

    pub fn next(&self) -> Option<&(Collision, f32)> {
        self.nearest.as_ref()
    }

    pub fn insert(&mut self, collision: Collision, t: f32) {
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

    pub fn check_collider(
        &mut self,
        collider: &ColliderQueryItem,
        candidate_id: Entity,
        candidates: &ColliderQueryItem,
        threshold: f32,
    ) {
        if let Some(t) = circle_collision(
            collider.position() - candidates.position(),
            collider.velocity() - candidates.velocity(),
            collider.radius() + candidates.radius(),
        ) {
            if t < threshold {
                let position = collider.position_at(t);
                let target_position = candidates.position_at(t);
                let collision = Collision {
                    position,
                    normal: (position - target_position).normalize(),
                    target: CollisionTarget::Collider {
                        id: candidate_id,
                        position: target_position,
                    },
                };
                self.insert(collision, t)
            }
        }
    }

    pub fn check_tile(
        &mut self,
        collider: &ColliderQueryItem,
        tile_position: TilePosition,
        tile: &Tile,
        delta_secs: f32,
    ) {
        let occupancy = tile.occupancy();
        if occupancy.contains(TileOccupancy::EAST) {
            self.check_tile_edge(
                collider,
                tile_position.with_offset(IVec2::new(1, 0)),
                Vec2::NEG_X,
                collider.position().x,
                tile_position.x() + 1,
                collider.velocity().x,
                collider.radius(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::NORTH) {
            self.check_tile_edge(
                collider,
                tile_position.with_offset(IVec2::new(0, 1)),
                Vec2::NEG_Y,
                collider.position().y,
                tile_position.y() + 1,
                collider.velocity().y,
                collider.radius(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::WEST) {
            self.check_tile_edge(
                collider,
                tile_position.with_offset(IVec2::new(-1, 0)),
                Vec2::X,
                collider.position().x,
                tile_position.x(),
                collider.velocity().x,
                collider.radius(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::SOUTH) {
            self.check_tile_edge(
                collider,
                tile_position.with_offset(IVec2::new(0, -1)),
                Vec2::Y,
                collider.position().y,
                tile_position.y(),
                collider.velocity().y,
                collider.radius(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::NORTH_EAST)
            && !occupancy.intersects(TileOccupancy::NORTH | TileOccupancy::EAST)
        {
            self.check_tile_corner(
                collider,
                tile_position.with_offset(IVec2::new(1, 1)),
                tile_position.position() + IVec2::ONE,
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::NORTH_WEST)
            && !occupancy.intersects(TileOccupancy::NORTH | TileOccupancy::WEST)
        {
            self.check_tile_corner(
                collider,
                tile_position.with_offset(IVec2::new(-1, 1)),
                tile_position.position() + IVec2::Y,
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::SOUTH_WEST)
            && !occupancy.intersects(TileOccupancy::SOUTH | TileOccupancy::WEST)
        {
            self.check_tile_corner(
                collider,
                tile_position.with_offset(IVec2::new(-1, -1)),
                tile_position.position(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::SOUTH_EAST)
            && !occupancy.intersects(TileOccupancy::SOUTH | TileOccupancy::EAST)
        {
            self.check_tile_corner(
                collider,
                tile_position.with_offset(IVec2::new(1, -1)),
                tile_position.position() + IVec2::X,
                delta_secs,
            );
        }
    }

    pub fn check_tile_edge(
        &mut self,
        collider: &ColliderQueryItem,
        tile_position: TilePosition,
        normal: Vec2,
        collider_position_component: f32,
        tile_position_component: i32,
        collider_velocity_component: f32,
        collider_radius: f32,
        delta_secs: f32,
    ) {
        if let Some(t) = axis_collision(
            collider_position_component - tile_position_component as f32,
            collider_velocity_component,
            collider_radius,
        ) {
            if t < delta_secs {
                let position = collider.position_at(t);
                let collision = Collision {
                    position,
                    normal,
                    target: CollisionTarget::Wall {
                        position: tile_position,
                    },
                };
                self.insert(collision, t)
            }
        }
    }

    pub fn check_tile_corner(
        &mut self,
        collider: &ColliderQueryItem,
        tile_position: TilePosition,
        corner_position: IVec2,
        delta_secs: f32,
    ) {
        if let Some(t) = circle_collision(
            collider.position() - corner_position.as_vec2(),
            collider.velocity(),
            collider.radius(),
        ) {
            if t < delta_secs {
                let position = collider.position_at(t);
                let target_position = corner_position.as_vec2();
                let collision = Collision {
                    position,
                    normal: (position - target_position).normalize(),
                    target: CollisionTarget::Wall {
                        position: tile_position,
                    },
                };
                self.insert(collision, t)
            }
        }
    }

    pub fn clear(&mut self) {
        self.active.clear();
        self.nearest = None;
    }
}

fn circle_collision(
    delta_position: Vec2,
    delta_velocity: Vec2,
    combined_radius: f32,
) -> Option<f32> {
    let a = delta_velocity.length_squared();
    let b = 2.0 * delta_position.dot(delta_velocity);
    let c = delta_position.length_squared() - combined_radius * combined_radius;

    if a == 0.0 {
        return None;
    }

    let discr = b * b - 4.0 * a * c;
    if discr < 0.0 {
        return None;
    }

    let t = (-b - discr.sqrt()) / (2.0 * a);

    if t > 0. || b < 0. { Some(t) } else { None }
}

fn axis_collision(delta_position: f32, delta_velocity: f32, combined_radius: f32) -> Option<f32> {
    if delta_velocity > 0.0 {
        Some((delta_position - combined_radius) / delta_velocity)
    } else {
        return None;
    }
}
