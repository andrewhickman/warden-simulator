use arrayvec::ArrayVec;
use bevy_ecs::prelude::*;
use wdn_physics::tile::adjacency::Adjacency;

use crate::path::region::{AddedRegions, RegionTiles};

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

pub fn update_door_regions(
    mut doors: Query<&mut DoorRegions>,
    regions: Query<(Entity, &RegionTiles)>,
    added_regions: Res<AddedRegions>,
) {
    regions
        .iter_many_unique(added_regions.iter())
        .for_each(|(region_id, region_tiles)| {
            let dead_end = region_tiles.door_count() == 1;
            for region_door in region_tiles.doors() {
                let mut door_regions = doors.get_mut(region_door.door()).expect("invalid door");
                door_regions.insert(
                    region_id,
                    region_door.flow_field(),
                    region_door.adjacency(),
                    dead_end,
                );
            }
        });
}

pub fn on_remove_region(
    trigger: On<Remove, RegionTiles>,
    regions: Query<&RegionTiles>,
    mut doors: Query<&mut DoorRegions>,
) -> Result {
    let region = regions.get(trigger.entity)?;
    for region_door in region.doors() {
        if let Ok(mut door_regions) = doors.get_mut(region_door.door()) {
            door_regions.remove(trigger.entity);
        }
    }

    Ok(())
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
                    || door_region.adjacency.intersects(adjacency)),
            "new door region {region:?} with adjacency {adjacency:?} overlaps with existing door regions: {:#?}",
            self.regions,
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
