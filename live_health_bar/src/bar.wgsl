@group(0) @binding(0)
var bar_texture: texture_2d_array<f32>;
@group(0) @binding(1)
var bar_sampler: sampler;

struct Param {
    index: u32,
    x: f32,
}

var<immediate> param: Param;

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, -1.0),
);

var<private> v_uvs: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
);

struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    var out: VertexOut;

    out.position = vec4<f32>(v_positions[idx], 0., 1.);
    out.uv = v_uvs[idx];

    return out;
}

// const srgb2p3 = mat3x3<f32>(
//     0.8225929, 0.1775330, 0.0,
//     0.0331995, 0.9667835, 0.0,
//     0.0170853, 0.0723957, 0.910301
// );

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var color = textureSample(bar_texture, bar_sampler, in.uv, param.index);
    color.a = select(0f, color.a, in.uv.x < param.x);
    return vec4(color.rgb * color.a, color.a);
    // if in.uv.y < 1. / 3. {
    //     return vec4(in.uv.x, 0f, 0f, 1f);
    // } else if in.uv.y < 1. / 3. * 2. {
    //     return vec4(0f, in.uv.x, 0f, 1f);
    // } else {
    //     return vec4(0f, 0f, in.uv.x, 1f);
    // }
}
