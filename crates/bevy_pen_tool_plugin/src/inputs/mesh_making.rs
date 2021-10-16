use super::inputs::Action;
use crate::util::*;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::{
    mesh::Indices,
    pipeline::{PipelineDescriptor, PrimitiveTopology, RenderPipeline},
    render_graph::{base, AssetRenderResourcesNode, RenderGraph},
    renderer::RenderResources,
    shader::{ShaderStage, ShaderStages},
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
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    query: Query<Entity, With<RoadMesh>>,

    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if action_event_reader.iter().any(|x| x == &Action::SpawnRoad) {
        if groups.iter().next() == None {
            return ();
        }
        let group = groups.iter().next().unwrap().1;

        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        let num_points = 200;

        let crop = 0.000001;
        let t_range: Vec<f32> = (0..num_points)
            .map(|x| (x as f32) / (num_points as f32 - 0.99999) / (1.0 + 2.0 * crop) + crop)
            .collect();
        let mut mesh_contour1: Vec<Vec3> = Vec::new();
        let mut mesh_contour2: Vec<Vec3> = Vec::new();
        let mut mesh_contour: Vec<Vec3> = Vec::new();

        let mut k: u32 = 0;
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
            // mesh_contour1.push(v1);
            //
            let v2 = Vec3::new(
                (position.x - normal.x * globals.road_width) as f32,
                (position.y - normal.y * globals.road_width) as f32,
                -88.0,
            );
            // mesh_contour2.push(v2);

            mesh_contour.push(v1);
            mesh_contour.push(v2);

            k += 1;

            // println!("t: {:?}", &t);
            // println!("contour1: {:?}", &v1);
            // println!("contour2: {:?}", &v2);
        }

        // indices
        let mut new_indices: Vec<u32> = Vec::new();
        for kk in 0..(num_points) {
            let k = kk * 2;
            let mut local_inds = vec![k, (k + 1), (k + 2), (k + 1), (k + 3), (k + 2)];
            new_indices.append(&mut local_inds);
        }
        // println!("indices len: {:?}", &new_indices.len());

        // uvs
        let path_length = group.standalone_lut.path_length;
        let num_repeats = path_length / 100.0;
        let mut mesh_attr_uvs: Vec<[f32; 2]> = Vec::new();
        for k in 0..num_points * 2 {
            // let (pos_x, pos_y) = (pos[0], pos[1]);
            let v = k as f32 / (num_points as f32 / num_repeats);
            mesh_attr_uvs.push([v % 1.0, (k as f32) % 2.0]);
        }

        // mesh_contour2 = mesh_contour2.iter().rev().cloned().collect();
        // mesh_contour1.append(&mut mesh_contour2);

        // let mut path_builder = Path::builder();

        // let first = mesh_contour[0];
        // path_builder.begin(point(first.x, first.y));

        // // let resto: Vec<Vec3> = mesh_contour1[1..].to_vec();
        // let resto: Vec<Vec3> = mesh_contour[1..].to_vec();

        // for e in resto.iter() {
        //     path_builder.line_to(point(e.x, e.y));
        // }

        // path_builder.end(true);
        // let path = path_builder.build();

        // // Create the destination vertex and index buffers.
        // let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

        // {
        //     let mut vertex_builder = simple_builder(&mut buffers);

        //     // Create the tessellator.
        //     let mut tessellator = FillTessellator::new();

        //     // Compute the tessellation.
        //     let result =
        //         tessellator.tessellate_path(&path, &FillOptions::default(), &mut vertex_builder);
        //     assert!(result.is_ok());
        // }

        let mut mesh_pos_attributes: Vec<[f32; 3]> = Vec::new();
        let mut mesh_attr_normals: Vec<[f32; 3]> = Vec::new();

        // let mut new_indices: Vec<u32> = Vec::new();
        // show points from look-up table
        let color = globals.picked_color.unwrap();
        let mut colors = Vec::new();

        for position in mesh_contour {
            // mesh_pos_attributes.push([position.x, position.y, 0.0]);
            mesh_pos_attributes.push([position.x, position.y, 0.0]);
            mesh_attr_normals.push([0.0, 0.0, 1.0]);

            colors.push([color.r(), color.g(), color.b()]);
        }

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

        // let mut bounds_x = bounds(&xs);
        // let size_x = bounds_x.1 - bounds_x.0;
        // let mut bounds_y = bounds(&ys);
        // let size_y = bounds_y.1 - bounds_y.0;

        // let half_len = mesh_pos_attributes.len() / 2;

        // let first_half: Vec<[f32; 3]> = mesh_pos_attributes[0..(p_len / 2)].to_vec();
        // let second_half: Vec<[f32; 3]> = mesh_pos_attributes[(p_len / 2)..p_len].to_vec();

        // let num_repeats = 1.0;

        // for k in 0..half_len {
        //     // let (pos_x, pos_y) = (pos[0], pos[1]);
        //     let v = k as f32 / (half_len as f32 / num_repeats);
        //     mesh_attr_uvs.push([v % 1.0, 0.0]);
        // }

        // for k in (0..half_len).rev() {
        //     // let (pos_x, pos_y) = (pos[0], pos[1]);
        //     let v = k as f32 / (half_len as f32 / num_repeats);
        //     mesh_attr_uvs.push([v % 1.0, 1.0]);
        // }

        // for ind in buffers.indices[..].iter().rev() {
        //     new_indices.push(ind.clone() as u32);
        // }

        // println!("POSITIONS: {:?}", &mesh_pos_attributes);
        // println!("UVs: {:?}", &mesh_attr_uvs);
        // println!("indices: {:?}", &new_indices);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, mesh_pos_attributes.clone());

        mesh.set_attribute("Vertex_Color", colors);
        mesh.set_attribute("Vertex_Normal", mesh_attr_normals);
        mesh.set_attribute("Vertex_Uv", mesh_attr_uvs);

        mesh.set_indices(Some(Indices::U32(new_indices)));

        let mesh_handle = meshes.add(mesh);
        // maps.group_meshes.insert("group_mesh", mesh_handle.clone());

        use std::{thread, time};
        let texture_handle: Handle<Texture> = asset_server.load("textures/road_texture.png");
        let hundred_millis = time::Duration::from_millis(100);
        thread::sleep(hundred_millis);

        // let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        //     vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        //     fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
        // }));

        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            reflectance: 0.02,
            unlit: false,
            ..Default::default()
        });

        commands
            .spawn_bundle(PbrBundle {
                mesh: mesh_handle,
                // render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                //     pipeline_handle,
                // )]),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -120.0)),
                material: material_handle,
                ..Default::default()
            })
            .insert(RoadMesh(color));

        // light
        commands.spawn_bundle(PointLightBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 50.0, -75.0)),
            point_light: PointLight {
                intensity: 50000.,
                range: 1000.,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}

