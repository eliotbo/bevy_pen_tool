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
    // uv_original.y = uv_original.y * aspect_ratio ;



    let cx =  material.size.x / 42.0;
    let cy =  material.size.y / 42.0;

    let p = 0.5 - 0.05 / 1.1;
    let bb_size = 0.1;

    let zoo =  material.zoom / 0.08; 

    let width = 0.00251 * 0.5;
    let sw = 0.0025 * zoo * 1.0;

    let w = sw / cx;
    let m = p + w;
    let l = p-w;
    let q = width / cx;

    let xdo = smoothstep( m+q, l+q, abs(uv_original.x ) );
    let xdi = 1.0 - smoothstep( m-q, l-q, abs(uv_original.x ) );

    let p = 0.5 - 0.05 / 1. - bb_size/cx;
    let w = sw / cx;
    let m = p + w;
    let l = p-w;
    let q = width / cx;

    // let xda= 1 - smoothstep( m+q, l+q, abs(uv.x ) );


    let p = 0.5 - 0.05 / 1.1;
    let w = sw / cy;
    let m = p + w;
    let l = p-w;
    let q = width / cy;

    let ydi = 1.0 - smoothstep( m-q, l-q, abs(uv_original.y ) );
    let ydo = smoothstep( m+q, l+q, abs(uv_original.y ) );
    

    // p = 0.5 - 0.05/1. - bb_size/cy;
    // w = sw / cy;
    // m = p + w;
    // l = p-w;
    // q = width / cy;
    // // let yda = smoothstep( p, p-0.01, 0.5 - abs(uv.y ) );
    // let yda= 1 - smoothstep( m+q, l+q, abs(uv.y ) );
    // // yda = step(0.5 - abs(uv.y ), p );


    let xd = xdi * xdo * ydo; // * xda * yda;
    let yd = ydi * ydo * xdo; // * xda * yda;


    let red = vec4<f32>(0.0, 0.0, 0.0, 0.00);
    let color2 = material.color;
    // color2.a = 0.3;

    let rect = mix( red ,color2 , min(yd + xd, 1.0));
    //  vec4 rect = mix( red ,color2 , xda);

    // o_Target = rect;
    

    return rect;
    // return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

