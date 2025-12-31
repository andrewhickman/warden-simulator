use bevy_app::prelude::*;
use bevy_asset::{AssetEventSystems, prelude::*};
use bevy_camera::prelude::*;
use bevy_color::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_image::prelude::*;
use bevy_log::prelude::*;
use bevy_math::prelude::*;
use bevy_mesh::prelude::*;
use bevy_sprite_render::{
    AlphaMode2d, PackedTileData, TileData, TilemapChunkMaterial, make_chunk_tile_data_image,
    prelude::*,
};
use bevy_transform::prelude::*;

use wdn_physics::tile::{
    CHUNK_SIZE, TileChunkPosition,
    storage::{TileChunk, TileLayer, TileMaterial},
};

use crate::assets::AssetHandles;

pub struct TilePlugin;

#[derive(Resource)]
pub struct TileChunkMesh(Handle<Mesh>);

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Mesh2d, MeshMaterial2d<TilemapChunkMaterial>, Transform)]
#[component(on_add = TileChunkRender::on_add)]
pub struct TileChunkRender;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileChunkMesh>();

        app.add_systems(PostUpdate, update_chunk_data.before(AssetEventSystems));

        app.register_required_components::<TileLayer, Visibility>();
        app.register_required_components::<TileChunk, TileChunkRender>();
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
    mut query: Query<(&TileChunk, &MeshMaterial2d<TilemapChunkMaterial>), Changed<TileChunk>>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    query.iter_mut().for_each(|(chunk, material)| {
        info!("updating tile chunk data for chunk {chunk:?}");

        let Some(material) = materials.get_mut(material.id()) else {
            error!("material asset not found for chunk {chunk:?}");
            return;
        };

        let Some(image) = images.get_mut(material.tile_data.id()) else {
            error!("image asset not found for chunk {chunk:?}");
            return;
        };

        let Some(data) = image.data.as_mut() else {
            error!("image data not found for chunk {chunk:?}");
            return;
        };

        data.clear();
        for tile in chunk.tiles() {
            let packed = pack_tile_chunk(tile.material());
            data.extend_from_slice(bytemuck::bytes_of(&packed));
        }
    });
}

impl TileChunkRender {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let mesh = world.resource::<TileChunkMesh>().0.clone();

        let chunk = world.get::<TileChunk>(context.entity).unwrap();
        let position = chunk.position();
        let packed_data = chunk
            .tiles()
            .map(|tile| pack_tile_chunk(tile.material()))
            .collect::<Vec<PackedTileData>>();

        let tileset = world.resource::<AssetHandles>().tileset.clone();
        let tile_data = world
            .resource_mut::<Assets<Image>>()
            .add(make_chunk_tile_data_image(
                &UVec2::splat(CHUNK_SIZE as u32),
                &packed_data,
            ));

        let material =
            world
                .resource_mut::<Assets<TilemapChunkMaterial>>()
                .add(TilemapChunkMaterial {
                    alpha_mode: AlphaMode2d::Opaque,
                    tileset,
                    tile_data,
                });

        let mut chunk = world.entity_mut(context.entity);
        chunk
            .get_mut::<MeshMaterial2d<TilemapChunkMaterial>>()
            .unwrap()
            .0 = material;
        chunk.get_mut::<Mesh2d>().unwrap().0 = mesh;
        *chunk.get_mut::<Transform>().unwrap() = tile_chunk_transform(position);
    }
}

fn pack_tile_chunk(material: TileMaterial) -> PackedTileData {
    PackedTileData::from(TileData {
        tileset_index: material as u16,
        color: Color::WHITE,
        visible: true,
    })
}

fn tile_chunk_transform(position: TileChunkPosition) -> Transform {
    Transform::from_translation(Vec3::new(
        position.x() as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0,
        position.y() as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0,
        0.0,
    ))
}
