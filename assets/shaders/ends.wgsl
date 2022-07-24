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

    var uv_original: vec2<f32> = (in.uv  - vec2<f32>(0.5));
    uv_original.x = uv_original.x / 2.0 ;


    // let zoo = material.zoom / 0.08;
    let zoo = 1.0;
    let w = -0.03 * zoo;

    let offset = 0.5;

    var d0 = sdCircle( uv_original - offset * vec2<f32>(0.5,0.), 0.5 );
    d0 = smoothstep(-w, w, d0);

    var d1 = sdCircle( uv_original - offset * vec2<f32>(0.5, 0.), 0.3 / (1.0 + pow(zoo, 0.5)));
    d1 = smoothstep(-w, w, d1);

    let d = 1. - d0 * (1. - d1);


    var color_with_t = material.color;
    

    var bg_color = material.clearcolor;
    bg_color.a = 0.0;
    let circle = mix( color_with_t ,bg_color  , d );

    return circle;
}



// void main( )
// {
//     vec2 pos = vec2(0.5, 0.5);
//     vec2 uv_original = (Vertex_Uv.xy-pos);
//     uv_original.x =  (uv_original.x -0.5) / 2.0;

//     float zoo = zoom / 0.08;
//     float w = -0.03 * zoo;

//     float d0 = sdCircle( uv_original, 0.5 );
//     d0 = smoothstep(-w, w, d0);

//     float d1 = sdCircle( uv_original, 0.3 / (1 + pow(zoo, 0.5)));
//     d1 = smoothstep(-w, w, d1);

//     float d = 1 - d0 * (1- d1);


//     // makes the middle quads disappear close to the ends
//     float transparency = smoothstep( 0.0, 0.05, 0.5-abs(t-0.5));
//     vec4 color_with_t = color;
//     color_with_t.a = transparency;

//     // vec4 other_color = vec4(110. / 255., 127. / 255., 128. / 255., 0.0);
//     vec4 bg_color = clear_color;
//     bg_color.a = 0.0;
//     vec4 circle = mix( color_with_t ,bg_color  , d );


//     o_Target = circle;
// }
