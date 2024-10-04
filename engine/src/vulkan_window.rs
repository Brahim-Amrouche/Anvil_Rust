use crate::vulkan_init;
use crate::vulkan_bindings;
use crate::system_window;

#[derive(Debug)]
pub enum VulkanWindowError
{
    CANT_LOAD_VULKAN_SURFACE,
    CANT_LOAD_SURFACE_CAPABILITIES
}

impl std::fmt::Display for VulkanWindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanWindowError::CANT_LOAD_VULKAN_SURFACE => write!(f, "Couldn't a vulkan surface"),
            VulkanWindowError::CANT_LOAD_SURFACE_CAPABILITIES => write!(f, "Couldn't load surface Capabilities")
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
    pub capabilites : vulkan_bindings::VkSurfaceCapabilitiesKHR
}

impl VulkanSurface {
    pub fn new(vk_instance: &vulkan_init::VulkanInstance) -> Result<Self, VulkanWindowError>
    {
        unsafe {
            let mut vk_surface = VulkanSurface {
                window : system_window::WindowParameters::new("Anvil".to_string()),
                surface: std::ptr::null_mut(),
                capabilites: std::mem::zeroed()
            };
            let vk_surface_create_info = vulkan_bindings::VkWin32SurfaceCreateInfoKHR {
                sType : vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
                pNext : std::ptr::null(),
                flags: 0,
                hinstance: vk_surface.window.Hinstance,
                hwnd: vk_surface.window.Hwnd
            };
            let fn_vkCreateWin32SurfaceKHR = vulkan_init::vkCreateWin32SurfaceKHR.unwrap();
            let result = fn_vkCreateWin32SurfaceKHR(vk_instance.instance, &vk_surface_create_info, std::ptr::null(), &mut vk_surface.surface);
            if result != vulkan_bindings::VkResult_VK_SUCCESS || vk_surface.surface == std::ptr::null_mut()
            {
                return Err(VulkanWindowError::CANT_LOAD_VULKAN_SURFACE);
            }
            Ok(vk_surface)
        }
    }

    pub fn load_surface_capabilities(&mut self, logical_device: &vulkan_init::VulkanLogicalDevice) -> Result<(), VulkanWindowError>
    {
        unsafe
        {
            let fn_vkGetPhysicalDeviceSurfaceCapabilitiesKHR = vulkan_init::vkGetPhysicalDeviceSurfaceCapabilitiesKHR.unwrap();
            let ref physical_device =  *logical_device.physical_device;
            let physical_device = physical_device.ph_device;
            let result = fn_vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, self.surface, &mut self.capabilites);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanWindowError::CANT_LOAD_SURFACE_CAPABILITIES)
            }
        }
        Ok(())
    }

    pub fn destroy(self)
    {
        self.window.destroy();
    }
}

pub fn vulkan_init_window()
{
    let global_exts = load_extension_names(&[vulkan_bindings::VK_KHR_SURFACE_EXTENSION_NAME, vulkan_bindings::VK_KHR_WIN32_SURFACE_EXTENSION_NAME]);
    let mut vk_instance = vulkan_init::initialize_vulkan(global_exts);
    let mut vk_surface = VulkanSurface::new(vk_instance).unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    let device_exts = load_extension_names(&[vulkan_bindings::VK_KHR_SWAPCHAIN_EXTENSION_NAME]);
    let logical_device;
    logical_device = vulkan_init::VulkanLogicalDevice::new(
        &mut vk_instance,
        device_exts, 
        &[(vulkan_bindings::VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT | vulkan_bindings::VkQueueFlagBits_VK_QUEUE_COMPUTE_BIT) as u32],
        &vk_surface.surface,
        vulkan_bindings::VkPresentModeKHR_VK_PRESENT_MODE_FIFO_KHR
    ).unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    vk_surface.load_surface_capabilities(&logical_device).unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    vk_surface.destroy();

}