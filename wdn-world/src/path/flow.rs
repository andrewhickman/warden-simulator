use std::{array, ops::Index};

use bevy_ecs::{
    entity::{EntityHashSet, hash_set},
    prelude::*,
};
use bevy_log::info;
use bevy_math::prelude::*;
use bevy_platform::collections::HashMap;
use smallvec::SmallVec;
use wdn_physics::tile::{
    adjacency::Adjacency,
    material::TileMoveSpeed,
    position::{TileLayerOffset, TilePosition},
};

use crate::path::region::{RegionTile, RegionTileIndex, RegionTiles};

pub const COST_MULTIPLIER: f32 = 5.0;
pub const SLOW_DIAGONAL_COST: u32 = 10;
pub const SLOW_CARDINAL_COST: u32 = 7;
pub const MEDIUM_DIAGONAL_COST: u32 = 7;
pub const MEDIUM_CARDINAL_COST: u32 = 5;
pub const FAST_DIAGONAL_COST: u32 = 4;
pub const FAST_CARDINAL_COST: u32 = 3;

#[derive(Component, Debug)]
pub struct FlowField {
    door_position: TilePosition,
    door_index: RegionTileIndex,
    door_adjacency: Adjacency,
    flow: HashMap<TileLayerOffset, FlowFieldEntry>,
}

#[derive(Resource, Default, Debug)]
pub struct AddedFlowFields {
    added_flow_fields: EntityHashSet,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FlowFieldEntry {
    dir: Dir2,
    cost: u32,
}

#[derive(Debug)]
pub struct CostField {
    costs: Vec<u32>,
}

struct CostNodeQueue<const N: usize> {
    buckets: [SmallVec<[CostNode; 32]>; N],
    min: u32,
}

#[derive(Clone, Copy, Debug)]
struct CostNode {
    priority: u32,
    index: RegionTileIndex,
    adjacency: Adjacency,
}

pub trait CostPolicy {
    const BUCKETS: usize;

    fn priority(&self, position: TileLayerOffset, cost: u32) -> u32;

