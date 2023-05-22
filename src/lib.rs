//! # Mepeyew
//!
//! ## Introduction
//!
//! Mepeyew is a rendering abstraction layer created for [`mewo`](https://github.com/davnotdev/mewo).
//! Essentially, Mepeyew allows you to draw graphics on the GPU without having to
//! worry about the platform specific details.
//! Additionally, Mepeyew has zero unnecessary dependencies, perfect for people who have
//! bundlephobia like me.
//! For more details, see the [Github page](https://github.com/davnotdev/mepeyew).
//!
//! ## Quick Reference
//!
//! Here's a quick example use to draw a triangle.
//!
//! For more comprehensive examples, see the [examples on github](https://github.com/davnotdev/mepeyew/tree/main/examples).
//!
//! Also, note that this doesn't include shaders nor windowing.
//!
//! ```
//! //  Create the context and initialize extensions.
//! let mut context = Context::new(&[
//!     (
//!         Api::Vulkan,
//!         &[
//!         //  I prefer this enabled, but note that not all errors are fatal.
//!             Extension::NativeDebug,
//!             Extension::Surface(surface::SurfaceConfiguration {
//!             //  All of this is up to you.
//!                 width: todo!(),
//!                 height: todo!(),
//!                 display: todo!(),
//!                 window: todo!(),
//!             }),
//!             Extension::NagaTranslation,
//!         ],
//!     ),
//!     (
//!         Api::WebGpu,
//!         &[
//!             //  See [`webgpu_init::WebGpuInitFromWindow`].
//!             Extension::WebGpuInitFromWindow(webgpu_init::WebGpuInitFromWindow {
//!                 adapter: todo!(),
//!                 device: todo!(),
//!                 canvas_id: todo!(),
//!             }),
//!             Extension::NagaTranslation,
//!         ],
//!     ),
//! ])
//! .unwrap();
//!
//! //  Load and create the shaders.
//! //  Overall, wgsl is the best supported language.
//! let vs = "...";
//! let fs = "...";
//!
//! let vs = context
//!     .naga_translation_extension_translate_shader_code(
//!         naga_translation::NagaTranslationStage::Vertex,
//!         naga_translation::NagaTranslationInput::Wgsl,
//!         vs,
//!         naga_translation::NagaTranslationExtensionTranslateShaderCodeExt::default(),
//!     )
//!     .unwrap();
//! let fs = context
//!     .naga_translation_extension_translate_shader_code(
//!         naga_translation::NagaTranslationStage::Fragment,
//!         naga_translation::NagaTranslationInput::Wgsl,
//!         fs,
//!         naga_translation::NagaTranslationExtensionTranslateShaderCodeExt::default(),
//!     )
//!     .unwrap();
//!
//! let program = context
//!     .new_program(
//!         &ShaderSet::shaders(&[
//!             (
//!                 ShaderType::Vertex(VertexBufferInput {
//!                     args: vec![VertexInputArgCount(3)],
//!                 }),
//!                 &vs,
//!             ),
//!             (ShaderType::Fragment, &fs),
//!         ]),
//!         &[],
//!         None,
//!     )
//!     .unwrap();
//!
//! //  Create a VBO and IBO for drawing a triangle.
//! #[rustfmt::skip]
//! let vertex_data: Vec<VertexBufferElement> = vec![
//!      0.0,  0.5, 0.0,
//!     -0.5, -0.5, 0.0,
//!      0.5, -0.5, 0.0,
//! ];
//! #[rustfmt::skip]
//! let index_data: Vec<IndexBufferElement> = vec![
//!     0, 1, 2
//! ];
//!
//! let vbo = context
//!     .new_vertex_buffer(&vertex_data, BufferStorageType::Static, None)
//!     .unwrap();
//! let ibo = context
//!     .new_index_buffer(&index_data, BufferStorageType::Static, None)
//!     .unwrap();
//!
//! //  Create our render pass.
//! let mut pass = Pass::new(
//! //  These width height numbers are based on the window.
//!     todo!(),
//!     todo!(),
//!     Some(NewPassExt {
//!         //  Yes, we would like this to be resizable.
//!         depends_on_surface_size: Some(()),
//!         //  Yes, we would like to clear the screen.
//!         surface_attachment_load_op: Some(PassInputLoadOpColorType::Clear),
//!         //  Unnecessary in this present, but prevents future updates from ruining everything.
//!         ..Default::default()
//!     }),
//! );
//!
//! let output_attachment = pass.get_surface_local_attachment();
//!
//! //  Each render pass has steps.
//! //  This step draws our VBO and IBO onto the surface.
//! {
//!     let pass_step = pass.add_step();
//!     pass_step
//!         .add_vertex_buffer(vbo)
//!         .set_index_buffer(ibo)
//!         .add_program(program)
//!         .add_write_color(output_attachment);
//! }
//!
//! //  Ok, we are ready to compile!
//! let compiled_pass = context.compile_pass(&pass, None).unwrap();
//!
//! //  Pretend that this is our render loop.
//! loop {
//!     //  Make sure to call [`Context::surface_extension_set_surface_size`] when the window resizes.
//!
//!     let mut submit = Submit::new();
//!
//!     //  Prepare to submit our compiled pass.
//!     let mut pass_submit = PassSubmitData::new(compiled_pass);
//!
//!     //  Each step MUST have its own submit data IN ORDER.
//!     {
//!         let mut step_submit = StepSubmitData::new();
//!         step_submit.draw_indexed(program, 0, index_data.len());
//!         pass_submit.set_attachment_clear_color(
//!             output_attachment,
//!             ClearColor {
//!                 r: 0.0,
//!                 g: 0.2,
//!                 b: 0.2,
//!                 a: 1.0,
//!             },
//!         );
//!         pass_submit.step(step_submit);
//!     }
//!
//!     //  Submit!
//!     submit.pass(pass_submit);
//!     context.submit(submit, None).unwrap();
//! }
//!
//! ```
//!
//! Sorry for the massive code dump!
//!
//! Once again, If you want a more comprehensive set of examples, [look here](https://github.com/davnotdev/mepeyew/tree/main/examples).
//!

pub mod prelude;

pub mod context;
mod error;
mod mock;

#[cfg(all(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    feature = "vulkan"
))]
mod vulkan;
#[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
mod webgpu;
