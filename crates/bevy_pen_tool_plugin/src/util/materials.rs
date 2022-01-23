use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{
            std140::{AsStd140, Std140},
            *,
        },
        renderer::RenderDevice,
    },
    sprite::Material2d,
    sprite::{Material2dPipeline, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::util::*;

// /////////////////////////////////  UiMat //////////////////////////////////////////

// #[derive(TypeUuid, Debug, Clone, AsStd140)]
// #[uuid = "1e08866c-0b8a-437e-8bce-37733b21137e"]
// #[allow(non_snake_case)]
// pub struct UiMat {
//     pub color: Vec4,
//     pub clearcolor: Vec4,
//     pub t: f32, // Bezier t-value for MiddleQuads, but is used for other purposes elsewhere
//     pub zoom: f32,
//     pub size: Vec2,
//     pub hovered: f32,
// }

// impl Default for UiMat {
//     fn default() -> Self {
//         Self {
//             color: Color::hex("F87575").unwrap().into(),
//             t: 0.5,
//             zoom: 0.15,
//             size: Vec2::new(1.0, 1.0),
//             clearcolor: Color::hex("6e7f80").unwrap().into(),
//             hovered: 0.0,
//         }
//     }
// }

// #[derive(Clone)]
// pub struct GpuUiMat {
//     _buffer: Buffer,
//     bind_group: BindGroup,
// }

// impl Material2d for UiMat {
//     fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
//         Some(asset_server.load("shaders/ui.frag"))
//     }

//     fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
//         Some(asset_server.load("shaders/bezier.vert"))
//     }

//     fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
//         &render_asset.bind_group
//     }

//     fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
//         render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
//             entries: &[BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: ShaderStages::FRAGMENT,
//                 ty: BindingType::Buffer {
//                     ty: BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: BufferSize::new(UiMat::std140_size_static() as u64),
//                 },
//                 count: None,
//             }],
//             label: None,
//         })
//     }
// }

// impl RenderAsset for UiMat {
//     type ExtractedAsset = UiMat;
//     type PreparedAsset = GpuUiMat;
//     type Param = (SRes<RenderDevice>, SRes<Material2dPipeline<Self>>);
//     fn extract_asset(&self) -> Self::ExtractedAsset {
//         self.clone()
//     }

//     fn prepare_asset(
//         extracted_asset: Self::ExtractedAsset,
//         (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
//     ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
//         let custom_material_std140 = extracted_asset.as_std140();
//         let custom_material_bytes = custom_material_std140.as_bytes();

//         let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
//             contents: custom_material_bytes,
//             label: None,
//             usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
//         });
//         let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
//             entries: &[BindGroupEntry {
//                 binding: 0,
//                 resource: buffer.as_entire_binding(),
//             }],
//             label: None,
//             layout: &material_pipeline.material2d_layout,
//         });

//         Ok(GpuUiMat {
//             _buffer: buffer,
//             bind_group,
//         })
//     }
// }

// /////////////////////////////////  UiMat //////////////////////////////////////////

#[macro_export]
macro_rules! make_mat {
    // ($($value:expr, $ty_of_val:ty, ),*) => {{
    ($( $name_of_mat:ident, $gpu_name_of_mat:ident, $vert_shader:expr, $frag_shader:expr, $uuid:expr ),*) => {

        $(
            #[derive(TypeUuid, Debug, Clone, AsStd140)]
            #[uuid = $uuid]
            #[allow(non_snake_case)]
            pub struct $name_of_mat {
                pub color: Vec4,
                pub clearcolor: Vec4,
                pub t: f32, // Bezier t-value for MiddleQuads, but is used for other purposes elsewhere
                pub zoom: f32,
                pub size: Vec2,
                pub hovered: f32,
            }

            impl Default for $name_of_mat {
                fn default() -> Self {

                    // println!("color: {:?}", Color::hex("6e7f80"));
                    Self {
                        color: Color::hex("F87575").unwrap().into(),
                        t: 0.5,
                        zoom: 0.15,
                        size: Vec2::new(1.0, 1.0),
                        clearcolor: Color::hex("6e7f80").unwrap().into(),
                        hovered: 0.0,
                    }
                }
            }


            #[derive(Clone)]
            pub struct $gpu_name_of_mat {
                _buffer: Buffer,
                bind_group: BindGroup,
            }

            impl Material2d for $name_of_mat {
                fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
                    Some(asset_server.load($frag_shader))
                }

                // fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
                //     Some(asset_server.load($frag_shader))
                // }

                fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
                    &render_asset.bind_group
                }

                fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
                    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        entries: &[BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: BufferSize::new($name_of_mat::std140_size_static() as u64),
                            },
                            count: None,
                        }],
                        label: None,
                    })
                }
            }




            impl RenderAsset for $name_of_mat {
                type ExtractedAsset = $name_of_mat;
                type PreparedAsset = $gpu_name_of_mat;
                type Param = (SRes<RenderDevice>, SRes<Material2dPipeline<Self>>);
                fn extract_asset(&self) -> Self::ExtractedAsset {
                    self.clone()
                }

                fn prepare_asset(
                    extracted_asset: Self::ExtractedAsset,
                    (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
                ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
                    let custom_material_std140 = extracted_asset.as_std140();
                    let custom_material_bytes = custom_material_std140.as_bytes();

                    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                        contents: custom_material_bytes,
                        label: None,
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    });
                    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                        entries: &[BindGroupEntry {
                            binding: 0,
                            resource: buffer.as_entire_binding(),
                        }],
                        label: None,
                        layout: &material_pipeline.material2d_layout,
                    });

                    Ok($gpu_name_of_mat {
                        _buffer: buffer,
                        bind_group,
                    })
                }
            }

        )*
    }
}

