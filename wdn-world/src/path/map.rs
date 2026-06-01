use std::{cmp::Ordering, collections::BinaryHeap};

use bevy_ecs::{entity::EntityHashMap, prelude::*};
use bevy_log::warn;
use bevy_math::{FloatOrd, FloatPow, prelude::*};
use bevy_platform::collections::{HashMap, HashSet, hash_map};
use wdn_physics::tile::{
    adjacency::Adjacency, index::TileIndex, position::TilePosition, storage::TileStorage,
};

use crate::path::region::{Region, TileChunkSections};

#[derive(Component, Default)]
pub struct RegionDoors {
    doors: EntityHashMap<Entity>,
}

#[derive(Component, Default)]
pub struct DoorRegions {
    north: Option<(Entity, Entity)>,
    south: Option<(Entity, Entity)>,
    east: Option<(Entity, Entity)>,
    west: Option<(Entity, Entity)>,
}

#[derive(Component, Debug)]
pub struct FlowField {
    door_position: TilePosition,
    door_adjacency: Adjacency,
    flow: HashMap<TilePosition, Dir2>,
}

pub fn update_region_doors(
    mut commands: Commands,
    index: Res<TileIndex>,
    mut regions: Query<(Entity, &Region, &mut RegionDoors), Added<Region>>,
    sections: Query<&TileChunkSections>,
    mut doors: Query<&mut DoorRegions>,
    mut flow_map: Local<EntityHashMap<FlowField>>,
) {
    regions
        .iter_mut()
        .for_each(|(region_id, region, mut region_doors)| {
            for (chunk_id, section_id) in region.sections() {
                let section = sections
                    .get(chunk_id)
                    .expect("chunk not found")
                    .section(section_id.chunk_offset());

                for (door_position, door_adjacency) in section.doors() {
                    let Some(id) = index.get_tile(door_position) else {
                        warn!("door at {:?} not found", door_position);
                        continue;
                    };

                    match flow_map.entry(id) {
                        hash_map::Entry::Occupied(mut entry) => {
                            entry.get_mut().door_adjacency.insert(door_adjacency);
                        }
                        hash_map::Entry::Vacant(entry) => {
                            entry.insert(FlowField {
                                door_position,
                                door_adjacency,
                                flow: HashMap::default(),
                            });
                        }
                    }
                }
            }

            region_doors.doors.reserve(flow_map.len());
            for (door_id, flow) in flow_map.drain() {
                let adjacency = flow.door_adjacency;
                let flow_id = commands.spawn((ChildOf(region_id), flow)).id();

                doors
                    .get_mut(door_id)
                    .expect("door not found")
                    .insert(region_id, flow_id, adjacency);
                region_doors.doors.insert(door_id, flow_id);
            }
        });
}

pub fn update_flow_fields(
    storage: TileStorage,
    regions: Query<&Region>,
    mut flow_fields: Query<(&ChildOf, &mut FlowField), Added<FlowField>>,
) {
    flow_fields.par_iter_mut().for_each(|(parent, mut flow)| {
        let region = regions.get(parent.parent()).expect("region not found");
        flow.generate_flow_field(&storage, region.size());
    });
}

pub fn on_remove_region_doors(
    trigger: On<Remove, RegionDoors>,
    regions: Query<&RegionDoors>,
    mut doors: Query<&mut DoorRegions>,
) -> Result {
    let region = regions.get(trigger.entity)?;
    for door_id in region.doors() {
        if let Ok(mut door) = doors.get_mut(door_id) {
            remove_door_region(&mut door.north, trigger.entity);
            remove_door_region(&mut door.south, trigger.entity);
            remove_door_region(&mut door.east, trigger.entity);
            remove_door_region(&mut door.west, trigger.entity);
        }
    }

    Ok(())
}

impl RegionDoors {
    pub fn doors(&self) -> impl Iterator<Item = Entity> {
        self.doors.keys().copied()
    }
}

