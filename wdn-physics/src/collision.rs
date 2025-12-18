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
#[require(Transform, Collisions, TilePosition)]
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
            FixedUpdate,
            resolve_collisions
                .in_set(PhysicsSystems::ResolveCollisions)
                .after(PhysicsSystems::UpdateTile),
        );
    }
}

pub fn resolve_collisions(
    index: Res<TileIndex>,
    storage: TileStorage,
    mut colliders: Query<(Entity, ColliderQuery, &TilePosition, &mut Collisions)>,
    candidates: Query<ColliderQuery>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();

    colliders
        .par_iter_mut()
        .for_each(|(collider_id, collider, &tile_position, mut collisions)| {
            collisions.clear();

            index.get_neighborhood(tile_position).for_each(|candidate| {
                if candidate == collider_id {
                    return;
                }

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

    pub fn check_collider(
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
                (tile_position.x() + 1) as f32 - collider.position().x,
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
                (tile_position.y() + 1) as f32 - collider.position().y,
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
                collider.position().x - tile_position.x() as f32,
                -collider.velocity().x,
                collider.radius(),
                delta_secs,
            );
        }

        if occupancy.contains(TileOccupancy::SOUTH) {
            self.check_tile_edge(
                collider,
                tile_position.with_offset(IVec2::new(0, -1)),
                Vec2::Y,
                collider.position().y - tile_position.y() as f32,
                -collider.velocity().y,
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

#[cfg(test)]
mod tests {
    use std::{f32::consts::FRAC_1_SQRT_2, time::Duration};

    use approx::assert_relative_eq;
    use bevy::{ecs::system::RunSystemOnce, prelude::*, time::TimeUpdateStrategy};

    use crate::{
        collision::{Collider, CollisionPlugin, CollisionTarget, Collisions},
        integrate::Velocity,
        tile::{
            TilePlugin, TilePosition,
            storage::{TileLayer, TileMaterial, TileStorageMut},
        },
    };

    #[test]
    fn collision_empty() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.5,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert!(collisions.next().is_none());
        assert!(collisions.next_time().is_none());
    }

    #[test]
    fn collision_collider() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.4, 0.1),
            Vec2::new(-0.5, 0.2),
            0.05,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(-0.1, 0.1),
            Vec2::new(0.5, 0.2),
            0.05,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 0);
        assert_relative_eq!(collisions1.next_time().unwrap(), 0.4);
        let collision1 = collisions1.next().unwrap();
        assert_relative_eq!(collision1.position, Vec2::new(0.2, 0.18));
        assert_relative_eq!(collision1.normal, Vec2::X);
        match collision1.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity2);
                assert_relative_eq!(position, Vec2::new(0.1, 0.18));
            }
            _ => panic!("Expected collider collision"),
        }

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 0);
        assert_relative_eq!(collisions2.next_time().unwrap(), 0.4);
        let collision2 = collisions2.next().unwrap();
        assert_relative_eq!(collision2.position, Vec2::new(0.1, 0.18));
        assert_relative_eq!(collision2.normal, Vec2::NEG_X);
        match collision2.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity1);
                assert_relative_eq!(position, Vec2::new(0.2, 0.18));
            }
            _ => panic!("Expected collider collision"),
        }
    }

    #[test]
    fn collision_collider_receding() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.4, 0.1),
            Vec2::new(0.5, 0.2),
            0.05,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(-0.1, 0.1),
            Vec2::new(-0.5, 0.2),
            0.05,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 0);
        assert!(collisions1.next_time().is_none());
        assert!(collisions1.next().is_none());

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 0);
        assert!(collisions2.next_time().is_none());
        assert!(collisions2.next().is_none());
    }

    #[test]
    fn collision_collider_touching() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.4, 0.1),
            Vec2::new(0.0, 0.0),
            0.1,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.2, 0.1),
            Vec2::new(0.0, 0.0),
            0.1,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 0);
        assert!(collisions1.next_time().is_none());
        assert!(collisions1.next().is_none());

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 0);
        assert!(collisions2.next_time().is_none());
        assert!(collisions2.next().is_none());
    }

    #[test]
    fn collision_collider_touching_and_receding() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.4, 0.1),
            Vec2::new(0.5, 0.2),
            0.1,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.2, 0.1),
            Vec2::new(-0.5, 0.2),
            0.1,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 0);
        assert!(collisions1.next_time().is_none());
        assert!(collisions1.next().is_none());

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 0);
        assert!(collisions2.next_time().is_none());
        assert!(collisions2.next().is_none());
    }

    #[test]
    fn collision_collider_touching_and_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.4, 0.1),
            Vec2::new(-0.5, 0.2),
            0.1,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.2, 0.1),
            Vec2::new(0.5, 0.2),
            0.1,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 1);
        assert!(collisions1.next_time().is_none());
        assert!(collisions1.next().is_none());
        let collision1 = collisions1.active().next().unwrap();
        assert_relative_eq!(collision1.position, Vec2::new(0.4, 0.1));
        assert_relative_eq!(collision1.normal, Vec2::X);
        match collision1.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity2);
                assert_relative_eq!(position, Vec2::new(0.2, 0.1));
            }
            _ => panic!("Expected collider collision"),
        }

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 1);
        assert!(collisions2.next_time().is_none());
        assert!(collisions2.next().is_none());
        let collision2 = collisions2.active().next().unwrap();
        assert_relative_eq!(collision2.position, Vec2::new(0.2, 0.1));
        assert_relative_eq!(collision2.normal, Vec2::NEG_X);
        match collision2.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity1);
                assert_relative_eq!(position, Vec2::new(0.4, 0.1));
            }
            _ => panic!("Expected collider collision"),
        }
    }

    #[test]
    fn collision_collider_intersecting() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.2, 0.1),
            Vec2::new(0.0, 0.0),
            0.1,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.15, 0.1),
            Vec2::new(0.0, 0.0),
            0.1,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 1);
        assert!(collisions1.next_time().is_none());
        assert!(collisions1.next().is_none());
        let collision1 = collisions1.active().next().unwrap();
        assert_relative_eq!(collision1.position, Vec2::new(0.2, 0.1));
        assert_relative_eq!(collision1.normal, Vec2::X);
        match collision1.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity2);
                assert_relative_eq!(position, Vec2::new(0.15, 0.1));
            }
            _ => panic!("Expected collider collision"),
        }

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 1);
        assert!(collisions2.next_time().is_none());
        assert!(collisions2.next().is_none());
        let collision2 = collisions2.active().next().unwrap();
        assert_relative_eq!(collision2.position, Vec2::new(0.15, 0.1));
        assert_relative_eq!(collision2.normal, Vec2::NEG_X);
        match collision2.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity1);
                assert_relative_eq!(position, Vec2::new(0.2, 0.1));
            }
            _ => panic!("Expected collider collision"),
        }
    }

    #[test]
    fn collision_collider_intersecting_and_receding() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.2, 0.1),
            Vec2::new(0.5, 0.0),
            0.1,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.15, 0.1),
            Vec2::new(-0.5, 0.0),
            0.1,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 1);
        assert!(collisions1.next_time().is_none());
        assert!(collisions1.next().is_none());
        let collision1 = collisions1.active().next().unwrap();
        assert_relative_eq!(collision1.position, Vec2::new(0.2, 0.1));
        assert_relative_eq!(collision1.normal, Vec2::X);
        match collision1.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity2);
                assert_relative_eq!(position, Vec2::new(0.15, 0.1));
            }
            _ => panic!("Expected collider collision"),
        }

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 1);
        assert!(collisions2.next_time().is_none());
        assert!(collisions2.next().is_none());
        let collision2 = collisions2.active().next().unwrap();
        assert_relative_eq!(collision2.position, Vec2::new(0.15, 0.1));
        assert_relative_eq!(collision2.normal, Vec2::NEG_X);
        match collision2.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity1);
                assert_relative_eq!(position, Vec2::new(0.2, 0.1));
            }
            _ => panic!("Expected collider collision"),
        }
    }

    #[test]
    fn collision_collider_intersecting_and_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.2, 0.1),
            Vec2::new(-0.5, 0.0),
            0.1,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.15, 0.1),
            Vec2::new(0.5, 0.0),
            0.1,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 1);
        assert!(collisions1.next_time().is_none());
        assert!(collisions1.next().is_none());
        let collision1 = collisions1.active().next().unwrap();
        assert_relative_eq!(collision1.position, Vec2::new(0.2, 0.1));
        assert_relative_eq!(collision1.normal, Vec2::X);
        match collision1.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity2);
                assert_relative_eq!(position, Vec2::new(0.15, 0.1));
            }
            _ => panic!("Expected collider collision"),
        }

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 1);
        assert!(collisions2.next_time().is_none());
        assert!(collisions2.next().is_none());
        let collision2 = collisions2.active().next().unwrap();
        assert_relative_eq!(collision2.position, Vec2::new(0.15, 0.1));
        assert_relative_eq!(collision2.normal, Vec2::NEG_X);
        match collision2.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity1);
                assert_relative_eq!(position, Vec2::new(0.2, 0.1));
            }
            _ => panic!("Expected collider collision"),
        }
    }

    #[test]
    fn collision_collider_angled() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.4, 0.3),
            Vec2::new(-0.5, 0.0),
            0.05,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.1, 0.22),
            Vec2::new(0.5, 0.0),
            0.05,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 0);
        assert_relative_eq!(collisions1.next_time().unwrap(), 0.24, epsilon = 0.0001);
        let collision1 = collisions1.next().unwrap();
        assert_relative_eq!(collision1.position, Vec2::new(0.28, 0.3), epsilon = 0.0001);
        assert_relative_eq!(collision1.normal, Vec2::new(0.6, 0.8), epsilon = 0.0001);
        match collision1.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity2);
                assert_relative_eq!(position, Vec2::new(0.22, 0.22), epsilon = 0.0001);
                assert_relative_eq!(
                    collision1.position.distance(position),
                    0.1,
                    epsilon = 0.0001,
                );
            }
            _ => panic!("Expected collider collision"),
        }

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 0);
        assert_relative_eq!(collisions2.next_time().unwrap(), 0.24, epsilon = 0.0001);
        let collision2 = collisions2.next().unwrap();
        assert_relative_eq!(collision2.position, Vec2::new(0.22, 0.22), epsilon = 0.0001);
        assert_relative_eq!(collision2.normal, Vec2::new(-0.6, -0.8), epsilon = 0.0001);
        match collision2.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity1);
                assert_relative_eq!(position, Vec2::new(0.28, 0.3), epsilon = 0.0001);
                assert_relative_eq!(
                    collision2.position.distance(position),
                    0.1,
                    epsilon = 0.0001,
                );
            }
            _ => panic!("Expected collider collision"),
        }
    }

    #[test]
    fn collision_collider_almost_touching_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        let t = 1e-6f32;
        let entity1 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.2 + t, 0.0),
            Vec2::new(-0.5, 0.0),
            0.1,
        );
        let entity2 = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.0, 0.0),
            Vec2::new(0.5, 0.0),
            0.1,
        );

        app.update();

        let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
        assert_eq!(collisions1.active().len(), 0);
        assert_relative_eq!(collisions1.next_time().unwrap(), t);
        let collision1 = collisions1.next().unwrap();
        assert_relative_eq!(collision1.position, Vec2::new(0.2 + t / 2.0, 0.0));
        assert_relative_eq!(collision1.normal, Vec2::X);
        match collision1.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity2);
                assert_relative_eq!(position, Vec2::new(t / 2.0, 0.0));
            }
            _ => panic!("Expected collider collision"),
        }

        let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
        assert_eq!(collisions2.active().len(), 0);
        assert_relative_eq!(collisions2.next_time().unwrap(), t);
        let collision2 = collisions2.next().unwrap();
        assert_relative_eq!(collision2.position, Vec2::new(t / 2.0, 0.0));
        assert_relative_eq!(collision2.normal, Vec2::NEG_X);
        match collision2.target {
            CollisionTarget::Collider { id, position } => {
                assert_eq!(id, entity1);
                assert_relative_eq!(position, Vec2::new(0.2 + t / 2.0, 0.0));
            }
            _ => panic!("Expected collider collision"),
        }
    }

    #[test]
    fn collision_wall_north_receding() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 0, 1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.0, 0.8),
            Vec2::new(0.0, -0.5),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
    }

    #[test]
    fn collision_wall_north_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 0, 1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.0, 0.8),
            Vec2::new(0.0, 0.5),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
        let collision = collisions.next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.0, 0.9));
        assert_relative_eq!(collision.normal, Vec2::NEG_Y);
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 0, 1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_wall_north_intersecting() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 0, 1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.0, 0.9),
            Vec2::new(0.0, 0.0),
            0.2,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 1);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
        let collision = collisions.active().next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.0, 0.9));
        assert_relative_eq!(collision.normal, Vec2::NEG_Y);
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 0, 1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_wall_south_receding() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 0, -1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.0, 0.2),
            Vec2::new(0.0, 0.5),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
    }

    #[test]
    fn collision_wall_south_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 0, -1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.0, 0.2),
            Vec2::new(0.0, -0.5),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
        let collision = collisions.next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.0, 0.1));
        assert_relative_eq!(collision.normal, Vec2::Y);
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 0, -1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_wall_south_intersecting() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 0, -1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.0, 0.1),
            Vec2::new(0.0, 0.0),
            0.2,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 1);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
        let collision = collisions.active().next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.0, 0.1));
        assert_relative_eq!(collision.normal, Vec2::Y);
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 0, -1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_wall_east_receding() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 1, 0));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.8, 0.0),
            Vec2::new(-0.5, 0.0),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
    }

    #[test]
    fn collision_wall_east_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 1, 0));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.8, 0.0),
            Vec2::new(0.5, 0.0),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
        let collision = collisions.next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.9, 0.0));
        assert_relative_eq!(collision.normal, Vec2::NEG_X);
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 1, 0));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_wall_east_intersecting() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 1, 0));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.9, 0.0),
            Vec2::new(0.0, 0.0),
            0.2,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 1);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
        let collision = collisions.active().next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.9, 0.0));
        assert_relative_eq!(collision.normal, Vec2::NEG_X);
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 1, 0));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_wall_west_receding() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, -1, 0));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.2, 0.0),
            Vec2::new(0.5, 0.0),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
    }

    #[test]
    fn collision_wall_west_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, -1, 0));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.2, 0.0),
            Vec2::new(-0.5, 0.0),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
        let collision = collisions.next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.1, 0.0));
        assert_relative_eq!(collision.normal, Vec2::X);
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, -1, 0));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_wall_west_intersecting() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, -1, 0));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.1, 0.0),
            Vec2::new(0.0, 0.0),
            0.2,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 1);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
        let collision = collisions.active().next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.1, 0.0));
        assert_relative_eq!(collision.normal, Vec2::X);
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, -1, 0));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_corner_north_east_receding() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 1, 1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.85, 0.85),
            Vec2::new(-0.5, -0.5),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
    }

    #[test]
    fn collision_corner_north_east_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 1, 1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.7, 0.7),
            Vec2::new(0.5, 0.5),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert_eq!(collisions.next_time().unwrap(), 0.45857865);
        let collision = collisions.next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.92928934, 0.92928934));
        assert_relative_eq!(
            collision.normal,
            Vec2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
            epsilon = 0.0001
        );
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 1, 1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_corner_north_east_intersecting() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 1, 1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.95, 0.95),
            Vec2::new(0.0, 0.0),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 1);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
        let collision = collisions.active().next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.95, 0.95), epsilon = 0.0001);
        assert_relative_eq!(
            collision.normal,
            Vec2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
            epsilon = 0.0001
        );
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 1, 1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_corner_north_west_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, -1, 1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.3, 0.7),
            Vec2::new(-0.5, 0.5),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert_eq!(collisions.next_time().unwrap(), 0.45857865);
        let collision = collisions.next().unwrap();
        assert_relative_eq!(
            collision.position,
            Vec2::new(0.07071069, 0.92928934),
            epsilon = 0.0001
        );
        assert_relative_eq!(
            collision.normal,
            Vec2::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
            epsilon = 0.0001
        );
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, -1, 1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_corner_north_west_intersecting() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, -1, 1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.05, 0.95),
            Vec2::new(0.0, 0.0),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 1);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
        let collision = collisions.active().next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.05, 0.95), epsilon = 0.0001);
        assert_relative_eq!(
            collision.normal,
            Vec2::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
            epsilon = 0.0001
        );
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, -1, 1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_corner_south_west_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, -1, -1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.3, 0.3),
            Vec2::new(-0.5, -0.5),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert_eq!(collisions.next_time().unwrap(), 0.45857865);
        let collision = collisions.next().unwrap();
        assert_relative_eq!(
            collision.position,
            Vec2::new(0.07071069, 0.07071069),
            epsilon = 0.0001
        );
        assert_relative_eq!(
            collision.normal,
            Vec2::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
            epsilon = 0.0001
        );
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, -1, -1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_corner_south_west_intersecting() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, -1, -1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.05, 0.05),
            Vec2::new(0.0, 0.0),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 1);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
        let collision = collisions.active().next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.05, 0.05), epsilon = 0.0001);
        assert_relative_eq!(
            collision.normal,
            Vec2::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
            epsilon = 0.0001
        );
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, -1, -1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_corner_south_east_closing() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 1, -1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.7, 0.3),
            Vec2::new(0.5, -0.5),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert_eq!(collisions.next_time().unwrap(), 0.45857865);
        let collision = collisions.next().unwrap();
        assert_relative_eq!(
            collision.position,
            Vec2::new(0.92928934, 0.07071069),
            epsilon = 0.0001
        );
        assert_relative_eq!(
            collision.normal,
            Vec2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
            epsilon = 0.0001
        );
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 1, -1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_corner_south_east_intersecting() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 1, -1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.95, 0.05),
            Vec2::new(0.0, 0.0),
            0.1,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 1);
        assert!(collisions.next_time().is_none());
        assert!(collisions.next().is_none());
        let collision = collisions.active().next().unwrap();
        assert_relative_eq!(collision.position, Vec2::new(0.95, 0.05), epsilon = 0.0001);
        assert_relative_eq!(
            collision.normal,
            Vec2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
            epsilon = 0.0001
        );
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 1, -1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    #[test]
    fn collision_corner_angled() {
        let mut app = make_app();
        let layer = spawn_layer(&mut app);

        set_tile(&mut app, TilePosition::new(layer, 1, 1));

        let entity = spawn_entity(
            &mut app,
            layer,
            Vec2::new(0.31, 0.6),
            Vec2::new(1.0, 0.5),
            0.05,
        );

        app.update();

        let collisions = app.world().get::<Collisions>(entity).unwrap();
        assert_eq!(collisions.active().len(), 0);
        assert_relative_eq!(collisions.next_time().unwrap(), 0.7040017, epsilon = 0.0001);
        let collision = collisions.next().unwrap();
        assert_relative_eq!(
            collision.position,
            Vec2::new(1.0140017, 0.95200086),
            epsilon = 0.0001
        );
        assert_relative_eq!(
            collision.normal,
            Vec2::new(0.28003645, -0.95998937),
            epsilon = 0.0001
        );
        match collision.target {
            CollisionTarget::Wall { position } => {
                assert_eq!(position, TilePosition::new(layer, 1, 1));
            }
            _ => panic!("Expected wall collision"),
        }
    }

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TilePlugin, CollisionPlugin));

        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs(1)));
        app.insert_resource(Time::<Virtual>::from_max_delta(Duration::MAX));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));

        app.world_mut()
            .resource_mut::<Time<Real>>()
            .update_with_duration(Duration::ZERO);

        app
    }

    fn spawn_layer(app: &mut App) -> Entity {
        app.world_mut().spawn(TileLayer {}).id()
    }

    fn spawn_entity(
        app: &mut App,
        layer: Entity,
        position: Vec2,
        velocity: Vec2,
        radius: f32,
    ) -> Entity {
        app.world_mut()
            .spawn((
                Collider { radius },
                Transform::from_translation(position.extend(0.0)),
                Velocity::new(velocity),
                ChildOf(layer),
            ))
            .id()
    }

    fn set_tile(app: &mut App, position: TilePosition) {
        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(position, TileMaterial::Wall);
            })
            .unwrap();
    }
}
