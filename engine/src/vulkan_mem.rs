use crate::vulkan_bindings;
use crate::vulkan_init;

#[derive(Debug)]
pub enum VulkanMemError
{
    COULDNT_ALLOCATE_BUFFER,
    COULDNT_ALLOCATE_DEVICE_MEMORY
}

impl std::fmt::Display for VulkanMemError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanMemError::COULDNT_ALLOCATE_BUFFER => write!(f,"Couldn't allocate a buffer memory"),
            VulkanMemError::COULDNT_ALLOCATE_DEVICE_MEMORY => write!(f, "Couldn't allocate device memory for buffer")
        }
    }
}

impl std::error::Error for VulkanMemError {}

pub struct VulkanBufferMem
{
    logical_device: *const vulkan_init::VulkanLogicalDevice,
    buffer_handle : vulkan_bindings::VkBuffer,
    size: u64,
    usage: vulkan_bindings::VkBufferUsageFlags,
    device_memory: vulkan_bindings::VkDeviceMemory
}

impl VulkanBufferMem
{
    pub fn new(logical_device : &vulkan_init::VulkanLogicalDevice, size:u64, usage:vulkan_bindings::VkBufferUsageFlags) -> Result<Self, VulkanMemError>
    {
        let buffer_create_info = vulkan_bindings::VkBufferCreateInfo {
            sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: std::ptr::null(),
            flags: 0,
            size,
            usage,
            sharingMode: vulkan_bindings::VkSharingMode_VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: std::ptr::null()
        };
        unsafe {
            let mut new_buffer = VulkanBufferMem {
                logical_device,
                buffer_handle: std::ptr::null_mut(),
                size,
                usage,
                device_memory: std::ptr::null_mut()
            };
            let fn_vkCreateBuffer = vulkan_init::vkCreateBuffer.unwrap();
            let result = fn_vkCreateBuffer(logical_device.device, &buffer_create_info, std::ptr::null(), &mut new_buffer.buffer_handle);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanMemError::COULDNT_ALLOCATE_BUFFER);
            }
            let mem_req = new_buffer.load_memory_requirements();
            new_buffer.allocate_memory(&mem_req, 0)?;
            Ok(new_buffer)
        }
    }

    pub fn load_memory_requirements(&mut self) -> vulkan_bindings::VkMemoryRequirements
    {
        unsafe
        {
            let mut mem_req = std::mem::zeroed();
            let fn_vkGetBufferMemoryRequirements = vulkan_init::vkGetBufferMemoryRequirements.unwrap();
            let logical_device = (*self.logical_device).device;
            fn_vkGetBufferMemoryRequirements(logical_device, self.buffer_handle, &mut mem_req);
            mem_req
        }
    }

    pub fn allocate_memory(&mut self,
        mem_req: &vulkan_bindings::VkMemoryRequirements,
        mem_props: vulkan_bindings::VkMemoryPropertyFlagBits
    ) -> Result<(), VulkanMemError>
    {
        unsafe
        {
            let ref logical_device = *self.logical_device;
            let ref ph_device = *logical_device.physical_device;
            let ref memory_properties = ph_device.mem_properties;
            let mut mem_type = 0;
            while mem_type < memory_properties.memoryTypeCount
            {
                if mem_req.memoryTypeBits & (1 << mem_type) != 0 &&
                memory_properties.memoryTypes[mem_type as usize].propertyFlags & (mem_props as u32) != 1
                {
                    let fn_vkAllocateMemory = vulkan_init::vkAllocateMemory.unwrap();
                    let allocation_info = vulkan_bindings::VkMemoryAllocateInfo {
                        sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                        pNext: std::ptr::null(),
                        allocationSize: mem_req.size,
                        memoryTypeIndex: mem_type
                    };
                    let result = fn_vkAllocateMemory(
                        logical_device.device,
                        &allocation_info,
                        std::ptr::null(),
                        &mut self.device_memory
                    );
                    if result != vulkan_bindings::VkResult_VK_SUCCESS || self.device_memory == std::ptr::null_mut()
                    {
                        return Err(VulkanMemError::COULDNT_ALLOCATE_DEVICE_MEMORY);
                    }
                    break;
                }
                mem_type += 1;
            }
            if self.device_memory == std::ptr::null_mut()
            {
                return Err(VulkanMemError::COULDNT_ALLOCATE_DEVICE_MEMORY);
            }
            let fn_vkBindBufferMemory = vulkan_init::vkBindBufferMemory.unwrap();
            let result = fn_vkBindBufferMemory(logical_device.device, self.buffer_handle, self.device_memory, 0);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanMemError::COULDNT_ALLOCATE_DEVICE_MEMORY);
            }
            Ok(())
        }
    }
}
