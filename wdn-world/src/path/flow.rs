use std::{cmp::Ordering, collections::BinaryHeap, mem::replace, ops::Index};

use bevy_ecs::prelude::*;
use bevy_log::error;
use bevy_math::{FloatOrd, FloatPow, prelude::*};
use bevy_platform::collections::{HashMap, hash_map};
use wdn_physics::tile::{
    adjacency::Adjacency,
    index::TileIndex,
    position::{TileLayerPosition, TilePosition},
    storage::TileStorage,
};

use crate::path::region::{Region, TileChunkSections};

#[derive(Clone, Component, Default, Debug)]
pub struct RegionDoors {
    doors: HashMap<TileLayerPosition, RegionDoor>,
}

#[derive(Clone, Copy, Debug)]
pub struct RegionDoor {
    door: Entity,
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
    flow: HashMap<TileLayerPosition, Dir2>,
}

const DOOR_COST: f32 = 1048576.0;

type CostField = HashMap<TileLayerPosition, CostEntry>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CostNode {
    cost: FloatOrd,
    position: TileLayerPosition,
    adjacency: Adjacency,
}

#[derive(Clone, Copy, Debug)]
struct CostEntry {
    cost: f32,
    adjacency: Adjacency,
    accepted: bool,
}

impl Ord for CostNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for CostNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
                    match region_doors.doors.entry(door_position) {
                        hash_map::Entry::Occupied(mut entry) => {
                            entry.get_mut().adjacency.insert(door_adjacency);
                        }
                        hash_map::Entry::Vacant(entry) => {
                            let Some(door) =
                                index.get_tile(TilePosition::from((region.layer(), door_position)))
                            else {
                                error!("door at {:?} not found", door_position);
                                continue;
                            };

                            entry.insert(RegionDoor {
                                door,
                                adjacency: door_adjacency,
                                flow_field: Entity::PLACEHOLDER,
                            });
                        }
                    }
                }
            }

            for (&door_position, door) in region_doors.doors.iter_mut() {
                door.flow_field = commands
                    .spawn((
                        ChildOf(region_id),
                        FlowField {
                            door_position: TilePosition::from((region.layer(), door_position)),
                            door_adjacency: door.adjacency,
                            flow: HashMap::default(),
                        },
                    ))
                    .id();

                doors.get_mut(door.door).expect("door not found").insert(
                    region_id,
                    door.flow_field,
                    door.adjacency,
                );
            }
        });
}

pub fn update_flow_fields(
    storage: TileStorage,
    regions: Query<(&Region, &RegionDoors)>,
    mut flow_fields: Query<(&ChildOf, &mut FlowField), Added<FlowField>>,
) {
    flow_fields.iter_mut().for_each(|(parent, mut flow)| {
        let (region, doors) = regions.get(parent.parent()).expect("region not found");
        flow.generate_flow_field(&storage, region, doors);
    });
}

pub fn on_remove_region_doors(
    trigger: On<Remove, RegionDoors>,
    regions: Query<&RegionDoors>,
    mut doors: Query<&mut DoorRegions>,
) -> Result {
    let region = regions.get(trigger.entity)?;
    for (_, door) in region.iter() {
        if let Ok(mut door) = doors.get_mut(door.door()) {
            remove_door_region(&mut door.north, trigger.entity);
            remove_door_region(&mut door.south, trigger.entity);
            remove_door_region(&mut door.east, trigger.entity);
            remove_door_region(&mut door.west, trigger.entity);
        }
    }

    Ok(())
}

impl RegionDoors {
    pub fn get(&self, position: TileLayerPosition) -> Option<&RegionDoor> {
        self.doors.get(&position)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TileLayerPosition, &RegionDoor)> {
        self.doors.iter().map(|(&pos, door)| (pos, door))
    }

    pub fn door_count(&self) -> usize {
        self.doors.len()
    }
}

impl RegionDoor {
    pub fn flow_field(&self) -> Entity {
        self.flow_field
    }

    pub fn door(&self) -> Entity {
        self.door
    }
}

impl DoorRegions {
    pub fn iter(&self) -> impl Iterator<Item = DoorRegion> {
        [self.north, self.south, self.east, self.west]
            .into_iter()
            .flatten()
    }

    pub fn flow_fields(&self) -> impl Iterator<Item = Entity> {
        self.iter().map(|door_region| door_region.flow_field())
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

    pub fn region(&self) -> Entity {
        self.region
    }

    pub fn flow_field(&self) -> Entity {
        self.flow_field
    }
}

impl FlowField {
    pub fn layer(&self) -> Entity {
        self.door_position.layer()
    }

    pub fn iter(&self) -> impl Iterator<Item = (TileLayerPosition, Dir2)> {
        self.flow.iter().map(|(&pos, &dir)| (pos, dir))
    }

    pub fn get(&self, position: TileLayerPosition) -> Option<Dir2> {
        self.flow.get(&position).copied()
    }

    fn generate_flow_field(&mut self, storage: &TileStorage, region: &Region, doors: &RegionDoors) {
        let costs = generate_cost_field(
            storage,
            doors,
            self.door_position,
            self.door_adjacency,
            region.size() + doors.door_count(),
        );
        debug_assert_eq!(costs.len(), region.size() + doors.door_count());

        self.flow.reserve(region.size() + doors.door_count() - 1);
        for (&position, &entry) in &costs {
            debug_assert!(entry.cost.is_finite());
            debug_assert!(entry.accepted);

            if position == self.door_position.layer_position() {
                continue;
            }

            self.flow.insert(
                position,
                flow_vector(&costs, position, entry.cost, entry.adjacency),
            );
        }

        debug_assert_eq!(self.flow.len(), region.size() + doors.door_count() - 1);
    }
}

impl Index<TilePosition> for FlowField {
    type Output = Dir2;

