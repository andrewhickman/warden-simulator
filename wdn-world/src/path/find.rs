use std::{
    collections::{BinaryHeap, VecDeque},
    hash::Hash,
};

use bevy_ecs::{entity::EntityHash, prelude::*, system::SystemParam};
use bevy_log::info;
use bevy_math::prelude::*;
use bevy_platform::collections::{HashMap, hash_map};
use wdn_physics::tile::{position::TilePosition, storage::TileStorage};

use crate::path::{
    door::{DoorRegions, RegionDoors},
    flow::{CostField, FlowField, PathPolicy},
    region::TileChunkSections,
};

#[derive(Debug)]
pub struct Path {
    cost: f32,
    entries: Vec<PathEntry>,
}

#[derive(Debug)]
pub enum PathEntry {
    ToDoor {
        flow_field: Entity,
        goal: TilePosition,
    },
    FromDoor {
        flow_field: Entity,
        goal: TilePosition,
    },
    InRegion {
        region: Entity,
        cost_field: CostField,
    },
}

#[derive(SystemParam)]
pub struct PathParam<'w, 's> {
    pub storage: TileStorage<'w, 's>,
    pub chunks: Query<'w, 's, &'static TileChunkSections>,
    pub flow_fields: Query<'w, 's, &'static FlowField>,
    pub doors: Query<'w, 's, &'static DoorRegions>,
    pub regions: Query<'w, 's, &'static RegionDoors>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SearchNodeId {
    Start,
    Door(Entity),
    Goal,
}

#[derive(Debug, Copy, Clone)]
struct SearchNode {
    region: Entity,
    id: SearchNodeId,
    position: TilePosition,
    cost: f32,
    estimated_cost: f32,
}

#[derive(Debug, Copy, Clone)]
struct SearchEntry {
    parent: SearchNodeId,
    flow_field: Entity,
    position: TilePosition,
    cost: f32,
}

impl PathParam<'_, '_> {
    pub fn find_path(&self, from: TilePosition, to: TilePosition) -> Option<Path> {
        if from.layer() != to.layer() {
            return None;
        }

        if from == to {
            return Some(Path {
                entries: vec![],
                cost: 0.0,
            });
        }

        let from_region = self.tile_region(from)?;
        let to_region = self.tile_region(to)?;

        if from_region != to_region {
            self.find_path_between_regions(from_region, from, to_region, to)
        } else {
            self.find_path_in_region(from_region, from, to)
        }
    }

    pub fn is_valid(&self, path: &Path) -> bool {
        match path.next() {
            Some(PathEntry::FromDoor { flow_field, .. })
            | Some(PathEntry::ToDoor { flow_field, .. }) => self.flow_fields.contains(*flow_field),
            Some(PathEntry::InRegion { region, .. }) => self.regions.contains(*region),
            None => false,
        }
    }

    pub fn path_dir(&self, path: &mut Path, position: TilePosition) -> Option<Dir2> {
        loop {
            match path.next() {
                Some(&PathEntry::ToDoor { flow_field, goal }) => {
                    if position == goal {
                        path.entries.pop();
                    } else {
                        return Some(
                            self.flow_fields
                                .get(flow_field)
                                .ok()?
                                .get(position.layer_offset())?
                                .dir(),
                        );
                    }
                }
                Some(&PathEntry::FromDoor { flow_field, goal }) => {
                    if position == goal {
                        path.entries.pop();
                    } else {
                        return Some(
                            self.flow_fields
                                .get(flow_field)
                                .ok()?
                                .get(position.layer_offset())?
                                .reverse_dir(),
                        );
                    }
                }
                Some(PathEntry::InRegion { cost_field, .. }) => {
                    let cost = cost_field.get_cost(position.layer_offset())?;
                    let dir = cost_field.flow_vector(
                        position.layer_offset(),
                        cost,
                        self.storage.get_adjacency(position).solid(),
                    );

                    return Some(dir);
                }
                None => return None,
            }
        }
    }

    fn find_path_in_region(
        &self,
        region: Entity,
        from: TilePosition,
        to: TilePosition,
    ) -> Option<Path> {
        info!(
            "Finding path in region {:?} from {:?} to {:?}",
            region, from, to
        );

        let mut cost_field = CostField::new();
        let region_doors = self.regions.get(region).expect("invalid region");
        let policy = PathPolicy::new(from.layer_offset());
        let adjacency = self.storage.get_adjacency(to).solid();

        cost_field.generate(
            &policy,
            &self.storage,
            region_doors,
            to,
            adjacency.complement(),
        );
        info!(
            "Generated cost field for region {:?} with {} entries",
            region,
            cost_field.len()
        );
        debug_assert!(cost_field.contains(from.layer_offset()));
        let cost = cost_field
            .get_cost(from.layer_offset())
            .expect("position not in cost field");

        Some(Path {
            cost,
            entries: vec![PathEntry::InRegion { region, cost_field }],
        })
    }

