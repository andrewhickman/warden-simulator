#[cfg(test)]
mod tests;

use std::mem;

use bevy_app::prelude::*;
use bevy_ecs::{batching::BatchingStrategy, prelude::*, query::QueryData};
use bevy_math::prelude::*;
use bevy_time::prelude::*;

use crate::{
    PhysicsSystems,
    kinematics::{GlobalPosition, GlobalVelocity},
    tile::{
        Tile, adjacency::Adjacency, index::TileIndex, position::TilePosition, storage::TileStorage,
    },
};

pub struct CollisionPlugin;

#[derive(Component, Clone, Copy, Debug)]
#[require(Collisions, TilePosition, GlobalPosition)]
pub struct Collider {
    radius: f32,
    solid: bool,
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Tile)]
pub struct TileCollider {
    solid: bool,
}

#[derive(QueryData, Debug)]
pub struct ColliderQuery {
    id: Entity,
    collider: &'static Collider,
    transform: &'static GlobalPosition,
    velocity: Option<&'static GlobalVelocity>,
}

#[derive(QueryData, Debug)]
#[query_data(derive(Clone, Copy, Debug))]
pub struct TileColliderQuery {
    id: Entity,
    collider: &'static TileCollider,
}

#[derive(Component, Clone, Debug, Default)]
pub struct Collisions {
    active: Vec<Collision>,
    previous: Vec<Collision>,
    nearest: Option<(Collision, f32)>,
}

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub position: Vec2,
    pub normal: Dir2,
    pub target: CollisionTarget,
    pub solid: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum CollisionTarget {
    Collider {
        id: Entity,
        position: Vec2,
    },
    Tile {
        id: Option<Entity>,
        position: TilePosition,
    },
}

static NEIGHBORS: [(IVec2, Adjacency); 9] = [
    (IVec2::new(-1, 1), Adjacency::NORTH_WEST),
    (IVec2::new(0, 1), Adjacency::NORTH),
    (IVec2::new(1, 1), Adjacency::NORTH_EAST),
    (IVec2::new(1, 0), Adjacency::EAST),
    (IVec2::new(1, -1), Adjacency::SOUTH_EAST),
    (IVec2::new(0, -1), Adjacency::SOUTH),
    (IVec2::new(-1, -1), Adjacency::SOUTH_WEST),
    (IVec2::new(-1, 0), Adjacency::WEST),
    (IVec2::new(0, 0), Adjacency::NONE),
];

#[derive(Default)]
struct TileColliderAdjacency<'w, 's> {
    tiles: [Option<TileColliderQueryItem<'w, 's>>; 8],
}

pub fn resolve_collisions(
    index: Res<TileIndex>,
    storage: TileStorage,
    mut colliders: Query<(ColliderQuery, &TilePosition, &mut Collisions)>,
    candidate_colliders: Query<ColliderQuery>,
    candidate_tiles: Query<TileColliderQuery>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();

    colliders
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(16))
        .for_each(|(collider, &tile_position, mut collisions)| {
            collisions.clear();

            let mut tile_colliders = TileColliderAdjacency::default();
            let mut wall_adjacency = storage.get_wall_adjacency(tile_position);

            for (neighbor_index, &(neighbor_offset, adjacency)) in NEIGHBORS.iter().enumerate() {
                let neighbor = tile_position.with_offset(neighbor_offset.x, neighbor_offset.y);
                let index_entry = match index.get(neighbor) {
                    Some(entry) => entry,
                    None => continue,
                };

                for &candidate in index_entry.objects() {
                    if candidate == collider.id {
                        continue;
                    }

                    let Ok(candidate_collider) = candidate_colliders.get(candidate) else {
                        continue;
                    };

                    collisions.check_collider(&collider, &candidate_collider, delta_secs);
                }

                if let Some(tile_collider_slot) = tile_colliders.tiles.get_mut(neighbor_index)
                    && let Some(candidate) = index_entry.tile()
                    && let Ok(candidate_tile) = candidate_tiles.get(candidate)
                {
                    wall_adjacency |= adjacency;
                    *tile_collider_slot = Some(candidate_tile);
                }
            }

            if wall_adjacency != Adjacency::NONE {
                collisions.check_tile(
                    &collider,
                    &tile_colliders,
                    tile_position,
                    wall_adjacency,
                    delta_secs,
                );
            }
        });
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            PhysicsSystems::Collisions.before(PhysicsSystems::Kinematics),
        );

        app.add_systems(
            FixedUpdate,
            resolve_collisions.in_set(PhysicsSystems::Collisions),
        );
    }
}

impl Collider {
    pub fn new(radius: f32, solid: bool) -> Self {
        Self { radius, solid }
    }

    pub fn solid(&self) -> bool {
        self.solid
    }

    pub fn set_solid(&mut self, solid: bool) {
        self.solid = solid;
    }
}

