use crate::vulkan_init;
use crate::vulkan_bindings;

#[derive(Debug)]
pub enum VulkanSynchroError
{
    COULDNT_CREATE_CMD_POOL,
    COULDNT_CREATE_CMD_BUFFER,
    FAILED_STARTING_PRIMARY_BUFFER_RECORDING,
    FAILED_STARTING_SECONDARY_BUFFER_RECORDING,
    FAILED_ENDING_PRIMARY_BUFFER_RECORDING,
    FAILED_RESETING_PRIMARY_BUFFER,
    FAILED_RESETING_POOL,
    FAILED_CREATING_SEMAPHORE,
    FAILED_CREATING_FENCE,
    COULDNT_WAIT_FOR_FENCES,
    COULDNT_RESET_FENCES,
    FAILED_SUBMITING_BUFFERS
}

impl std::fmt::Display for  VulkanSynchroError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanSynchroError::COULDNT_CREATE_CMD_POOL => write!(f, "Couldnt create a Command pool"),
            VulkanSynchroError::COULDNT_CREATE_CMD_BUFFER => write!(f, "Couldn't create a Command Buffer"),
            VulkanSynchroError::FAILED_STARTING_PRIMARY_BUFFER_RECORDING => write!(f, "Failed Starting primary buffer recording"),
            VulkanSynchroError::FAILED_STARTING_SECONDARY_BUFFER_RECORDING => write!(f, "Failed Starting secondary buffer recording"),
            VulkanSynchroError::FAILED_ENDING_PRIMARY_BUFFER_RECORDING => write!(f, "Failed Closing primary buffer recording"),
            VulkanSynchroError::FAILED_RESETING_PRIMARY_BUFFER => write!(f, "Failed resetting primary buffer"),
            VulkanSynchroError::FAILED_RESETING_POOL => write!(f, "Failed resetting pool"),
            VulkanSynchroError::FAILED_CREATING_SEMAPHORE => write!(f, "Failed creating semaphore"),
            VulkanSynchroError::FAILED_CREATING_FENCE => write!(f, "Failed creating fence"),
            VulkanSynchroError::COULDNT_WAIT_FOR_FENCES => write!(f, "Couldnt wait for fences"),
            VulkanSynchroError::COULDNT_RESET_FENCES => write!(f, "Couldn't reset fences"),
            VulkanSynchroError::FAILED_SUBMITING_BUFFERS => write!(f, "Failed Submiting buffers")
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

    pub fn reset_pool(&mut self, release_mem: bool) -> Result<(), VulkanSynchroError>
    {
        unsafe
        {
            let fn_vkResetCommandPool = vulkan_init::vkResetCommandPool.unwrap();
            let logical_device = (*self.logical_device).device;
            let result = fn_vkResetCommandPool(
                logical_device, 
                self.cmd_pool_handle,
                if release_mem { vulkan_bindings::VkCommandPoolResetFlagBits_VK_COMMAND_POOL_RESET_RELEASE_RESOURCES_BIT as u32 } else { 0 } 
            );
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanSynchroError::FAILED_RESETING_POOL);
            }
        }
        Ok(())
    }

    pub fn submit_buffers(&mut self,queue: vulkan_bindings::VkQueue,  wait_sems: &VulkanWaitSemaphoresInfo)-> Result<(), VulkanSynchroError>
    {
        let buffer = self.cmd_buffers.as_ref().unwrap();
        unsafe
        {
            let ref logical_device = *self.logical_device;
            let sig_sem = init_semaphore(logical_device)?;
            let buffer_submit_info =  vulkan_bindings::VkSubmitInfo
            {
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_SUBMIT_INFO,
                pNext: std::ptr::null(),
                waitSemaphoreCount: wait_sems.semaphores.len() as u32,
                pWaitSemaphores: wait_sems.semaphores.as_ptr(),
                pWaitDstStageMask: wait_sems.waiting_stage.as_ptr(),
                commandBufferCount: buffer.primary_buffers.len() as u32,
                pCommandBuffers: buffer.primary_buffers.as_ptr(),
                signalSemaphoreCount: 1,
                pSignalSemaphores: &sig_sem,
            };
            let fn_vkQueueSubmit = vulkan_init::vkQueueSubmit.unwrap();
            let result = fn_vkQueueSubmit(queue, 1, &buffer_submit_info, std::ptr::null_mut());
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanSynchroError::FAILED_SUBMITING_BUFFERS);
            }
        }
        Ok(())
    }

}

pub enum VulkanBufferType
{
    PRIMARY,
    SECONDARY
}

pub struct VulkanCmdBuffer
{
    cmd_pool : *const VulkanCmdPool,
    pub primary_buffers : Vec<vulkan_bindings::VkCommandBuffer>,
    pub secondary_buffers: Vec<vulkan_bindings::VkCommandBuffer>,
    started_buffers : Vec<(VulkanBufferType, usize)>
}

