#import bevy_pbr::mesh_view_bindings

@group(1) @binding(0)
var sprite_texture: texture_2d<f32>;
@group(1) @binding(1)
var sprite_sampler: sampler;

struct RoadMat {
    center_of_mass: vec2<f32>,
    show_com: f32,
};

@group(1) @binding(2)
var<uniform> uni: RoadMat;


@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    var color = textureSample(sprite_texture, sprite_sampler, uv); 
    if uni.show_com > 0.5 {
        let selector_color = vec4<f32>(0.5, 0.5, 0.5, 0.5);
        let mixed_color = mix(color, selector_color, 0.2);
        return mixed_color;
    }
    return color;
}