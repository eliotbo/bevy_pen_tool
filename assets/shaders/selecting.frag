// #version 450


// layout(location = 1) in vec4 v_Position;
// layout(location = 2) in vec2 Vertex_Uv;

// layout(location = 0) out vec4 o_Target;

// layout(set = 2, binding = 0) uniform MyShader_color{
//     vec4 color;
// };
// layout(set = 2, binding = 1) uniform MyShader_t{
//     float t;
// };

// layout(set = 2, binding = 2) uniform MyShader_zoom{
//     float zoom;
// };
// layout(set = 2, binding = 3) uniform MyShader_size{
//     vec2 qsize;
// };


// void main( )
// {
//     vec2 pos = vec2(0.5, 0.5);
//     vec2 uv = (Vertex_Uv.xy-pos);


//     float cx =  qsize.x/42.0;
//     float cy =  qsize.y/42.0;

//     float p = 0.5 - 0.05/1.1;
//     float bb_size = 0.1;

//     float zoo = zoom / 0.08; 

//     float width = 0.00251 * 0.5;
//     float sw = 0.0025 * zoo * 1.0;

//     float w = sw / cx;
//     float m = p + w;
//     float l = p-w;
//     float q = width / cx;

//     float xdo = smoothstep( m+q, l+q, abs(uv.x ) );
//     float xdi = 1 - smoothstep( m-q, l-q, abs(uv.x ) );

//     p = 0.5 - 0.05/1. - bb_size/cx;
//     w = sw / cx;
//     m = p + w;
//     l = p-w;
//     q = width / cx;

//     // float xda= 1 - smoothstep( m+q, l+q, abs(uv.x ) );


//     p = 0.5 - 0.05/1.1;
//     w = sw / cy;
//     m = p + w;
//     l = p-w;
//     q = width / cy;

//     float ydi = 1 - smoothstep( m-q, l-q, abs(uv.y ) );
//     float ydo = smoothstep( m+q, l+q, abs(uv.y ) );
    

//     // p = 0.5 - 0.05/1. - bb_size/cy;
//     // w = sw / cy;
//     // m = p + w;
//     // l = p-w;
//     // q = width / cy;
//     // // float yda = smoothstep( p, p-0.01, 0.5 - abs(uv.y ) );
//     // float yda= 1 - smoothstep( m+q, l+q, abs(uv.y ) );
//     // // yda = step(0.5 - abs(uv.y ), p );


//     float xd = xdi * xdo * ydo; // * xda * yda;
//     float yd = ydi * ydo * xdo; // * xda * yda;


//     vec4 red = vec4(0.0, 0.0, 0.0, 0.00);
//     vec4 color2 = color;
//     // color2.a = 0.3;

//     vec4 rect = mix( red ,color2 , min(yd + xd, 1));
//     //  vec4 rect = mix( red ,color2 , xda);

//     o_Target = rect;
// }



#import bevy_sprite::mesh2d_struct
#import bevy_sprite::mesh2d_view_bind_group

type float4 = vec4<f32>;
type float2 = vec2<f32>;

struct UiMat {
    color: vec4<f32>;
    clearcolor: vec4<f32>;
    t: f32;
    zoom: f32;
    size: vec2<f32>;
    hovered: f32;
};

[[group(1), binding(0)]]
var<uniform> material: UiMat;


struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
#ifdef VERTEX_TANGENTS
    [[location(3)]] tangent: vec4<f32>;
#endif
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
#ifdef VERTEX_TANGENTS
    [[location(3)]] world_tangent: vec4<f32>;
#endif
};

[[group(0), binding(0)]]
var<uniform> view: View;

[[group(2), binding(0)]]
var<uniform> mesh: Mesh2d;

[[stage(vertex)]]
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
    [[builtin(front_facing)]] is_front: bool;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
#ifdef VERTEX_TANGENTS
    [[location(3)]] world_tangent: vec4<f32>;
#endif
};

fn sdBox(p: vec2<f32>, b: vec2<f32>) -> f32 {
  let d = abs(p) - b;
  return length(max(d, vec2<f32>(0.))) + min(max(d.x, d.y), 0.);
}

fn sdCircle(p: vec2<f32>, r: f32) -> f32 {
  return length(p) - r;
}



[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {

    let aspect_ratio = material.size.y / material.size.x ;

    var uv_original: float2 = in.uv  - float2(0.5);
    uv_original.y = uv_original.y * aspect_ratio ;


    let margin = 0.15;
    var d = sdBox( uv_original, float2(0.5 - margin ,  0.5*aspect_ratio - margin ) );
    let offset = 0.1;
    d = smoothStep(0.01+offset, -0.01+offset, d);

    var bg_color = (material.clearcolor);
    bg_color.a = 0.0;

    var color_mod = (material.color);
    color_mod.a = 0.3;

    var rect = mix( color_mod ,  bg_color , 1.0 -  d );
    

    return rect;
    // return float4(1.0, 1.0, 1.0, 1.0);
}

