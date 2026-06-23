@group(0) @binding(0)
var<storage, read> framebuffer: array<u32, 15360>;

@group(0) @binding(1)
var<uniform> palette: array<vec4f, 64>;

@group(0) @binding(2)
var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 1)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>
) 
{
    // There are 256 pixels,
    // 64 4x1 pixel strips per scanline,
    // and each work group handles one pixel strip.
    let strip_index = gid.x + gid.y * 64u;
    let strip = framebuffer[strip_index];
    
    // Since the palette is only 64 colors large
    // we want to avoid overflows by masking out
    // the most significant two bits.
    let byte_index = (strip >> (lid.x * 8u)) & 0x3F;

    textureStore(output_tex, vec2u(gid.x + lid.x, gid.y), palette[byte_index]);
}