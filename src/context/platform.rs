use super::*;

#[cfg(target_os = "windows")]
fn platform_prefered() -> Vec<Api> {
    vec![
        #[cfg(feature = "vulkan")]
        Api::Vulkan,
    ]
}

#[cfg(target_os = "macos")]
fn platform_prefered() -> Vec<Api> {
    vec![
        #[cfg(feature = "vulkan")]
        Api::Vulkan,
    ]
}

#[cfg(all(not(target_os = "macos"), target_family = "unix"))]
fn platform_prefered() -> Vec<Api> {
    vec![
        #[cfg(feature = "vulkan")]
        Api::Vulkan,
    ]
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn platform_prefered() -> Vec<Api> {
    vec![Api::WebGpu]
}

impl Context {
    /// Declare the extensions you plan on using if a specific api is selected.
    /// The platform preference order has nothing to do with this extensions list.
    /// An error is thrown only after all apis have failed to initialize.
    /// ```
    ///     let mut context = Context::new(&[(
    ///         Api::Vulkan,
    ///         &[
    ///             Extension::NativeDebug,
    ///             Extension::Surface(surface::SurfaceConfiguration {
    ///                 ...
    ///             }),
    ///         ],
    ///     )]);
    /// ```
    pub fn new(extensions: Extensions) -> Result<Self, Vec<GpuError>>
    where
        Self: Sized,
    {
        let mut fails = vec![];
        for api in platform_prefered() {
            match api {
                #[cfg(all(
                    not(all(target_arch = "wasm32", target_os = "unknown")),
                    feature = "vulkan"
                ))]
                Api::Vulkan => match VkContext::new(extensions.clone()) {
                    Ok(context) => return Ok(Context::Vulkan(context)),
                    Err(fail) => fails.push(fail),
                },
                #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
                Api::WebGpu => match WebGpuContext::new(extensions.clone()) {
                    Ok(context) => return Ok(Context::WebGpu(context)),
                    Err(fail) => fails.push(fail),
                },
                _ => continue,
            }
        }

        Err(fails)
    }
}
