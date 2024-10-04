use crate::vulkan_init;
use crate::vulkan_bindings;
use crate::system_window;

#[derive(Debug)]
pub enum VulkanWindowError
{
    CANT_LOAD_VULKAN_SURFACE
}

impl std::fmt::Display for VulkanWindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanWindowError::CANT_LOAD_VULKAN_SURFACE => write!(f, "Couldn't a vulkan surface")
        }
    }
}

impl std::error::Error for VulkanWindowError {}


pub fn load_extension_names(extensions: &[&[u8]]) -> Vec<String>
{
    let mut desired_extensions :Vec<String> = Vec::with_capacity(extensions.len());
    for ext in extensions
    {
        desired_extensions.push(String::from_utf8(ext.to_vec()).unwrap().trim_end_matches('\0').to_string());
    }
    desired_extensions
}

pub struct VulkanSurface {
    pub window : system_window::WindowParameters,
    pub surface : vulkan_bindings::VkSurfaceKHR,
}

impl VulkanSurface {
    pub fn new(vk_instance: &vulkan_init::VulkanInstance) -> Result<Self, VulkanWindowError>
    {
        let mut vk_surface = VulkanSurface {
            window : system_window::WindowParameters::new("Anvil".to_string()),
            surface: std::ptr::null_mut()
        };
        let vk_surface_create_info = vulkan_bindings::VkWin32SurfaceCreateInfoKHR {
            sType : vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
            pNext : std::ptr::null(),
            flags: 0,
            hinstance: vk_surface.window.Hinstance,
            hwnd: vk_surface.window.Hwnd
        };

        unsafe {
            let fn_vkCreateWin32SurfaceKHR = vulkan_init::vkCreateWin32SurfaceKHR.unwrap();
            let result = fn_vkCreateWin32SurfaceKHR(vk_instance.instance, &vk_surface_create_info, std::ptr::null(), &mut vk_surface.surface);
            if result != vulkan_bindings::VkResult_VK_SUCCESS || vk_surface.surface == std::ptr::null_mut()
            {
                return Err(VulkanWindowError::CANT_LOAD_VULKAN_SURFACE);
            }
            Ok(vk_surface)
        }
    }
}

pub fn vulkan_init_window()
{
    let global_exts = load_extension_names(&[vulkan_bindings::VK_KHR_SURFACE_EXTENSION_NAME, vulkan_bindings::VK_KHR_WIN32_SURFACE_EXTENSION_NAME]);
    let vk_instance = vulkan_init::initialize_vulkan(global_exts);
    let vk_surface = match VulkanSurface::new(vk_instance)
    {
        Ok(surface) => surface,
        Err(e) =>
        {
            eprintln!("{}",e);
            std::process::exit(1);
        }
    };
    let device_exts = load_extension_names(&[vulkan_bindings::VK_KHR_SWAPCHAIN_EXTENSION_NAME]);
    let logical_device;
    logical_device = match vk_instance.create_logical_device(
        device_exts, 
        &[(vulkan_bindings::VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT | vulkan_bindings::VkQueueFlagBits_VK_QUEUE_COMPUTE_BIT) as u32],
        &vk_surface.surface
    ) {
        Ok(l) => l,
        Err(e) =>
        {
            eprintln!("{}",e);
            std::process::exit(1);
        }
    };

}