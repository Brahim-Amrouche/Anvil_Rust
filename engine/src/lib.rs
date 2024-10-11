#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod vulkan_bindings;
pub mod vulkan_init;
pub mod vulkan_window;
pub mod vulkan_synchro;
mod system_window;

pub fn render()
{
    let global_exts = vulkan_init::load_extension_names(&[vulkan_bindings::VK_KHR_SURFACE_EXTENSION_NAME, vulkan_bindings::VK_KHR_WIN32_SURFACE_EXTENSION_NAME]);
    let mut vk_instance = vulkan_init::initialize_vulkan(global_exts);
    let mut vk_surface = vulkan_window::VulkanSurface::new(vk_instance).unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    let device_exts = vulkan_init::load_extension_names(&[vulkan_bindings::VK_KHR_SWAPCHAIN_EXTENSION_NAME]);
    let logical_device = vulkan_init::VulkanLogicalDevice::new(
        &mut vk_instance,
        device_exts, 
        &[(vulkan_bindings::VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT | vulkan_bindings::VkQueueFlagBits_VK_QUEUE_COMPUTE_BIT) as u32],
        &vk_surface.surface,
        vulkan_bindings::VkPresentModeKHR_VK_PRESENT_MODE_MAILBOX_KHR
    ).unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    let mut cmd_pool= vulkan_synchro::VulkanCmdPool::new(&logical_device).unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    let buffer = cmd_pool.create_buffers(3, 3).unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    buffer.begin_primary_buffer(0, vulkan_bindings::VkCommandBufferUsageFlagBits_VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT).unwrap();
    buffer.end_primary_buffer(0).unwrap();
    buffer.reset_primary_buffer(0, true).unwrap();
    cmd_pool.reset_pool(true).unwrap();
    // let fence = vulkan_synchro::init_fence(&logical_device).unwrap();
    // vulkan_synchro::wait_fences(&logical_device, &vec![fence], vulkan_bindings::VK_TRUE, 20000000).unwrap();
    // vulkan_synchro::reset_fences(&logical_device, &vec![fence]).unwrap();
    vk_surface.destroy();
    logical_device.destroy();
    vk_instance.destroy();
}