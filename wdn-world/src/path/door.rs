use std::ops::Index;

use arrayvec::ArrayVec;
use bevy_ecs::prelude::*;
use bevy_log::error;
use bevy_platform::collections::{HashMap, hash_map};
use wdn_physics::tile::{
    adjacency::Adjacency,
    index::TileIndex,
    position::{TileLayerOffset, TilePosition},
};

use crate::path::{
    flow::FlowField,
    region::{AddedRegions, Region, TileChunkSections},
};

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
    regions: ArrayVec<DoorRegion, 4>,
}

#[derive(Copy, Clone, Debug)]
pub struct DoorRegion {
    region: Entity,
    flow_field: Entity,
    adjacency: Adjacency,
    dead_end: bool,
}

pub fn update_region_doors(
    mut commands: Commands,
    index: Res<TileIndex>,
    mut regions: Query<(Entity, &Region, &mut RegionDoors), Added<Region>>,
    sections: Query<&TileChunkSections>,
    added_regions: Res<AddedRegions>,
) {
    regions.iter_many_unique_mut(added_regions.iter()).for_each(
        |(region_id, region, mut region_doors)| {
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
                        FlowField::new(
                            TilePosition::from((region.layer(), door_position)),
                            door.adjacency,
                        ),
                    ))
                    .id();
            }
        },
    );
}

pub fn update_door_regions(
    mut doors: Query<&mut DoorRegions>,
    regions: Query<(Entity, &RegionDoors)>,
    added_regions: Res<AddedRegions>,
) {
    regions
        .iter_many_unique(added_regions.iter())
        .for_each(|(region_id, region_doors)| {
            let dead_end = region_doors.door_count() == 1;
            for region_door in region_doors.iter_values() {
                let mut door_regions = doors.get_mut(region_door.door()).unwrap();
                door_regions.insert(
                    region_id,
                    region_door.flow_field(),
                    region_door.adjacency(),
                    dead_end,
                );
            }
        });
}

pub fn on_remove_region_doors(
    trigger: On<Remove, RegionDoors>,
    regions: Query<&RegionDoors>,
    mut doors: Query<&mut DoorRegions>,
) -> Result {
    let region = regions.get(trigger.entity)?;
    for (_, region_door) in region.iter() {
        if let Ok(mut door_regions) = doors.get_mut(region_door.door()) {
            door_regions
                .regions
                .retain(|door_region| door_region.region() != trigger.entity);
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

    pub fn iter_values(&self) -> impl Iterator<Item = &RegionDoor> {
        self.doors.values()
    }

    pub fn door_count(&self) -> usize {
        self.doors.len()
    }
}

impl Index<TileLayerOffset> for RegionDoors {
    type Output = RegionDoor;

    fn index(&self, index: TileLayerOffset) -> &Self::Output {
        self.doors.get(&index).expect("door not found at position")
    }
}

impl RegionDoor {
    pub fn adjacency(&self) -> Adjacency {
        self.adjacency
    }

    pub fn flow_field(&self) -> Entity {
        self.flow_field
    }

    pub fn door(&self) -> Entity {
        self.door
    }
}

impl DoorRegions {
    pub fn iter(&self) -> impl Iterator<Item = &DoorRegion> {
        self.regions.iter()
    }

    pub fn north(&self) -> Option<&DoorRegion> {
        self.iter()
            .find(|door_region| door_region.adjacency().contains(Adjacency::NORTH))
    }

    pub fn west(&self) -> Option<&DoorRegion> {
        self.iter()
            .find(|door_region| door_region.adjacency().contains(Adjacency::WEST))
    }

    pub fn south(&self) -> Option<&DoorRegion> {
        self.iter()
            .find(|door_region| door_region.adjacency().contains(Adjacency::SOUTH))
    }

    pub fn east(&self) -> Option<&DoorRegion> {
        self.iter()
            .find(|door_region| door_region.adjacency().contains(Adjacency::EAST))
    }

    pub fn flow_fields(&self) -> impl Iterator<Item = Entity> {
        self.iter().map(|door_region| door_region.flow_field())
    }

    pub fn insert(&mut self, region: Entity, flow: Entity, adjacency: Adjacency, dead_end: bool) {
        debug_assert!(
            !self
                .regions
                .iter()
                .any(|door_region| door_region.region() == region
                    || door_region.adjacency.intersects(adjacency))
        );

        self.regions.push(DoorRegion {
            region,
            flow_field: flow,
            adjacency,
            dead_end,
        });
    }

    pub fn remove(&mut self, region: Entity) {
        if let Some(index) = self
            .regions
            .iter()
            .position(|door_region| door_region.region() == region)
        {
            self.regions.swap_remove(index);
        }
    }
}

impl DoorRegion {
    pub fn region(&self) -> Entity {
        self.region
    }

    pub fn flow_field(&self) -> Entity {
        self.flow_field
    }

    pub fn adjacency(&self) -> Adjacency {
        self.adjacency
    }

    pub fn dead_end(&self) -> bool {
        self.dead_end
    }
}
