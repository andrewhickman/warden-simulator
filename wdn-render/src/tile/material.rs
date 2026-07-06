use std::mem;

use bevy_app::{App, Plugin};
use bevy_asset::{Asset, AssetPath, Handle, RenderAssetUsages, embedded_asset, embedded_path};
use bevy_image::{Image, ImageSampler};
use bevy_mesh::MeshVertexBufferLayoutRef;
use bevy_reflect::TypePath;
use bevy_render::render_resource::*;
use bevy_shader::{ShaderDefVal, ShaderRef};
use bevy_sprite_render::{AlphaMode2d, Material2d, Material2dKey, Material2dPlugin};
use bytemuck::{Pod, Zeroable};
use wdn_physics::tile::{CHUNK_SIZE, CHUNK_SIZE_SQUARED};

pub struct TileChunkMaterialPlugin;

impl Plugin for TileChunkMaterialPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "tile.wgsl");

        app.add_plugins(Material2dPlugin::<TileChunkMaterial>::default());
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct TileChunkMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub tileset: Handle<Image>,

    #[texture(2, sample_type = "u_int")]
    pub tile_data: Handle<Image>,
}

impl Material2d for TileChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("tile.wgsl")).with_source("embedded"),
        )
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let fragment_state = descriptor
            .fragment
            .as_mut()
            .expect("no fragment shader for Mesh2d pipeline");

        fragment_state.shader_defs.push(ShaderDefVal::UInt(
            "CHUNK_WIDTH".into(),
            CHUNK_SIZE as u32 * 2,
        ));
        fragment_state
            .shader_defs
            .push(ShaderDefVal::UInt("CHUNK_HEIGHT".into(), CHUNK_SIZE as u32));

        descriptor
            .depth_stencil
            .as_mut()
            .expect("no depth stencil for Mesh2d pipeline")
            .depth_write_enabled = Some(true);
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PackedTileData {
    index: u16,
    depth: u16,
}

pub fn make_tile_chunk_image() -> Image {
    Image {
        data: Some(Vec::with_capacity(
            CHUNK_SIZE_SQUARED * 2 * mem::size_of::<PackedTileData>(),
        )),
        data_order: TextureDataOrder::default(),
        texture_descriptor: TextureDescriptor {
            size: Extent3d {
                height: CHUNK_SIZE as u32,
                width: CHUNK_SIZE as u32 * 2,
                depth_or_array_layers: 1,
            },
            dimension: TextureDimension::D2,
            format: TextureFormat::Rg16Uint,
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        sampler: ImageSampler::nearest(),
        texture_view_descriptor: None,
        asset_usage: RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        copy_on_resize: false,
    }
}

impl PackedTileData {
    pub const FLIP_X: u16 = 1 << 15;

    pub fn new(index: u16, depth: u16, flip_x: bool) -> Self {
        let index = if flip_x {
            index | PackedTileData::FLIP_X
        } else {
            index
        };

        Self { index, depth }
    }

    pub fn index(&self) -> u16 {
        self.index & !PackedTileData::FLIP_X
    }

    pub fn depth(&self) -> u16 {
        self.depth
    }

    pub fn flip_x(&self) -> bool {
        (self.index & PackedTileData::FLIP_X) != 0
    }
}
