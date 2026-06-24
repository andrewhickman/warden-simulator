use std::{
    array,
    cmp::Ordering,
    collections::{BinaryHeap, VecDeque},
    fmt,
    ops::Index,
};

use bevy_ecs::{
    entity::{EntityHashSet, hash_set},
    prelude::*,
};
use bevy_log::warn;
use bevy_math::prelude::*;
use bevy_platform::collections::HashMap;
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
    flow: Vec<Dir2>,
    costs: CostField,
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

#[derive(Debug, Clone)]
pub struct CostField {
    costs: Vec<u32>,
}

#[derive(Clone, Copy, Debug)]
pub struct CostNode {
    priority: u32,
    cost: u32,
    index: RegionTileIndex,
    adjacency: Adjacency,
}

pub struct CostNodeQueue<const N: usize> {
    buckets: [VecDeque<CostNode>; N],
    current: u32,
    len: u32,
}

pub trait CostPolicy {
    type Queue: CostPolicyQueue + Default;

    fn priority(&self, position: TileLayerOffset, cost: u32) -> u32;

    fn finished(&self, index: RegionTileIndex) -> bool;
}

pub trait CostPolicyQueue {
    fn push(&mut self, node: CostNode);

    fn pop(&mut self) -> Option<CostNode>;
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
    let start = std::time::Instant::now();

    flow_fields
        .par_iter_many_unique_mut(added_flow_fields.iter())
        .for_each(|(parent, mut flow)| {
            let tiles = regions.get(parent.parent()).expect("region not found");
            flow.generate(tiles);
            flow.populate_flow(tiles);
        });

    let elapsed = start.elapsed();
    if elapsed > std::time::Duration::from_secs_f32(0.001953125) {
        let mut regions = HashMap::<Entity, (usize, usize)>::new();

        for (parent, flow) in flow_fields.iter_many(added_flow_fields.iter()) {
            let entry = regions.entry(parent.parent()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 = flow.len();
        }

        warn!(
            "updating {} flow fields took {:.2?}",
            added_flow_fields.added_flow_fields.len(),
            elapsed,
        );
    }
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
            flow: Vec::default(),
            costs: CostField::new(0),
        }
    }

    pub fn from_cost_field(
        door_position: TilePosition,
        door_index: RegionTileIndex,
        door_adjacency: Adjacency,
        costs: CostField,
    ) -> Self {
        FlowField {
            door_position,
            door_index,
            door_adjacency,
            flow: Vec::default(),
            costs,
        }
    }

    pub fn layer(&self) -> Entity {
        self.door_position.layer()
    }

    pub fn iter(
        &self,
        tiles: &RegionTiles,
    ) -> impl Iterator<Item = (TileLayerOffset, FlowFieldEntry)> {
        (0..self.costs.len()).filter_map(move |index| {
            let index = index as RegionTileIndex;
            if index == self.door_index {
                return None;
            }

            let dir = self.flow[index as usize];
            let cost = self.costs[index];

            Some((tiles[index].position(), FlowFieldEntry::new(dir, cost)))
        })
    }

    pub fn get(&self, tiles: &RegionTiles, position: TileLayerOffset) -> Option<FlowFieldEntry> {
        let index = tiles.get_tile_index(position)?;
        if index == self.door_index {
            return None;
        }

        let dir = self.flow[index as usize];
        let cost = self.costs[index];

        Some(FlowFieldEntry::new(dir, cost))
    }

    pub fn len(&self) -> usize {
        self.costs.len() - 1
    }

    pub fn generate(&mut self, tiles: &RegionTiles) {
        self.costs.resize(tiles.size());
        self.costs.generate::<FlowPolicy>(
            &FlowPolicy,
            tiles,
            self.door_index,
            self.door_position.layer_offset(),
            self.door_adjacency,
        );
    }

    pub fn populate_flow(&mut self, tiles: &RegionTiles) {
        self.flow.reserve(tiles.size());

        for (index, tile) in tiles.tiles() {
            let position = tile.position();
            if position == self.door_position.layer_offset() {
                self.flow.push(Dir2::NORTH);
                continue;
            }

            let cost = self.costs[index];
            debug_assert_ne!(
                cost,
                u32::MAX,
                "{:?} is unreachable from door {:?}",
                position,
                self.door_position
            );

            let dir = self.costs.flow_vector(tile, cost);
            self.flow.push(dir);
        }

        debug_assert_eq!(self.flow.len(), tiles.size());
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

    pub fn resize(&mut self, size: usize) {
        self.costs.resize(size, u32::MAX);
    }

    pub fn generate<S: CostPolicy>(
        &mut self,
        policy: &S,
        tiles: &RegionTiles,
        start: RegionTileIndex,
        start_position: TileLayerOffset,
        start_adjacency: Adjacency,
    ) {
        let mut open = S::Queue::default();

        let start_priority = policy.priority(start_position, 0);
        self.insert(start, 0);
        open.push(CostNode::new(start, start_adjacency, 0, start_priority));

        while let Some(node) = open.pop() {
            debug_assert!(self.contains(node.index));

            if policy.finished(node.index) {
                break;
            }

            if self[node.index] < node.cost {
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

                if self.insert(neighbor, new_cost) {
                    open.push(CostNode::new(neighbor, adjacency, new_cost, priority));
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
    fn new(index: RegionTileIndex, adjacency: Adjacency, cost: u32, priority: u32) -> Self {
        CostNode {
            index,
            adjacency,
            cost,
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

impl PartialEq for CostNode {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!()
    }
}

impl Eq for CostNode {}

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

impl<const N: usize> CostNodeQueue<N> {
    #[inline]
    fn advance(&mut self) {
        self.buckets.rotate_left(1);
        self.buckets[N - 1].clear();
        self.current += 1;
    }
}

impl<const N: usize> Default for CostNodeQueue<N> {
    fn default() -> Self {
        Self {
            buckets: array::from_fn(|_| VecDeque::new()),
            current: 0,
            len: 0,
        }
    }
}

impl<const N: usize> CostPolicyQueue for CostNodeQueue<N> {
    fn push(&mut self, node: CostNode) {
        debug_assert!(node.priority >= self.current);

        let delta = node.priority - self.current;

        debug_assert!(
            delta < N as u32,
            "priority {} exceeds window [{}, {}]",
            node.priority,
            self.current,
            self.current + (N as u32 - 1),
        );

        self.buckets[delta as usize].push_back(node);
        self.len += 1;
    }

    fn pop(&mut self) -> Option<CostNode> {
        if self.len == 0 {
            return None;
        }

        while self.buckets[0].is_empty() {
            self.advance();
        }

        self.len -= 1;
        self.buckets[0].pop_front()
    }
}

impl CostPolicy for FlowPolicy {
    type Queue = CostNodeQueue<{ SLOW_DIAGONAL_COST as usize + 1 }>;

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
    type Queue = BinaryHeap<CostNode>;

    fn priority(&self, position: TileLayerOffset, cost: u32) -> u32 {
        cost + octile_cost(position, self.goal, TileMoveSpeed::Slow)
    }

    fn finished(&self, index: RegionTileIndex) -> bool {
        index == self.goal_index
    }
}

impl CostPolicyQueue for BinaryHeap<CostNode> {
    fn push(&mut self, node: CostNode) {
        self.push(node);
    }

    fn pop(&mut self) -> Option<CostNode> {
        self.pop()
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
