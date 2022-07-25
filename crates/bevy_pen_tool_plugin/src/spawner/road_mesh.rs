use bevy::asset::HandleId;
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState, RenderPipelineDescriptor,
            SamplerBindingType, ShaderStages, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, TextureSampleType, TextureViewDimension,
            VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::VisibleEntities,
        Extract, MainWorld, RenderApp, RenderStage,
    },
    sprite::{
        DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform,
        SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
    utils::FloatOrd,
};

use std::collections::HashMap;

/// A marker component for colored 2d meshes
#[derive(Component, Default)]
pub struct RoadMesh2d;

/// Custom pipeline for 2d meshes with vertex colors
pub struct RoadMesh2dPipeline {
    /// this pipeline wraps the standard [`Mesh2dPipeline`]
    mesh2d_pipeline: Mesh2dPipeline,
    material_layout: BindGroupLayout,
}

impl FromWorld for RoadMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh2d_pipeline = Mesh2dPipeline::from_world(world).clone();
        let world_cell = world.cell();
        let render_device = world_cell.get_resource::<RenderDevice>().unwrap();
        let material_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("sprite_material_layout"),
        });

        Self {
            mesh2d_pipeline,
            material_layout,
        }
    }
}

// We implement `SpecializedPipeline` to customize the default rendering from `Mesh2dPipeline`
impl SpecializedRenderPipeline for RoadMesh2dPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let vertex_attributes = vec![
            // Position (GOTCHA! Vertex_Position isn't first in the buffer due to how Mesh sorts attributes (alphabetically))
            VertexAttribute {
                format: VertexFormat::Float32x3,
                // this offset is the size of the color attribute, which is stored first
                offset: 16,
                // position is available at location 0 in the shader
                shader_location: 0,
            },
            // Color
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 1,
            },
            // uv
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 28,
                shader_location: 2,
            },
        ];
        // This is the sum of the size of position, color uv attributes (12 + 16 + 8 = 36)
        let vertex_array_stride = 36;

        RenderPipelineDescriptor {
            vertex: VertexState {
                // Use our custom shader
                shader: COLORED_MESH2D_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                // Use our custom vertex buffer
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                // Use our custom shader
                shader: COLORED_MESH2D_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            // Use the two standard uniforms for 2d meshes
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is the mesh uniform
                self.mesh2d_pipeline.mesh_layout.clone(),
                // texture
                self.material_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("colored_mesh2d_pipeline".into()),
        }
    }
}

// This specifies how to render a colored 2d mesh
type DrawRoadMesh2d = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Set the material uniform as bind group 2
    SetSpriteTextureBindGroup<2>,
    // Draw the mesh
    DrawMesh2d,
);

/// Plugin that renders [`RoadMesh2d`]s
pub struct RoadMesh2dPlugin;

/// Handle to the custom shader with a unique random ID
pub const COLORED_MESH2D_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13828845414312094821);

pub struct ShaderHandle {
    maybe_handle: Option<Handle<Shader>>,
}

// impl Plugin for SpritePlugin {
//     fn build(&self, app: &mut App) {
//         let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
//         let sprite_shader = Shader::from_wgsl(include_str!("render/sprite.wgsl"));
//         shaders.set_untracked(SPRITE_SHADER_HANDLE, sprite_shader);
//         app.add_asset::<TextureAtlas>()
//             .register_type::<Sprite>()
//             .add_plugin(Mesh2dRenderPlugin)
//             .add_plugin(ColorMaterialPlugin);
//         let render_app = app.sub_app_mut(RenderApp);
//         render_app
//             // .init_resource::<ImageBindGroups>()
//             // .init_resource::<SpritePipeline>()
//             // .init_resource::<SpecializedPipelines<SpritePipeline>>()
//             .init_resource::<SpriteMeta>()
//             .init_resource::<ExtractedSprites>()
//             // .init_resource::<SpriteAssetEvents>()
//             // .add_render_command::<Transparent2d, DrawSprite>()
//             .add_system_to_stage(
//                 RenderStage::Extract,
//                 render::extract_sprites.label(SpriteSystem::ExtractSprites),
//             )
//             // .add_system_to_stage(RenderStage::Extract, render::extract_texture_events)
//             .add_system_to_stage(RenderStage::Queue, queue_sprites);
//     }
// }

impl Plugin for RoadMesh2dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShaderHandle { maybe_handle: None })
            .add_startup_system(setup_shader)
            .add_system(finalize_setup_shader);

        // Register our custom draw function and pipeline, and add our render systems
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .add_render_command::<Transparent2d, DrawRoadMesh2d>()
            .init_resource::<ImageBindGroups>()
            .init_resource::<RoadMesh2dPipeline>()
            .init_resource::<SpriteAssetEvents>()
            .init_resource::<SpecializedRenderPipelines<RoadMesh2dPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_texture_events)
            .add_system_to_stage(RenderStage::Extract, extract_colored_mesh2d)
            .add_system_to_stage(RenderStage::Queue, queue_colored_mesh2d);
    }
}

pub fn setup_shader(asset_server: Res<AssetServer>, mut shader_handle: ResMut<ShaderHandle>) {
    let shader: Handle<Shader> = asset_server.load("shaders/road_mesh.wgsl");
    shader_handle.maybe_handle = Some(shader);

    use std::{thread, time};
    let hundred_millis = time::Duration::from_millis(100);
    thread::sleep(hundred_millis);
}

pub fn finalize_setup_shader(
    mut shaders: ResMut<Assets<Shader>>,
    shader_handle: ResMut<ShaderHandle>,
) {
    let s = shaders
        .get(&shader_handle.maybe_handle.clone().unwrap())
        .unwrap()
        .clone();

    shaders.set_untracked(COLORED_MESH2D_SHADER_HANDLE, s);
}

