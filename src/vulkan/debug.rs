use super::*;
use std::ffi::CStr;

pub struct VkDebug {
    debug_ext: extensions::ext::DebugUtils,
    debug: vk::DebugUtilsMessengerEXT,
}

impl VkDebug {
    pub fn new(entry: &Entry, instance: &Instance) -> GResult<Self> {
        let debug_ext = extensions::ext::DebugUtils::new(entry, instance);
        let debug_create = Self::get_debug_create();
        unsafe { debug_ext.create_debug_utils_messenger(&debug_create, None) }
            .map_err(|e| gpu_api_err!("vulkan debug init {}", e))
            .map(|debug| VkDebug { debug_ext, debug })
    }

    pub fn get_debug_create() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                // | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(vulkan_debug_utils_callback))
            .build()
    }

    pub const fn get_additional_extensions() -> [&'static CStr; 1] {
        [extensions::ext::DebugUtils::name()]
    }

    pub const fn get_additional_layers() -> &'static [&'static str] {
        &["VK_LAYER_KHRONOS_validation"]
    }

    pub unsafe fn destory(&mut self) {
        self.debug_ext
            .destroy_debug_utils_messenger(self.debug, None);
    }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    //  TODO FIX: Change to something else?
    eprintln!("[Debug][{}][{}] {:?}\n\t---", severity, ty, message);
    vk::FALSE
}
