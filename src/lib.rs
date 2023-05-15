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
//! let mut context = Context::new(&[(
//!     Api::Vulkan,
//!     &[
//!         //  I prefer this enabled, but it doesn't always help and may resource intensive.
//!         Extension::NativeDebug,
//!         Extension::Surface(surface::SurfaceConfiguration {
//!         //  Yes, these numbers are arbitrary.
//!             width: 640,
//!             height: 480,
//!         //  You are in charge of this.
//!             display: unimplemented!(),
//!             window: unimplemented!(), 
//!         }),
//!         Extension::ShaderReflection,
//!     ],
//! )])
//! .unwrap();
//!
//! //  Load and create the shaders.
//! let vs = "...";
//! let fs = "...";
//!
//! let vs_reflect = context
//!     .shader_reflection_extension_reflect(
//!         vs,
//!         shader_reflection::ReflectionShaderTypeHint::Vertex,
//!     )
//!     .unwrap();
//! let fs_reflect = context
//!     .shader_reflection_extension_reflect(
//!         fs,
//!         shader_reflection::ReflectionShaderTypeHint::Fragment,
//!     )
//!     .unwrap();
//!
//! let program = context
//!     .new_program(
//!         &ShaderSet::shaders(&[(vs_reflect, vs), (fs_reflect, fs)]),
//!         &[],
//!         None,
//!     )
//!    .unwrap();
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
//! //  These width height numbers are arbitrary as well.
//!     640,
//!     480,
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
//!         .set_program(program)
//!         .add_write_color(output_attachment);
//! }
//!
//! //  Ok, we are ready to compile!
//! let compiled_pass = context.compile_pass(&pass, None).unwrap();
//!
//! //  Pretend that this is our render loop.
//! loop {
//!     let mut submit = Submit::new();
//!
//!     //  Prepare to submit our compiled pass.
//!     let mut pass_submit = PassSubmitData::new(compiled_pass);
//!
//!     //  Each step MUST have its own submit data IN ORDER.
//!     {
//!         let mut step_submit = StepSubmitData::new();
//!         step_submit.draw_indexed(0, index_data.len());
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
//! > Sorry for the massive code dump!

pub mod prelude;

pub mod context;
mod error;
mod mock;

#[cfg(feature = "vulkan")]
mod vulkan;
