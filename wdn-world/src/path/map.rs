use bevy_ecs::prelude::*;
use bevy_log::warn;
use bevy_platform::collections::{HashMap, hash_map};
use bevy_tasks::ComputeTaskPool;
use wdn_physics::tile::{
    adjacency::DoorAdjacency, index::TileIndex, position::TilePosition, storage::TileStorage,
};

use crate::path::region::{LayerRegion, TileChunkSections};

#[derive(Component, Default)]
pub struct LayerRegionMap {
    doors: HashMap<TilePosition, DoorFlowMap>,
}

pub struct DoorFlowMap {
    pub id: Entity,
    pub adjacency: DoorAdjacency,
    pub flow: HashMap<TilePosition, f32>,
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
            let doors = sections
                .get(chunk_id)
                .expect("chunk not found")
                .doors(section_id.chunk_offset())
                .expect("section not found");

            for (door_position, adjacency) in doors {
                match map.doors.entry(door_position) {
                    hash_map::Entry::Occupied(mut entry) => {
                        entry.get_mut().adjacency.insert(adjacency);
                    }
                    hash_map::Entry::Vacant(entry) => {
                        let Some(id) = index.get_tile(door_position) else {
                            warn!("door at {:?} not found", door_position);
                            continue;
                        };

                        entry.insert(DoorFlowMap {
                            id,
                            adjacency,
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
                        generate_flow_map(door_position, flow, storage);
                    });
                });
            });
        } else {
            map.doors.iter_mut().for_each(|(&door_position, flow)| {
                generate_flow_map(door_position, flow, storage);
            });
        }
    });
}

impl LayerRegionMap {
    pub fn doors(&self) -> impl Iterator<Item = Entity> {
        self.doors.values().map(|flow| flow.id)
    }
}

fn generate_flow_map(
    _door_position: TilePosition,
    _flow: &mut DoorFlowMap,
    _storage: &TileStorage,
) {
}
