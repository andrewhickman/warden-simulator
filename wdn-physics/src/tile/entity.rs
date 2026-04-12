use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};

use crate::tile::{
    TilePosition,
    storage::{TileMaterial, TileStorageMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
#[require(TilePosition)]
pub struct TileEntity {
    material: TileMaterial,
}

impl TileEntity {
    pub fn new(material: TileMaterial) -> Self {
        Self { material }
    }

    pub fn material(&self) -> TileMaterial {
        self.material
    }
}

fn tile_inserted(
    trigger: On<Insert, TileEntity>,
    tiles: Query<(&TileEntity, &TilePosition)>,
    mut storage: TileStorageMut,
) -> Result {
    let (tile, position) = tiles.get(trigger.entity)?;
    debug_assert_ne!(*position, TilePosition::default());
    storage.set_material(*position, tile.material);
    Ok(())
}

fn tile_removed(
    trigger: On<Remove, TileEntity>,
    tiles: Query<&TilePosition>,
    mut storage: TileStorageMut,
) -> Result {
    let position = tiles.get(trigger.entity)?;
    debug_assert_ne!(*position, TilePosition::default());
    storage.set_material(*position, TileMaterial::Empty);
    Ok(())
}
