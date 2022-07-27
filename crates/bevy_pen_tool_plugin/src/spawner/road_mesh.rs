use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Component, Default)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f95a4b"]
pub struct RoadMesh2dMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub road_texture: Handle<Image>,
}

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
}
