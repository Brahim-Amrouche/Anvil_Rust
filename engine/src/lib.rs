#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod vulkan_bindings;
pub mod vulkan_init;
pub mod vulkan_window;
pub mod vulkan_synchro;
pub mod vulkan_mem;
mod system_window;

pub fn render()
{
    let global_exts = vulkan_init::load_extension_names(&[vulkan_bindings::VK_KHR_SURFACE_EXTENSION_NAME, vulkan_bindings::VK_KHR_WIN32_SURFACE_EXTENSION_NAME]);
    let mut vk_instance = vulkan_init::initialize_vulkan(global_exts);
    let vk_surface = vulkan_window::VulkanSurface::new(vk_instance).unwrap_or_else(|e| {
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

    //Vulkan Window Test
    // let desired_surface_format = vulkan_bindings::VkSurfaceFormatKHR {
    //     format : vulkan_bindings::VkFormat_VK_FORMAT_R8G8B8A8_UNORM,
    //     colorSpace: vulkan_bindings::VkColorSpaceKHR_VK_COLOR_SPACE_SRGB_NONLINEAR_KHR
    // };
    // vk_surface.configure_swapchain(&logical_device, 
    //     3,
    //     (vulkan_bindings::VkImageUsageFlagBits_VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT) as u32,
    //     vulkan_bindings::VkSurfaceTransformFlagBitsKHR_VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR as u32,
    //     &desired_surface_format
    // ).unwrap_or_else(|e| {
    //         eprintln!("{}",e);
    //         std::process::exit(1);
    // });
    // vk_surface.present_image().unwrap();

    //Vulkan Synchro Test
    // let mut cmd_pool= vulkan_synchro::VulkanCmdPool::new(&logical_device).unwrap_or_else(|e| {
    //     eprintln!("{}",e);
    //     std::process::exit(1);
    // });
    // let buffer = cmd_pool.create_buffers(3, 3).unwrap_or_else(|e| {
    //     eprintln!("{}",e);
    //     std::process::exit(1);
    // });
    // buffer.begin_primary_buffer(0, vulkan_bindings::VkCommandBufferUsageFlagBits_VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT).unwrap();
    // buffer.end_primary_buffer(0).unwrap();
    // buffer.reset_primary_buffer(0, true).unwrap();
    // cmd_pool.reset_pool(true).unwrap();
    // let sem = vulkan_synchro::init_semaphore(&logical_device).unwrap();
    // vulkan_synchro::destroy_semaphore(&logical_device, sem);
    // let fence = vulkan_synchro::init_fence(&logical_device).unwrap();
    // vulkan_synchro::wait_fences(&logical_device, &vec![fence], vulkan_bindings::VK_TRUE, 20000000).unwrap();
    // vulkan_synchro::reset_fences(&logical_device, &vec![fence]).unwrap();
    // vulkan_synchro::destroy_fence(&logical_device, fence);
    // let waiting_sems = vulkan_synchro::VulkanWaitSemaphoresInfo {
        //     semaphores:Vec::new(),
        //     waiting_stage: Vec::new()
        // };
        // let queue  = logical_device.get_device_queue(vulkan_bindings::VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT as u32, 0).unwrap();
        // cmd_pool.submit_buffers(queue, &waiting_sems).unwrap();
        // println!("{}", vulkan_synchro::check_queue_idle(queue));
        // println!("{}", logical_device.is_idle());
    // cmd_pool.destroy();

    //Vulkan Mem Tests
    let mut buffer = vulkan_mem::VulkanBufferMem::new(
        &logical_device, 
        100,
        vulkan_bindings::VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFER_SRC_BIT as u32
    ).unwrap();
    // buffer.create_buffer_view(vulkan_bindings::VkFormat_VK_FORMAT_R8G8B8A8_UNORM).unwrap();
    // let mut image = vulkan_mem::VulkanImageMem::new(
    //     &logical_device,
    //     vulkan_bindings::VkImageType_VK_IMAGE_TYPE_2D,
    //     vulkan_bindings::VkFormat_VK_FORMAT_R8G8B8A8_UNORM,
    //     vulkan_bindings::VkExtent3D {width: 100, height: 100, depth: 1},
    //     1,
    //     6,
    //     1,
    //     vulkan_bindings::VkImageUsageFlagBits_VK_IMAGE_USAGE_TRANSFER_SRC_BIT as u32,
    // ).unwrap();
    // image.create_image_view(vulkan_bindings::VkImageViewType_VK_IMAGE_VIEW_TYPE_CUBE, 0).unwrap();
    buffer.destroy();
    // image.destroy();
    vk_surface.destroy();
    logical_device.destroy();
    vk_instance.destroy();
}