impl DoorRegions {
    pub fn insert(&mut self, region: Entity, flow: Entity, adjacency: Adjacency) {
        if adjacency.contains(Adjacency::NORTH) {
            debug_assert!(self.north.is_none());
            self.north = Some((region, flow));
        }

        if adjacency.contains(Adjacency::SOUTH) {
            debug_assert!(self.south.is_none());
            self.south = Some((region, flow));
        }

        if adjacency.contains(Adjacency::EAST) {
            debug_assert!(self.east.is_none());
            self.east = Some((region, flow));
        }

        if adjacency.contains(Adjacency::WEST) {
            debug_assert!(self.west.is_none());
            self.west = Some((region, flow));
        }
    }
}

impl FlowField {
    fn generate_flow_field(&mut self, storage: &TileStorage, size_hint: usize) {
        let distance = generate_distance_field(
            storage,
            self.door_position,
            self.door_adjacency,
            size_hint + 1,
        );
        debug_assert_eq!(distance.len(), size_hint + 1);

        self.flow.reserve(size_hint);
        for (&tile, &(cost, adjacency)) in &distance {
            if tile == self.door_position {
                continue;
            }

            self.flow
                .insert(tile, flow_vector(&distance, tile, cost, adjacency));
        }

        debug_assert_eq!(self.flow.len(), size_hint);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Node {
    cost: FloatOrd,
    position: TilePosition,
    adjacency: Adjacency,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn generate_distance_field(
    storage: &TileStorage,
    door: TilePosition,
    door_adjacency: Adjacency,
    size_hint: usize,
) -> HashMap<TilePosition, (f32, Adjacency)> {
    let mut open = BinaryHeap::new();
    let mut distance = HashMap::with_capacity(size_hint);
    let mut accepted = HashSet::with_capacity(size_hint);

    distance.insert(door, (0.0, Adjacency::NONE));
    accepted.insert(door);

    visit_door_neighbors(door, door_adjacency, |neighbor, adjacency| {
        let Some(neighbor_data) = storage.get(neighbor) else {
            return;
        };

        let neighbor_adjacency = neighbor_data.solid_adjacency().difference(adjacency);

        distance.insert(neighbor, (1.0, neighbor_adjacency));
        open.push(Node {
            position: neighbor,
            adjacency: neighbor_adjacency,
            cost: FloatOrd(1.0),
        });
    });

    while let Some(tile) = open.pop() {
        if !accepted.insert(tile.position) {
            continue;
        }

        visit_neighbours(tile.position, tile.adjacency, |neighbor| {
            if accepted.contains(&neighbor) {
                return;
            }

            let Some(neighbor_data) = storage.get(neighbor) else {
                return;
            };

            let neighbor_adjacency = neighbor_data.solid_adjacency();

            let new_distance = eikonal_distance(
                &distance,
                neighbor,
                neighbor_adjacency,
                neighbor_data.move_cost(),
            );

            match distance.entry(neighbor) {
                hash_map::Entry::Occupied(entry) if new_distance >= entry.get().0 => return,
                entry => {
                    entry.insert((new_distance, neighbor_adjacency));
                    open.push(Node {
                        position: neighbor,
                        adjacency: neighbor_adjacency,
                        cost: FloatOrd(new_distance),
                    });
                }
            }
        });
    }

    distance
}

fn flow_vector(
    distance: &HashMap<TilePosition, (f32, Adjacency)>,
    tile: TilePosition,
    cost: f32,
    adjacency: Adjacency,
) -> Dir2 {
    let mut dir = Vec2::ZERO;

    visit_neighbours_with_diagonals(tile, adjacency, |neighbor, direction| {
        let Some((neighbor_cost, _)) = distance.get(&neighbor) else {
            return;
        };

        let delta = cost - neighbor_cost;
        if delta <= 0.0 {
            return;
        }

        dir += direction.as_vec2() * delta;
    });

    Dir2::new(dir).expect("flow vector should not be zero")
}

fn eikonal_distance(
    distance: &HashMap<TilePosition, (f32, Adjacency)>,
    tile: TilePosition,
    adjacency: Adjacency,
    cost: f32,
) -> f32 {
    let west = if !adjacency.contains(Adjacency::WEST) {
        get_distance(distance, tile.west())
    } else {
        None
    };

    let east = if !adjacency.contains(Adjacency::EAST) {
        get_distance(distance, tile.east())
    } else {
        None
    };

    let north = if !adjacency.contains(Adjacency::NORTH) {
        get_distance(distance, tile.north())
    } else {
        None
    };

    let south = if !adjacency.contains(Adjacency::SOUTH) {
        get_distance(distance, tile.south())
    } else {
        None
    };

    let a = min_distance(west, east);
    let b = min_distance(north, south);

    match (a, b) {
        (Some(a), None) | (None, Some(a)) => a + cost,
        (Some(a), Some(b)) if (a - b).abs() >= cost => a.min(b) + cost,
        (Some(a), Some(b)) => (a + b + (2.0 * cost.squared() - (a - b).squared()).sqrt()) * 0.5,
        (None, None) => panic!("tile {:?} is not reachable", tile),
    }
}

fn get_distance(
    distance: &HashMap<TilePosition, (f32, Adjacency)>,
    tile: TilePosition,
) -> Option<f32> {
    distance.get(&tile).map(|&(d, _)| d)
}

fn min_distance(a: Option<f32>, b: Option<f32>) -> Option<f32> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) | (None, Some(a)) => Some(a),
        (None, None) => None,
    }
}