make_mat![
    UiMat,
    GpuUiMat,
    "shaders/bezier.vert",
    "shaders/ui.wgsl",
    "6cf5ad10-8906-45d9-b29b-eba9ec6c0de8"
];

// make_mat![
//     BezierMat,
//     GpuBezier,
//     "shaders/bezier.vert",
//     "shaders/bezier.frag",
//     "2e08866c-0b8a-437e-8bce-37733b21137e"
// ];

make_mat![
    SelectionMat,
    GpuSelectionMat,
    "shaders/bezier.vert",
    "shaders/bounding_box.wgsl",
    "3e08866c-0b8a-437e-8bce-37733b21137e"
];

make_mat![
    SelectingMat,
    GpuSelectingMat,
    "shaders/bezier.vert",
    "shaders/selecting.wgsl",
    "4e08866c-0b8a-437e-8bce-37733b21137e"
];

make_mat![
    ButtonMat,
    GpuButtonMat,
    "shaders/bezier.vert",
    "shaders/button.wgsl",
    "5e08866c-0b8a-437e-8bce-37733b21137e"
];

make_mat![
    BezierEndsMat,
    GpuBezierEnds,
    "shaders/bezier.vert",
    "shaders/ends.wgsl",
    "6e08866c-0b8a-437e-8bce-37733b21137e"
];

make_mat![
    BezierControlsMat,
    GpuBezierControlsMat,
    "shaders/bezier.vert",
    "shaders/controls.wgsl",
    "7e08866c-0b8a-437e-8bce-37733b21137e"
];

make_mat![
    BezierMidMat,
    GpuUiBezierMidMat,
    "shaders/bezier.vert",
    "shaders/mids.wgsl",
    "8e08866c-0b8a-437e-8bce-37733b21137e"
];

make_mat![
    FillMat,
    GpuFillMat,
    "shaders/bezier.vert",
    "shaders/mids.wgsl",
    "9e08866c-0b8a-437e-8bce-37733b21137e"
];
