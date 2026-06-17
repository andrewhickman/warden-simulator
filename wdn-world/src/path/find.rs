use std::{collections::BinaryHeap, hash::Hash};

use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_log::info;
use bevy_math::prelude::*;
use bevy_platform::collections::{HashMap, hash_map};
use wdn_physics::tile::{index::TileIndex, position::TilePosition, storage::TileStorage};

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
    InRegion {
        region: Entity,
        cost_field: CostField,
        current: Option<(TilePosition, Dir2)>,
    },
}

#[derive(SystemParam)]
pub struct PathParam<'w, 's> {
    pub storage: TileStorage<'w, 's>,
    pub index: Res<'w, TileIndex>,
    pub chunks: Query<'w, 's, &'static TileChunkSections>,
    pub flow_fields: Query<'w, 's, &'static FlowField>,
    pub doors: Query<'w, 's, &'static DoorRegions>,
    pub regions: Query<'w, 's, &'static RegionDoors>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum SearchNodeId {
    Door(Entity),
    Position(Entity, TilePosition),
}

#[derive(Debug, Copy, Clone)]
struct SearchNode {
    id: SearchNodeId,
    position: TilePosition,
    cost: f32,
    estimated_cost: f32,
}

#[derive(Debug)]
struct SearchEntry {
    parent: SearchNodeId,
    path: SearchEntryPath,
    position: TilePosition,
    cost: f32,
}

#[derive(Debug)]
enum SearchEntryPath {
    FlowField(Entity),
    CostField(Entity, CostField),
    LazyCostField(Entity, TilePosition),
}

impl PathParam<'_, '_> {
    pub fn find_path(&self, start: TilePosition, goal: TilePosition) -> Option<Path> {
        if start.layer() != goal.layer() {
            return None;
        }

        if start == goal {
            return Some(Path {
                entries: vec![],
                cost: 0.0,
            });
        }

        let start_id = self.position_id(start)?;
        let goal_id = self.position_id(goal)?;

        let mut open: BinaryHeap<SearchNode> = BinaryHeap::new();
        let mut map: HashMap<SearchNodeId, SearchEntry> = HashMap::default();

        open.push(SearchNode {
            id: start_id,
            position: start,
            cost: 0.0,
            estimated_cost: start.distance(goal),
        });

        while let Some(node) = open.pop() {
            if node.id == goal_id {
                return Some(self.collect_path(map, goal_id));
            }

            if let Some(entry) = map.get(&node.id) {
                if node.cost > entry.cost {
                    continue;
                }
            }

            self.visit_neighbors(&node, goal, goal_id, |id, position, path, cost| {
                let new_cost = node.cost + cost;

                match map.entry(id) {
                    hash_map::Entry::Occupied(entry) if new_cost >= entry.get().cost => {
                        return;
                    }
                    entry => {
                        entry.insert(SearchEntry {
                            parent: node.id,
                            path,
                            cost: new_cost,
                            position,
                        });
                    }
                }

                let estimated_cost = new_cost + position.distance(goal);
                open.push(SearchNode {
                    id,
                    position,
                    cost: new_cost,
                    estimated_cost,
                });
            });
        }

        None
    }

    pub fn is_valid(&self, path: &Path) -> bool {
        match path.next() {
            Some(PathEntry::ToDoor { flow_field, .. }) => self.flow_fields.contains(*flow_field),
            Some(PathEntry::InRegion { region, .. }) => self.regions.contains(*region),
            None => false,
        }
    }

