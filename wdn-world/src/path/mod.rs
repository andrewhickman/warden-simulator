pub mod map;
pub mod region;
#[cfg(test)]
mod tests;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use wdn_physics::tile::storage::TileChunk;

use crate::{
    WorldSystems,
    path::{
        map::update_region_maps,
        region::{
            TileChunkSectionChanges, TileChunkSections, chunk_sections_changed,
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
                (update_regions, update_region_maps)
                    .chain()
                    .run_if(chunk_sections_changed),
            )
                .in_set(WorldSystems::UpdatePaths)
                .chain(),
        );
    }
}
