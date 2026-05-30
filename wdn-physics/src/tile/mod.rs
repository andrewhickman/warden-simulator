pub mod adjacency;
pub mod index;
pub mod material;
pub mod position;
pub mod storage;
#[cfg(test)]
mod tests;

use std::any::type_name_of_val;

use bevy_app::prelude::*;
use bevy_ecs::{component::Component, name::Name};

use crate::tile::{
    adjacency::{TileAdjacency, on_add_adjacency},
    index::TileIndex,
    material::{TileMaterial, on_insert_material},
    position::TilePosition,
    storage::{TileMap, TileMapBuffer},
};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_SQUARED: usize = CHUNK_SIZE * CHUNK_SIZE;

pub struct TilePlugin;

#[derive(Component, Clone, Copy, Debug, Default)]
#[require(TilePosition, TileMaterial, TileAdjacency)]
pub struct Tile;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileIndex>()
            .init_resource::<TileMap>()
            .init_resource::<TileMapBuffer>();

        app.world_mut()
            .add_observer(on_insert_material)
            .insert(Name::new(format!(
                "Observer({})",
                type_name_of_val(&on_insert_material)
            )));
        app.world_mut()
            .add_observer(on_add_adjacency)
            .insert(Name::new(format!(
                "Observer({})",
                type_name_of_val(&on_add_adjacency)
            )));
    }
}
