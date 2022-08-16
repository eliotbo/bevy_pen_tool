#import bevy_pbr::mesh_view_bindings

// This uniform is not needed here, as color attributes are being sent.
// But in the future, the uniform will be used to highlight selected 
// fill meshes
struct FillMat {
    color: vec4<f32>, 
};

@group(1) @binding(0)
var<uniform> uni: FillMat;

// // Converts a color from sRGB gamma to linear light gamma
fn toLinear(sRGB: vec4<f32>) -> vec4<f32>
{
    let cutoff = vec4<f32>(sRGB < vec4<f32>(0.04045));
    let higher = pow((sRGB + vec4<f32>(0.055))/vec4<f32>(1.055), vec4<f32>(2.4));
    let lower = sRGB/vec4<f32>(12.92);
    return mix(higher, lower, cutoff);
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    return toLinear(color);
}