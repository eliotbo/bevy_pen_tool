use crate::inputs::Action;
use crate::spawner::RoadMesh2d;
use crate::util::*;
// use bevy::reflect::TypeUuid;
// use bevy::render::mesh::Indices;
// use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        render_asset::RenderAssets,
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, SpecializedRenderPipeline, SpecializedRenderPipelines,
            TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
            VertexStepMode,
        },
        texture::BevyDefault,
        view::VisibleEntities,
        MainWorld, RenderApp, RenderStage,
    },
    sprite::{
        DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform,
        SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
    utils::FloatOrd,
};

use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::Path;
use lyon::tessellation::{FillOptions, FillTessellator, VertexBuffers};

pub fn make_road(
    mut action_event_reader: EventReader<Action>,
    mut commands: Commands,
    curves: ResMut<Assets<Bezier>>,
    globals: Res<Globals>,
    groups: Res<Assets<Group>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<Entity, With<RoadMesh>>,

    // mut materials: ResMut<Assets<StandardMaterial>>,
    // asset_server: Res<AssetServer>,
    maps: ResMut<Maps>,
) {
    if action_event_reader.iter().any(|x| x == &Action::SpawnRoad) {
        if groups.iter().next() == None {
            return ();
        }

        let group = groups.iter().next().unwrap().1;

        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        let num_points = 204;

        let crop = 0.000001;
        let t_range: Vec<f32> = (0..num_points)
            .map(|x| (x as f32) / (num_points as f32 - 0.99999) / (1.0 + 2.0 * crop) + crop)
            .collect();

        let mut mesh_contour: Vec<Vec3> = Vec::new();

        for t in t_range {
            let position = group.compute_position_with_lut(t);
            let normal = group
                .compute_normal_with_bezier(&curves, t as f64)
                .normalize();

            let v1 = Vec3::new(
                (position.x + normal.x * globals.road_width) as f32,
                (position.y + normal.y * globals.road_width) as f32,
                -88.0,
            );

            let v2 = Vec3::new(
                (position.x - normal.x * globals.road_width) as f32,
                (position.y - normal.y * globals.road_width) as f32,
                -88.0,
            );

            mesh_contour.push(v1);
            mesh_contour.push(v2);
        }

        // indices
        let mut new_indices: Vec<u32> = Vec::new();
        for kk in 0..(num_points - 1) {
            let k = kk * 2;
            let mut local_inds = vec![k, (k + 1), (k + 2), (k + 1), (k + 3), (k + 2)];
            new_indices.append(&mut local_inds);
        }

        // uvs
        let path_length = group.standalone_lut.path_length;
        let num_repeats = path_length / 100.0;
        let mut mesh_attr_uvs: Vec<[f32; 2]> = Vec::new();
        for k in 0..num_points * 2 {
            // let (pos_x, pos_y) = (pos[0], pos[1]);
            let v = k as f32 / (num_points as f32 / num_repeats);
            mesh_attr_uvs.push([v % 1.0, (k as f32) % 2.0]);
        }

        let mut mesh_pos_attributes: Vec<[f32; 3]> = Vec::new();
        let mut mesh_attr_normals: Vec<[f32; 3]> = Vec::new();

        // show points from look-up table
        let color = globals.picked_color.unwrap();
        let mut colors = Vec::new();

        for position in mesh_contour {
            // mesh_pos_attributes.push([position.x, position.y, 0.0]);
            mesh_pos_attributes.push([position.x, position.y, 0.0]);
            mesh_attr_normals.push([0.0, 0.0, 1.0]);

            colors.push([color.r(), color.g(), color.b(), 1.0]);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_pos_attributes.clone());

        // mesh.set_attribute("Vertex_Color", colors);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

        // mesh.set_attribute("Vertex_Normal", mesh_attr_normals);
        // mesh.set_attribute("Vertex_Uv", mesh_attr_uvs);

        mesh.set_indices(Some(Indices::U32(new_indices)));

        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_attr_uvs);

        // use std::{thread, time};
        // let texture_handle: Handle<Texture> = asset_server.load("textures/road_texture.png");

        // let material_handle = materials.add(StandardMaterial {
        //     base_color_texture: Some(texture_handle),
        //     reflectance: 0.02,
        //     unlit: false,
        //     ..Default::default()
        // });

        // println!("material_handle: {:?}", "yaaaaa");

        // TODO: add texture to road
        // let texture_handle: Handle<Image> = asset_server.load("textures/single_lane_road.png");

        // let hundred_millis = time::Duration::from_millis(200);
        // thread::sleep(hundred_millis);

        let texture_handle = maps.textures.get("single_lane_road").unwrap();

        commands.spawn_bundle((
            RoadMesh2d::default(),
            Mesh2dHandle(meshes.add(mesh)),
            // Transform::default(),
            GlobalTransform::default(),
            Transform::from_translation(Vec3::new(0.0, 0.0, 310.0)),
            Visibility::default(),
            ComputedVisibility::default(),
            texture_handle.clone(),
        ));
    }
}

