use bevy_ecs::prelude::*;
use bevy_log::warn;
use bevy_platform::collections::{HashMap, hash_map};
use wdn_physics::tile::{adjacency::Adjacency, index::TileIndex, position::TilePosition};

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
    mut regions: Query<(&LayerRegion, &mut LayerRegionMap), Added<LayerRegion>>,
    sections: Query<&TileChunkSections>,
) {
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
    });
}

impl LayerRegionMap {
    pub fn doors(&self) -> impl Iterator<Item = Entity> {
        self.doors.values().map(|flow| flow.id)
    }
}
