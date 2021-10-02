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
layout(set = 2, binding = 3) uniform MyShader_clearcolor{
    vec4 clear_color;
};


/////////////// unused ///////////////
float sdBox( in vec2 p, in vec2 b )
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

    float zoo = zoom / 0.08;
    float w = -0.03 * zoo;

    float d0 = sdCircle( uv_original, 0.5 );
    d0 = smoothstep(-w, w, d0);

    float d1 = sdCircle( uv_original, 0.3 / (1 + pow(zoo, 0.5)));
    d1 = smoothstep(-w, w, d1);

    float d = 1 - d0 * (1- d1);


    // makes the middle quads disappear close to the ends
    float transparency = smoothstep( 0.0, 0.05, 0.5-abs(t-0.5));
    vec4 color_with_t = color;
    color_with_t.a = transparency;

    // vec4 other_color = vec4(110. / 255., 127. / 255., 128. / 255., 0.0);
    vec4 bg_color = clear_color;
    bg_color.a = 0.0;
    vec4 circle = mix( color_with_t ,bg_color  , d );


    o_Target = circle;
}
