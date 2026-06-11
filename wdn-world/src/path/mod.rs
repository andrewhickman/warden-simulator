pub mod find;
pub mod flow;
pub mod region;
#[cfg(test)]
mod tests;

use std::any::type_name_of_val;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use wdn_physics::tile::storage::TileChunk;

use crate::{
    WorldSystems,
    path::{
        flow::{on_remove_region_doors, update_flow_fields, update_region_doors},
        region::{
            TileChunkSectionChanges, TileChunkSections, chunk_sections_changed, on_add_region,
            update_chunk_sections, update_regions,
        },
    },
};

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<TileChunk, TileChunkSections>();

        app.init_resource::<TileChunkSectionChanges>();

        app.add_systems(
            FixedUpdate,
            (
                update_chunk_sections,
                (update_regions, update_region_doors, update_flow_fields)
                    .chain()
                    .run_if(chunk_sections_changed),
            )
                .in_set(WorldSystems::UpdateRegions)
                .chain(),
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
