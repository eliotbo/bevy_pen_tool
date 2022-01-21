// #version 450
// #define PI 3.1415926538

// layout(location = 0) in vec4 gl_FragCoord;
// layout(location = 1) in vec3 v_Position;
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
//     vec2 size;
// };
// layout(set = 2, binding = 4) uniform MyShader_clearcolor{
//     vec4 clear_color;
// };
// layout(set = 2, binding = 5) uniform MyShader_hovered{
//     float hovered;
// };

// /////////////// unused ///////////////
// float sdBox( in vec2 p, in vec2 b )
// {
//     vec2 d = abs(p)-b;
//     return length(max(d,0.0)) + min(max(d.x,d.y),0.0);
// }

// float sdBoxar( in vec2 p, in vec2 b )
// {
//     vec2 d = abs(p)-b;
//     return length(max(d,0.0)) + min(max(d.x,d.y),0.0);
// }


// float sdSegment( in vec2 p, in vec2 a, in vec2 b )
// {
//     vec2 pa = p-a, ba = b-a;
//     float h = clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
//     return length( pa - ba*h );
// }


// float sdSquareEdge(vec2 p, float r, float w)
// {
//     float d = sdBox( p, vec2(r,r) );
//     float s1 = smoothStep(-0.005, 0.01, d);

//     float width = 0.01;
//     float s2 = smoothStep(-0.005-w, 0.002-w, d);
//     return 1.0 - abs(s1-s2);
// }
// /////////////// unused ///////////////


// float sdCircle( vec2 p, float r)
// {
//     float d = length(p) - r;
//     return d;
// }


// void main( )
// {
//     vec2 pos = vec2(0.5, 0.5);
//     vec2 uv_original = (Vertex_Uv.xy-pos);
    
//     float aspect_ratio = size.y / size.x ;
//     vec2 uv_sized = uv_original * size; 
//     vec2 uv_ar = vec2(uv_original.x, uv_original.y * aspect_ratio); // aspect ratio corrected
//     vec2 maxes = size / 2 ;


    // float margin = 0.2;
    // float d = sdBox( uv_ar, vec2(0.5 - margin ,  0.5*aspect_ratio - margin ) );
    // float offset = 0.1;
    // d = smoothStep(0.01+offset, -0.01+offset, d);


    // vec4 other_color = vec4(.0, .0, .0, 0.0);
    // vec4 white = vec4(1.0, 1.0, 1.0, 1.0);
    // vec4 yellow = vec4(0.6, 0.75, 0.04, 1.0);
    // vec4 red = vec4(0.8, 0.45, 0.04, 1.0);

    // vec4 bg_color = clear_color;
    // bg_color.a = 0.0;

    // vec4 rect = mix( color ,  bg_color , 1 - d );
    // vec4 rect_memory = rect;



    // // add white contour when color is selected
    // if (t > 0.5) {
    //     float r = 0.3; // size
    //     float w = 0.06; // contour width
    //     float d = sdBox( uv_original, vec2(r,r) );
    //     float c = 0.15; // contour roundness
    //     float b = 0.03; // smoothing
    //     float s1 = smoothStep(-b+w+c, b+w+c, d);
    //     float s2 = smoothStep(-b-w+c, b-w+c, d);

    //     d = (1-s1) * (s2);
   
    //     rect = mix( white ,  other_color , 1 - d );
    //     rect = mix(rect_memory, rect,  d);
    // }


    // if (hovered > 0.9) {
    //     float r = 0.3; // size
    //     float w = 0.03; // contour width
    //     float d = sdBox( uv_original, vec2(r,r) );
    //     float c = 0.15; // contour roundness
    //     float b = 0.03; // smoothing
    //     float s1 = smoothStep(-b+w+c, b+w+c, d);
    //     float s2 = smoothStep(-b-w+c, b-w+c, d);

    //     d = (1-s1) * (s2);
   
    //     vec4 rect_hover = mix( yellow ,  other_color , 1 - d );
    //     rect = mix(rect, rect_hover,  d);
    // } else if (hovered>0.7) // if pressed
    // {
    //     float r = 0.3; // size
    //     float w = 0.03; // contour width
    //     float d = sdBox( uv_original, vec2(r,r) );
    //     float c = 0.15; // contour roundness
    //     float b = 0.03; // smoothing
    //     float s1 = smoothStep(-b+w+c, b+w+c, d);
    //     float s2 = smoothStep(-b-w+c, b-w+c, d);

    //     d = (1-s1) * (s2);
   
    //     vec4 rect_hover = mix( red ,  other_color , 1 - d );
    //     rect = mix(rect, rect_hover,  d);
    // }

    

//     o_Target = rect;
// }


#import bevy_sprite::mesh2d_struct
#import bevy_sprite::mesh2d_view_bind_group

type float4 = vec4<f32>;
type float2 = vec2<f32>;

struct ButtonMat {
    color: vec4<f32>;
    clearcolor: vec4<f32>;
    t: f32;
    zoom: f32;
    size: vec2<f32>;
    hovered: f32;
};

[[group(1), binding(0)]]
var<uniform> material: ButtonMat;


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
    uv_original.x = uv_original.x * aspect_ratio ;

    let margin = 0.2;
    var d = sdBox( uv_original, float2(0.5 - margin ,  0.5*aspect_ratio - margin ) );
    let offset = 0.1;
    d = smoothStep(0.01+offset, -0.01+offset, d);


    let other_color = float4(.0, .0, .0, 0.0);
    let white = float4(1.0, 1.0, 1.0, 1.0);
    let yellow = float4(0.6, 0.75, 0.04, 1.0);
    let red = float4(0.8, 0.45, 0.04, 1.0);

    var bg_color = material.clearcolor;
    bg_color.a = 0.0;

    var rect = mix( material.color ,  bg_color , 1. - d );
    let rect_memory = rect;



    // add white contour when color is selected
    if (material.hovered > 0.5) {
        let r = 0.3; // size
        let w = 0.06; // contour width
        var d = sdBox( uv_original, float2(r,r) );
        let c = 0.15; // contour roundness
        let b = 0.03; // smoothing
        let s1 = smoothStep(-b+w+c, b+w+c, d);
        let s2 = smoothStep(-b-w+c, b-w+c, d);

        d = (1.-s1) * (s2);
   
        rect = mix( white ,  other_color , 1. - d );
        rect = mix(rect_memory, rect,  d);
    }


    if (material.hovered > 0.9) {
        let r = 0.3; // size
        let w = 0.03; // contour width
        var d = sdBox( uv_original, float2(r,r) );
        let c = 0.15; // contour roundness
        let b = 0.03; // smoothing
        let s1 = smoothStep(-b+w+c, b+w+c, d);
        let s2 = smoothStep(-b-w+c, b-w+c, d);

        d = (1.-s1) * (s2);
   
        let rect_hover = mix( yellow ,  other_color , 1. - d );
        rect = mix(rect, rect_hover,  d);
    } else if (material.hovered>0.7) // if pressed
    {
        let r = 0.3; // size
        let w = 0.03; // contour width
        var d = sdBox( uv_original, float2(r,r) );
        let c = 0.15; // contour roundness
        let b = 0.03; // smoothing
        let s1 = smoothStep(-b+w+c, b+w+c, d);
        let s2 = smoothStep(-b-w+c, b-w+c, d);

        d = (1.-s1) * (s2);
   
        let rect_hover = mix( red ,  other_color , 1. - d );
        rect = mix(rect, rect_hover,  d);
    }

    // rect = float4(rect.xyz * 0.5, 1.0);
    // rect.w = 1.0;


    return rect;
}
