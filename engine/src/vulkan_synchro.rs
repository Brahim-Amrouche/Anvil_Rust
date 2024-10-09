use crate::vulkan_init;
use crate::vulkan_bindings;

#[derive(Debug)]
pub enum VulkanSynchroError
{
    COULDNT_CREATE_CMD_POOL
}

impl std::fmt::Display for  VulkanSynchroError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanSynchroError::COULDNT_CREATE_CMD_POOL => write!(f, "Couldnt Create a Command pool")
        }
    }
}

impl std::error::Error for VulkanSynchroError
{}


pub struct VulkanCmdPool
{
    cmd_pool_handle: vulkan_bindings::VkCommandPool,
    logical_device : *const vulkan_init::VulkanLogicalDevice
}

impl VulkanCmdPool
{
    pub fn new(logical_device : &vulkan_init::VulkanLogicalDevice) -> Result<Self, VulkanSynchroError>
    {
        let mut vk_cmd_pool = VulkanCmdPool {
            cmd_pool_handle : std::ptr::null_mut(),
            logical_device
        };
        unsafe
        {
            let fn_vkCreateCommandPool = vulkan_init::vkCreateCommandPool.unwrap();
            let buffer_params = vulkan_bindings::VkCommandPoolCreateFlagBits_VK_COMMAND_POOL_CREATE_TRANSIENT_BIT | vulkan_bindings::VkCommandPoolCreateFlagBits_VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
            let cmd_pool_create_info = vulkan_bindings::VkCommandPoolCreateInfo {
                sType : vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
                pNext: std::ptr::null(),
                flags: buffer_params as u32,
                queueFamilyIndex: 0
            };
            let result = fn_vkCreateCommandPool(logical_device.device, &cmd_pool_create_info, std::ptr::null(), &mut vk_cmd_pool.cmd_pool_handle);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanSynchroError::COULDNT_CREATE_CMD_POOL);
            }
        }
        Ok(vk_cmd_pool)
    }
}