    fn find_path_between_regions(
        &self,
        start_region: Entity,
        start: TilePosition,
        goal_region: Entity,
        goal: TilePosition,
    ) -> Option<Path> {
        let mut open: BinaryHeap<SearchNode> = BinaryHeap::new();
        let mut map: HashMap<SearchNodeId, SearchEntry, EntityHash> = HashMap::default();

        open.push(SearchNode {
            region: start_region,
            id: SearchNodeId::Start,
            position: start,
            cost: 0.0,
            estimated_cost: start.distance(goal),
        });

        while let Some(node) = open.pop() {
            info!("pop node {:?} at {:?}", node.id, node.position);

            if node.id == SearchNodeId::Goal {
                return Some(self.collect_path(map));
            }

            if let Some(entry) = map.get(&node.id) {
                if node.cost > entry.cost {
                    info!(
                        "skip node {:?} at {:?} with cost {} (existing cost {})",
                        node.id, node.position, node.cost, entry.cost
                    );
                    continue;
                }
            }

            self.visit_neighbors(
                &node,
                goal_region,
                goal,
                |id, position, region, flow_field, cost| {
                    info!(
                        "visit neighbor {:?} at {:?} with cost {}",
                        id, position, cost
                    );

                    let new_cost = node.cost + cost;

                    match map.entry(id) {
                        hash_map::Entry::Occupied(entry) if new_cost >= entry.get().cost => {
                            return;
                        }
                        entry => {
                            entry.insert(SearchEntry {
                                parent: node.id,
                                flow_field,
                                cost: new_cost,
                                position,
                            });
                        }
                    }

                    let estimated_cost = new_cost + position.distance(goal);
                    open.push(SearchNode {
                        region,
                        id,
                        position,
                        cost: new_cost,
                        estimated_cost,
                    });
                },
            );
        }

        None
    }

    fn tile_region(&self, position: TilePosition) -> Option<Entity> {
        let chunk_id = self.storage.chunk_id(position.chunk_position())?;
        let chunk_sections = self.chunks.get(chunk_id).ok()?;
        chunk_sections.region(position.chunk_offset())
    }

    fn visit_neighbors(
        &self,
        node: &SearchNode,
        goal_region: Entity,
        goal: TilePosition,
        mut f: impl FnMut(SearchNodeId, TilePosition, Entity, Entity, f32),
    ) {
        match node.id {
            SearchNodeId::Start => {
                let region_doors = self.regions.get(node.region).expect("invalid region");
                for (door_position, region_door) in region_doors.iter() {
                    let flow_field = self
                        .flow_fields
                        .get(region_door.flow_field())
                        .expect("invalid flow field");
                    let cost = flow_field
                        .get(node.position.layer_offset())
                        .expect("position not in flow field")
                        .cost();
                    f(
                        SearchNodeId::Door(region_door.door()),
                        TilePosition::from((node.position.layer(), door_position)),
                        node.region,
                        region_door.flow_field(),
                        cost,
                    );
                }
            }
            SearchNodeId::Door(door) => {
                info!("evaluate neighbors of door {:?}", door);

                let door_regions = self.doors.get(door).expect("invalid door");
                for door_region in door_regions.iter() {
                    let flow_field = self
                        .flow_fields
                        .get(door_region.flow_field())
                        .expect("invalid flow field");
                    for (neighbor_position, cost, neighbor_door) in flow_field.doors() {
                        info!("neighbor door: {:?}", neighbor_door);
                        f(
                            SearchNodeId::Door(neighbor_door),
                            TilePosition::from((node.position.layer(), neighbor_position)),
                            door_region.region(),
                            door_region.flow_field(),
                            cost,
                        );
                    }

                    if door_region.region() == goal_region {
                        info!("door region is goal region, adding goal node");
                        let cost = flow_field
                            .get(goal.layer_offset())
                            .expect("goal not in flow field")
                            .cost();
                        f(
                            SearchNodeId::Goal,
                            goal,
                            door_region.region(),
                            door_region.flow_field(),
                            cost,
                        );
                    }
                }
            }
            SearchNodeId::Goal => unreachable!(),
        }
    }

    fn collect_path(&self, mut map: HashMap<SearchNodeId, SearchEntry, EntityHash>) -> Path {
        let mut entries = Vec::new();

        let goal_entry = map[&SearchNodeId::Goal];

        entries.push(PathEntry::FromDoor {
            flow_field: goal_entry.flow_field,
            goal: goal_entry.position,
        });

        let mut current = goal_entry.parent;
        while current != SearchNodeId::Start {
            let entry = map.remove(&current).expect("invalid path");
            entries.push(PathEntry::ToDoor {
                flow_field: entry.flow_field,
                goal: entry.position,
            });
            current = entry.parent;
        }

        info!(
            "found path with cost {:?} and entries: {:#?} entries",
            goal_entry.cost, entries
        );

        Path {
            cost: goal_entry.cost,
            entries,
        }
    }
}

impl Path {
    pub fn cost(&self) -> f32 {
        self.cost
    }

    pub fn next(&self) -> Option<&PathEntry> {
        self.entries.last()
    }
}

impl PartialEq for SearchNode {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!()
    }
}

impl Eq for SearchNode {}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.estimated_cost.total_cmp(&self.estimated_cost)
    }
}

impl Hash for SearchNodeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            SearchNodeId::Start => u64::MAX.hash(state),
            SearchNodeId::Door(entity) => entity.hash(state),
            SearchNodeId::Goal => (u64::MAX - 1).hash(state),
        }
    }
}
