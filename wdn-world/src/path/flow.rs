use std::{cmp::Ordering, collections::BinaryHeap, mem::replace};

use bevy_ecs::{entity::EntityHashMap, prelude::*};
use bevy_log::error;
use bevy_math::{FloatOrd, FloatPow, prelude::*};
use bevy_platform::collections::{HashMap, hash_map};
use wdn_physics::tile::{
    adjacency::Adjacency, index::TileIndex, position::TilePosition, storage::TileStorage,
};

use crate::path::region::{Region, TileChunkSections};

#[derive(Component, Default)]
pub struct RegionDoors {
    doors: EntityHashMap<RegionDoor>,
}

#[derive(Copy, Clone, Debug)]
pub struct RegionDoor {
    position: TilePosition,
    adjacency: Adjacency,
    flow_field: Entity,
}

#[derive(Component, Default)]
pub struct DoorRegions {
    north: Option<DoorRegion>,
    south: Option<DoorRegion>,
    east: Option<DoorRegion>,
    west: Option<DoorRegion>,
}

#[derive(Copy, Clone, Debug)]
pub struct DoorRegion {
    region: Entity,
    flow_field: Entity,
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
) {
    regions
        .iter_mut()
        .for_each(|(region_id, region, mut region_doors)| {
            for (chunk_id, section_id) in region.sections() {
                let section = sections
                    .get(chunk_id)
                    .expect("chunk not found")
                    .section(section_id.chunk_offset());

                region_doors.doors.reserve(section.door_count());
                for (door_position, door_adjacency) in section.doors() {
                    let Some(id) = index.get_tile(door_position) else {
                        error!("door at {:?} not found", door_position);
                        continue;
                    };

                    match region_doors.doors.entry(id) {
                        hash_map::Entry::Occupied(mut entry) => {
                            entry.get_mut().adjacency.insert(door_adjacency);
                        }
                        hash_map::Entry::Vacant(entry) => {
                            let flow_field = commands
                                .spawn((
                                    ChildOf(region_id),
                                    FlowField {
                                        door_position,
                                        door_adjacency,
                                        flow: HashMap::default(),
                                    },
                                ))
                                .id();
                            entry.insert(RegionDoor {
                                position: door_position,
                                adjacency: door_adjacency,
                                flow_field,
                            });
                        }
                    }
                }
            }

            for (door_id, door) in region_doors.doors() {
                doors.get_mut(door_id).expect("door not found").insert(
                    region_id,
                    door.flow_field,
                    door.adjacency,
                );
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
    for (_, door) in region.doors() {
        if let Ok(mut door) = doors.get_mut(door.flow_field) {
            remove_door_region(&mut door.north, trigger.entity);
            remove_door_region(&mut door.south, trigger.entity);
            remove_door_region(&mut door.east, trigger.entity);
            remove_door_region(&mut door.west, trigger.entity);
        }
    }

    Ok(())
}

impl RegionDoors {
    pub fn doors(&self) -> impl Iterator<Item = (Entity, &RegionDoor)> {
        self.doors.iter().map(|(&id, door)| (id, door))
    }
}

impl RegionDoor {
    pub fn position(&self) -> TilePosition {
        self.position
    }

    pub fn flow_field(&self) -> Entity {
        self.flow_field
    }
}

impl DoorRegions {
    pub fn flow_fields(&self) -> impl Iterator<Item = Entity> {
        [self.north, self.south, self.east, self.west]
            .into_iter()
            .flatten()
            .map(|door_region| door_region.flow_field)
    }

    pub fn insert(&mut self, region: Entity, flow: Entity, adjacency: Adjacency) {
        if adjacency.contains(Adjacency::NORTH) {
            debug_assert!(self.north.is_none());
            self.north = Some(DoorRegion::new(region, flow));
        }

        if adjacency.contains(Adjacency::SOUTH) {
            debug_assert!(self.south.is_none());
            self.south = Some(DoorRegion::new(region, flow));
        }

        if adjacency.contains(Adjacency::EAST) {
            debug_assert!(self.east.is_none());
            self.east = Some(DoorRegion::new(region, flow));
        }

        if adjacency.contains(Adjacency::WEST) {
            debug_assert!(self.west.is_none());
            self.west = Some(DoorRegion::new(region, flow));
        }
    }
}

impl DoorRegion {
    pub fn new(region: Entity, flow_field: Entity) -> Self {
        Self { region, flow_field }
    }

    pub fn flow_field(&self) -> Entity {
        self.flow_field
    }
}

impl FlowField {
    pub fn iter(&self) -> impl Iterator<Item = (TilePosition, Dir2)> {
        self.flow.iter().map(|(&pos, &dir)| (pos, dir))
    }

    fn generate_flow_field(&mut self, storage: &TileStorage, size_hint: usize) {
        let distance = generate_distance_field(
            storage,
            self.door_position,
            self.door_adjacency,
            size_hint + 1,
        );
        debug_assert_eq!(distance.len(), size_hint + 1);

        self.flow.reserve(size_hint);
        for (&tile, &(cost, adjacency, accepted)) in &distance {
            debug_assert!(cost.is_finite());
            debug_assert!(accepted);

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

type DistanceField = HashMap<TilePosition, (f32, Adjacency, bool)>;

fn generate_distance_field(
    storage: &TileStorage,
    door: TilePosition,
    door_adjacency: Adjacency,
    size_hint: usize,
) -> DistanceField {
    let mut open = BinaryHeap::new();
    let mut distance = HashMap::with_capacity(size_hint);

    distance.insert(door, (0.0, Adjacency::NONE, true));

    visit_door_neighbors(door, door_adjacency, |neighbor, adjacency| {
        let Some(neighbor_data) = storage.get(neighbor) else {
            return;
        };

        let neighbor_adjacency = neighbor_data.solid_adjacency().difference(adjacency);

        distance.insert(neighbor, (1.0, neighbor_adjacency, false));
        open.push(Node {
            position: neighbor,
            adjacency: neighbor_adjacency,
            cost: FloatOrd(1.0),
        });
    });

    while let Some(tile) = open.pop() {
        if replace(&mut distance.get_mut(&tile.position).unwrap().2, true) {
            continue;
        }

        visit_neighbours(tile.position, tile.adjacency, |neighbor| {
            if distance
                .get(&neighbor)
                .is_some_and(|&(_, _, accepted)| accepted)
            {
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
                    entry.insert((new_distance, neighbor_adjacency, false));
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
    distance: &DistanceField,
    tile: TilePosition,
    cost: f32,
    adjacency: Adjacency,
) -> Dir2 {
    let mut north = if !adjacency.contains(Adjacency::NORTH) {
        flow_delta(distance, tile.north(), cost)
    } else {
        None
    };
    let mut south = if !adjacency.contains(Adjacency::SOUTH) {
        flow_delta(distance, tile.south(), cost)
    } else {
        None
    };
    let mut east = if !adjacency.contains(Adjacency::EAST) {
        flow_delta(distance, tile.east(), cost)
    } else {
        None
    };
    let mut west = if !adjacency.contains(Adjacency::WEST) {
        flow_delta(distance, tile.west(), cost)
    } else {
        None
    };

    flow_tiebreak(&mut north, &mut south);
    flow_tiebreak(&mut east, &mut west);

    if adjacency.contains(Adjacency::NORTH_WEST) {
        flow_tiebreak(&mut north, &mut west);
    }

    if adjacency.contains(Adjacency::NORTH_EAST) {
        flow_tiebreak(&mut north, &mut east);
    }

    if adjacency.contains(Adjacency::SOUTH_EAST) {
        flow_tiebreak(&mut south, &mut east);
    }

    if adjacency.contains(Adjacency::SOUTH_WEST) {
        flow_tiebreak(&mut south, &mut west);
    }

    let mut dir = Vec2::ZERO;

    if let Some(north) = north {
        dir += Vec2::Y * north;
    }

    if let Some(south) = south {
        dir -= Vec2::Y * south;
    }

    if let Some(east) = east {
        dir += Vec2::X * east;
    }

    if let Some(west) = west {
        dir -= Vec2::X * west;
    }

    Dir2::new(dir).expect("flow vector should not be zero")
}

fn flow_delta(distance: &DistanceField, neighbor: TilePosition, cost: f32) -> Option<f32> {
    let &(neighbor_cost, _, _) = distance.get(&neighbor)?;
    let delta = cost - neighbor_cost;
    if delta > 0.0 { Some(delta) } else { None }
}

fn flow_tiebreak(a_flow: &mut Option<f32>, b_flow: &mut Option<f32>) {
    if let (Some(a), Some(b)) = (*a_flow, *b_flow) {
        if a < b {
            *b_flow = None;
        } else {
            *a_flow = None;
        }
    }
}

fn eikonal_distance(
    distance: &DistanceField,
    tile: TilePosition,
    adjacency: Adjacency,
    cost: f32,
) -> f32 {
    let west = if !adjacency.contains(Adjacency::WEST) {
        get_accepted_distance(distance, tile.west())
    } else {
        f32::INFINITY
    };

    let east = if !adjacency.contains(Adjacency::EAST) {
        get_accepted_distance(distance, tile.east())
    } else {
        f32::INFINITY
    };

    let north = if !adjacency.contains(Adjacency::NORTH) {
        get_accepted_distance(distance, tile.north())
    } else {
        f32::INFINITY
    };

    let south = if !adjacency.contains(Adjacency::SOUTH) {
        get_accepted_distance(distance, tile.south())
    } else {
        f32::INFINITY
    };

    let a = west.min(east);
    let b = north.min(south);

    if (a - b).abs() >= cost {
        a.min(b) + cost
    } else {
        let discr = 2.0 * cost.squared() - (a - b).squared();
        (a + b + discr.sqrt()) * 0.5
    }
}

fn get_accepted_distance(distance: &DistanceField, tile: TilePosition) -> f32 {
    match distance.get(&tile) {
        Some(&(cost, _, true)) => cost,
        _ => f32::INFINITY,
    }
}

fn remove_door_region(flow: &mut Option<DoorRegion>, region: Entity) {
    if matches!(flow, Some(door_region) if door_region.region == region) {
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
