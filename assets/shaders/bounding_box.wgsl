
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

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {

    let aspect_ratio = material.size.y / material.size.x ;

    var uv_original: float2 = in.uv  - float2(0.5);
    uv_original.y = uv_original.y * aspect_ratio ;

    let cx =  material.size.x / 42.0;
    let cy =  material.size.y / 42.0;

    let p = 0.5 - 0.05 / 1.1;
    let bb_size = 0.1;

    let zoo = material.zoom / 0.08; 

    let width = 0.0051;
    let sw = 0.0025 * zoo;

    let w = sw / cx;
    let m = p + w;
    let l = p-w;
    let q = width / cx;

    let xdo = smoothStep( m+q, l+q, abs(uv_original.x ) );
    let xdi = 1.0 - smoothStep( m-q, l-q, abs(uv_original.x ) );

    let p = 0.5 - 0.05 / 1.0 - bb_size / cx;
    let w = sw / cx;
    let m = p + w;
    let l = p-w;
    let q = width / cx;
    // let xda = 1 -smoothStep( m+q, l+q, (0.5-abs(uv_original.x) )/1.1 );
    let xda= 1.0 - smoothStep( m+q, l+q, abs(uv_original.x ) );
    // xda = 1 - step(uv.x*material.size.x, material.size.x/2 - 10.0 );
    // xda = step(uv.x*material.size.x, material.size.x*0.9/2 );

    let p = 0.5 - 0.05 / 1.1;
    let w = sw / cy;
    let m = p + w;
    let l = p-w;
    let q = width / cy;

    let ydi = 1.0 - smoothStep( m-q, l-q, abs(uv_original.y ) );
    let ydo = smoothStep( m+q, l+q, abs(uv_original.y ) );
    

    let p = 0.5 - 0.05 / 1.0 - bb_size / cy;
    let w = sw / cy;
    let m = p + w;
    let l = p-w;
    let q = width / cy;
    // let yda = smoothStep( p, p-0.01, 0.5 - abs(uv_original.y ) );
    let yda= 1.0 - smoothStep( m+q, l+q, abs(uv_original.y ) );
    // yda = step(0.5 - abs(uv_original.y ), p );


    let xd = xdi * xdo * ydo * xda * yda;
    let yd = ydi * ydo * xdo * xda * yda;


    let red = float4(0.0, 0.0, 0.0, 0.00);
    let color2 = material.color;
    // color2.a = 0.3;

    let rect = mix( red ,color2 , min(yd + xd, 1.0));
    //  let rect = mix( red ,color2 , xda);




    return rect;
    // return float4(1.0, 0.0, 0.0, 1.0);

}