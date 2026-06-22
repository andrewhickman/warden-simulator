pub mod door;
pub mod find;
pub mod flow;
pub mod region;
pub mod section;
#[cfg(test)]
mod tests;

use std::any::type_name_of_val;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use wdn_physics::tile::storage::TileChunk;

use crate::{
    WorldSystems,
    path::{
        door::{on_remove_region_doors, update_door_regions},
        flow::{AddedFlowFields, clear_added_flow_fields, flow_fields_added, update_flow_fields},
        region::{
            AddedRegions, clear_added_regions, on_add_region, regions_added, update_region_doors,
            update_region_tiles, update_regions,
        },
        section::{
            TileChunkSectionChanges, TileChunkSections, chunk_sections_changed,
            update_chunk_sections,
        },
    },
};

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<TileChunk, TileChunkSections>();

        app.init_resource::<TileChunkSectionChanges>()
            .init_resource::<AddedRegions>()
            .init_resource::<AddedFlowFields>();

        app.add_systems(
            FixedUpdate,
            (
                update_chunk_sections,
                update_regions.run_if(chunk_sections_changed),
                (
                    update_region_tiles,
                    update_region_doors,
                    (update_flow_fields, clear_added_flow_fields)
                        .chain()
                        .run_if(flow_fields_added),
                    update_door_regions,
                    clear_added_regions,
                )
                    .chain()
                    .run_if(regions_added),
            )
                .chain()
                .in_set(WorldSystems::UpdateRegions),
        );

        app.world_mut()
            .add_observer(on_add_region)
            .insert(Name::new(format!(
                "Observer({})",
                type_name_of_val(&on_add_region)
            )));
        app.world_mut()
            .add_observer(on_remove_region_doors)
            .insert(Name::new(format!(
                "Observer({})",
                type_name_of_val(&on_remove_region_doors)
            )));
    }
}