    pub fn path_dir(&self, path: &mut Path, position: TilePosition) -> Option<Dir2> {
        loop {
            match path.entries.last_mut() {
                Some(&mut PathEntry::ToDoor { flow_field, goal }) => {
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
                Some(PathEntry::InRegion {
                    cost_field,
                    current,
                    ..
                }) => {
                    if let Some((current_position, current_dir)) = current {
                        if *current_position == position {
                            return Some(*current_dir);
                        }
                    }

                    let cost = cost_field.get_cost(position.layer_offset())?;
                    let dir = cost_field.flow_vector(
                        position.layer_offset(),
                        cost,
                        self.storage.get_adjacency(position).solid(),
                    );

                    *current = Some((position, dir));
                    return Some(dir);
                }
                None => return None,
            }
        }
    }

    fn generate_cost_field_path(
        &self,
        region: Entity,
        start: TilePosition,
        goal: TilePosition,
    ) -> CostField {
        let mut cost_field = CostField::new();
        let region_doors = self.regions.get(region).expect("invalid region");
        let policy = PathPolicy::new(start.layer_offset());
        let adjacency = self.storage.get_adjacency(goal).walls();

        cost_field.generate(
            &policy,
            &self.storage,
            region_doors,
            goal,
            adjacency.complement(),
        );
        debug_assert!(
            cost_field.contains(start.layer_offset()),
            "cost field does not contain start position: {:#?}",
            cost_field
        );

        cost_field
    }

    fn position_id(&self, position: TilePosition) -> Option<SearchNodeId> {
        let chunk_id = self.storage.chunk_id(position.chunk_position())?;
        let chunk_sections = self.chunks.get(chunk_id).ok()?;

        if let Some(region) = chunk_sections.region(position.chunk_offset()) {
            Some(SearchNodeId::Position(region, position))
        } else if let Some(door) = self.index.get_tile(position) {
            Some(SearchNodeId::Door(door))
        } else {
            None
        }
    }

    fn visit_neighbors(
        &self,
        node: &SearchNode,
        goal: TilePosition,
        goal_id: SearchNodeId,
        mut f: impl FnMut(SearchNodeId, TilePosition, SearchEntryPath, f32),
    ) {
        match node.id {
            SearchNodeId::Position(region, position) => {
                let region_doors = self.regions.get(region).expect("invalid region");
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
                        SearchEntryPath::FlowField(region_door.flow_field()),
                        cost,
                    );
                }

                if goal_id.in_region(region) {
                    let cost_field = self.generate_cost_field_path(region, position, goal);
                    let cost = cost_field
                        .get_cost(position.layer_offset())
                        .expect("position not in cost field");
                    f(
                        goal_id,
                        goal,
                        SearchEntryPath::CostField(region, cost_field),
                        cost,
                    );
                }
            }
            SearchNodeId::Door(door) => {
                let door_regions = self.doors.get(door).expect("invalid door");
                for door_region in door_regions.iter() {
                    let region_doors = self
                        .regions
                        .get(door_region.region())
                        .expect("invalid region");
                    let flow_field = self
                        .flow_fields
                        .get(door_region.flow_field())
                        .expect("invalid flow field");

                    for (neighbor_position, region_door) in region_doors.iter() {
                        if region_door.door() == door {
                            continue;
                        }

                        let cost = flow_field
                            .get(neighbor_position)
                            .expect("position not in flow field")
                            .cost();
                        f(
                            SearchNodeId::Door(region_door.door()),
                            TilePosition::from((node.position.layer(), neighbor_position)),
                            SearchEntryPath::FlowField(region_door.flow_field()),
                            cost,
                        );
                    }

                    if goal_id.in_region(door_region.region()) {
                        let cost = flow_field
                            .get(goal.layer_offset())
                            .expect("goal not in flow field")
                            .cost();
                        f(
                            goal_id,
                            goal,
                            SearchEntryPath::LazyCostField(door_region.region(), node.position),
                            cost,
                        );
                    }
                }
            }
        }
    }

    fn collect_path(
        &self,
        mut map: HashMap<SearchNodeId, SearchEntry>,
        goal: SearchNodeId,
    ) -> Path {
        let mut entries = Vec::new();

        let mut current = goal;
        let cost = map[&current].cost;

        while let Some(entry) = map.remove(&current) {
            let path_entry = match entry.path {
                SearchEntryPath::FlowField(flow_field) => PathEntry::ToDoor {
                    flow_field,
                    goal: entry.position,
                },
                SearchEntryPath::CostField(region, cost_field) => PathEntry::InRegion {
                    region,
                    cost_field,
                    current: None,
                },
                SearchEntryPath::LazyCostField(region, start) => PathEntry::InRegion {
                    region: region,
                    cost_field: self.generate_cost_field_path(region, start, entry.position),
                    current: None,
                },
            };

            entries.push(path_entry);
            current = entry.parent;
        }

        info!(
            "found path with cost {:?} and entries: {:#?} entries",
            cost, entries
        );

        Path { cost, entries }
    }
}

impl Path {
    pub fn cost(&self) -> f32 {
        self.cost
    }

    pub fn iter(&self) -> impl Iterator<Item = &PathEntry> {
        self.entries.iter().rev()
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

impl SearchNodeId {
    fn in_region(&self, region: Entity) -> bool {
        match self {
            SearchNodeId::Position(r, _) => *r == region,
            SearchNodeId::Door(_) => false,
        }
    }
}
