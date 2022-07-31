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

fn sdTriangle(   p: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32> ) -> f32
{
    let e0 = p1-p0;
    let e1 = p2-p1;
    let e2 = p0-p2;
    let v0 = p -p0;
    let v1 = p -p1;
    let v2 = p -p2;
    let pq0 = v0 - e0*clamp( dot(v0,e0)/dot(e0,e0), 0.0, 1.0 );
    let pq1 = v1 - e1*clamp( dot(v1,e1)/dot(e1,e1), 0.0, 1.0 );
    let pq2 = v2 - e2*clamp( dot(v2,e2)/dot(e2,e2), 0.0, 1.0 );
    let s = sign( e0.x*e2.y - e0.y*e2.x );
    let d = min(min(vec2<f32>(dot(pq0,pq0), s*(v0.x*e0.y-v0.y*e0.x)),
                     vec2<f32>(dot(pq1,pq1), s*(v1.x*e1.y-v1.y*e1.x))),
                     vec2<f32>(dot(pq2,pq2), s*(v2.x*e2.y-v2.y*e2.x)));
    return -sqrt(d.x)*sign(d.y);
}


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

    // let pos = vec2<f32>(0.5, 0.5);
    // let uv_original = (Vertex_Uv.xy-pos);



    let p0 = vec2<f32>(0.0, -0.4);
    let p1 = vec2<f32>(0.4, 0.4);
    let p2 = vec2<f32>(-0.4, 0.4);
    var d0 = sdTriangle( uv_original,  p0,  p1,  p2 );

    let zoo = material.zoom / 0.08 / 5.;
    let w = -0.02 * zoo;
    d0 = smoothstep(-w, w, d0);

    let p3 = vec2<f32>(0.0, -0.0);
    let p4 = vec2<f32>(0.4, 0.45);
    let p5 = vec2<f32>(-0.4, 0.45);
    
    var d1 = sdTriangle( uv_original,  p3,  p4,  p5 );
    d1 = smoothstep(-w, w, d1);


    let d = d0 * (1.0 - d1);

    // let color_with_t = material.color;

    var bg_color = material.clearcolor;
    bg_color.a = 0.0;

    let circle = mix( material.color, bg_color,  1.0 - d );


    // o_Target = circle;

    

    return circle;
    // return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
