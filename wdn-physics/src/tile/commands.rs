use bevy_ecs::prelude::*;

use crate::tile::{material::TileMaterial, position::TilePosition, storage::TileStorageMut};

pub trait TileCommandsExt {
    fn set_material(&mut self, position: TilePosition, material: TileMaterial);

    fn spawn_tile(
        &mut self,
        position: TilePosition,
        material: TileMaterial,
        bundle: impl Bundle,
    ) -> Entity;
}

impl TileCommandsExt for Commands<'_, '_> {
    fn set_material(&mut self, position: TilePosition, material: TileMaterial) {
        self.run_system_cached_with(set_material, (position, material));
    }

    fn spawn_tile(
        &mut self,
        position: TilePosition,
        material: TileMaterial,
        bundle: impl Bundle,
    ) -> Entity {
        self.set_material(position, material);
        self.spawn((position, material, bundle)).id()
    }
}

impl TileCommandsExt for World {
    fn set_material(&mut self, position: TilePosition, material: TileMaterial) {
        self.run_system_cached_with(set_material, (position, material))
            .unwrap();
    }

    fn spawn_tile(
        &mut self,
        position: TilePosition,
        material: TileMaterial,
        bundle: impl Bundle,
    ) -> Entity {
        self.set_material(position, material);
        self.spawn((position, material, bundle)).id()
    }
}

fn set_material(
    In((position, material)): In<(TilePosition, TileMaterial)>,
    mut storage: TileStorageMut,
) {
    storage.set_material(position, material);
}
