pub mod adjacency;
pub mod index;
pub mod material;
pub mod position;
pub mod storage;
#[cfg(test)]
mod tests;

use std::any::type_name_of_val;

use bevy_app::prelude::*;
use bevy_ecs::name::Name;

use crate::tile::{
    index::TileIndex,
    material::{on_insert_material, on_remove_material},
    storage::{TileMap, TileMapBuffer},
};

pub const CHUNK_SIZE: usize = 32;

pub struct TilePlugin;

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
            .add_observer(on_remove_material)
            .insert(Name::new(format!(
                "Observer({})",
                type_name_of_val(&on_remove_material)
            )));
    }
}
