use std::ops::Index;

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
    region::{Region, TileChunkSections},
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
    north: Option<DoorRegion>,
    south: Option<DoorRegion>,
    east: Option<DoorRegion>,
    west: Option<DoorRegion>,
}

#[derive(Copy, Clone, Debug)]
pub struct DoorRegion {
    region: Entity,
    flow_field: Entity,
    dead_end: bool,
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

            let dead_end = region_doors.door_count() == 1;
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

                doors.get_mut(door.door).expect("door not found").insert(
                    region_id,
                    door.flow_field,
                    door.adjacency,
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
        [
            self.north.as_ref(),
            self.south.as_ref(),
            self.east.as_ref(),
            self.west.as_ref(),
        ]
        .into_iter()
        .flatten()
    }

    pub fn north(&self) -> Option<&DoorRegion> {
        self.north.as_ref()
    }

    pub fn south(&self) -> Option<&DoorRegion> {
        self.south.as_ref()
    }

    pub fn east(&self) -> Option<&DoorRegion> {
        self.east.as_ref()
    }

    pub fn west(&self) -> Option<&DoorRegion> {
        self.west.as_ref()
    }

    pub fn flow_fields(&self) -> impl Iterator<Item = Entity> {
        self.iter().map(|door_region| door_region.flow_field())
    }

    pub fn insert(&mut self, region: Entity, flow: Entity, adjacency: Adjacency, dead_end: bool) {
        if adjacency.contains(Adjacency::NORTH) {
            debug_assert!(self.north.is_none());
            self.north = Some(DoorRegion::new(region, flow, dead_end));
        }

        if adjacency.contains(Adjacency::SOUTH) {
            debug_assert!(self.south.is_none());
            self.south = Some(DoorRegion::new(region, flow, dead_end));
        }

        if adjacency.contains(Adjacency::EAST) {
            debug_assert!(self.east.is_none());
            self.east = Some(DoorRegion::new(region, flow, dead_end));
        }

        if adjacency.contains(Adjacency::WEST) {
            debug_assert!(self.west.is_none());
            self.west = Some(DoorRegion::new(region, flow, dead_end));
        }
    }
}

impl DoorRegion {
    pub fn new(region: Entity, flow_field: Entity, dead_end: bool) -> Self {
        Self {
            region,
            flow_field,
            dead_end,
        }
    }

    pub fn region(&self) -> Entity {
        self.region
    }

    pub fn flow_field(&self) -> Entity {
        self.flow_field
    }

    pub fn dead_end(&self) -> bool {
        self.dead_end
    }
}

fn remove_door_region(flow: &mut Option<DoorRegion>, region: Entity) {
    if matches!(flow, Some(door_region) if door_region.region == region) {
        *flow = None;
    }
}