fn remove_door_region(flow: &mut Option<(Entity, Entity)>, region: Entity) {
    if matches!(flow, Some((r, _)) if *r == region) {
        *flow = None;
    }
}

fn visit_door_neighbors(
    tile: TilePosition,
    adjacency: Adjacency,
    mut f: impl FnMut(TilePosition, Adjacency),
) {
    if adjacency.contains(Adjacency::NORTH) {
        f(tile.north(), Adjacency::SOUTH);
    }

    if adjacency.contains(Adjacency::SOUTH) {
        f(tile.south(), Adjacency::NORTH);
    }

    if adjacency.contains(Adjacency::EAST) {
        f(tile.east(), Adjacency::WEST);
    }

    if adjacency.contains(Adjacency::WEST) {
        f(tile.west(), Adjacency::EAST);
    }
}

fn visit_neighbours(tile: TilePosition, adjacency: Adjacency, mut f: impl FnMut(TilePosition)) {
    if !adjacency.contains(Adjacency::NORTH) {
        f(tile.north());
    }

    if !adjacency.contains(Adjacency::SOUTH) {
        f(tile.south());
    }

    if !adjacency.contains(Adjacency::EAST) {
        f(tile.east());
    }

    if !adjacency.contains(Adjacency::WEST) {
        f(tile.west());
    }
}

fn visit_neighbours_with_diagonals(
    tile: TilePosition,
    adjacency: Adjacency,
    mut f: impl FnMut(TilePosition, Dir2),
) {
    if !adjacency.contains(Adjacency::NORTH) {
        f(tile.north(), Dir2::NORTH);
    }

    if !adjacency.contains(Adjacency::SOUTH) {
        f(tile.south(), Dir2::SOUTH);
    }

    if !adjacency.contains(Adjacency::EAST) {
        f(tile.east(), Dir2::EAST);
    }

    if !adjacency.contains(Adjacency::WEST) {
        f(tile.west(), Dir2::WEST);
    }

    if !adjacency.intersects(Adjacency::NORTH | Adjacency::NORTH_EAST | Adjacency::EAST) {
        f(tile.north().east(), Dir2::NORTH_EAST);
    }

    if !adjacency.intersects(Adjacency::NORTH | Adjacency::NORTH_WEST | Adjacency::WEST) {
        f(tile.north().west(), Dir2::NORTH_WEST);
    }

    if !adjacency.intersects(Adjacency::SOUTH | Adjacency::SOUTH_EAST | Adjacency::EAST) {
        f(tile.south().east(), Dir2::SOUTH_EAST);
    }

    if !adjacency.intersects(Adjacency::SOUTH | Adjacency::SOUTH_WEST | Adjacency::WEST) {
        f(tile.south().west(), Dir2::SOUTH_WEST);
    }
}