    fn index(&self, position: TilePosition) -> &Self::Output {
        &self[position.layer_position()]
    }
}

impl Index<TileLayerPosition> for FlowField {
    type Output = Dir2;

    fn index(&self, position: TileLayerPosition) -> &Self::Output {
        match self.flow.get(&position) {
            Some(dir) => dir,
            None => panic!(
                "{:?} not found in flow field {:?}",
                position, self.door_position
            ),
        }
    }
}

impl CostNode {
    fn new(position: TileLayerPosition, adjacency: Adjacency, cost: f32) -> Self {
        Self {
            position,
            adjacency,
            cost: FloatOrd(cost),
        }
    }

    fn visit_neighbors(&self, mut f: impl FnMut(TileLayerPosition)) {
        if !self.adjacency.contains(Adjacency::NORTH) {
            f(self.position.north());
        }

        if !self.adjacency.contains(Adjacency::SOUTH) {
            f(self.position.south());
        }

        if !self.adjacency.contains(Adjacency::EAST) {
            f(self.position.east());
        }

        if !self.adjacency.contains(Adjacency::WEST) {
            f(self.position.west());
        }
    }
}

impl CostEntry {
    fn new(cost: f32, adjacency: Adjacency, accepted: bool) -> Self {
        Self {
            cost,
            adjacency,
            accepted,
        }
    }

    fn accept(&mut self) -> bool {
        !replace(&mut self.accepted, true)
    }
}

fn generate_cost_field(
    storage: &TileStorage,
    doors: &RegionDoors,
    door: TilePosition,
    door_adjacency: Adjacency,
    size_hint: usize,
) -> CostField {
    let mut open = BinaryHeap::new();
    let mut costs = HashMap::with_capacity(size_hint);

    costs.insert(
        door.layer_position(),
        CostEntry::new(0.0, Adjacency::NONE, false),
    );
    open.push(CostNode::new(
        door.layer_position(),
        door_adjacency.complement(),
        0.0,
    ));

    while let Some(node) = open.pop() {
        if !costs.get_mut(&node.position).unwrap().accept() {
            continue;
        }

        node.visit_neighbors(|neighbor| {
            if costs.get(&neighbor).is_some_and(|e| e.accepted) {
                return;
            }

            let Some(neighbor_data) = storage.get(TilePosition::from((door.layer(), neighbor)))
            else {
                return;
            };

            let (adjacency, move_cost) = if neighbor_data.material().is_empty() {
                (neighbor_data.wall_adjacency(), neighbor_data.move_cost())
            } else {
                (doors.doors[&neighbor].adjacency.complement(), DOOR_COST)
            };

            let new_cost = eikonal_cost(&costs, neighbor, adjacency, move_cost);

            match costs.entry(neighbor) {
                hash_map::Entry::Occupied(entry) if new_cost >= entry.get().cost => return,
                entry => {
                    entry.insert(CostEntry::new(new_cost, adjacency, false));
                    open.push(CostNode::new(neighbor, adjacency, new_cost));
                }
            }
        });
    }

    costs
}

fn eikonal_cost(
    costs: &CostField,
    position: TileLayerPosition,
    adjacency: Adjacency,
    cost: f32,
) -> f32 {
    let west = if !adjacency.contains(Adjacency::WEST) {
        get_accepted_cost(costs, position.west())
    } else {
        f32::INFINITY
    };

    let east = if !adjacency.contains(Adjacency::EAST) {
        get_accepted_cost(costs, position.east())
    } else {
        f32::INFINITY
    };

    let north = if !adjacency.contains(Adjacency::NORTH) {
        get_accepted_cost(costs, position.north())
    } else {
        f32::INFINITY
    };

    let south = if !adjacency.contains(Adjacency::SOUTH) {
        get_accepted_cost(costs, position.south())
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

fn get_accepted_cost(costs: &CostField, position: TileLayerPosition) -> f32 {
    match costs.get(&position) {
        Some(entry) if entry.accepted => entry.cost,
        _ => f32::INFINITY,
    }
}

fn flow_vector(
    costs: &CostField,
    position: TileLayerPosition,
    cost: f32,
    adjacency: Adjacency,
) -> Dir2 {
    let mut north = if !adjacency.contains(Adjacency::NORTH) {
        flow_delta(costs, position.north(), cost)
    } else {
        None
    };
    let mut south = if !adjacency.contains(Adjacency::SOUTH) {
        flow_delta(costs, position.south(), cost)
    } else {
        None
    };
    let mut east = if !adjacency.contains(Adjacency::EAST) {
        flow_delta(costs, position.east(), cost)
    } else {
        None
    };
    let mut west = if !adjacency.contains(Adjacency::WEST) {
        flow_delta(costs, position.west(), cost)
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

fn flow_delta(costs: &CostField, neighbor: TileLayerPosition, cost: f32) -> Option<f32> {
    let delta = cost - costs.get(&neighbor)?.cost;
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

fn remove_door_region(flow: &mut Option<DoorRegion>, region: Entity) {
    if matches!(flow, Some(door_region) if door_region.region == region) {
        *flow = None;
    }
}
