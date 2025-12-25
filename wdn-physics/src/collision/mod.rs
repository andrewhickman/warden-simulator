#[cfg(test)]
mod tests;

use bevy::{ecs::query::QueryData, math::CompassOctant, prelude::*};

use crate::{
    PhysicsSystems,
    integrate::Velocity,
    tile::{
        TilePosition,
        index::TileIndex,
        storage::{TileOccupancy, TileStorage},
    },
};

pub struct CollisionPlugin;

#[derive(Component, Clone, Copy, Debug)]
#[require(Transform, Collisions, TilePosition)]
pub struct Collider {
    radius: f32,
}

#[derive(Component, Clone, Copy, Debug, Default)]
#[require(TilePosition)]
pub struct TileCollider;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ColliderDisabled;

#[derive(QueryData, Debug)]
pub struct ColliderQuery {
    collider: &'static Collider,
    transform: &'static Transform,
    velocity: Option<&'static Velocity>,
}

#[derive(QueryData, Debug)]
pub struct TileColliderQuery {
    collider: &'static TileCollider,
    position: &'static TilePosition,
}

#[derive(Component, Clone, Debug, Default)]
pub struct Collisions {
    active: Vec<Collision>,
    nearest: Option<(Collision, f32)>,
}

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub position: Vec2,
    pub normal: Dir2,
    pub target: CollisionTarget,
}

#[derive(Clone, Copy, Debug)]
pub enum CollisionTarget {
    Collider {
        id: Entity,
        position: Vec2,
    },
    Wall {
        id: Option<Entity>,
        position: TilePosition,
    },
}

struct TileColliderLookup {
    entities: [Option<Entity>; 8],
}

pub fn resolve_collisions(
    index: Res<TileIndex>,
    storage: TileStorage,
    mut colliders: Query<(
        Entity,
        ColliderQuery,
        &TilePosition,
        &mut Collisions,
        Has<ColliderDisabled>,
    )>,
    candidates: Query<AnyOf<(ColliderQuery, TileColliderQuery)>, Without<ColliderDisabled>>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();

    colliders.par_iter_mut().for_each(
        |(collider_id, collider, &tile_position, mut collisions, disabled)| {
            collisions.clear();

            if disabled {
                return;
            }

            let mut tile_occupancy = storage.get_occupancy(tile_position);
            let mut tile_colliders = TileColliderLookup::new();

            index.get_neighborhood(tile_position).for_each(|candidate| {
                if candidate == collider_id {
                    return;
                }

                let Ok((candidate_collider, candidate_tile)) = candidates.get(candidate) else {
                    return;
                };

                if let Some(candidate_collider) = candidate_collider {
                    collisions.check_collider(
                        &collider,
                        candidate,
                        &candidate_collider,
                        delta_secs,
                    );
                }

                if let Some(candidate_tile) = candidate_tile {
                    if let Some(octant) = offset_to_octant(
                        tile_position.position(),
                        candidate_tile.position.position(),
                    ) {
                        tile_occupancy |= TileOccupancy::from_octant(octant);
                        tile_colliders.insert(octant, candidate);
                    }
                }
            });

            if tile_occupancy != TileOccupancy::NONE {
                collisions.check_tile(
                    &collider,
                    &tile_colliders,
                    tile_position,
                    tile_occupancy,
                    delta_secs,
                );
            }
        },
    );
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            resolve_collisions
                .in_set(PhysicsSystems::ResolveCollisions)
                .after(PhysicsSystems::UpdateTile),
        );
    }
}

impl Collider {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
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
    pub fn active(&self) -> impl ExactSizeIterator<Item = Collision> {
        self.active.iter().copied()
    }

    pub fn next(&self) -> Option<Collision> {
        match self.nearest {
            Some((collision, _)) => Some(collision),
            None => None,
        }
    }

    pub fn next_time(&self) -> Option<f32> {
        match self.nearest {
            Some((_, t)) => Some(t),
            None => None,
        }
    }

