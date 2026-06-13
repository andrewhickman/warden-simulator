use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_log::info;
use bevy_math::Dir2;
use wdn_physics::tile::{position::TilePosition, storage::TileStorage};

use crate::path::{
    flow::{CostField, DoorRegions, FlowField, PathPolicy, RegionDoors},
    region::TileChunkSections,
};

#[derive(Debug)]
pub struct Path {
    entries: Vec<PathEntry>,
}

#[derive(Debug)]
pub enum PathEntry {
    ToDoor {
        flow_field: Entity,
    },
    FromDoor {
        flow_field: Entity,
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

impl PathParam<'_, '_> {
    pub fn find_path(&self, from: TilePosition, to: TilePosition) -> Option<Path> {
        if from.layer() != to.layer() {
            return None;
        }

        if from == to {
            return Some(Path { entries: vec![] });
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
            Some(PathEntry::FromDoor { flow_field }) | Some(PathEntry::ToDoor { flow_field }) => {
                self.flow_fields.contains(*flow_field)
            }
            Some(PathEntry::InRegion { region, .. }) => self.regions.contains(*region),
            None => false,
        }
    }

    pub fn path_dir(&self, path: &Path, position: TilePosition) -> Option<Dir2> {
        match path.next() {
            Some(PathEntry::ToDoor { flow_field }) => Some(
                self.flow_fields
                    .get(*flow_field)
                    .ok()?
                    .get(position.layer_offset())?
                    .dir(),
            ),
            Some(PathEntry::FromDoor { flow_field }) => Some(
                self.flow_fields
                    .get(*flow_field)
                    .ok()?
                    .get(position.layer_offset())?
                    .reverse_dir(),
            ),
            Some(PathEntry::InRegion { region, cost_field }) => {
                if !self.regions.contains(*region) {
                    return None;
                }

                let cost = cost_field.get_cost(position.layer_offset())?;
                let dir = cost_field.flow_vector(
                    position.layer_offset(),
                    cost,
                    self.storage.get_adjacency(position).solid(),
                );

                Some(dir)
            }
            None => None,
        }
    }

    fn find_path_between_regions(
        &self,
        from_region: Entity,
        from: TilePosition,
        to_region: Entity,
        to: TilePosition,
    ) -> Option<Path> {
        todo!()
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

        Some(Path {
            entries: vec![PathEntry::InRegion { region, cost_field }],
        })
    }

    fn tile_region(&self, position: TilePosition) -> Option<Entity> {
        let chunk_id = self.storage.chunk_id(position.chunk_position())?;
        let chunk_sections = self.chunks.get(chunk_id).ok()?;
        chunk_sections.region(position.chunk_offset())
    }
}

impl Path {
    pub fn next(&self) -> Option<&PathEntry> {
        self.entries.last()
    }
}
