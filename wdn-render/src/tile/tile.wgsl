#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::view,
    mesh2d_vertex_output::VertexOutput,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var tileset: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var tileset_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var tile_data: texture_2d<u32>;

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let chunk_size = textureDimensions(tile_data, 0);
    let tile_uv = in.uv * vec2<f32>(chunk_size);
    var tile_coord = clamp(vec2<u32>(floor(tile_uv)), vec2<u32>(0), chunk_size - 1);
    var local_uv = tile_uv - vec2<f32>(tile_coord);

    tile_coord.y = chunk_size.y - 1 - tile_coord.y;

    let data = textureLoad(tile_data, tile_coord, 0);
    let base_index = data.r;
    let top_index = data.g;

    let base_color = textureSample(tileset, tileset_sampler, local_uv, base_index);
    let top_color = textureSample(tileset, tileset_sampler, local_uv, top_index);

    let color = mix(base_color, top_color, top_color.a);
    let depth = select(0.0, 1.0, top_color.a > 0.0);
    return FragmentOutput(color, depth);
}
