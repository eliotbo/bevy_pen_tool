#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_types

// [[group(0), binding(0)]]

@group(0) @binding(0)
var<uniform> view: View;


@group(1) @binding(0)
var<uniform> mesh: Mesh2d;




struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,

};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

/// Entry point for the vertex shader
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;
    out.uv = vertex.uv;
    return out;
}

fn fromLinear(linearRGB: vec4<f32>) -> vec4<f32>
{
    let cutoff: vec4<f32> = vec4<f32>(linearRGB < vec4<f32>(0.0031308));
    let higher: vec4<f32> = vec4<f32>(1.055)*pow(linearRGB, vec4<f32>(1.0/2.4)) - vec4<f32>(0.055);
    let lower: vec4<f32> = linearRGB * vec4<f32>(12.92);

    return mix(higher, lower, cutoff);
}

// // Converts a color from sRGB gamma to linear light gamma
fn toLinear(sRGB: vec4<f32>) -> vec4<f32>
{
    let cutoff = vec4<f32>(sRGB < vec4<f32>(0.04045));
    let higher = pow((sRGB + vec4<f32>(0.055))/vec4<f32>(1.055), vec4<f32>(2.4));
    let lower = sRGB/vec4<f32>(12.92);

    return mix(higher, lower, cutoff);
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    // The color is interpolated between vertices by default
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};
/// Entry point for the fragment shader
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return toLinear(in.color);
    // return vec4<f32>(1.0,1.0, 0.0, 1.0);
}