//! # Mepeyew
//!
//! ## Introduction
//!
//! Mepeyew is a rendering abstraction layer created for [`mewo`](https://github.com/davnotdev/mewo).
//! Essentially, Mepeyew allows you to draw graphics on the GPU without having to
//! worry about the platform specific details.
//! Additionally, Mepeyew has zero unnecessary dependencies, perfect for people who have
//! bundlephobia (like me).
//! For more details, see the [Github page](https://github.com/davnotdev/mepeyew).
//!
//! ## Usage
//!
//! Graphics programming is complicated...
//!
//! For this reason, my best advice for you is to have a look at the examples on the [Github page](https://github.com/davnotdev/mepeyew/tree/main/examples).
//!
//! ## Platform Dependent Nastiness
//!
//! Unfortunately, not everything can be fully abstracted.
//!
//! Here's the list of oddities to look out for.
//!
//! ### Uniform Padding
//!
//! Vulkan has very specific alignment requirements for uniform buffers (of any kind).
//! Failing to conform with these requirements leads to strange shader behaviour.
//! You can read [Vulkan's specification here](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap15.html#interfaces-resources-layout)
//!
//! ### Step Dependencies
//!
//! Certain methods such as [`prelude::PassStep::set_wait_for_depth_from_step`] only cause errors with
//! Vulkan.
//! For this reason, you should always use these methods even if you code appears to work
//! without it.

pub mod prelude;

pub(crate) mod alignment;
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
