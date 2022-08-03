#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_types




struct UiMat {
    color: vec4<f32>,
    clearcolor: vec4<f32>,
    t: f32,
    zoom: f32,
    size: vec2<f32>,
    hovered: f32,
};


@group(1) @binding(0)
var<uniform> material: UiMat;


struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
#ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
#endif
};


@group(0) @binding(0)
var<uniform> view: View;


@group(2) @binding(0)
var<uniform> mesh: Mesh2d;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = mesh.model * vec4<f32>(vertex.position, 1.0);

    var out: VertexOutput;
    out.uv = vertex.uv;
    out.world_position = world_position;
    out.clip_position = view.view_proj * world_position;
    out.world_normal = mat3x3<f32>(
        mesh.inverse_transpose_model[0].xyz,
        mesh.inverse_transpose_model[1].xyz,
        mesh.inverse_transpose_model[2].xyz
    ) * vertex.normal;
#ifdef VERTEX_TANGENTS
    out.world_tangent = vec4<f32>(
        mat3x3<f32>(
            mesh.model[0].xyz,
            mesh.model[1].xyz,
            mesh.model[2].xyz
        ) * vertex.tangent.xyz,
        vertex.tangent.w
    );
#endif
    return out;
}

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
#ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
#endif
};

fn sdBox(p: vec2<f32>, b: vec2<f32>) -> f32 {
  let d = abs(p) - b;
  return length(max(d, vec2<f32>(0.))) + min(max(d.x, d.y), 0.);
}

fn sdCircle(p: vec2<f32>, r: f32) -> f32 {
  return length(p) - r;
}



@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {

    let aspect_ratio = material.size.y / material.size.x ;

    var uv_original: vec2<f32> = in.uv  - vec2<f32>(0.5);
    uv_original.y = uv_original.y * aspect_ratio ;


    let margin = 0.15;
    var d = sdBox( uv_original, vec2<f32>(0.5 - margin ,  0.5*aspect_ratio - margin ) );
    let offset = 0.1;
    d = smoothstep(0.01+offset, -0.01+offset, d);

    var bg_color = (material.clearcolor);
    bg_color.a = 0.0;

    var color_mod = (material.color);
    color_mod.a = 0.3;

    var rect = mix( color_mod ,  bg_color , 1.0 -  d );
    

    return rect;
}
