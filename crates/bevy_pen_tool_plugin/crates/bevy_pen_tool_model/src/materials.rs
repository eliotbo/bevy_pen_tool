use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::*, sprite::Material2d};

#[macro_export]
macro_rules! make_mat {
    ($( $name_of_mat:ident, $gpu_name_of_mat:ident, $vert_shader:expr, $frag_shader:expr, $uuid:expr ),*) => {

        $(
            #[derive(TypeUuid, Debug, Clone, AsBindGroup)]
            #[uuid = $uuid]
            #[allow(non_snake_case)]
            pub struct $name_of_mat {
                #[uniform(0)]
                pub color: Vec4,
                #[uniform(0)]
                pub clearcolor: Vec4,
                #[uniform(0)]
                pub t: f32, // Bezier t-value for MiddleQuads, but is used for other purposes elsewhere
                #[uniform(0)]
                pub zoom: f32,
                #[uniform(0)]
                pub size: Vec2,
                #[uniform(0)]
                pub hovered: f32,
            }

            impl Default for $name_of_mat {
                fn default() -> Self {

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

            impl Material2d for $name_of_mat {
                fn fragment_shader() -> ShaderRef {
                    $frag_shader.into()
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
