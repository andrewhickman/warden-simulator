use std::{cmp::Ordering, collections::BinaryHeap, mem::replace, ops::Index};

use bevy_ecs::prelude::*;
use bevy_math::{FloatOrd, FloatPow, prelude::*};
use bevy_platform::collections::{HashMap, hash_map};
use wdn_physics::tile::{
    adjacency::Adjacency,
    position::{TileLayerOffset, TilePosition},
    storage::TileStorage,
};

use crate::path::{door::RegionDoors, region::Region};

const DOOR_COST: f32 = 1048576.0;

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
    base_cost: f32,
    adjacency: Adjacency,
    accepted: bool,
    door: bool,
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

impl FlowField {
    pub fn new(door_position: TilePosition, door_adjacency: Adjacency) -> Self {
        Self {
            door_position,
            door_adjacency,
            flow: HashMap::default(),
        }
    }

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
        debug_assert!(costs.costs.values().all(|e| e.accepted));

        self.flow.reserve(region.size() + doors.door_count() - 1);
        self.flow
            .extend(costs.iter_flow(self.door_position.layer_offset()));

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
                    doors[neighbor].adjacency().complement()
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
                    base_cost: cost,
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

    pub fn iter_flow(
        &self,
        start: TileLayerOffset,
    ) -> impl Iterator<Item = (TileLayerOffset, FlowFieldEntry)> {
        self.costs
            .iter()
            .filter(move |&(&pos, _)| pos != start)
            .map(|(&pos, entry)| {
                debug_assert!(entry.base_cost.is_finite());

                let flow = self.flow_vector(pos, entry.cost(), entry.adjacency);

                (pos, FlowFieldEntry::new(flow, entry.base_cost))
            })
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
            self.base_cost + DOOR_COST
        } else {
            self.base_cost
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
