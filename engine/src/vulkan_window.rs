use crate::vulkan_init;
use crate::vulkan_bindings;
use crate::system_window;

pub fn load_extension_names(extensions: &[&[u8]]) -> Vec<String>
{
    let mut desired_extensions :Vec<String> = Vec::with_capacity(extensions.len());
    for ext in extensions
    {
        desired_extensions.push(String::from_utf8(ext.to_vec()).unwrap().trim_end_matches('\0').to_string());
    }
    desired_extensions
}


pub fn vulkan_init_window()
{
    let global_exts = load_extension_names(&[vulkan_bindings::VK_KHR_SURFACE_EXTENSION_NAME, vulkan_bindings::VK_KHR_WIN32_SURFACE_EXTENSION_NAME]);
    let vk_instance = vulkan_init::initialize_vulkan(global_exts);
    let window = system_window::WindowParameters::new("Anvil".to_string());
    window.destroy()
    // load_extension_names(extensions);
    // logical_device = vk_instance.create_logical_device(&["VK_KHR_swapchain"], &[(vulkan_bindings::VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT | vulkan_bindings::VkQueueFlagBits_VK_QUEUE_COMPUTE_BIT) as u32]);

}