    pub fn insert(&mut self, collision: Collision, t: f32) {
        if t > 0.0 {
            match self.next_time() {
                Some(next_t) if next_t <= t => {}
                _ => {
                    self.nearest = Some((collision, t));
                }
            }
        } else {
            self.active.push(collision);
        }
    }

    pub fn clear(&mut self) {
        self.active.clear();
        self.nearest = None;
    }

    fn check_collider(
        &mut self,
        collider: &ColliderQueryItem,
        candidate_id: Entity,
        candidate: &ColliderQueryItem,
        threshold: f32,
    ) {
        if let Some(t) = collider_collision(
            collider.position() - candidate.position(),
            collider.velocity() - candidate.velocity(),
            collider.radius() + candidate.radius(),
        ) {
            if t < threshold {
                let position = collider.position_at(t);
                let target_position = candidate.position_at(t);
                let collision = Collision {
                    position,
                    normal: Dir2::new(position - target_position).unwrap_or(Dir2::X),
                    target: CollisionTarget::Collider {
                        id: candidate_id,
                        position: target_position,
                    },
                };
                self.insert(collision, t)
            }
        }
    }

    fn check_tile(
        &mut self,
        collider: &ColliderQueryItem,
        tile_colliders: &TileColliderLookup,
        tile_position: TilePosition,
        occupancy: TileOccupancy,
        delta_secs: f32,
    ) {
        if occupancy.contains(TileOccupancy::EAST) {
            let neighbor_pos = tile_position.with_offset(IVec2::new(1, 0));
            self.check_tile_edge(
                collider,
                tile_colliders.get(CompassOctant::East),
                neighbor_pos,
                Dir2::NEG_X,
                (tile_position.x() + 1) as f32 - collider.position().x,
                collider.velocity().x,
                collider.radius(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::NORTH) {
            let neighbor_pos = tile_position.with_offset(IVec2::new(0, 1));
            self.check_tile_edge(
                collider,
                tile_colliders.get(CompassOctant::North),
                neighbor_pos,
                Dir2::NEG_Y,
                (tile_position.y() + 1) as f32 - collider.position().y,
                collider.velocity().y,
                collider.radius(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::WEST) {
            let neighbor_pos = tile_position.with_offset(IVec2::new(-1, 0));
            self.check_tile_edge(
                collider,
                tile_colliders.get(CompassOctant::West),
                neighbor_pos,
                Dir2::X,
                collider.position().x - tile_position.x() as f32,
                -collider.velocity().x,
                collider.radius(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::SOUTH) {
            let neighbor_pos = tile_position.with_offset(IVec2::new(0, -1));
            self.check_tile_edge(
                collider,
                tile_colliders.get(CompassOctant::South),
                neighbor_pos,
                Dir2::Y,
                collider.position().y - tile_position.y() as f32,
                -collider.velocity().y,
                collider.radius(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::NORTH_EAST)
            && !occupancy.intersects(TileOccupancy::NORTH | TileOccupancy::EAST)
        {
            let neighbor_pos = tile_position.with_offset(IVec2::new(1, 1));
            self.check_tile_corner(
                collider,
                tile_colliders.get(CompassOctant::NorthEast),
                neighbor_pos,
                tile_position.position() + IVec2::ONE,
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::NORTH_WEST)
            && !occupancy.intersects(TileOccupancy::NORTH | TileOccupancy::WEST)
        {
            let neighbor_pos = tile_position.with_offset(IVec2::new(-1, 1));
            self.check_tile_corner(
                collider,
                tile_colliders.get(CompassOctant::NorthWest),
                neighbor_pos,
                tile_position.position() + IVec2::Y,
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::SOUTH_WEST)
            && !occupancy.intersects(TileOccupancy::SOUTH | TileOccupancy::WEST)
        {
            let neighbor_pos = tile_position.with_offset(IVec2::new(-1, -1));
            self.check_tile_corner(
                collider,
                tile_colliders.get(CompassOctant::SouthWest),
                neighbor_pos,
                tile_position.position(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::SOUTH_EAST)
            && !occupancy.intersects(TileOccupancy::SOUTH | TileOccupancy::EAST)
        {
            let neighbor_pos = tile_position.with_offset(IVec2::new(1, -1));
            self.check_tile_corner(
                collider,
                tile_colliders.get(CompassOctant::SouthEast),
                neighbor_pos,
                tile_position.position() + IVec2::X,
                delta_secs,
            );
        }
    }

    fn check_tile_edge(
        &mut self,
        collider: &ColliderQueryItem,
        tile_collider_id: Option<Entity>,
        tile_position: TilePosition,
        normal: Dir2,
        delta_position_component: f32,
        collider_velocity_component: f32,
        collider_radius: f32,
        delta_secs: f32,
    ) {
        if let Some(t) = wall_collision(
            delta_position_component,
            collider_velocity_component,
            collider_radius,
        ) {
            if t < delta_secs {
                let position = collider.position_at(t);
                let collision = Collision {
                    position,
                    normal,
                    target: CollisionTarget::Wall {
                        id: tile_collider_id,
                        position: tile_position,
                    },
                };
                self.insert(collision, t)
            }
        }
    }

    fn check_tile_corner(
        &mut self,
        collider: &ColliderQueryItem,
        tile_collider_id: Option<Entity>,
        tile_position: TilePosition,
        corner_position: IVec2,
        delta_secs: f32,
    ) {
        if let Some(t) = collider_collision(
            collider.position() - corner_position.as_vec2(),
            collider.velocity(),
            collider.radius(),
        ) {
            if t < delta_secs {
                let position = collider.position_at(t);
                let target_position = corner_position.as_vec2();
                let collision = Collision {
                    position,
                    normal: Dir2::new(position - target_position).unwrap_or(Dir2::X),
                    target: CollisionTarget::Wall {
                        id: tile_collider_id,
                        position: tile_position,
                    },
                };
                self.insert(collision, t)
            }
        }
    }
}

impl TileColliderLookup {
    fn new() -> Self {
        Self {
            entities: [None; 8],
        }
    }

    fn insert(&mut self, octant: CompassOctant, id: Entity) {
        self.entities[octant.to_index()] = Some(id);
    }

    fn get(&self, octant: CompassOctant) -> Option<Entity> {
        self.entities[octant.to_index()]
    }
}

fn collider_collision(
    delta_position: Vec2,
    delta_velocity: Vec2,
    combined_radius: f32,
) -> Option<f32> {
    let c = delta_position.length_squared() - combined_radius * combined_radius;
    if c < 0.0 {
        return Some(0.0);
    }

    let a = delta_velocity.length_squared();
    if a == 0.0 {
        return None;
    }

    let b = 2.0 * delta_position.dot(delta_velocity);
    let discr = b * b - 4.0 * a * c;
    if discr < 0.0 {
        return None;
    }

    let t = (-b - discr.sqrt()) / (2.0 * a);

    if t >= 0. { Some(t) } else { None }
}

fn wall_collision(distance: f32, speed: f32, radius: f32) -> Option<f32> {
    if distance <= radius {
        return Some(0.0);
    }

    if speed <= 0.0 {
        return None;
    }

    Some((distance - radius) / speed)
}

fn offset_to_octant(center: IVec2, collider: IVec2) -> Option<CompassOctant> {
    let direction = collider - center;
    match direction {
        IVec2 { x: 0, y: 1 } => Some(CompassOctant::North),
        IVec2 { x: 1, y: 1 } => Some(CompassOctant::NorthEast),
        IVec2 { x: 1, y: 0 } => Some(CompassOctant::East),
        IVec2 { x: 1, y: -1 } => Some(CompassOctant::SouthEast),
        IVec2 { x: 0, y: -1 } => Some(CompassOctant::South),
        IVec2 { x: -1, y: -1 } => Some(CompassOctant::SouthWest),
        IVec2 { x: -1, y: 0 } => Some(CompassOctant::West),
        IVec2 { x: -1, y: 1 } => Some(CompassOctant::NorthWest),
        _ => None,
    }
}
