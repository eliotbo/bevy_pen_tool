#version 450
#define PI 3.1415926538

layout(location = 0) in vec4 gl_FragCoord;
layout(location = 1) in vec3 v_Position;
layout(location = 2) in vec2 Vertex_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform MyShader_color{
    vec4 color;
};
layout(set = 2, binding = 1) uniform MyShader_t{
    float t;
};
layout(set = 2, binding = 2) uniform MyShader_zoom{
    float zoom;
};
layout(set = 2, binding = 3) uniform MyShader_size{
    vec2 size;
};
layout(set = 2, binding = 4) uniform MyShader_clearcolor{
    vec4 clear_color;
};

/////////////// unused ///////////////
float sdBox( in vec2 p, in vec2 b )
{
    vec2 d = abs(p)-b;
    return length(max(d,0.0)) + min(max(d.x,d.y),0.0);
}

float sdBoxar( in vec2 p, in vec2 b )
{
    vec2 d = abs(p)-b;
    return length(max(d,0.0)) + min(max(d.x,d.y),0.0);
}


float sdSegment( in vec2 p, in vec2 a, in vec2 b )
{
    vec2 pa = p-a, ba = b-a;
    float h = clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
    return length( pa - ba*h );
}


float sdSquareEdge(vec2 p, float r, float w)
{
    float d = sdBox( p, vec2(r,r) );
    float s1 = smoothstep(-0.005, 0.01, d);

    float width = 0.01;
    float s2 = smoothstep(-0.005-w, 0.002-w, d);
    return 1.0 - abs(s1-s2);
}
/////////////// unused ///////////////


float sdCircle( vec2 p, float r)
{
    float d = length(p) - r;
    return d;
}


void main( )
{
    vec2 pos = vec2(0.5, 0.5);
    vec2 uv_original = (Vertex_Uv.xy-pos);
    
    float aspect_ratio = size.y / size.x ;
    vec2 uv_sized = uv_original * size; 
    vec2 uv_ar = vec2(uv_original.x, uv_original.y * aspect_ratio); // aspect ratio corrected
    vec2 maxes = size / 2 ;


    float margin = 0.15;
    float d = sdBox( uv_ar, vec2(0.5 - margin ,  0.5*aspect_ratio - margin ) );
    float offset = 0.1;
    d = smoothstep(0.01+offset, -0.01+offset, d);





    vec4 bg_color = clear_color;
    bg_color.a = 0.0;

    vec4 color_mod = color;
    color_mod.a = 0.3;
    vec4 rect = mix( color_mod ,  bg_color , 1 - d );
    

    o_Target = rect;
}
