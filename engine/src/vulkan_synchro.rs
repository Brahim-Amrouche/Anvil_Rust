use crate::vulkan_init;
use crate::vulkan_bindings;

#[derive(Debug)]
pub enum VulkanSynchroError
{
    COULDNT_CREATE_CMD_POOL,
    COULDNT_CREATE_CMD_BUFFER
}

impl std::fmt::Display for  VulkanSynchroError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanSynchroError::COULDNT_CREATE_CMD_POOL => write!(f, "Couldnt create a Command pool"),
            VulkanSynchroError::COULDNT_CREATE_CMD_BUFFER => write!(f, "Couldn't create a Command Buffer")
        }
    }
}

impl std::error::Error for VulkanSynchroError
{}


pub struct VulkanCmdPool
{
    cmd_pool_handle: vulkan_bindings::VkCommandPool,
    logical_device : *const vulkan_init::VulkanLogicalDevice,
    cmd_buffers : Vec<VulkanCmdBuffer>
}

impl VulkanCmdPool
{
    pub fn new(logical_device : &vulkan_init::VulkanLogicalDevice) -> Result<Self, VulkanSynchroError>
    {
        let mut vk_cmd_pool = VulkanCmdPool {
            cmd_pool_handle : std::ptr::null_mut(),
            logical_device,
            cmd_buffers: Vec::new()
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

    pub fn create_buffers(&mut self, count: u32) -> Result<&VulkanCmdBuffer, VulkanSynchroError>
    {
        self.cmd_buffers.push(VulkanCmdBuffer::new(self, count)?);   
        Ok(&self.cmd_buffers[self.cmd_buffers.len() - 1])
    }
}

pub struct VulkanCmdBuffer
{
    cmd_pool : *const VulkanCmdPool,
    buffers : Vec<vulkan_bindings::VkCommandBuffer>
}

impl VulkanCmdBuffer
{
    pub fn new(cmd_pool: &VulkanCmdPool, count: u32 ) -> Result<Self, VulkanSynchroError>
    {
        let mut cmd_buffer = VulkanCmdBuffer {
            cmd_pool,
            buffers: vec![std::ptr::null_mut(); count as usize]
        };
        unsafe {
            let buffer_create_info = vulkan_bindings::VkCommandBufferAllocateInfo {
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
                pNext: std::ptr::null(),
                commandPool: (*cmd_buffer.cmd_pool).cmd_pool_handle,
                level: vulkan_bindings::VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_PRIMARY,
                commandBufferCount: count
            };
            let fn_vkAllocateCommandBuffers = vulkan_init::vkAllocateCommandBuffers.unwrap();
            let logical_device = (*(*cmd_buffer.cmd_pool).logical_device).device;
            let result = fn_vkAllocateCommandBuffers(logical_device, &buffer_create_info, cmd_buffer.buffers.as_mut_ptr());
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanSynchroError::COULDNT_CREATE_CMD_BUFFER);
            }
        }
        Ok(cmd_buffer)
    }
}