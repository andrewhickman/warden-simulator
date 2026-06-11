use std::{cmp::Ordering, collections::BinaryHeap, mem::replace, ops::Index};

use bevy_ecs::prelude::*;
use bevy_log::error;
use bevy_math::{FloatOrd, FloatPow, prelude::*};
use bevy_platform::collections::{HashMap, hash_map};
use wdn_physics::tile::{
    adjacency::Adjacency,
    index::TileIndex,
    position::{TileLayerOffset, TilePosition},
    storage::TileStorage,
};

use crate::path::region::{Region, TileChunkSections};

const DOOR_COST: f32 = 1048576.0;

#[derive(Clone, Component, Default, Debug)]
pub struct RegionDoors {
    doors: HashMap<TileLayerOffset, RegionDoor>,
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
    flow: HashMap<TileLayerOffset, FlowFieldEntry>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FlowFieldEntry {
    dir: Dir2,
    cost: f32,
}

#[derive(Debug)]
pub struct CostField {
    costs: HashMap<TileLayerOffset, CostEntry>,
}

pub trait CostPolicy {
    fn priority(&self, position: TileLayerOffset, cost: f32) -> f32;

    fn finished(&self, position: TileLayerOffset) -> bool;
}

pub struct FlowPolicy;

pub struct PathPolicy {
    goal: TileLayerOffset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CostNode {
    priority: FloatOrd,
    cost: FloatOrd,
    position: TileLayerOffset,
    adjacency: Adjacency,
}

#[derive(Clone, Copy, Debug)]
struct CostEntry {
    cost: f32,
    adjacency: Adjacency,
    accepted: bool,
    door: bool,
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
    flow_fields.par_iter_mut().for_each(|(parent, mut flow)| {
        let (region, doors) = regions.get(parent.parent()).expect("region not found");
        flow.generate(&storage, region, doors);
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
    pub fn get(&self, position: TileLayerOffset) -> Option<&RegionDoor> {
        self.doors.get(&position)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TileLayerOffset, &RegionDoor)> {
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

    pub fn iter(&self) -> impl Iterator<Item = (TileLayerOffset, FlowFieldEntry)> {
        self.flow.iter().map(|(&pos, &entry)| (pos, entry))
    }

    pub fn get(&self, position: TileLayerOffset) -> Option<FlowFieldEntry> {
        self.flow.get(&position).copied()
    }

    pub fn len(&self) -> usize {
        self.flow.len()
    }

    fn generate(&mut self, storage: &TileStorage, region: &Region, doors: &RegionDoors) {
        let mut costs = CostField::with_capacity(region.size() + doors.door_count());
        costs.generate(
            &FlowPolicy,
            storage,
            doors,
            self.door_position,
            self.door_adjacency,
        );
        debug_assert_eq!(costs.len(), region.size() + doors.door_count());

        self.flow.reserve(region.size() + doors.door_count() - 1);
        for (position, entry) in costs.iter() {
            debug_assert!(entry.cost.is_finite());
            debug_assert!(entry.accepted);

            if position == self.door_position.layer_offset() {
                continue;
            }

            let dir = costs.flow_vector(position, entry.cost(), entry.adjacency);
            self.flow
                .insert(position, FlowFieldEntry::new(dir, entry.cost));
        }

        debug_assert_eq!(self.flow.len(), region.size() + doors.door_count() - 1);
    }
}

impl Index<TilePosition> for FlowField {
    type Output = FlowFieldEntry;

    fn index(&self, position: TilePosition) -> &Self::Output {
        &self[position.layer_offset()]
    }
}

impl Index<TileLayerOffset> for FlowField {
    type Output = FlowFieldEntry;

    fn index(&self, position: TileLayerOffset) -> &Self::Output {
        match self.flow.get(&position) {
            Some(entry) => entry,
            None => panic!(
                "{:?} not found in flow field {:?}",
                position, self.door_position
            ),
        }
    }
}

impl FlowFieldEntry {
    pub fn new(dir: Dir2, cost: f32) -> Self {
        FlowFieldEntry { dir, cost }
    }

    pub fn dir(&self) -> Dir2 {
        self.dir
    }

    pub fn reverse_dir(&self) -> Dir2 {
        Dir2::from_xy_unchecked(-self.dir.x, -self.dir.y)
    }

    pub fn cost(&self) -> f32 {
        self.cost
    }
}

impl CostField {
    pub fn new() -> Self {
        Self {
            costs: HashMap::default(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            costs: HashMap::with_capacity(capacity),
        }
    }

    pub fn generate<S: CostPolicy>(
        &mut self,
        policy: &S,
        storage: &TileStorage,
        doors: &RegionDoors,
        start: TilePosition,
        start_adjacency: Adjacency,
    ) {
        let mut open = BinaryHeap::new();

        self.insert(start.layer_offset(), 0.0, Adjacency::NONE, false);
        open.push(CostNode::new(
            policy,
            start.layer_offset(),
            start_adjacency.complement(),
            0.0,
        ));

        while let Some(node) = open.pop() {
            if !self.accept(node.position) {
                continue;
            }

            if policy.finished(node.position) {
                break;
            }

            node.visit_neighbors(|neighbor| {
                if self.is_accepted(neighbor) {
                    return;
                }

                let Some(neighbor_data) =
                    storage.get(TilePosition::from((start.layer(), neighbor)))
                else {
                    return;
                };

                let is_door = !neighbor_data.material().is_empty();
                let move_cost = neighbor_data.move_cost();
                let adjacency = if is_door {
                    doors.doors[&neighbor].adjacency.complement()
                } else {
                    neighbor_data.wall_adjacency()
                };

                let new_cost = self.eikonal_cost(neighbor, adjacency, move_cost);

                if self.insert(neighbor, new_cost, adjacency, is_door) {
                    open.push(CostNode::new(policy, neighbor, adjacency, new_cost));
                }
            });
        }
    }

    pub fn flow_vector(&self, position: TileLayerOffset, cost: f32, adjacency: Adjacency) -> Dir2 {
        let mut north = if !adjacency.contains(Adjacency::NORTH) {
            self.flow_delta(position.north(), cost)
        } else {
            None
        };
        let mut south = if !adjacency.contains(Adjacency::SOUTH) {
            self.flow_delta(position.south(), cost)
        } else {
            None
        };
        let mut east = if !adjacency.contains(Adjacency::EAST) {
            self.flow_delta(position.east(), cost)
        } else {
            None
        };
        let mut west = if !adjacency.contains(Adjacency::WEST) {
            self.flow_delta(position.west(), cost)
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

        match Dir2::new(dir) {
            Ok(dir) => dir,
            Err(_) => panic!(
                "failed to resolve flow vector for position {:?} with cost {}",
                position, cost
            ),
        }
    }

    pub fn contains(&self, position: TileLayerOffset) -> bool {
        self.costs.contains_key(&position)
    }

    pub fn get_cost(&self, position: TileLayerOffset) -> Option<f32> {
        Some(self.costs.get(&position)?.cost())
    }

    fn insert(
        &mut self,
        position: TileLayerOffset,
        cost: f32,
        adjacency: Adjacency,
        door: bool,
    ) -> bool {
        match self.costs.entry(position) {
            hash_map::Entry::Occupied(entry) if cost >= entry.get().cost() => false,
            entry => {
                entry.insert(CostEntry {
                    cost,
                    adjacency,
                    door,
                    accepted: false,
                });
                true
            }
        }
    }

    pub fn len(&self) -> usize {
        self.costs.len()
    }

    fn iter(&self) -> impl Iterator<Item = (TileLayerOffset, &CostEntry)> {
        self.costs.iter().map(|(&pos, entry)| (pos, entry))
    }

    fn accept(&mut self, position: TileLayerOffset) -> bool {
        self.costs.get_mut(&position).unwrap().accept()
    }

    fn is_accepted(&self, position: TileLayerOffset) -> bool {
        self.costs
            .get(&position)
            .is_some_and(|entry| entry.accepted)
    }

    fn get_accepted_cost(&self, position: TileLayerOffset) -> f32 {
        match self.costs.get(&position) {
            Some(entry) if entry.accepted => entry.cost(),
            _ => f32::INFINITY,
        }
    }

    fn eikonal_cost(&self, position: TileLayerOffset, adjacency: Adjacency, cost: f32) -> f32 {
        let west = if !adjacency.contains(Adjacency::WEST) {
            self.get_accepted_cost(position.west())
        } else {
            f32::INFINITY
        };

        let east = if !adjacency.contains(Adjacency::EAST) {
            self.get_accepted_cost(position.east())
        } else {
            f32::INFINITY
        };

        let north = if !adjacency.contains(Adjacency::NORTH) {
            self.get_accepted_cost(position.north())
        } else {
            f32::INFINITY
        };

        let south = if !adjacency.contains(Adjacency::SOUTH) {
            self.get_accepted_cost(position.south())
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

    fn flow_delta(&self, neighbor: TileLayerOffset, cost: f32) -> Option<f32> {
        let delta = cost - self.get_cost(neighbor)?;
        if delta > 0.0 { Some(delta) } else { None }
    }
}

impl CostNode {
    fn new<S: CostPolicy>(
        policy: &S,
        position: TileLayerOffset,
        adjacency: Adjacency,
        cost: f32,
    ) -> Self {
        Self {
            position,
            adjacency,
            cost: FloatOrd(cost),
            priority: FloatOrd(policy.priority(position, cost)),
        }
    }

    fn visit_neighbors(&self, mut f: impl FnMut(TileLayerOffset)) {
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
    fn accept(&mut self) -> bool {
        !replace(&mut self.accepted, true)
    }

    fn cost(&self) -> f32 {
        if self.door {
            self.cost + DOOR_COST
        } else {
            self.cost
        }
    }
}

impl Ord for CostNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for CostNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl CostPolicy for FlowPolicy {
    fn priority(&self, _position: TileLayerOffset, cost: f32) -> f32 {
        cost
    }

    fn finished(&self, _position: TileLayerOffset) -> bool {
        false
    }
}

impl PathPolicy {
    pub fn new(goal: TileLayerOffset) -> Self {
        Self { goal }
    }
}

impl CostPolicy for PathPolicy {
    fn priority(&self, position: TileLayerOffset, cost: f32) -> f32 {
        cost + (position.position().distance_squared(self.goal.position()) as f32).sqrt()
    }

    fn finished(&self, position: TileLayerOffset) -> bool {
        position == self.goal
    }
}

fn flow_tiebreak(a_flow: &mut Option<f32>, b_flow: &mut Option<f32>) {
    if let (Some(a), Some(b)) = (*a_flow, *b_flow) {
        if b > a {
            *a_flow = None;
        } else {
            *b_flow = None;
        }
    }
}

fn remove_door_region(flow: &mut Option<DoorRegion>, region: Entity) {
    if matches!(flow, Some(door_region) if door_region.region == region) {
        *flow = None;
    }
}
