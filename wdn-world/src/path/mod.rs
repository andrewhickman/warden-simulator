pub mod region;
#[cfg(test)]
mod tests;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use wdn_physics::tile::storage::TileChunk;

use crate::{
    WorldSystems,
    path::region::{
        TileChunkSections, TileChunkSectionsChanged, update_layer_regions,
        update_tile_chunk_sections,
    },
};

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TileChunkSectionsChanged>();

        app.register_required_components::<TileChunk, TileChunkSections>();

        app.add_systems(
            FixedUpdate,
            (update_tile_chunk_sections, update_layer_regions)
                .chain()
                .in_set(WorldSystems::UpdatePaths),
        );
    }
}
