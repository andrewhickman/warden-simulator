use bevy_ecs::prelude::*;

use crate::tile::{
    index::TileIndex, material::TileMaterial, position::TilePosition, storage::TileStorageMut,
};

pub trait TileCommandsExt {
    fn set_material(&mut self, position: TilePosition, material: TileMaterial);

    fn spawn_tile(
        &mut self,
        position: TilePosition,
        material: TileMaterial,
        bundle: impl Bundle,
    ) -> Entity;

    fn despawn_tile(&mut self, position: TilePosition, material: TileMaterial);
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

    fn despawn_tile(&mut self, position: TilePosition, material: TileMaterial) {
        self.queue(move |world: &mut World| despawn_tile(world, position));
        self.set_material(position, material);
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

    fn despawn_tile(&mut self, position: TilePosition, material: TileMaterial) {
        despawn_tile(self, position);
        self.set_material(position, material);
    }
}

fn despawn_tile(world: &mut World, tile: TilePosition) {
    if let Some(index) = world.resource::<TileIndex>().get(tile) {
        if let Some(entity) = index.tile() {
            world.entity_mut(entity).despawn();
        }
    }
}

fn set_material(
    In((position, material)): In<(TilePosition, TileMaterial)>,
    mut storage: TileStorageMut,
) {
    storage.set_material(position, material);
}