impl VulkanCmdBuffer
{
    pub fn new(cmd_pool: &VulkanCmdPool, primary_count: u32 , secondary_count: u32) -> Result<Self, VulkanSynchroError>
    {
        let mut cmd_buffer = VulkanCmdBuffer {
            cmd_pool,
            primary_buffers: Vec::new(),
            secondary_buffers : Vec::new(),
            started_buffers: Vec::new()
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
        self.started_buffers.push((VulkanBufferType::PRIMARY, buffer_idx));
        Ok(())
    }

    pub fn end_primary_buffer(&mut self, buffer_idx: usize) -> Result<(), VulkanSynchroError>
    {  
        let mut i = 0;
        while i < self.started_buffers.len()
        {
            let ref started =  self.started_buffers[i];
            if let VulkanBufferType::PRIMARY = started.0 
            {
                if started.1 == buffer_idx
                {
                    unsafe{
                        let fn_vkEndCommandBuffer = vulkan_init::vkEndCommandBuffer.unwrap();
                        let result = fn_vkEndCommandBuffer(self.primary_buffers[buffer_idx]);
                        if result != vulkan_bindings::VkResult_VK_SUCCESS
                        {
                            return Err(VulkanSynchroError::FAILED_ENDING_PRIMARY_BUFFER_RECORDING);
                        }
                    }
                    self.started_buffers.remove(i);
                    break;
                }
            }
            i += 1;
        }
        Ok(())
    }

    pub fn reset_primary_buffer(&mut self, idx: usize, release_mem: bool) -> Result<(), VulkanSynchroError>
    {
        if idx >= self.primary_buffers.len()
        {
            return Err(VulkanSynchroError::FAILED_RESETING_PRIMARY_BUFFER);
        }
        unsafe
        {
            let fn_vkResetCommandBuffer = vulkan_init::vkResetCommandBuffer.unwrap();
            let result = fn_vkResetCommandBuffer(self.primary_buffers[idx],
                if release_mem { vulkan_bindings::VkCommandBufferResetFlagBits_VK_COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT as u32 } else { 0 }
            );
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return  Err(VulkanSynchroError::FAILED_RESETING_PRIMARY_BUFFER);
            }
        }
        Ok(())
    }
    
}

pub struct VulkanWaitSemaphoresInfo
{
    pub semaphores : Vec<vulkan_bindings::VkSemaphore>,
    pub waiting_stage: Vec<vulkan_bindings::VkPipelineStageFlags>
}

pub fn init_semaphore(logical_device: &vulkan_init::VulkanLogicalDevice) -> Result<vulkan_bindings::VkSemaphore, VulkanSynchroError>
{
    let sem_create_info = vulkan_bindings::VkSemaphoreCreateInfo {
        sType : vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
        pNext : std::ptr::null(),
        flags: 0
    };
    unsafe{
        let fn_vkCreateSemaphore = vulkan_init::vkCreateSemaphore.unwrap();
        let logical_device = logical_device.device;
        let mut sem : vulkan_bindings::VkSemaphore = std::ptr::null_mut();
        let result = fn_vkCreateSemaphore(logical_device, &sem_create_info, std::ptr::null(), &mut sem);
        if result != vulkan_bindings::VkResult_VK_SUCCESS
        {
            return Err(VulkanSynchroError::FAILED_CREATING_SEMAPHORE);
        }
        Ok(sem)
    }
}

pub fn init_fence(logical_device: &vulkan_init::VulkanLogicalDevice) -> Result<vulkan_bindings::VkFence, VulkanSynchroError>
{
    let fence_create_info = vulkan_bindings::VkFenceCreateInfo{
        sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags:0
    };
    unsafe {
        let fn_vkCreateFence = vulkan_init::vkCreateFence.unwrap();
        let logical_device = logical_device.device;
        let mut fence: vulkan_bindings::VkFence = std::ptr::null_mut();
        let result = fn_vkCreateFence(logical_device, &fence_create_info, std::ptr::null(), &mut fence);
        if result != vulkan_bindings::VkResult_VK_SUCCESS
        {
            return Err(VulkanSynchroError::FAILED_CREATING_FENCE);
        }
        Ok(fence)
    }
}

pub fn wait_fences(logical_device: &vulkan_init::VulkanLogicalDevice, fences: &Vec<vulkan_bindings::VkFence>, wait_all: vulkan_bindings::VkBool32, timeout: u64) -> Result<(), VulkanSynchroError>
{
    unsafe
    {
        let logical_device = logical_device.device;
        let fn_vkWaitForFences = vulkan_init::vkWaitForFences.unwrap();
        let result = fn_vkWaitForFences(
            logical_device,
            fences.len() as u32,
            fences.as_ptr(),
            wait_all,
            timeout
        );
        if result !=  vulkan_bindings::VkResult_VK_SUCCESS
        {
            return Err(VulkanSynchroError::COULDNT_WAIT_FOR_FENCES);
        }
    }
    Ok(())
}

pub fn reset_fences(logical_device: &vulkan_init::VulkanLogicalDevice, fences: &Vec<vulkan_bindings::VkFence>) -> Result<(), VulkanSynchroError>
{
    unsafe
    {
        let logical_device = logical_device.device;
        let fn_vkResetFences = vulkan_init::vkResetFences.unwrap();
        let result = fn_vkResetFences(logical_device, fences.len() as u32, fences.as_ptr());
        if result != vulkan_bindings::VkResult_VK_SUCCESS
        {
            return Err(VulkanSynchroError::COULDNT_RESET_FENCES);
        }
    }
    Ok(())
}