pub fn make_mesh(
    mut action_event_reader: EventReader<Action>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    globals: Res<Globals>,
    groups: Res<Assets<Group>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    query: Query<Entity, With<GroupMesh>>,

    asset_server: Res<AssetServer>,
    // mut render_graph: ResMut<RenderGraph>,
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
        let mut mesh_attr_normals: Vec<[f32; 3]> = Vec::new();
        let mut mesh_attr_uvs: Vec<[f32; 2]> = Vec::new();

        let mut new_indices: Vec<u32> = Vec::new();
        // show points from look-up table
        let color = globals.picked_color.unwrap();
        let mut colors = Vec::new();

        for position in buffers.vertices[..].iter() {
            mesh_pos_attributes.push([position.x, position.y, 0.0]);
            mesh_attr_normals.push([0.0, 0.0, 1.0]);

            colors.push([color.r(), color.g(), color.b()]);
        }

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

        let mut bounds_x = bounds(&xs);
        let size_x = bounds_x.1 - bounds_x.0;
        let mut bounds_y = bounds(&ys);
        let size_y = bounds_y.1 - bounds_y.0;

        for pos in &mesh_pos_attributes {
            let (pos_x, pos_y) = (pos[0], pos[1]);

            mesh_attr_uvs.push([
                1.0 * (pos_x - bounds_x.0) / size_x,
                1.0 * (pos_y - bounds_y.0) / size_y,
            ]);

            // mesh_attr_uvs.push([
            //     1.0 * (pos_x - bounds_x.0 * 0.0),
            //     1.0 * (pos_y - bounds_y.0 * 0.0),
            // ]);
        }

        for ind in buffers.indices[..].iter().rev() {
            new_indices.push(ind.clone() as u32);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, mesh_pos_attributes.clone());

        mesh.set_attribute("Vertex_Color", colors);
        mesh.set_attribute("Vertex_Normal", mesh_attr_normals);
        mesh.set_attribute("Vertex_Uv", mesh_attr_uvs);

        mesh.set_indices(Some(Indices::U32(new_indices)));

        let mesh_handle = meshes.add(mesh);
        // maps.group_meshes.insert("group_mesh", mesh_handle.clone());

        use std::{thread, time};
        let texture_handle: Handle<Texture> = asset_server.load("textures/road_texture.png");
        let hundred_millis = time::Duration::from_millis(100);
        thread::sleep(hundred_millis);

        // let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        //     vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        //     fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
        // }));

        let material_handle = materials.add(StandardMaterial {
            // base_color_texture: Some(texture_handle),
            reflectance: 0.02,
            base_color: color,
            unlit: false,
            ..Default::default()
        });

        commands
            .spawn_bundle(PbrBundle {
                mesh: mesh_handle,
                // render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                //     pipeline_handle,
                // )]),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -150.0)),
                material: material_handle,
                ..Default::default()
            })
            .insert(GroupMesh(color));
    }
}

// struct MyPipeline(Handle<PipelineDescriptor>);

#[derive(Component, RenderResources, Default, TypeUuid)]
#[uuid = "93fb26fc-6c05-489b-9029-601edf703b6b"]
pub struct MyArrayTexture {
    pub texture: Handle<Texture>,
}

// const VERTEX_SHADER: &str = r"
// #version 450
// layout(location = 0) in vec3 Vertex_Position;
// layout(location = 1) in vec3 Vertex_Color;
// layout(location = 0) out vec4 v_Position;
// layout(location = 1) out vec3 v_Color;
// layout(set = 0, binding = 0) uniform CameraViewProj {
//     mat4 ViewProj;
// };
// layout(set = 1, binding = 0) uniform Transform {
//     mat4 Model;
// };
// void main() {
//     v_Color = Vertex_Color;
//     v_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
//     gl_Position = v_Position;
// }
// ";

// const FRAGMENT_SHADER: &str = r"
// #version 450
// layout(location = 0) in vec4 v_Position;
// layout(location = 1) in vec3 v_Color;
// layout(location = 0) out vec4 o_Target;

// layout(set = 2, binding = 0) uniform texture2DArray MyArrayTexture_texture;
// layout(set = 2, binding = 1) uniform sampler MyArrayTexture_texture_sampler;

// void main() {
//     vec2 ss = v_Position.xy / v_Position.w;
//     // o_Target = vec4(v_Color.r, v_Color.g, v_Color.b, 1.0);
//     vec2 uv = (ss + vec2(1.0)) / 2.0;
//     // o_Target = texture(sampler2DArray(MyArrayTexture_texture, MyArrayTexture_texture_sampler), vec3(uv, 0));

//     o_Target =  vec4(v_Color, 0.5);
// }
// ";
