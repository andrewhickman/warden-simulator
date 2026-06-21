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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileMoveSpeed {
    Slow = 3,
    Medium = 5,
    Fast = 7,
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

impl TileMaterial {
    pub fn is_empty(&self) -> bool {
        matches!(self, TileMaterial::Empty)
    }
}

impl TileMoveSpeed {
    pub fn factor(&self) -> f32 {
        match self {
            TileMoveSpeed::Slow => 0.6,
            TileMoveSpeed::Medium => 1.0,
            TileMoveSpeed::Fast => 1.4,
        }
    }
}
