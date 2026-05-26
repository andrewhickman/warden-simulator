# import bevy_sprite::{
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
    let chunk_size = vec2<u32>(#{CHUNK_SIZE});
    let tile_uv = in.uv * vec2<f32>(chunk_size);
    var tile_coord = clamp(vec2<u32>(floor(tile_uv)), vec2<u32>(0), chunk_size - 1u);
    var local_uv = tile_uv - vec2<f32>(tile_coord);

    tile_coord.y = chunk_size.y - 1u - tile_coord.y;

    let data = textureLoad(tile_data, tile_coord, 0);
    let index = data.r;
    let depth = select(f32(data.g), 0.0, local_uv.y < 0.5);

    var color = textureSample(tileset, tileset_sampler, local_uv, index);
    if color.a < 0.7 {
        discard;
    }

    return FragmentOutput(color, depth);
}
