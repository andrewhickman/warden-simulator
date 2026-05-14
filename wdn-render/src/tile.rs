use bevy_app::prelude::*;
use bevy_asset::{AssetEventSystems, prelude::*};
use bevy_camera::prelude::*;
use bevy_color::prelude::*;
use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_image::prelude::*;
use bevy_log::prelude::*;
use bevy_math::prelude::*;
use bevy_mesh::prelude::*;
use bevy_sprite_render::{
    AlphaMode2d, PackedTileData, TileData, TilemapChunkMaterial, make_chunk_tile_data_image,
    prelude::*,
};
use bevy_transform::prelude::*;

use wdn_physics::{
    layer::Layer,
    tile::{
        CHUNK_SIZE, TileChunkOffset, TileChunkPosition,
        storage::{Tile, TileChunk, TileMaterial, TileOccupancy},
    },
};

use crate::{
    assets::AssetHandles,
    layers::{BASE_LAYER, TOP_LAYER},
};

pub const SPRITE_CHUNK_SIZE: u16 = 16;

pub const DIRT_OFFSET: u16 = 0;
pub const WALL_TOP_OFFSET: u16 = DIRT_OFFSET + 256;
pub const WALL_BASE_OFFSET: u16 = WALL_TOP_OFFSET + 13;

pub struct TilePlugin;

#[derive(Resource)]
pub struct TileChunkMesh(Handle<Mesh>);

#[derive(Clone, Component, Default, Debug)]
#[require(Transform, Visibility)]
pub struct TileChunkSprites {
    base: Handle<TilemapChunkMaterial>,
    base_image: Handle<Image>,
    top: Handle<TilemapChunkMaterial>,
    top_image: Handle<Image>,
}

#[derive(SystemParam)]
pub struct TileChunkSpriteParam<'w, 's> {
    commands: Commands<'w, 's>,
    assets: Res<'w, AssetHandles>,
    mesh: Res<'w, TileChunkMesh>,
    materials: ResMut<'w, Assets<TilemapChunkMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileChunkMesh>();

        app.add_systems(PostUpdate, update_chunk_data.before(AssetEventSystems));

        app.register_required_components::<Layer, Visibility>();
        app.register_required_components::<TileChunk, TileChunkSprites>();
    }
}

impl FromWorld for TileChunkMesh {
    fn from_world(world: &mut World) -> Self {
        let handle = world
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::from_length(CHUNK_SIZE as f32));
        TileChunkMesh(handle)
    }
}

pub fn update_chunk_data(
    mut param: TileChunkSpriteParam,
    mut chunks: Query<(Entity, &TileChunk, &mut TileChunkSprites), Changed<TileChunk>>,
) {
    chunks.iter_mut().for_each(|(id, chunk, mut sprites)| {
        if sprites.is_added() {
            let position = chunk.position();
            let (base, base_image) = param.spawn_chunk_image(id, chunk_base_transform(position));
            let (top, top_image) = param.spawn_chunk_image(id, chunk_top_transform(position));

            sprites.base = base;
            sprites.base_image = base_image;
            sprites.top = top;
            sprites.top_image = top_image;
        }

        param.update_chunk_image(
            sprites.base.id(),
            sprites.base_image.id(),
            chunk,
            pack_tile_base,
        );
        param.update_chunk_image(
            sprites.top.id(),
            sprites.top_image.id(),
            chunk,
            pack_tile_top,
        );
    });
}

impl TileChunkSpriteParam<'_, '_> {
    fn spawn_chunk_image(
        &mut self,
        id: Entity,
        transform: Transform,
    ) -> (Handle<TilemapChunkMaterial>, Handle<Image>) {
        let tile_data = self.images.add(make_chunk_tile_data_image(
            &UVec2::splat(CHUNK_SIZE as u32),
            &[PackedTileData::from(None); CHUNK_SIZE * CHUNK_SIZE],
        ));
        let material = self.materials.add(TilemapChunkMaterial {
            alpha_mode: AlphaMode2d::Blend,
            tileset: self.assets.tileset(),
            tile_data: tile_data.clone(),
        });
        self.commands.spawn((
            ChildOf(id),
            Mesh2d(self.mesh.0.clone()),
            MeshMaterial2d(material.clone()),
            transform,
        ));
        (material, tile_data)
    }

