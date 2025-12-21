use bevy::{
    asset::AssetEventSystems,
    prelude::*,
    sprite_render::{
        AlphaMode2d, PackedTileData, TileData, TilemapChunkMaterial, make_chunk_tile_data_image,
    },
};

use wdn_physics::tile::{CHUNK_SIZE, storage::TileChunk};

use crate::assets::AssetHandles;

pub struct TilePlugin;

#[derive(Resource)]
pub struct TileChunkMesh(Mesh2d);

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileChunkMesh>();

        app.add_systems(PostUpdate, update_chunk_material.before(AssetEventSystems));

        app.register_required_components::<TileChunk, MeshMaterial2d<TilemapChunkMaterial>>();
        app.register_required_components::<TileChunk, Mesh2d>();
    }
}

impl FromWorld for TileChunkMesh {
    fn from_world(world: &mut World) -> Self {
        let handle = world
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::from_length((CHUNK_SIZE * 16) as f32));
        TileChunkMesh(Mesh2d(handle))
    }
}

pub fn update_chunk_material(
    assets: Res<AssetHandles>,
    chunk_mesh: Res<TileChunkMesh>,
    mut query: Query<
        (
            &TileChunk,
            &mut MeshMaterial2d<TilemapChunkMaterial>,
            &mut Mesh2d,
        ),
        Changed<TileChunk>,
    >,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut tile_buf: Local<Vec<PackedTileData>>,
) {
    query
        .iter_mut()
        .for_each(|(chunk, mut material, mut mesh)| {
            mesh.clone_from_if_neq(&chunk_mesh.0);

            tile_buf.clear();
            tile_buf.extend(chunk.tiles().map(|tile| {
                PackedTileData::from(TileData {
                    tileset_index: tile.material() as u16,
                    color: Color::WHITE,
                    visible: true,
                })
            }));

            if material.id() != AssetId::default()
                && let Some(material) = materials.get_mut(material.id())
            {
                let Some(image) = images.get_mut(material.tile_data.id()) else {
                    error!("image asset not found for chunk {chunk:?}");
                    return;
                };

                *image = make_chunk_tile_data_image(&UVec2::splat(CHUNK_SIZE as u32), &tile_buf);
            } else {
                material.0 = materials.add(TilemapChunkMaterial {
                    alpha_mode: AlphaMode2d::Opaque,
                    tileset: assets.tileset.clone(),
                    tile_data: images.add(make_chunk_tile_data_image(
                        &UVec2::splat(CHUNK_SIZE as u32),
                        &tile_buf,
                    )),
                });
            }
        });
}