#[derive(Component)]
pub struct ImageComponent {
    pub image: Handle<Image>,
}

/// Extract the [`RoadMesh2d`] marker component into the render app
pub fn extract_colored_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Extract<Query<(Entity, &Handle<Image>, &ComputedVisibility), With<RoadMesh2d>>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, texture, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible() {
            continue;
        }
        values.push((
            entity,
            (
                RoadMesh2d,
                ImageComponent {
                    image: texture.clone(),
                },
            ),
        ));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

/// Queue the 2d meshes marked with [`RoadMesh2d`] using our custom pipeline and draw function
#[allow(clippy::too_many_arguments)]
pub fn queue_colored_mesh2d(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<RoadMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<RoadMesh2dPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    mut image_bind_groups: ResMut<ImageBindGroups>,
    gpu_images: Res<RenderAssets<Image>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    colored_mesh2d: Query<(&Mesh2dHandle, &ImageComponent, &Mesh2dUniform), With<RoadMesh2d>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
    events: Res<SpriteAssetEvents>,
) {
    // If an image has changed, the GpuImage has (probably) changed
    for event in &events.images {
        match event {
            AssetEvent::Created { .. } => None,
            AssetEvent::Modified { handle } => image_bind_groups.values.remove(handle),
            AssetEvent::Removed { handle } => image_bind_groups.values.remove(handle),
        };
    }

    if colored_mesh2d.is_empty() {
        return;
    }

    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_colored_mesh2d = transparent_draw_functions
            .read()
            .get_id::<DrawRoadMesh2d>()
            .unwrap();

        // let draw_sprite_function = draw_functions.read().get_id::<DrawSprite>().unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, texture, mesh2d_uniform)) =
                colored_mesh2d.get(*visible_entity)
            {
                let new_batch = SpriteBatch {
                    image_handle_id: texture.image.id,
                };

                let _current_batch_entity = commands.spawn_bundle((new_batch,)).id();

                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                if let Some(gpu_image) = gpu_images.get(&Handle::weak(texture.image.id)) {
                    image_bind_groups
                        .values
                        .entry(Handle::weak(texture.image.id))
                        .or_insert_with(|| {
                            render_device.create_bind_group(&BindGroupDescriptor {
                                entries: &[
                                    BindGroupEntry {
                                        binding: 0,
                                        resource: BindingResource::TextureView(
                                            &gpu_image.texture_view,
                                        ),
                                    },
                                    BindGroupEntry {
                                        binding: 1,
                                        resource: BindingResource::Sampler(&gpu_image.sampler),
                                    },
                                ],
                                label: Some("sprite_material_bind_group"),
                                layout: &colored_mesh2d_pipeline.material_layout,
                            })
                        });
                }

                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &colored_mesh2d_pipeline, mesh2d_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    // entity: current_batch_entity,
                    draw_function: draw_colored_mesh2d,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: None,
                });
            }
        }
    }
}

#[derive(Default)]
pub struct SpriteAssetEvents {
    pub images: Vec<AssetEvent<Image>>,
}

// pub fn extract_texture_events(
//     mut render_world: ResMut<MainWorld>,
//     mut image_events: EventReader<AssetEvent<Image>>,
// ) {
//     let mut events = render_world
//         .get_resource_mut::<SpriteAssetEvents>()
//         .unwrap();

//     let SpriteAssetEvents { ref mut images } = *events;
//     images.clear();

//     for image in image_events.iter() {
//         // AssetEvent: !Clone

//         images.push(match image {
//             AssetEvent::Created { handle } => AssetEvent::Created {
//                 handle: handle.clone_weak(),
//             },
//             AssetEvent::Modified { handle } => AssetEvent::Modified {
//                 handle: handle.clone_weak(),
//             },
//             AssetEvent::Removed { handle } => AssetEvent::Removed {
//                 handle: handle.clone_weak(),
//             },
//         });
//     }
// }

pub fn extract_texture_events(
    mut render_world: ResMut<MainWorld>,
    mut sprite_asset_event: ResMut<SpriteAssetEvents>,
    mut image_events: EventReader<AssetEvent<Image>>,
) {
    // let mut events = render_world
    //     .get_resource_mut::<SpriteAssetEvents>()
    //     .unwrap();

    // let SpriteAssetEvents { ref mut images } = *events;

    let SpriteAssetEvents { ref mut images } = *sprite_asset_event;
    images.clear();

    for image in image_events.iter() {
        // AssetEvent: !Clone

        images.push(match image {
            AssetEvent::Created { handle } => AssetEvent::Created {
                handle: handle.clone_weak(),
            },
            AssetEvent::Modified { handle } => AssetEvent::Modified {
                handle: handle.clone_weak(),
            },
            AssetEvent::Removed { handle } => AssetEvent::Removed {
                handle: handle.clone_weak(),
            },
        });
    }
}

#[derive(Default)]
pub struct ImageBindGroups {
    values: HashMap<Handle<Image>, BindGroup>,
}

#[derive(Component, Eq, PartialEq, Copy, Clone)]
pub struct SpriteBatch {
    image_handle_id: HandleId,
}

pub struct SetSpriteTextureBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetSpriteTextureBindGroup<I> {
    type Param = (SRes<ImageBindGroups>, SQuery<Read<SpriteBatch>>);

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        (image_bind_groups, query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        //
        // TODO: handle this query safely
        let sprite_batch = query_batch.single();
        let image_bind_groups = image_bind_groups.into_inner();

        pass.set_bind_group(
            I,
            image_bind_groups
                .values
                .get(&Handle::weak(sprite_batch.image_handle_id))
                .unwrap(),
            &[],
        );
        RenderCommandResult::Success
    }
}
