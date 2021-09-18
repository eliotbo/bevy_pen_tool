#version 450


layout(location = 1) in vec4 v_Position;
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
    vec2 qsize;
};


void main( )
{
    vec2 pos = vec2(0.5, 0.5);
    vec2 uv = (Vertex_Uv.xy-pos);


    float cx =  qsize.x/42.0;
    float cy =  qsize.y/42.0;

    float p = 0.5 - 0.05/1.1;
    float bb_size = 0.1;

    float zoo = zoom / 0.08; 

    float width = 0.0051;
    float sw = 0.0025 * zoo;

    float w = sw / cx;
    float m = p + w;
    float l = p-w;
    float q = width / cx;

    float xdo = smoothstep( m+q, l+q, abs(uv.x ) );
    float xdi = 1 - smoothstep( m-q, l-q, abs(uv.x ) );

    p = 0.5 - 0.05/1. - bb_size/cx;
    w = sw / cx;
    m = p + w;
    l = p-w;
    q = width / cx;
    // float xda = 1 -smoothstep( m+q, l+q, (0.5-abs(uv.x) )/1.1 );
    float xda= 1 - smoothstep( m+q, l+q, abs(uv.x ) );
    // xda = 1 - step(uv.x*qsize.x, qsize.x/2 - 10.0 );
    // xda = step(uv.x*qsize.x, qsize.x*0.9/2 );

    p = 0.5 - 0.05/1.1;
    w = sw / cy;
    m = p + w;
    l = p-w;
    q = width / cy;

    float ydi = 1 - smoothstep( m-q, l-q, abs(uv.y ) );
    float ydo = smoothstep( m+q, l+q, abs(uv.y ) );
    

    p = 0.5 - 0.05/1. - bb_size/cy;
    w = sw / cy;
    m = p + w;
    l = p-w;
    q = width / cy;
    // float yda = smoothstep( p, p-0.01, 0.5 - abs(uv.y ) );
    float yda= 1 - smoothstep( m+q, l+q, abs(uv.y ) );
    // yda = step(0.5 - abs(uv.y ), p );


    float xd = xdi * xdo * ydo * xda * yda;
    float yd = ydi * ydo * xdo * xda * yda;


    vec4 red = vec4(0.0, 0.0, 0.0, 0.00);
    vec4 color2 = color;
    // color2.a = 0.3;

    vec4 rect = mix( red ,color2 , min(yd + xd, 1));
    //  vec4 rect = mix( red ,color2 , xda);

    o_Target = rect;
}
