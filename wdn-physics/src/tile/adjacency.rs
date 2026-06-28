use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bitflags::bitflags;

use crate::tile::{position::TilePosition, storage::TileStorage};

#[derive(Debug, Default, Clone, Copy, Component, PartialEq, Eq)]
pub struct TileAdjacency {
    pub(crate) walls: Adjacency,
    pub(crate) doors: Adjacency,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Adjacency : u8 {
        const NONE = 0b0000_0000;
        const NORTH_WEST = 0b0000_0001;
        const NORTH = 0b0000_0010;
        const NORTH_EAST = 0b0000_0100;
        const EAST = 0b0000_1000;
        const SOUTH_EAST = 0b0001_0000;
        const SOUTH = 0b0010_0000;
        const SOUTH_WEST = 0b0100_0000;
        const WEST = 0b1000_0000;
    }
}

impl TileAdjacency {
    pub const NONE: Self = Self {
        walls: Adjacency::NONE,
        doors: Adjacency::NONE,
    };

    pub fn new(walls: Adjacency, doors: Adjacency) -> Self {
        Self { walls, doors }
    }

    pub fn solid(&self) -> Adjacency {
        self.walls | self.doors
    }

    pub fn empty(&self) -> Adjacency {
        self.solid().complement()
    }

    pub fn walls(&self) -> Adjacency {
        self.walls
    }

    pub fn doors(&self) -> Adjacency {
        self.doors
    }
}

impl Adjacency {
    pub(crate) const OFFSETS: [(Adjacency, IVec2); 8] = [
        (Adjacency::NORTH_WEST, IVec2::new(1, -1)),
        (Adjacency::NORTH, IVec2::new(0, -1)),
        (Adjacency::NORTH_EAST, IVec2::new(-1, -1)),
        (Adjacency::EAST, IVec2::new(-1, 0)),
        (Adjacency::SOUTH_EAST, IVec2::new(-1, 1)),
        (Adjacency::SOUTH, IVec2::new(0, 1)),
        (Adjacency::SOUTH_WEST, IVec2::new(1, 1)),
        (Adjacency::WEST, IVec2::new(1, 0)),
    ];

    pub fn opposite(&self) -> Self {
        Adjacency::from_bits_retain(self.bits().rotate_right(4))
    }

    pub fn values() -> impl Iterator<Item = Self> {
        (0..=255u8).map(Self::from_bits_retain)
    }
}

pub fn on_add_adjacency(
    trigger: On<Add, TileAdjacency>,
    mut tiles: Query<(&mut TileAdjacency, &TilePosition)>,
    storage: TileStorage,
) -> Result {
    let (mut adjacency, position) = tiles.get_mut(trigger.entity)?;
    if let Some(tile) = storage.get(*position) {
        if adjacency.walls != tile.wall_adjacency() {
            adjacency.walls = tile.wall_adjacency();
        }
        if adjacency.doors != tile.door_adjacency() {
            adjacency.doors = tile.door_adjacency();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy_app::prelude::*;
    use bevy_ecs::{prelude::*, system::RunSystemOnce};

    use crate::{
        layer::Layer,
        tile::{
            Tile, TilePlugin,
            adjacency::{Adjacency, TileAdjacency},
            material::TileMaterial,
            position::TilePosition,
            storage::{TileStorage, TileStorageMut},
        },
    };

    #[test]
    fn spawn_tile() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(Layer::default()).id();
        let position = TilePosition::new(layer, 5, 5);

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(position.west(), TileMaterial::Wall);
                storage.set_material(position.east(), TileMaterial::Wall);
            })
            .unwrap();

        let tile = app
            .world_mut()
            .spawn((Tile, position, TileMaterial::Door))
            .id();

        assert_eq!(
            app.world()
                .entity(tile)
                .get::<TileAdjacency>()
                .unwrap()
                .walls(),
            Adjacency::WEST | Adjacency::EAST
        );

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                assert_eq!(storage.get_material(position), TileMaterial::Door);
                assert_eq!(storage.get_material(position), TileMaterial::Door);
            })
            .unwrap();
    }

    #[test]
    fn update_tile() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(Layer::default()).id();
        let position = TilePosition::new(layer, 5, 5);

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(position, TileMaterial::Empty);
            })
            .unwrap();

        let tile = app
            .world_mut()
            .spawn((Tile, position, TileMaterial::Door))
            .id();

        assert_eq!(
            app.world()
                .entity(tile)
                .get::<TileAdjacency>()
                .unwrap()
                .walls(),
            Adjacency::NONE
        );

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                assert_eq!(storage.get_material(position), TileMaterial::Door);

                storage.set_material(position.west(), TileMaterial::Wall);
                storage.set_material(position.east(), TileMaterial::Wall);
            })
            .unwrap();

        assert_eq!(
            app.world()
                .entity(tile)
                .get::<TileAdjacency>()
                .unwrap()
                .walls(),
            Adjacency::EAST | Adjacency::WEST
        );
    }

    #[test]
    fn spawn_tile_storage_not_found() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(Layer::default()).id();
        let position = TilePosition::new(layer, 5, 5);
        let tile = app
            .world_mut()
            .spawn((Tile, position, TileMaterial::Door))
            .id();

        assert_eq!(
            app.world()
                .entity(tile)
                .get::<TileAdjacency>()
                .unwrap()
                .walls(),
            Adjacency::NONE
        );

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                assert_eq!(storage.get_material(position), TileMaterial::Door);

                storage.set_material(position.west(), TileMaterial::Wall);
                storage.set_material(position.east(), TileMaterial::Wall);
            })
            .unwrap();

        assert_eq!(
            app.world()
                .entity(tile)
                .get::<TileAdjacency>()
                .unwrap()
                .walls(),
            Adjacency::EAST | Adjacency::WEST
        );
    }

    #[test]
    fn spawn_tile_storage_nested_buffer() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(Layer::default()).id();
        let position = TilePosition::new(layer, 5, 5);

        let tile = app
            .world_mut()
            .run_system_once(move |mut commands: Commands, mut storage: TileStorageMut| {
                storage.set_material(position.west(), TileMaterial::Wall);
                storage.set_material(position.east(), TileMaterial::Wall);

                commands.spawn((Tile, position, TileMaterial::Door)).id()
            })
            .unwrap();

        assert_eq!(
            app.world()
                .entity(tile)
                .get::<TileAdjacency>()
                .unwrap()
                .walls(),
            Adjacency::EAST | Adjacency::WEST
        );
    }
}
