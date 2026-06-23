@group(0) @binding(0)
var<storage, read> framebuffer: array<u32, 15360>;

@group(0) @binding(1)
var<uniform> palette: array<vec4f, 64>;

@group(0) @binding(2)
var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 1)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wgid: vec3<u32> // <-- Added this to track the strip index
) 
{
    // wgid.x ranges from 0 to 63, representing the 64 strips per scanline.
    let strip_index = wgid.x + gid.y * 64u;
    let strip = framebuffer[strip_index];
    
    // Each of the 4 threads extracts its specific 8-bit color index from the 32-bit strip.
    let byte_index = (strip >> (lid.x * 8u)) & 0x3Fu;

    // gid.x is already (wgid.x * 4 + lid.x), mapping perfectly to columns 0-255.
    textureStore(output_tex, vec2u(gid.x, gid.y), palette[byte_index]);
}