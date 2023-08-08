use mepeyew::*;
use nalgebra_glm as glm;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

const SPHERE_ROW_COUNT: usize = 9;
const SPHERE_COL_COUNT: usize = 9;

fn main() {
    #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
    wasm::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let window_size = get_window_size(&window);

    //
    //  --- Begin Setup Code ---
    //
    let mut extensions = Extensions::new();
    extensions
        .native_debug(NativeDebugConfiguration::default())
        .naga_translation()
        .surface_extension(SurfaceConfiguration {
            width: window_size.0,
            height: window_size.1,
            display: window.raw_display_handle(),
            window: window.raw_window_handle(),
        })
        .webgpu_init_from_window(WebGpuInitFromWindow {
            adapter: String::from("mepeyewAdapter"),
            device: String::from("mepeyewDevice"),
            canvas_id: Some(String::from("canvas")),
        });

    let mut context = Context::new(extensions, None).unwrap();

    let vs = include_bytes!("shaders/pbr_sphere/vs.vert");
    let fs = include_bytes!("shaders/pbr_sphere/fs.frag");

    let vs = context
        .naga_translate_shader_code(
            naga_translation::NagaTranslationStage::Vertex,
            naga_translation::NagaTranslationInput::Glsl,
            vs,
            naga_translation::NagaTranslationExtensionTranslateShaderCodeExt::default(),
        )
        .unwrap();
    let fs = context
        .naga_translate_shader_code(
            naga_translation::NagaTranslationStage::Fragment,
            naga_translation::NagaTranslationInput::Glsl,
            fs,
            naga_translation::NagaTranslationExtensionTranslateShaderCodeExt::default(),
        )
        .unwrap();

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct SceneData {
        camera_position: glm::Vec3,
        _p: f32,
        light_positions: [glm::Vec4; 4],
        light_colors: [glm::Vec4; 4],
        view: glm::Mat4,
        projection: glm::Mat4,
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct ObjectData {
        albedo: glm::Vec3,
        metallic: f32,
        _p1: glm::Vec3,
        roughness: f32,
        _p2: glm::Vec3,
        ao: f32,
        model: glm::Mat4,
        normal_matrix: glm::Mat4,
    }

    let (scene_ubo, scene_ubo_guard) = context
        .new_uniform_buffer(&SceneData::default(), None)
        .unwrap();

    let (obj_ubo, obj_ubo_guard) = context
        .new_dynamic_uniform_buffer(
            &[ObjectData::default(); SPHERE_ROW_COUNT * SPHERE_COL_COUNT],
            None,
        )
        .unwrap();

    let program = context
        .new_program(
            &ShaderSet::shaders(&[
                (
                    ShaderType::Vertex(VertexBufferInput {
                        args: vec![3, 3, 2],
                    }),
                    &vs,
                ),
                (ShaderType::Fragment, &fs),
            ]),
            &[
                ShaderUniform {
                    set: 0,
                    binding: 0,
                    ty: ShaderUniformType::UniformBuffer(scene_ubo),
                },
                ShaderUniform {
                    set: 1,
                    binding: 0,
                    ty: ShaderUniformType::DynamicUniformBuffer(obj_ubo),
                },
            ],
            Some(NewProgramExt {
                enable_depth_test: Some(()),
                enable_depth_write: Some(()),
                primitive_topology: Some(ShaderPrimitiveTopology::TriangleStrip),
                ..Default::default()
            }),
        )
        .unwrap();

    let vertex_data = get_sphere_vbo();
    let index_data = get_sphere_ibo();

    let vbo = context
        .new_vertex_buffer(&vertex_data, BufferStorageType::Static, None)
        .unwrap();
    let ibo = context
        .new_index_buffer(&index_data, BufferStorageType::Static, None)
        .unwrap();

    let mut pass = Pass::new(
        window_size.0,
        window_size.1,
        Some(NewPassExt {
            depends_on_surface_size: Some(()),
            surface_attachment_load_op: Some(PassInputLoadOpColorType::Clear),
            ..Default::default()
        }),
    );

    let depth_attachment_image = context
        .new_attachment_image(
            window_size.0,
            window_size.1,
            AttachmentImageUsage::DepthAttachment,
            Some(NewAttachmentImageExt {
                depends_on_surface_size: Some(()),
                ..Default::default()
            }),
        )
        .unwrap();

    let depth_attachment = pass.add_attachment_depth_image(
        depth_attachment_image,
        PassInputLoadOpDepthStencilType::Clear,
    );

    let output_attachment = pass.get_surface_local_attachment();
    {
        let pass_step = pass.add_step();
        pass_step
            .add_vertex_buffer(vbo)
            .set_index_buffer(ibo)
            .add_program(program)
            .add_write_color(output_attachment)
            .set_write_depth(depth_attachment);
    }

    let compiled_pass = context.compile_pass(&pass, None).unwrap();

    //
    //  --- End Setup Code ---
    //

    let mut last_window_size = window_size;
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
            } if window_id == window.id() => {
                context
                    .set_surface_size(size.width as usize, size.height as usize)
                    .unwrap();
            }
            Event::MainEventsCleared => {
                let window_size = get_window_size(&window);
                if last_window_size.0 != window_size.0 || last_window_size.1 != window_size.1 {
                    context
                        .set_surface_size(window_size.0, window_size.1)
                        .unwrap();
                }
                last_window_size = window_size;

                //
                //  --- Begin Render Code ---
                //

                let mut submit = Submit::new();

                let projection = glm::perspective(
                    window_size.0 as f32 / window_size.1 as f32,
                    40.0 * (glm::pi::<f32>() / 180.0),
                    0.1,
                    100.0,
                );

                let camera_translate = glm::vec3(0.0, 0.0, -35.0);
                let view = glm::identity();
                let view = glm::translate(&view, &camera_translate);

                let scene_data = SceneData {
                    view,
                    projection,
                    camera_position: -camera_translate,
                    light_positions: [
                        glm::vec4(10.0, 10.0, 10.0, 1.0),
                        glm::vec4(-10.0, 10.0, 10.0, 1.0),
                        glm::vec4(10.0, -10.0, 10.0, 1.0),
                        glm::vec4(-10.0, -10.0, 10.0, 1.0),
                    ],
                    light_colors: [
                        glm::vec4(300.0, 300.0, 300.0, 1.0),
                        glm::vec4(300.0, 300.0, 300.0, 1.0),
                        glm::vec4(300.0, 300.0, 300.0, 1.0),
                        glm::vec4(300.0, 300.0, 300.0, 1.0),
                    ],
                    ..Default::default()
                };

                submit.transfer_into_uniform_buffer(scene_ubo_guard, &scene_data);

                let spacing = 2.5;

                let mut obj_datas = vec![];
                for row in 0..SPHERE_ROW_COUNT {
                    let metallic = row as f32 / SPHERE_ROW_COUNT as f32;
                    for col in 0..SPHERE_COL_COUNT {
                        let roughness = col as f32 / SPHERE_COL_COUNT as f32;

                        let model = glm::identity();
                        let model = glm::translate(
                            &model,
                            &glm::vec3(
                                (col as f32 - (SPHERE_ROW_COUNT as f32 / 2.0)) * spacing,
                                (row as f32 - (SPHERE_COL_COUNT as f32 / 2.0)) * spacing,
                                0.0,
                            ),
                        );
                        let normal_matrix = glm::transpose(&glm::inverse(&model));

                        let obj_data = ObjectData {
                            albedo: glm::vec3(0.5, 0.0, 0.0),
                            metallic,

                            roughness,
                            ao: 1.0,
                            model,
                            normal_matrix,
                            ..Default::default()
                        };

                        obj_datas.push(obj_data);
                    }
                }

                obj_datas.iter().enumerate().for_each(|(idx, obj_data)| {
                    submit.transfer_into_dynamic_uniform_buffer(obj_ubo_guard, obj_data, idx);
                });

                let mut pass_submit = PassSubmitData::new(compiled_pass);

                {
                    let mut step_submit = StepSubmitData::new();

                    for dyn_idx in 0..obj_datas.len() {
                        step_submit
                            .draw_indexed(program, 0, index_data.len())
                            .set_viewport(DrawViewport {
                                x: 0.0,
                                y: 0.0,
                                width: window_size.0 as f32,
                                height: window_size.1 as f32,
                            })
                            .set_dynamic_uniform_buffer_index(obj_ubo, dyn_idx);
                    }

                    pass_submit.set_attachment_clear_color(
                        output_attachment,
                        ClearColor {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        },
                    );
                    pass_submit.set_attachment_clear_depth_stencil(
                        depth_attachment,
                        ClearDepthStencil {
                            depth: 1.0,
                            stencil: 0,
                        },
                    );
                    pass_submit.step(step_submit);
                }

                submit.pass(pass_submit);
                context.submit(submit, None).unwrap();

                //
                //  --- End Render Code ---
                //
                window.request_redraw();
            }
            _ => (),
        }
    });
}

