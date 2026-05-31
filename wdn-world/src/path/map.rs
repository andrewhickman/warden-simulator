use bevy_ecs::prelude::*;
use bevy_log::warn;
use bevy_platform::collections::{HashMap, hash_map};
use bevy_tasks::ComputeTaskPool;
use wdn_physics::tile::{
    adjacency::{Adjacency, TileAdjacency},
    index::TileIndex,
    position::TilePosition,
    storage::TileStorage,
};

use crate::path::region::{LayerRegion, TileChunkSections};

#[derive(Component, Default)]
pub struct LayerRegionMap {
    doors: HashMap<TilePosition, DoorFlowMap>,
}

pub struct DoorFlowMap {
    pub id: Entity,
    pub door_adjacency: Adjacency,
    pub flow: HashMap<TilePosition, u32>,
}

pub fn update_region_maps(
    index: Res<TileIndex>,
    storage: TileStorage,
    mut regions: Query<(&LayerRegion, &mut LayerRegionMap), Added<LayerRegion>>,
    sections: Query<&TileChunkSections>,
) {
    let storage = &storage;
    regions.par_iter_mut().for_each(|(region, mut map)| {
        for (chunk_id, section_id) in region.sections() {
            let section = sections
                .get(chunk_id)
                .expect("chunk not found")
                .section(section_id.chunk_offset());

            for (door_position, door_adjacency) in section.doors() {
                match map.doors.entry(door_position) {
                    hash_map::Entry::Occupied(mut entry) => {
                        entry.get_mut().door_adjacency.insert(door_adjacency);
                    }
                    hash_map::Entry::Vacant(entry) => {
                        let Some(id) = index.get_tile(door_position) else {
                            warn!("door at {:?} not found", door_position);
                            continue;
                        };

                        entry.insert(DoorFlowMap {
                            id,
                            door_adjacency,
                            flow: HashMap::default(),
                        });
                    }
                }
            }
        }

        if map.doors.len() > 1 {
            ComputeTaskPool::get().scope(|scope| {
                map.doors.iter_mut().for_each(|(&door_position, flow)| {
                    scope.spawn(async move {
                        flow.generate(door_position, storage);
                    });
                });
            });
        } else {
            map.doors.iter_mut().for_each(|(&door_position, flow)| {
                flow.generate(door_position, storage);
            });
        }
    });
}

impl LayerRegionMap {
    pub fn doors(&self) -> impl Iterator<Item = Entity> {
        self.doors.values().map(|flow| flow.id)
    }
}

impl DoorFlowMap {
    fn generate(&mut self, door: TilePosition, storage: &TileStorage) {
        self.flow.insert(door, 0);

        let mut open = Vec::new();
        if self.door_adjacency.contains(Adjacency::NORTH) {
            self.flow.insert(door.south(), 1);
            open.push(door.south());
        }

        if self.door_adjacency.contains(Adjacency::SOUTH) {
            self.flow.insert(door.north(), 1);
            open.push(door.north());
        }

        if self.door_adjacency.contains(Adjacency::EAST) {
            self.flow.insert(door.west(), 1);
            open.push(door.west());
        }

        if self.door_adjacency.contains(Adjacency::WEST) {
            self.flow.insert(door.east(), 1);
            open.push(door.east());
        }

        while let Some(position) = open.pop() {
            let adjacency = storage.get_adjacency(position);
        }
    }
}

fn visit_tile_neighbors(
    adjacency: TileAdjacency,
    position: TilePosition,
    mut f: impl FnMut(TilePosition),
) {
    let adjacency = adjacency.walls() | adjacency.doors();

    if adjacency.contains(Adjacency::NORTH) {
        f(position.north());
    }

    if adjacency.contains(Adjacency::SOUTH) {
        f(position.south());
    }

    if adjacency.contains(Adjacency::EAST) {
        f(position.east());
    }

    if adjacency.contains(Adjacency::WEST) {
        f(position.west());
    }

    if !adjacency.intersects(other)
}