impl ColliderQueryItem<'_, '_> {
    pub fn radius(&self) -> f32 {
        self.collider.radius
    }

    pub fn position(&self) -> Vec2 {
        self.transform.position()
    }

    pub fn position_at(&self, t: f32) -> Vec2 {
        match self.velocity {
            Some(velocity) if t > 0.0 => self.transform.position() + velocity.linear() * t,
            _ => self.transform.position(),
        }
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity.map_or(Vec2::ZERO, |v| v.linear())
    }

    pub fn solid(&self) -> bool {
        self.collider.solid
    }
}

impl TileCollider {
    pub fn new(solid: bool) -> Self {
        Self { solid }
    }

    pub fn solid(&self) -> bool {
        self.solid
    }

    pub fn set_solid(&mut self, solid: bool) {
        self.solid = solid;
    }
}

impl Default for TileCollider {
    fn default() -> Self {
        Self { solid: true }
    }
}

impl TileColliderQueryItem<'_, '_> {
    pub fn solid(&self) -> bool {
        self.collider.solid
    }
}

impl Collisions {
    pub fn active(&self) -> impl ExactSizeIterator<Item = Collision> {
        self.active.iter().copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = Collision> {
        self.active().chain(self.next_collision())
    }

    pub fn previous(&self) -> impl ExactSizeIterator<Item = Collision> + '_ {
        self.previous.iter().copied()
    }

    pub fn next(&self) -> Option<(Collision, f32)> {
        self.nearest
    }

    pub fn next_collision(&self) -> Option<Collision> {
        self.nearest.map(|(collision, _)| collision)
    }

    pub fn next_time(&self) -> Option<f32> {
        self.nearest.map(|(_, t)| t)
    }

    pub fn insert(&mut self, collision: Collision, t: f32) {
        if collision.solid && t > 0.0 {
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
        self.previous.clear();
        mem::swap(&mut self.previous, &mut self.active);
        if let Some((collision, _)) = self.nearest.take() {
            self.previous.push(collision);
        }
    }

    pub fn started(&self) -> impl Iterator<Item = Collision> + '_ {
        self.iter()
            .filter(|ac| !self.previous().any(|pc| pc.target.contains(&ac.target)))
    }