pub fn make_mesh(
    mut action_event_reader: EventReader<Action>,
    mut commands: Commands,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    globals: Res<Globals>,
    groups: Res<Assets<Group>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<Entity, With<GroupMesh>>,
) {
    if action_event_reader.iter().any(|x| x == &Action::MakeMesh) {
        if groups.iter().next() == None {
            return ();
        }
        let group = groups.iter().next().unwrap();

        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        let mut path_builder = Path::builder();

        let lut = group.1.standalone_lut.lut.clone();

        let first = lut[0];
        path_builder.begin(point(first.x, first.y));

        let resto: Vec<Vec2> = lut[1..].to_vec();

        for e in resto.iter() {
            path_builder.line_to(point(e.x, e.y));
        }

        path_builder.end(true);
        let path = path_builder.build();

        // Create the destination vertex and index buffers.
        let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

        {
            let mut vertex_builder = simple_builder(&mut buffers);

            // Create the tessellator.
            let mut tessellator = FillTessellator::new();

            // Compute the tessellation.
            let result =
                tessellator.tessellate_path(&path, &FillOptions::default(), &mut vertex_builder);
            assert!(result.is_ok());
        }

        let mut mesh_pos_attributes: Vec<[f32; 3]> = Vec::new();
        let mut mesh_attr_uvs: Vec<[f32; 2]> = Vec::new();
        let mut new_indices: Vec<u32> = Vec::new();

        // show points from look-up table
        let color = globals.picked_color.unwrap();
        let mut colors = Vec::new();

        for position in buffers.vertices[..].iter() {
            mesh_pos_attributes.push([position.x, position.y, 0.0]);

            colors.push([color.r(), color.g(), color.b(), 1.0]);
        }

        //////////////////////////// uvs ///////////////////////////////
        let xs: Vec<f32> = mesh_pos_attributes.iter().map(|v| v[0]).collect();
        let ys: Vec<f32> = mesh_pos_attributes.iter().map(|v| v[1]).collect();

        use std::cmp::Ordering;

        fn bounds(v: &Vec<f32>) -> (f32, f32) {
            let max_v: &f32 = v
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                .unwrap();

            let min_v: &f32 = v
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                .unwrap();

            return (*min_v, *max_v);
        }

        let bounds_x = bounds(&xs);
        let size_x = bounds_x.1 - bounds_x.0;
        let bounds_y = bounds(&ys);
        let size_y = bounds_y.1 - bounds_y.0;

        for pos in &mesh_pos_attributes {
            let (pos_x, pos_y) = (pos[0], pos[1]);

            mesh_attr_uvs.push([
                1.0 * (pos_x - bounds_x.0) / size_x,
                1.0 * (pos_y - bounds_y.0) / size_y,
            ]);
        }

        for ind in buffers.indices[..].iter().rev() {
            new_indices.push(ind.clone() as u32);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_pos_attributes.clone());

        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

        mesh.set_indices(Some(Indices::U32(new_indices)));

        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_attr_uvs);

        // We can now spawn the entities for the star and the camera
        commands.spawn_bundle((
            ColoredMesh2d::default(),
            Mesh2dHandle(meshes.add(mesh)),
            // Transform::default(),
            GlobalTransform::default(),
            Transform::from_translation(Vec3::new(0.0, 0.0, 309.99)),
            Visibility::default(),
            ComputedVisibility::default(),
        ));
    }
}

/// A marker component for colored 2d meshes
#[derive(Component, Default)]
pub struct ColoredMesh2d;

/// Custom pipeline for 2d meshes with vertex colors
pub struct ColoredMesh2dPipeline {
    /// this pipeline wraps the standard [`Mesh2dPipeline`]
    mesh2d_pipeline: Mesh2dPipeline,
    // material_layout: BindGroupLayout,
}

impl FromWorld for ColoredMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh2d_pipeline = Mesh2dPipeline::from_world(world).clone();
        Self { mesh2d_pipeline }
    }
}

// We implement `SpecializedPipeline` to customize the default rendering from `Mesh2dPipeline`
impl SpecializedRenderPipeline for ColoredMesh2dPipeline {
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
                // self.material_layout.clone(),
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
type DrawColoredMesh2d = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Draw the mesh
    DrawMesh2d,
);

/// Plugin that renders [`ColoredMesh2d`]s
pub struct ColoredMesh2dPlugin;

/// Handle to the custom shader with a unique random ID
pub const COLORED_MESH2D_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13828845428412094821);

pub struct ShaderHandle {
    maybe_handle: Option<Handle<Shader>>,
}

impl Plugin for ColoredMesh2dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShaderHandle { maybe_handle: None })
            .add_startup_system(setup_shader)
            .add_system(finalize_setup_shader);

        // Register our custom draw function and pipeline, and add our render systems
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .add_render_command::<Transparent2d, DrawColoredMesh2d>()
            .init_resource::<ColoredMesh2dPipeline>()
            .init_resource::<SpecializedRenderPipelines<ColoredMesh2dPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_colored_mesh2d)
            .add_system_to_stage(RenderStage::Queue, queue_colored_mesh2d);
    }
}

pub fn setup_shader(asset_server: Res<AssetServer>, mut shader_handle: ResMut<ShaderHandle>) {
    let shader: Handle<Shader> = asset_server.load("shaders/fill_mesh.wgsl");
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

// TODO!
/// Extract the [`ColoredMesh2d`] marker component into the render app
pub fn extract_colored_mesh2d(
    mut render_world: ResMut<MainWorld>,
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &ComputedVisibility), With<ColoredMesh2d>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible() {
            continue;
        }
        values.push((entity, (ColoredMesh2d,)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

/// Queue the 2d meshes marked with [`ColoredMesh2d`] using our custom pipeline and draw function
#[allow(clippy::too_many_arguments)]
pub fn queue_colored_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<ColoredMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<ColoredMesh2dPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    colored_mesh2d: Query<(&Mesh2dHandle, &Mesh2dUniform), With<ColoredMesh2d>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if colored_mesh2d.is_empty() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_colored_mesh2d = transparent_draw_functions
            .read()
            .get_id::<DrawColoredMesh2d>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) = colored_mesh2d.get(*visible_entity) {
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &colored_mesh2d_pipeline, mesh2d_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
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
