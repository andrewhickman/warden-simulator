pub mod region;
#[cfg(test)]
mod tests;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use wdn_physics::tile::storage::TileChunk;

use crate::{
    WorldSystems,
    path::region::{TileChunkSections, update_tile_chunk_sections as update_regions},
};

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<TileChunk, TileChunkSections>();

        app.add_systems(
            FixedUpdate,
            update_regions.in_set(WorldSystems::UpdatePaths),
        );
    }
}
