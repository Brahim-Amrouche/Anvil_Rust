use crate::vulkan_init;
use crate::vulkan_bindings;

#[derive(Debug)]
pub enum VulkanSynchroError
{
    COULDNT_CREATE_CMD_POOL,
    COULDNT_CREATE_CMD_BUFFER,
    FAILED_STARTING_PRIMARY_BUFFER_RECORDING,
    FAILED_STARTING_SECONDARY_BUFFER_RECORDING
}

impl std::fmt::Display for  VulkanSynchroError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanSynchroError::COULDNT_CREATE_CMD_POOL => write!(f, "Couldnt create a Command pool"),
            VulkanSynchroError::COULDNT_CREATE_CMD_BUFFER => write!(f, "Couldn't create a Command Buffer"),
            VulkanSynchroError::FAILED_STARTING_PRIMARY_BUFFER_RECORDING => write!(f, "Failed Starting primary buffer recording"),
            VulkanSynchroError::FAILED_STARTING_SECONDARY_BUFFER_RECORDING => write!(f, "Failed Starting secondary buffer recording")
        }
    }
}

impl std::error::Error for VulkanSynchroError
{}

pub struct VulkanCmdPool
{
    cmd_pool_handle: vulkan_bindings::VkCommandPool,
    logical_device : *const vulkan_init::VulkanLogicalDevice,
    cmd_buffers : Option<VulkanCmdBuffer>,
}

impl VulkanCmdPool
{
    pub fn new(logical_device : &vulkan_init::VulkanLogicalDevice) -> Result<Self, VulkanSynchroError>
    {
        let mut vk_cmd_pool = VulkanCmdPool {
            cmd_pool_handle : std::ptr::null_mut(),
            logical_device,
            cmd_buffers : None,
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

    pub fn create_buffers(&mut self, primary_count: u32,  secondary_count: u32) -> Result<&mut VulkanCmdBuffer, VulkanSynchroError>
    {
        self.cmd_buffers = Some(VulkanCmdBuffer::new(self, primary_count, secondary_count)?);   
        Ok(self.cmd_buffers.as_mut().unwrap())
    }

}

pub struct VulkanCmdBuffer
{
    cmd_pool : *const VulkanCmdPool,
    primary_buffers : Vec<vulkan_bindings::VkCommandBuffer>,
    secondary_buffers: Vec<vulkan_bindings::VkCommandBuffer>
}

impl VulkanCmdBuffer
{
    pub fn new(cmd_pool: &VulkanCmdPool, primary_count: u32 , secondary_count: u32) -> Result<Self, VulkanSynchroError>
    {
        let mut cmd_buffer = VulkanCmdBuffer {
            cmd_pool,
            primary_buffers: Vec::new(),
            secondary_buffers : Vec::new(),
        };
        cmd_buffer.primary_buffers = cmd_buffer.create_buffer(primary_count, vulkan_bindings::VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_PRIMARY)?;
        if secondary_count > 0
        {
            cmd_buffer.secondary_buffers = cmd_buffer.create_buffer(secondary_count, vulkan_bindings::VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_SECONDARY)?;
        }
        Ok(cmd_buffer)
    }

    pub fn create_buffer(&self, count: u32, level: vulkan_bindings::VkCommandBufferLevel ) -> Result<Vec<vulkan_bindings::VkCommandBuffer>, VulkanSynchroError>
    {
        unsafe {
            let buffer_create_info = vulkan_bindings::VkCommandBufferAllocateInfo {
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
                pNext: std::ptr::null(),
                commandPool: (*self.cmd_pool).cmd_pool_handle,
                level,
                commandBufferCount: count
            };
            let fn_vkAllocateCommandBuffers = vulkan_init::vkAllocateCommandBuffers.unwrap();
            let logical_device = (*(*self.cmd_pool).logical_device).device;
            let mut cmd_buffers : Vec<vulkan_bindings::VkCommandBuffer> = vec![std::ptr::null_mut(); count as usize];
            let result = fn_vkAllocateCommandBuffers(logical_device, &buffer_create_info, cmd_buffers.as_mut_ptr());
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanSynchroError::COULDNT_CREATE_CMD_BUFFER);
            }
            Ok(cmd_buffers)
        }
    }
    
    pub fn begin_primary_buffer(&mut self, buffer_idx: usize, usage: vulkan_bindings::VkBufferUsageFlagBits) -> Result<(), VulkanSynchroError>
    {
        if buffer_idx >= self.primary_buffers.len()
        {
            return Err(VulkanSynchroError::FAILED_STARTING_PRIMARY_BUFFER_RECORDING);
        }
        let buffer_begin_info = vulkan_bindings::VkCommandBufferBeginInfo {
            sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext : std::ptr::null(),
            flags: usage as u32,
            pInheritanceInfo: std::ptr::null()
        };
        unsafe 
        {
            let fn_vkBeginCommandBuffer = vulkan_init::vkBeginCommandBuffer.unwrap();
            let result = fn_vkBeginCommandBuffer(self.primary_buffers[buffer_idx], &buffer_begin_info);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanSynchroError::FAILED_STARTING_PRIMARY_BUFFER_RECORDING);
            }
        }
        Ok(())
    }
}