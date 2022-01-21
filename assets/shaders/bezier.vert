// #version 450


// layout(location = 0) in vec3 Vertex_Position;
// layout(location = 2) in vec2 Vertex_Uv;

// // layout(location = 5) in vec3 Instance_Position;

// layout(location = 2) out vec2 Vertex_Uv_O;
// layout(location = 0) out vec4 v_Position;

// // layout(set = 2, binding = 1) uniform DamnUniform{
// //     float ya ;
// //     // vec3 ba;
// // };


// layout(set = 0, binding = 0) uniform CameraViewProj {
//     mat4 ViewProj;
// };
// layout(set = 1, binding = 0) uniform Transform {
//     mat4 Model;
// };

// void main() {
    
//     float d = 1.0;
//     if (gl_InstanceIndex == 0) {
//     // if (gl_BaseInstance == 1) {
//         d = 0.0;
//     }
//     vec3 pos = vec3(50.0*d, 0.0, 0.0) ;
//     gl_Position = ViewProj * Model * vec4(Vertex_Position + pos, 1.0);
//     Vertex_Uv_O = Vertex_Uv;
//     v_Position = gl_Position;

// }


#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
    mat4 InverseView;
    mat4 Projection;
    vec3 WorldPosition;
    float near;
    float far;
    float width;
    float height;
};

layout(set = 2, binding = 0) uniform Mesh {
    mat4 Model;
    mat4 InverseTransposeModel;
    uint flags;
};

void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
