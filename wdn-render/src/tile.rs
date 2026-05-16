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
        storage::{Tile, TileChunk, TileOccupancy},
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
    top: Handle<TilemapChunkMaterial>,
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

        app.add_systems(PostUpdate, update_chunk.before(AssetEventSystems));

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

pub fn update_chunk(
    mut param: TileChunkSpriteParam,
    mut chunks: Query<
        (Entity, &TileChunk, &mut Transform, &mut TileChunkSprites),
        Changed<TileChunk>,
    >,
) {
    chunks
        .iter_mut()
        .for_each(|(id, chunk, mut transform, mut sprites)| {
            if sprites.is_added() {
                let position = chunk.position();
                *transform = chunk_transform(position);

                sprites.base = param.spawn_chunk_material(id, BASE_LAYER);
                sprites.top = param.spawn_chunk_material(id, TOP_LAYER);
            }

            param.update_chunk_material(sprites.base.id(), chunk, pack_tile_base);
            param.update_chunk_material(sprites.top.id(), chunk, pack_tile_top);
        });
}

impl TileChunkSpriteParam<'_, '_> {
    fn spawn_chunk_material(&mut self, id: Entity, depth: f32) -> Handle<TilemapChunkMaterial> {
        let tile_data = self.images.add(make_chunk_tile_data_image(
            &UVec2::splat(CHUNK_SIZE as u32),
            &[PackedTileData::from(None); CHUNK_SIZE * CHUNK_SIZE],
        ));
        let material = self.materials.add(TilemapChunkMaterial {
            alpha_mode: AlphaMode2d::Blend,
            tileset: self.assets.tileset(),
            tile_data: tile_data,
        });

        self.commands.spawn((
            ChildOf(id),
            Mesh2d(self.mesh.0.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(0.0, 0.0, depth),
        ));

        material
    }

    fn update_chunk_material(
        &mut self,
        material: AssetId<TilemapChunkMaterial>,
        chunk: &TileChunk,
        pack: impl Fn(TileChunkOffset, Tile) -> PackedTileData,
    ) {
        let Some(material) = self.materials.get_mut(material) else {
            error!("material asset not found for chunk {chunk:?}");
            return;
        };

        let Some(image) = self.images.get_mut(material.tile_data.id()) else {
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

fn pack_tile_base(offset: TileChunkOffset, _tile: Tile) -> PackedTileData {
    let tileset_index = DIRT_OFFSET + dirt_sprite_offset(offset);

    PackedTileData::from(TileData {
        tileset_index,
        color: Color::WHITE,
        visible: true,
    })
}

fn pack_tile_top(_: TileChunkOffset, tile: Tile) -> PackedTileData {
    let tileset_index = WALL_TOP_OFFSET + wall_sprite_offset(tile.is_solid(), tile.occupancy());

    PackedTileData::from(TileData {
        tileset_index,
        color: Color::WHITE,
        visible: true,
    })
}

fn chunk_transform(position: TileChunkPosition) -> Transform {
    Transform::from_xyz(
        chunk_coord_transform(position.x()),
        chunk_coord_transform(position.y()),
        0.0,
    )
}

fn chunk_coord_transform(d: i16) -> f32 {
    d as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0
}

fn dirt_sprite_offset(position: TileChunkOffset) -> u16 {
    (SPRITE_CHUNK_SIZE - 1 - position.y().rem_euclid(SPRITE_CHUNK_SIZE)) * SPRITE_CHUNK_SIZE
        + position.x().rem_euclid(SPRITE_CHUNK_SIZE)
}

fn wall_sprite_offset(solid: bool, occupancy: TileOccupancy) -> u16 {
    const EMPTY_LOOKUP: [u8; 256] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2,
        2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4,
        4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2,
        2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3,
        4, 4, 4, 4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1,
        1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3,
        3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
        1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
    ];

    const SOLID_LOOKUP: [u8; 256] = [
        5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9, 10, 10,
        10, 10, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 11, 11, 11, 11, 12, 12, 12, 12, 13,
        13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 16, 16, 16, 16,
        17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20, 15, 15, 15, 15, 16, 16, 16,
        16, 15, 15, 15, 15, 16, 16, 16, 16, 21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 24, 24,
        24, 24, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9,
        10, 10, 10, 10, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 11, 11, 11, 11, 12, 12, 12,
        12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 16, 16,
        16, 16, 17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20, 15, 15, 15, 15, 16,
        16, 16, 16, 15, 15, 15, 15, 16, 16, 16, 16, 21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23,
        24, 24, 24, 24,
    ];

    if solid {
        SOLID_LOOKUP[occupancy.bits() as usize] as u16
    } else {
        EMPTY_LOOKUP[occupancy.bits() as usize] as u16
    }
}

#[test]
fn test_tile_sprite_index() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<(bool, TileOccupancy), u16> = HashMap::new();

    for solid in [false, true] {
        for i in 0..=255u8 {
            let occupancy = TileOccupancy::from_bits_retain(i);
            let mut normal = occupancy.intersection(
                TileOccupancy::SOUTH
                    | TileOccupancy::SOUTH_WEST
                    | TileOccupancy::SOUTH_EAST
                    | TileOccupancy::WEST
                    | TileOccupancy::EAST,
            );

            if solid {
                if !normal.contains(TileOccupancy::SOUTH) {
                    normal.remove(TileOccupancy::SOUTH_WEST);
                    normal.remove(TileOccupancy::SOUTH_EAST);
                }
            } else {
                if !normal.contains(TileOccupancy::SOUTH) {
                    normal = TileOccupancy::NONE;
                }

                normal.remove(TileOccupancy::EAST);
                normal.remove(TileOccupancy::WEST);
            }

            let index = patterns.len() as u16;

            match patterns.entry((solid, normal)) {
                hash_map::Entry::Occupied(entry) => {
                    assert_eq!(
                        wall_sprite_offset(solid, occupancy),
                        *entry.get() as u16,
                        "unexpected sprite index for solid={solid}, occupancy={occupancy:?}, normal={normal:?}"
                    );
                }
                hash_map::Entry::Vacant(entry) => {
                    println!(
                        "new pattern: solid={solid}, occupancy={occupancy:?}, normal={normal:?} => index={index}"
                    );
                    assert_eq!(
                        wall_sprite_offset(solid, occupancy),
                        index,
                        "unexpected sprite index for solid={solid}, occupancy={occupancy:?}, normal={normal:?}"
                    );
                    entry.insert(index);
                }
            }
        }
    }

    assert_eq!(patterns.len(), 25);
}
