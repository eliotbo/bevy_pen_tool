// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_view_bind_group
[[group(0), binding(0)]]
var<uniform> view: View;
#import bevy_sprite::mesh2d_struct
[[group(1), binding(0)]]
var<uniform> mesh: Mesh2d;

type float4 = vec4<f32>;
type float2 = vec2<f32>;

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2)]] uv: vec2<f32>;
};
struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    [[builtin(position)]] clip_position: vec4<f32>;
    // We pass the vertex color to the framgent shader in location 0
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] uv: vec2<f32>;
};
/// Entry point for the vertex shader
[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;
    out.uv = vertex.uv;
    return out;
}

fn fromLinear(linearRGB: float4) -> float4
{
    let cutoff: vec4<f32> = vec4<f32>(linearRGB < float4(0.0031308));
    let higher: vec4<f32> = float4(1.055)*pow(linearRGB, float4(1.0/2.4)) - float4(0.055);
    let lower: vec4<f32> = linearRGB * float4(12.92);

    return mix(higher, lower, cutoff);
}

// // Converts a color from sRGB gamma to linear light gamma
fn toLinear(sRGB: float4) -> float4
{
    let cutoff = vec4<f32>(sRGB < float4(0.04045));
    let higher = pow((sRGB + float4(0.055))/float4(1.055), float4(2.4));
    let lower = sRGB/float4(12.92);

    return mix(higher, lower, cutoff);
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    // The color is interpolated between vertices by default
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] uv: vec2<f32>;
};

/// Entry point for the fragment shader
[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    return toLinear(in.color);
    // return float4(1.0,1.0, 0.0, 1.0);
}