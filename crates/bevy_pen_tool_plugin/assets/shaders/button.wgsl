#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_types




struct ButtonMat {
    color: vec4<f32>,
    clearcolor: vec4<f32>,
    t: f32,
    zoom: f32,
    size: vec2<f32>,
    hovered: f32,
};


@group(1) @binding(0)
var<uniform> material: ButtonMat;


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


@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {

    let aspect_ratio = material.size.y / material.size.x ;

    var uv_original: vec2<f32> = in.uv  - vec2<f32>(0.5);
    uv_original.x = uv_original.x * aspect_ratio ;

    let margin = 0.2;
    var d = sdBox( uv_original, vec2<f32>(0.5 - margin ,  0.5*aspect_ratio - margin ) );
    let offset = 0.1;
    d = smoothstep(0.01+offset, -0.01+offset, d);


    let other_color = vec4<f32>(.0, .0, .0, 0.0);
    let white = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let yellow = vec4<f32>(0.6, 0.75, 0.04, 1.0);
    let red = vec4<f32>(0.8, 0.45, 0.04, 1.0);

    var bg_color = material.clearcolor;
    bg_color.a = 0.0;

    var rect = mix( material.color ,  bg_color , 1. - d );
    let rect_memory = rect;



    // add white contour when color is selected
    if (material.hovered > 0.5) {
        let r = 0.3; // size
        let w = 0.06; // contour width
        var d = sdBox( uv_original, vec2<f32>(r,r) );
        let c = 0.15; // contour roundness
        let b = 0.03; // smoothing
        let s1 = smoothstep(-b+w+c, b+w+c, d);
        let s2 = smoothstep(-b-w+c, b-w+c, d);

        d = (1.-s1) * (s2);
   
        rect = mix( white ,  other_color , 1. - d );
        rect = mix(rect_memory, rect,  d);
    }


    if (material.hovered > 0.9) {
        let r = 0.3; // size
        let w = 0.03; // contour width
        var d = sdBox( uv_original, vec2<f32>(r,r) );
        let c = 0.15; // contour roundness
        let b = 0.03; // smoothing
        let s1 = smoothstep(-b+w+c, b+w+c, d);
        let s2 = smoothstep(-b-w+c, b-w+c, d);

        d = (1.-s1) * (s2);
   
        let rect_hover = mix( yellow ,  other_color , 1. - d );
        rect = mix(rect, rect_hover,  d);
    } else if (material.hovered>0.7) // if pressed
    {
        let r = 0.3; // size
        let w = 0.03; // contour width
        var d = sdBox( uv_original, vec2<f32>(r,r) );
        let c = 0.15; // contour roundness
        let b = 0.03; // smoothing
        let s1 = smoothstep(-b+w+c, b+w+c, d);
        let s2 = smoothstep(-b-w+c, b-w+c, d);

        d = (1.-s1) * (s2);
   
        let rect_hover = mix( red ,  other_color , 1. - d );
        rect = mix(rect, rect_hover,  d);
    }

    // rect = vec4<f32>(rect.xyz * 0.5, 1.0);
    // rect.w = 1.0;


    return toLinear( rect);
}