const X_SEGMENTS: usize = 64;
const Y_SEGMENTS: usize = 64;

fn get_sphere_vbo() -> Vec<VertexBufferElement> {
    let mut out = vec![];
    let pi = std::f32::consts::PI;
    for x in 0..=X_SEGMENTS {
        for y in 0..=Y_SEGMENTS {
            let x_seg = x as f32 / X_SEGMENTS as f32;
            let y_seg = y as f32 / Y_SEGMENTS as f32;
            let px = (x_seg * 2.0 * pi).cos() * (y_seg * pi).sin();
            let py = (y_seg * pi).cos();
            let pz = (x_seg * 2.0 * pi).sin() * (y_seg * pi).sin();

            out.append(&mut vec![px, py, pz, px, py, pz, x_seg, y_seg]);
        }
    }
    out
}

fn get_sphere_ibo() -> Vec<IndexBufferElement> {
    let mut out = vec![];
    let mut is_odd = false;
    for y in 0..Y_SEGMENTS as u32 {
        if !is_odd {
            for x in 0..=X_SEGMENTS as u32 {
                out.push(y * (X_SEGMENTS as u32 + 1) + x);
                out.push((y + 1) * (X_SEGMENTS as u32 + 1) + x);
            }
        } else {
            for x in (0..=X_SEGMENTS as u32).rev() {
                out.push((y + 1) * (X_SEGMENTS as u32 + 1) + x);
                out.push(y * (X_SEGMENTS as u32 + 1) + x);
            }
        }
        is_odd = !is_odd;
    }
    out
}

#[allow(unused_variables)]
fn get_window_size(window: &Window) -> (usize, usize) {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    return wasm::get_window_size();

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        let size = window.inner_size();
        (size.width as usize, size.height as usize)
    }
}

#[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
mod wasm {
    use wasm_bindgen::prelude::*;

    pub fn init() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    pub fn get_window_size() -> (usize, usize) {
        let window = web_sys::window().unwrap();
        let canvas = window
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        (
            canvas.client_width() as usize,
            canvas.client_height() as usize,
        )
    }
}