    fn finished(&self, index: RegionTileIndex) -> bool;
}

pub struct FlowPolicy;

pub struct PathPolicy {
    goal: TileLayerOffset,
    goal_index: RegionTileIndex,
}

pub fn update_flow_fields(
    regions: Query<&RegionTiles>,
    mut flow_fields: Query<(&ChildOf, &mut FlowField)>,
    added_flow_fields: Res<AddedFlowFields>,
) {
    info!(
        "updating {} flow fields",
        added_flow_fields.added_flow_fields.len()
    );
    flow_fields
        .par_iter_many_unique_mut(added_flow_fields.iter())
        .for_each(|(parent, mut flow)| {
            let tiles = regions.get(parent.parent()).expect("region not found");
            flow.generate(tiles);
        });
}

pub fn flow_fields_added(changes: Res<AddedFlowFields>) -> bool {
    changes.has_flow_fields()
}

pub fn clear_added_flow_fields(mut changes: ResMut<AddedFlowFields>) {
    changes.clear();
}

pub fn octile_cost(a: TileLayerOffset, b: TileLayerOffset, move_speed: TileMoveSpeed) -> u32 {
    let dx = a.x().abs_diff(b.x());
    let dy = a.y().abs_diff(b.y());

    let (min, max) = if dx < dy { (dx, dy) } else { (dy, dx) };
    let (cardinal_cost, diagonal_cost) = move_cost(move_speed);

    cardinal_cost * max + (diagonal_cost - cardinal_cost) * min
}

pub fn move_cost(move_speed: TileMoveSpeed) -> (u32, u32) {
    match move_speed {
        TileMoveSpeed::Slow => (SLOW_CARDINAL_COST, SLOW_DIAGONAL_COST),
        TileMoveSpeed::Medium => (MEDIUM_CARDINAL_COST, MEDIUM_DIAGONAL_COST),
        TileMoveSpeed::Fast => (FAST_CARDINAL_COST, FAST_DIAGONAL_COST),
    }
}

impl FlowField {
    pub fn new(
        door_position: TilePosition,
        door_index: RegionTileIndex,
        door_adjacency: Adjacency,
    ) -> Self {
        FlowField {
            door_position,
            door_index,
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

    fn generate(&mut self, tiles: &RegionTiles) {
        let mut costs = CostField::new(tiles.size());
        costs.generate::<FlowPolicy, { FlowPolicy::BUCKETS }>(
            &FlowPolicy,
            tiles,
            self.door_index,
            self.door_position.layer_offset(),
            self.door_adjacency,
        );

        self.flow.reserve(tiles.size() - 1);

        for (index, tile) in tiles.tiles() {
            let position = tile.position();
            if position == self.door_position.layer_offset() {
                continue;
            }

            let cost = costs[index];
            debug_assert_ne!(
                cost,
                u32::MAX,
                "{:?} is unreachable from door {:?}",
                position,
                self.door_position
            );

            let dir = costs.flow_vector(tile, cost);
            self.flow.insert(position, FlowFieldEntry::new(dir, cost));
        }

        debug_assert_eq!(self.flow.len(), tiles.size() - 1);
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

impl AddedFlowFields {
    pub fn insert(&mut self, flow_field: Entity) {
        self.added_flow_fields.insert(flow_field);
    }

    pub fn clear(&mut self) {
        self.added_flow_fields.clear();
    }

    pub fn has_flow_fields(&self) -> bool {
        !self.added_flow_fields.is_empty()
    }

    pub fn iter(&'_ self) -> hash_set::Iter<'_> {
        self.added_flow_fields.iter()
    }
}

impl FlowFieldEntry {
    pub fn new(dir: Dir2, cost: u32) -> Self {
        FlowFieldEntry { dir, cost }
    }

    pub fn dir(&self) -> Dir2 {
        self.dir
    }

    pub fn cost(&self) -> u32 {
        self.cost
    }
}

impl CostField {
    pub fn new(size: usize) -> Self {
        Self {
            costs: vec![u32::MAX; size],
        }
    }

    pub fn generate<S: CostPolicy, const BUCKETS: usize>(
        &mut self,
        policy: &S,
        tiles: &RegionTiles,
        start: RegionTileIndex,
        start_position: TileLayerOffset,
        start_adjacency: Adjacency,
    ) {
        let mut open = CostNodeQueue::<BUCKETS>::new();

        let start_priority = policy.priority(start_position, 0);
        self.insert(start, 0);
        open.push(CostNode::new(start, start_adjacency, start_priority));

        while let Some(node) = open.pop() {
            debug_assert!(self.contains(node.index));

            if policy.finished(node.index) {
                break;
            }

            if self[node.index] < node.priority {
                continue;
            }

            node.visit_neighbors(tiles, |neighbor, cost| {
                let neighbor_data = &tiles[neighbor];

                let adjacency = if neighbor_data.is_door() {
                    Adjacency::NONE
                } else {
                    neighbor_data.adjacency()
                };

                let new_cost = self[node.index] + cost;
                let priority = policy.priority(neighbor_data.position(), new_cost);

                debug_assert!(
                    node.priority.abs_diff(priority) < BUCKETS as u32,
                    "priority difference between {:?} and {:?} is too large for the queue with bucket size {}. Base cost: {}, new cost: {}",
                    node.index,
                    neighbor,
                    BUCKETS,
                    self[node.index],
                    new_cost
                );

                if self.insert(neighbor, new_cost) {
                    open.push(CostNode::new(neighbor, adjacency, priority));
                }
            });
        }
    }

    pub fn flow_vector(&self, tile: &RegionTile, cost: u32) -> Dir2 {
        let mut north = if let Some(north) = tile.north() {
            self.flow_delta(north, cost)
        } else {
            None
        };
        let mut south = if let Some(south) = tile.south() {
            self.flow_delta(south, cost)
        } else {
            None
        };
        let mut east = if let Some(east) = tile.east() {
            self.flow_delta(east, cost)
        } else {
            None
        };
        let mut west = if let Some(west) = tile.west() {
            self.flow_delta(west, cost)
        } else {
            None
        };

        flow_tiebreak(&mut north, &mut south);
        flow_tiebreak(&mut east, &mut west);

        let adjacency = tile.adjacency();

        if !adjacency.contains(Adjacency::NORTH_WEST) {
            flow_tiebreak(&mut north, &mut west);
        }

        if !adjacency.contains(Adjacency::NORTH_EAST) {
            flow_tiebreak(&mut north, &mut east);
        }

        if !adjacency.contains(Adjacency::SOUTH_EAST) {
            flow_tiebreak(&mut south, &mut east);
        }

        if !adjacency.contains(Adjacency::SOUTH_WEST) {
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
                "failed to resolve flow vector for tile {:?} with cost {}",
                tile.position(),
                cost
            ),
        }
    }

    pub fn contains(&self, index: RegionTileIndex) -> bool {
        self.costs[index as usize] != u32::MAX
    }

    fn insert(&mut self, index: RegionTileIndex, cost: u32) -> bool {
        let entry = &mut self.costs[index as usize];
        if cost < *entry {
            *entry = cost;
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.costs.len()
    }

    fn flow_delta(&self, neighbor: RegionTileIndex, cost: u32) -> Option<f32> {
        let delta = cost.checked_sub(self[neighbor])?;
        if delta > 0 { Some(delta as f32) } else { None }
    }
}

impl Index<RegionTileIndex> for CostField {
    type Output = u32;

    fn index(&self, index: RegionTileIndex) -> &Self::Output {
        &self.costs[index as usize]
    }
}

impl CostNode {
    fn new(index: RegionTileIndex, adjacency: Adjacency, priority: u32) -> Self {
        CostNode {
            index,
            adjacency,
            priority,
        }
    }

    fn visit_neighbors<F>(&self, tiles: &RegionTiles, mut f: F)
    where
        F: FnMut(RegionTileIndex, u32),
    {
        let tile = &tiles[self.index];
        let (cardinal_cost, _diagonal_cost) = move_cost(tile.move_speed());

        if self.adjacency.contains(Adjacency::NORTH)
            && let Some(north) = tile.north()
        {
            f(north, cardinal_cost);
        }

        if self.adjacency.contains(Adjacency::SOUTH)
            && let Some(south) = tile.south()
        {
            f(south, cardinal_cost);
        }

        if self.adjacency.contains(Adjacency::EAST)
            && let Some(east) = tile.east()
        {
            f(east, cardinal_cost);
        }

        if self.adjacency.contains(Adjacency::WEST)
            && let Some(west) = tile.west()
        {
            f(west, cardinal_cost);
        }
    }
}

impl<const N: usize> CostNodeQueue<N> {
    fn new() -> Self {
        Self {
            buckets: array::from_fn(|_| SmallVec::new()),
            min: N as u32,
        }
    }

    fn pop(&mut self) -> Option<CostNode> {
        for i in self.min as usize..N {
            if let Some(node) = self.buckets[i].pop() {
                self.min = i as u32;
                return Some(node);
            }
        }

        None
    }

    fn push(&mut self, node: CostNode) {
        let index = node.priority % N as u32;
        self.buckets[index as usize].push(node);
        if index < self.min {
            self.min = index;
        }
    }
}

impl CostPolicy for FlowPolicy {
    const BUCKETS: usize = SLOW_DIAGONAL_COST as usize + 1;

    fn priority(&self, _position: TileLayerOffset, cost: u32) -> u32 {
        cost
    }

    fn finished(&self, _index: RegionTileIndex) -> bool {
        false
    }
}

impl PathPolicy {
    pub fn new(goal: TileLayerOffset, goal_index: RegionTileIndex) -> Self {
        Self { goal, goal_index }
    }
}

impl CostPolicy for PathPolicy {
    const BUCKETS: usize = SLOW_DIAGONAL_COST as usize * 2 + 1;

    fn priority(&self, position: TileLayerOffset, cost: u32) -> u32 {
        cost + octile_cost(position, self.goal, TileMoveSpeed::Slow)
    }

    fn finished(&self, index: RegionTileIndex) -> bool {
        index == self.goal_index
    }
}

fn flow_tiebreak(a_flow: &mut Option<f32>, b_flow: &mut Option<f32>) {
    // todo revisit equal?
    if let (Some(a), Some(b)) = (*a_flow, *b_flow) {
        if b > a {
            *a_flow = None;
        } else {
            *b_flow = None;
        }
    }
}