    pub fn ended(&self) -> impl Iterator<Item = Collision> + '_ {
        self.previous()
            .filter(|pc| !self.iter().any(|ac| ac.target.contains(&pc.target)))
    }

    fn check_collider(
        &mut self,
        collider: &ColliderQueryItem,
        candidate: &ColliderQueryItem,
        threshold: f32,
    ) {
        if let Some(t) = collider_collision(
            collider.position() - candidate.position(),
            collider.velocity() - candidate.velocity(),
            collider.radius() + candidate.radius(),
        ) && t < threshold
        {
            let position = collider.position_at(t);
            let target_position = candidate.position_at(t);
            let collision = Collision {
                position,
                normal: Dir2::new(position - target_position).unwrap_or(Dir2::X),
                target: CollisionTarget::Collider {
                    id: candidate.id,
                    position: target_position,
                },
                solid: collider.solid() && candidate.solid(),
            };
            self.insert(collision, t)
        }
    }

    fn check_tile(
        &mut self,
        collider: &ColliderQueryItem,
        candidates: &TileColliderAdjacency,
        tile_position: TilePosition,
        adjacency: Adjacency,
        delta_secs: f32,
    ) {
        if adjacency.contains(Adjacency::EAST) {
            let candidate_position = tile_position.east();
            self.check_tile_edge(
                collider,
                candidates.east(),
                candidate_position,
                Dir2::NEG_X,
                candidate_position.x() as f32 - collider.position().x,
                collider.velocity().x,
                collider.radius(),
                delta_secs,
            );
        }

        if adjacency.contains(Adjacency::NORTH) {
            let candidate_position = tile_position.north();
            self.check_tile_edge(
                collider,
                candidates.north(),
                candidate_position,
                Dir2::NEG_Y,
                candidate_position.y() as f32 - collider.position().y,
                collider.velocity().y,
                collider.radius(),
                delta_secs,
            );
        }

        if adjacency.contains(Adjacency::WEST) {
            let candidate_position = tile_position.west();
            self.check_tile_edge(
                collider,
                candidates.west(),
                candidate_position,
                Dir2::X,
                collider.position().x - tile_position.x() as f32,
                -collider.velocity().x,
                collider.radius(),
                delta_secs,
            );
        }

        if adjacency.contains(Adjacency::SOUTH) {
            let candidate_position = tile_position.south();
            self.check_tile_edge(
                collider,
                candidates.south(),
                candidate_position,
                Dir2::Y,
                collider.position().y - tile_position.y() as f32,
                -collider.velocity().y,
                collider.radius(),
                delta_secs,
            );
        }

        if adjacency.contains(Adjacency::NORTH_EAST)
            && !adjacency.intersects(Adjacency::NORTH | Adjacency::EAST)
        {
            let candidate_position = tile_position.north().east();
            self.check_tile_corner(
                collider,
                candidates.north_east(),
                candidate_position,
                candidate_position.position(),
                delta_secs,
            );
        }

        if adjacency.contains(Adjacency::NORTH_WEST)
            && !adjacency.intersects(Adjacency::NORTH | Adjacency::WEST)
        {
            let candidate_position = tile_position.north().west();
            self.check_tile_corner(
                collider,
                candidates.north_west(),
                candidate_position,
                IVec2::new(tile_position.x(), candidate_position.y()),
                delta_secs,
            );
        }

        if adjacency.contains(Adjacency::SOUTH_WEST)
            && !adjacency.intersects(Adjacency::SOUTH | Adjacency::WEST)
        {
            let candidate_position = tile_position.south().west();
            self.check_tile_corner(
                collider,
                candidates.south_west(),
                candidate_position,
                tile_position.position(),
                delta_secs,
            );
        }

        if adjacency.contains(Adjacency::SOUTH_EAST)
            && !adjacency.intersects(Adjacency::SOUTH | Adjacency::EAST)
        {
            let candidate_position = tile_position.south().east();
            self.check_tile_corner(
                collider,
                candidates.south_east(),
                candidate_position,
                IVec2::new(candidate_position.x(), tile_position.y()),
                delta_secs,
            );
        }
    }

    fn check_tile_edge(
        &mut self,
        collider: &ColliderQueryItem,
        candidate: Option<&TileColliderQueryItem>,
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
        ) && t < delta_secs
        {
            let position = collider.position_at(t);
            let (id, solid) = match candidate {
                Some(candidate) => (Some(candidate.id), collider.solid() && candidate.solid()),
                None => (None, collider.solid()),
            };

            let collision = Collision {
                position,
                normal,
                target: CollisionTarget::Tile {
                    id,
                    position: tile_position,
                },
                solid,
            };
            self.insert(collision, t)
        }
    }

    fn check_tile_corner(
        &mut self,
        collider: &ColliderQueryItem,
        candidate: Option<&TileColliderQueryItem>,
        tile_position: TilePosition,
        corner_position: IVec2,
        delta_secs: f32,
    ) {
        if let Some(t) = collider_collision(
            collider.position() - corner_position.as_vec2(),
            collider.velocity(),
            collider.radius(),
        ) && t < delta_secs
        {
            let position = collider.position_at(t);
            let target_position = corner_position.as_vec2();
            let (id, solid) = match candidate {
                Some(candidate) => (Some(candidate.id), collider.solid() && candidate.solid()),
                None => (None, collider.solid()),
            };

            let collision = Collision {
                position,
                normal: Dir2::new(position - target_position).unwrap_or(Dir2::X),
                target: CollisionTarget::Tile {
                    id,
                    position: tile_position,
                },
                solid,
            };
            self.insert(collision, t)
        }
    }
}

impl CollisionTarget {
    pub fn contains(&self, other: &Self) -> bool {
        match (self, other) {
            (
                CollisionTarget::Collider { id: id1, .. },
                CollisionTarget::Collider { id: id2, .. },
            ) => id1 == id2,
            (
                CollisionTarget::Tile {
                    id: Some(_),
                    position: pos1,
                },
                CollisionTarget::Tile {
                    id: None,
                    position: pos2,
                },
            ) => pos1 == pos2,
            (
                CollisionTarget::Tile {
                    id: Some(id1),
                    position: pos1,
                },
                CollisionTarget::Tile {
                    id: Some(id2),
                    position: pos2,
                },
            ) => pos1 == pos2 && id1 == id2,
            _ => false,
        }
    }
}

impl<'w, 's> TileColliderAdjacency<'w, 's> {
    fn north_west(&self) -> Option<&TileColliderQueryItem<'w, 's>> {
        self.tiles[0].as_ref()
    }

    fn north(&self) -> Option<&TileColliderQueryItem<'w, 's>> {
        self.tiles[1].as_ref()
    }

    fn north_east(&self) -> Option<&TileColliderQueryItem<'w, 's>> {
        self.tiles[2].as_ref()
    }

    fn east(&self) -> Option<&TileColliderQueryItem<'w, 's>> {
        self.tiles[3].as_ref()
    }

    fn south_east(&self) -> Option<&TileColliderQueryItem<'w, 's>> {
        self.tiles[4].as_ref()
    }

    fn south(&self) -> Option<&TileColliderQueryItem<'w, 's>> {
        self.tiles[5].as_ref()
    }

    fn south_west(&self) -> Option<&TileColliderQueryItem<'w, 's>> {
        self.tiles[6].as_ref()
    }

    fn west(&self) -> Option<&TileColliderQueryItem<'w, 's>> {
        self.tiles[7].as_ref()
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