    fn update_chunk_image(
        &mut self,
        material: AssetId<TilemapChunkMaterial>,
        image: AssetId<Image>,
        chunk: &TileChunk,
        pack: impl Fn(TileChunkOffset, Tile) -> PackedTileData,
    ) {
        // Mark material as changed to trigger redraw
        if self.materials.get_mut(material).is_none() {
            error!("material asset not found for chunk {chunk:?}");
            return;
        }

        let Some(image) = self.images.get_mut(image) else {
            error!("image asset not found for chunk {chunk:?}");
            return;
        };

        let Some(data) = image.data.as_mut() else {
            error!("image data not found for chunk {chunk:?}");
            return;
        };

        data.clear();
        for (offset, tile) in chunk.tiles() {
            let packed = pack(offset, tile);
            data.extend_from_slice(bytemuck::bytes_of(&packed));
        }
    }
}

fn pack_tile_base(offset: TileChunkOffset, tile: Tile) -> PackedTileData {
    let tileset_index = match tile.material() {
        TileMaterial::Empty => DIRT_OFFSET + dirt_sprite_offset(offset),
        TileMaterial::Wall => WALL_BASE_OFFSET + wall_base_sprite_offset(tile.occupancy()),
    };

    PackedTileData::from(TileData {
        tileset_index,
        color: Color::WHITE,
        visible: true,
    })
}

fn pack_tile_top(_: TileChunkOffset, tile: Tile) -> PackedTileData {
    let tileset_index = match tile.material() {
        TileMaterial::Empty => return PackedTileData::from(None),
        TileMaterial::Wall => WALL_TOP_OFFSET + wall_top_sprite_offset(tile.occupancy()),
    };

    PackedTileData::from(TileData {
        tileset_index,
        color: Color::WHITE,
        visible: true,
    })
}

fn chunk_top_transform(position: TileChunkPosition) -> Transform {
    let t = Transform::from_xyz(
        chunk_coord_transform(position.x()),
        chunk_coord_transform(position.y()) + 1.0,
        TOP_LAYER,
    );
    println!("base transform: {t:#?}");
    t
}

fn chunk_base_transform(position: TileChunkPosition) -> Transform {
    let t = Transform::from_xyz(
        chunk_coord_transform(position.x()),
        chunk_coord_transform(position.y()),
        BASE_LAYER,
    );
    println!("base transform: {t:#?}");
    t
}

fn chunk_coord_transform(d: i16) -> f32 {
    d as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0
}

fn dirt_sprite_offset(position: TileChunkOffset) -> u16 {
    (SPRITE_CHUNK_SIZE - 1 - position.y().rem_euclid(SPRITE_CHUNK_SIZE)) * SPRITE_CHUNK_SIZE
        + position.x().rem_euclid(SPRITE_CHUNK_SIZE)
}

fn wall_base_sprite_offset(occupancy: TileOccupancy) -> u16 {
    const LOOKUP: [u8; 256] = [
        0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4,
        4, 4, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 2,
        4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 7, 7,
        7, 7, 9, 9, 9, 9, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 10, 10, 10, 10, 11, 11,
        11, 11, 10, 10, 10, 10, 12, 12, 12, 12, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 2,
        2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1,
        1, 2, 2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6,
        6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 7, 7, 7, 7, 9, 9, 9, 9, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5,
        5, 6, 6, 6, 6, 10, 10, 10, 10, 11, 11, 11, 11, 10, 10, 10, 10, 12, 12, 12, 12,
    ];

    LOOKUP[occupancy.bits() as usize] as u16
}

