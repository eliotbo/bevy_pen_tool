use super::inputs::Action;
use crate::util::*;
use bevy::prelude::*;
use bevy::render::{
    mesh::Indices,
    pipeline::{PipelineDescriptor, PrimitiveTopology, RenderPipeline},
    shader::{ShaderStage, ShaderStages},
};

use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::Path;
use lyon::tessellation::{FillOptions, FillTessellator, VertexBuffers};

pub fn make_mesh(
    mut action_event_reader: EventReader<Action>,
    mut commands: Commands,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    // mut lut: ResMut<Lut>,
    // mut inds: ResMut<Inds>,
    globals: Res<Globals>,
    groups: Res<Assets<Group>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut maps: ResMut<Maps>,
    query: Query<Entity, With<GroupMesh>>,
) {
    match (action_event_reader.iter().next(), groups.iter().next()) {
        (Some(Action::MakeMesh), Some(group)) => {
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
                let result = tessellator.tessellate_path(
                    &path,
                    &FillOptions::default(),
                    &mut vertex_builder,
                );
                assert!(result.is_ok());
            }

            let mut new_lut = Vec::new();
            let mut mesh_attributes: Vec<[f32; 3]> = Vec::new();
            let mut new_indices: Vec<u32> = Vec::new();
            // show points from look-up table
            let color = globals.picked_color.unwrap();
            let mut colors = Vec::new();
            for position in buffers.vertices[..].iter() {
                // // draw points in standalone_lut
                // let v = Vec3::new(position.x, position.y, -100.0);
                // commands.spawn_bundle(SpriteBundle {
                //     material: materials.add(Color::rgb(0.9, 0.5, 0.8).into()),
                //     transform: Transform::from_translation(v),
                //     sprite: Sprite::new(Vec2::new(1.2, 1.2)),

                //     ..Default::default()
                // });

                new_lut.push(Vec2::new(position.x, position.y));
                mesh_attributes.push([position.x, position.y, 0.0]);
                colors.push([color.r(), color.g(), color.b()]);
            }

            for ind in buffers.indices[..].iter().rev() {
                new_indices.push(ind.clone() as u32);
            }

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, mesh_attributes);

            mesh.set_attribute("Vertex_Color", colors);
            mesh.set_indices(Some(Indices::U32(new_indices)));

            let mesh_handle = meshes.add(mesh);
            // maps.group_meshes.insert("group_mesh", mesh_handle.clone());

            let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
                vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
                fragment: Some(
                    shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER)),
                ),
            }));

            commands
                .spawn_bundle(MeshBundle {
                    mesh: mesh_handle,
                    render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        pipeline_handle,
                    )]),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, -12000.0)),
                    ..Default::default()
                })
                .insert(GroupMesh(color));
        }

        _ => {}
    }
}

const VERTEX_SHADER: &str = r"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Color;
layout(location = 1) out vec3 v_Color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    v_Color = Vertex_Color;
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
";

const FRAGMENT_SHADER: &str = r"
#version 450
layout(location = 1) in vec3 v_Color;
layout(location = 0) out vec4 o_Target;
void main() {
    o_Target = vec4(v_Color, 1.0);
}
";
