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
    ///         Extension::ShaderReflection,
    ///         ],
    ///     )]);
    /// ```
    pub fn new(extensions: &[(Api, &[Extension])]) -> Result<Self, Vec<GpuError>>
    where
        Self: Sized,
    {
        let mut fails = vec![];
        for api in platform_prefered() {
            let api_extensions = extensions
                .iter()
                .find_map(|(eapi, extensions)| (api == *eapi).then_some(*extensions))
                .unwrap_or(&[]);
            match api {
                #[cfg(feature = "vulkan")]
                Api::Vulkan => match VkContext::new(api_extensions) {
                    Ok(context) => return Ok(Context::Vulkan(context)),
                    Err(fail) => fails.push(fail),
                },
            }
        }

        Err(fails)
    }
}