fn wall_top_sprite_offset(occupancy: TileOccupancy) -> u16 {
    const LOOKUP: [u8; 256] = [
        0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3,
        2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1,
        2, 3, 2, 1, 4, 5, 4, 5, 6, 7, 6, 5, 4, 5, 4, 5, 6, 7, 6, 5, 4, 5, 4, 5, 6, 7, 6, 5, 4, 5,
        4, 5, 6, 7, 6, 5, 4, 5, 4, 5, 6, 7, 6, 5, 4, 5, 4, 5, 6, 7, 6, 5, 4, 5, 4, 5, 6, 7, 6, 5,
        4, 5, 4, 5, 6, 7, 6, 5, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3,
        2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1,
        2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 4, 1, 4, 1, 6, 3, 6, 1, 4, 1, 4, 1, 6, 3, 6, 1, 4, 1,
        4, 1, 6, 3, 6, 1, 4, 1, 4, 1, 6, 3, 6, 1, 4, 1, 4, 1, 6, 3, 6, 1, 4, 1, 4, 1, 6, 3, 6, 1,
        4, 1, 4, 1, 6, 3, 6, 1, 4, 1, 4, 1, 6, 3, 6, 1,
    ];

    LOOKUP[occupancy.bits() as usize] as u16
}

#[test]
fn test_tile_sprite_index_bottom() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<TileOccupancy, u16> = HashMap::new();

    let base_mask = TileOccupancy::SOUTH
        | TileOccupancy::SOUTH_WEST
        | TileOccupancy::SOUTH_EAST
        | TileOccupancy::WEST
        | TileOccupancy::EAST;

    for i in 0..=255u8 {
        let occupancy = TileOccupancy::from_bits_retain(i) & base_mask;
        let mut normal = occupancy;

        if !normal.contains(TileOccupancy::NORTH | TileOccupancy::WEST) {
            normal.remove(TileOccupancy::NORTH_WEST);
        }

        if !normal.contains(TileOccupancy::NORTH | TileOccupancy::EAST) {
            normal.remove(TileOccupancy::NORTH_EAST);
        }

        if !normal.contains(TileOccupancy::SOUTH | TileOccupancy::WEST) {
            normal.remove(TileOccupancy::SOUTH_WEST);
        }

        if !normal.contains(TileOccupancy::SOUTH | TileOccupancy::EAST) {
            normal.remove(TileOccupancy::SOUTH_EAST);
        }

        let index = patterns.len() as u16;

        match patterns.entry(normal) {
            hash_map::Entry::Occupied(entry) => {
                assert_eq!(wall_base_sprite_offset(occupancy), *entry.get() as u16);
            }
            hash_map::Entry::Vacant(entry) => {
                assert_eq!(wall_base_sprite_offset(occupancy), index);
                entry.insert(index);
            }
        }
    }

    assert_eq!(patterns.len(), 13);
}

#[test]
fn test_tile_sprite_index_top() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<TileOccupancy, u16> = HashMap::new();

    let top_mask = TileOccupancy::NORTH
        | TileOccupancy::WEST
        | TileOccupancy::EAST
        | TileOccupancy::NORTH_WEST
        | TileOccupancy::NORTH_EAST;

    for i in 0..=255u8 {
        let occupancy = TileOccupancy::from_bits_retain(i) & top_mask;
        let mut normal = occupancy;

        if normal.contains(TileOccupancy::NORTH) {
            if normal.contains(TileOccupancy::NORTH_EAST) {
                normal.remove(TileOccupancy::EAST | TileOccupancy::NORTH_EAST);
            }

            if normal.contains(TileOccupancy::NORTH_WEST) {
                normal.remove(TileOccupancy::WEST | TileOccupancy::NORTH_WEST);
            }
        } else {
            normal.remove(TileOccupancy::NORTH_EAST);
            normal.remove(TileOccupancy::NORTH_WEST);
        }

        let index = patterns.len() as u16;

        match patterns.entry(normal) {
            hash_map::Entry::Occupied(entry) => {
                assert_eq!(wall_top_sprite_offset(occupancy), *entry.get() as u16);
            }
            hash_map::Entry::Vacant(entry) => {
                assert_eq!(wall_top_sprite_offset(occupancy), index);
                entry.insert(index);
            }
        }
    }

    assert_eq!(patterns.len(), 8);
}
