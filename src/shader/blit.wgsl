struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0,  1.0), 
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),

        vec2<f32>(-1.0,  1.0), 
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0), 
    );

    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), 
        vec2<f32>(0.0, 1.0), 
        vec2<f32>(1.0, 1.0), 

        vec2<f32>(0.0, 0.0), 
        vec2<f32>(1.0, 1.0), 
        vec2<f32>(1.0, 0.0),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.tex_coords = uvs[vertex_index];
    return out;
}

@group(0) @binding(0) var t_screen: texture_2d<f32>;
@group(0) @binding(1) var s_screen: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_screen, s_screen, in.tex_coords);
}