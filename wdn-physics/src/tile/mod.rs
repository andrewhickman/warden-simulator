pub mod adjacency;
pub mod index;
pub mod position;
pub mod storage;
#[cfg(test)]
mod tests;

use bevy_app::prelude::*;

use crate::tile::{index::TileIndex, storage::TileMap};

pub const CHUNK_SIZE: usize = 32;

pub struct TilePlugin;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileMaterial {
    #[default]
    Empty,
    Wall,
    Door,
}

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileIndex>().init_resource::<TileMap>();
    }
}
