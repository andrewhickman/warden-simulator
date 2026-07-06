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
    let chunk_size = vec2<u32>(#{CHUNK_WIDTH}, #{CHUNK_HEIGHT});
    let tile_uv = in.uv * vec2<f32>(chunk_size);
    var tile_coord = clamp(vec2<u32>(floor(tile_uv)), vec2<u32>(0), chunk_size - 1u);
    var local_uv = tile_uv - vec2<f32>(tile_coord);

    tile_coord.y = chunk_size.y - 1u - tile_coord.y;

    let data = textureLoad(tile_data, tile_coord, 0);
    let raw_index = data.r;
    let flip_x = (raw_index & 0x8000u) != 0u;
    let index = raw_index & 0x7FFFu;
    let depth = select(f32(data.g), 0.0, local_uv.y < 0.5);

    let sample_uv = vec2<f32>(select(local_uv.x, 1.0 - local_uv.x, flip_x), local_uv.y);
    var color = textureSample(tileset, tileset_sampler, sample_uv, index);
    if color.a < 0.7 {
        discard;
    }

    return FragmentOutput(color, depth);
}
