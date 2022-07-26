use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::InnerMeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, Mesh2dPipeline},
    utils::{FixedState, Hashed},
};

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone, Component, Default)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f95a4b"]
pub struct RoadMesh2dMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub road_texture: Handle<Image>,
    // pub mesh2d_pipeline: Mesh2dPipeline,
}

/// Plugin that renders [`RoadMesh2d`]s
pub struct RoadMesh2dPlugin;

impl Plugin for RoadMesh2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<RoadMesh2dMaterial>::default());
    }
}

impl Material2d for RoadMesh2dMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/road_mesh.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &Hashed<InnerMeshVertexBufferLayout, FixedState>,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // let formats = vec![
        //     // Position
        //     VertexFormat::Float32x3,
        //     VertexFormat::Float32x4,
        //     // UV
        //     VertexFormat::Float32x2,
        // ];

        // let vertex_layout =
        //     VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        // descriptor.vertex.buffers = vec![vertex_layout];

        println!("descriptor : {:?}", descriptor.layout);

        // descriptor.layout = Some(vec![
        //     // // Bind group 0 is the view uniform
        //     // self.mesh2d_pipeline.view_layout.clone(),
        //     // // Bind group 1 is the mesh uniform
        //     // self.mesh2d_pipeline.mesh_layout.clone(),
        //     // // texture
        //     // self.material_layout.clone(),
        // ]);

        // *descriptor = RenderPipelineDescriptor {
        //     vertex: VertexState {
        //         // Use our custom shader
        //         shader: COLORED_MESH2D_SHADER_HANDLE.typed::<Shader>(),
        //         entry_point: "vertex".into(),
        //         shader_defs: Vec::new(),
        //         // Use our custom vertex buffer
        //         // buffers: vec![VertexBufferLayout {
        //         //     array_stride: vertex_array_stride,
        //         //     step_mode: VertexStepMode::Vertex,
        //         //     attributes: vertex_attributes,
        //         // }],
        //         buffers: vec![vertex_layout],
        //     },
        //     fragment: Some(FragmentState {
        //         // Use our custom shader
        //         shader: COLORED_MESH2D_SHADER_HANDLE.typed::<Shader>(),
        //         shader_defs: Vec::new(),
        //         entry_point: "fragment".into(),
        //         targets: vec![Some(ColorTargetState {
        //             format: TextureFormat::bevy_default(),
        //             blend: Some(BlendState::ALPHA_BLENDING),
        //             write_mask: ColorWrites::ALL,
        //         })],
        //     }),
        //     // Use the two standard uniforms for 2d meshes
        //     layout: Some(vec![
        //         // Bind group 0 is the view uniform
        //         self.mesh2d_pipeline.view_layout.clone(),
        //         // Bind group 1 is the mesh uniform
        //         self.mesh2d_pipeline.mesh_layout.clone(),
        //         // texture
        //         self.material_layout.clone(),
        //     ]),
        //     primitive: PrimitiveState {
        //         front_face: FrontFace::Ccw,
        //         cull_mode: Some(Face::Back),
        //         unclipped_depth: false,
        //         polygon_mode: PolygonMode::Fill,
        //         conservative: false,
        //         topology: key.primitive_topology(),
        //         strip_index_format: None,
        //     },
        //     depth_stencil: None,
        //     multisample: MultisampleState {
        //         count: key.msaa_samples(),
        //         mask: !0,
        //         alpha_to_coverage_enabled: false,
        //     },
        //     label: Some("colored_mesh2d_pipeline".into()),
        // }

        Ok(())
    }
}
