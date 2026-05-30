use bevy_ecs::prelude::*;

use crate::tile::{position::TilePosition, storage::TileStorageMut};

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[component(immutable)]
#[require(TilePosition)]
pub enum TileMaterial {
    #[default]
    Empty,
    Wall,
    Door,
}

pub fn on_insert_material(
    trigger: On<Insert, TileMaterial>,
    tiles: Query<(&TileMaterial, &TilePosition)>,
    mut storage: TileStorageMut,
) -> Result {
    let (material, position) = tiles.get(trigger.entity)?;
    storage.set_material(*position, *material);
    Ok(())
}
