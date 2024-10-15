use crate::vulkan_bindings;
use crate::vulkan_init;

#[derive(Debug)]
pub enum VulkanMemError
{
    COULDNT_ALLOCATE_BUFFER,
    COULDNT_ALLOCATE_DEVICE_MEMORY,
    FAILED_CREATING_BUFFER_VIEW,
    COULDNT_ALLOCATE_IMAGE
}

impl std::fmt::Display for VulkanMemError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanMemError::COULDNT_ALLOCATE_BUFFER => write!(f,"Couldn't allocate a buffer memory"),
            VulkanMemError::COULDNT_ALLOCATE_DEVICE_MEMORY => write!(f, "Couldn't allocate device memory for buffer"),
            VulkanMemError::FAILED_CREATING_BUFFER_VIEW => write!(f, "Failed creating buffer view"),
            VulkanMemError::COULDNT_ALLOCATE_IMAGE => write!(f, "Couldn't allocat an image")
        }
    }
}

impl std::error::Error for VulkanMemError {}

pub struct VulkanBufferTransition
{
    pub buffer: vulkan_bindings::VkBuffer,
    pub current_access: vulkan_bindings::VkAccessFlags,
    pub new_access: vulkan_bindings::VkAccessFlags,
    pub current_fam_queue: u32,
    pub new_fam_queue : u32
}

pub struct VulkanDeviceMemory
{
    pub handle: vulkan_bindings::VkDeviceMemory,
    pub size: u64,
    pub properties:vulkan_bindings::VkMemoryPropertyFlagBits
}

impl VulkanDeviceMemory
{
    pub fn new(
        logical_device: &vulkan_init::VulkanLogicalDevice,
        mem_req: &vulkan_bindings::VkMemoryRequirements,
        mem_props: vulkan_bindings::VkMemoryPropertyFlagBits
    ) -> Result< Self, VulkanMemError>
    {
        unsafe
        {
            let mut new_memory = VulkanDeviceMemory {
                handle : std::ptr::null_mut(),
                size: mem_req.size,
                properties: mem_props,
            };
            let ref ph_device = *logical_device.physical_device;
            let ref memory_properties = ph_device.mem_properties;
            let mut mem_type = 0;
            while mem_type < memory_properties.memoryTypeCount
            {
                if mem_req.memoryTypeBits & (1 << mem_type) != 0 &&
                memory_properties.memoryTypes[mem_type as usize].propertyFlags & (new_memory.properties as u32) != 1
                {
                    let fn_vkAllocateMemory = vulkan_init::vkAllocateMemory.unwrap();
                    let allocation_info = vulkan_bindings::VkMemoryAllocateInfo {
                        sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                        pNext: std::ptr::null(),
                        allocationSize: new_memory.size,
                        memoryTypeIndex: mem_type
                    };
                    let result = fn_vkAllocateMemory(
                        logical_device.device,
                        &allocation_info,
                        std::ptr::null(),
                        &mut new_memory.handle
                    );
                    if result != vulkan_bindings::VkResult_VK_SUCCESS
                    {
                        return Err(VulkanMemError::COULDNT_ALLOCATE_DEVICE_MEMORY);
                    }
                    break;
                }
                mem_type += 1;
            }
            if new_memory.handle == std::ptr::null_mut()
            {
                return Err(VulkanMemError::COULDNT_ALLOCATE_DEVICE_MEMORY);
            }
            Ok(new_memory)
        }   
    }
}

pub struct VulkanBufferMem
{
    logical_device: *const vulkan_init::VulkanLogicalDevice,
    handle : vulkan_bindings::VkBuffer,
    size: u64,
    device_memory: Option<VulkanDeviceMemory>,
    buffer_view: vulkan_bindings::VkBufferView
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
                handle: std::ptr::null_mut(),
                size,
                device_memory: None,
                buffer_view: std::ptr::null_mut()
            };
            let fn_vkCreateBuffer = vulkan_init::vkCreateBuffer.unwrap();
            let result = fn_vkCreateBuffer(logical_device.device, &buffer_create_info, std::ptr::null(), &mut new_buffer.handle);
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
            fn_vkGetBufferMemoryRequirements(logical_device, self.handle, &mut mem_req);
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
            let device_memory = VulkanDeviceMemory::new(logical_device, mem_req, mem_props)?;
            let fn_vkBindBufferMemory = vulkan_init::vkBindBufferMemory.unwrap();
            let result = fn_vkBindBufferMemory(logical_device.device, self.handle, device_memory.handle, 0);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanMemError::COULDNT_ALLOCATE_DEVICE_MEMORY);
            }
            self.device_memory = Some(device_memory);
            Ok(())
        }
    }

    pub fn create_buffers_barriers(
        transitions: Vec<VulkanBufferTransition>,
        cmd_buffer: vulkan_bindings::VkCommandBuffer,
        generating_stages: vulkan_bindings::VkPipelineStageFlags,
        consuming_stages: vulkan_bindings::VkPipelineStageFlags
    ) -> Result<(), VulkanMemError>
    {
        let mut buffers_mem_barriers :Vec<vulkan_bindings::VkBufferMemoryBarrier>  = Vec::with_capacity(transitions.len());
        for transition in transitions.into_iter()
        {
            buffers_mem_barriers.push(vulkan_bindings::VkBufferMemoryBarrier{
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER,
                pNext: std::ptr::null(),
                buffer: transition.buffer,
                srcAccessMask: transition.current_access,
                dstAccessMask: transition.new_access,
                srcQueueFamilyIndex: transition.current_fam_queue,
                dstQueueFamilyIndex: transition.new_fam_queue,
                offset: 0,
                size: vulkan_bindings::VK_WHOLE_SIZE as u64
            });
        }
        if buffers_mem_barriers.len() > 0
        {
            unsafe
            {
                let fn_vkCmdPipelineBarrier = vulkan_init::vkCmdPipelineBarrier.unwrap();
                fn_vkCmdPipelineBarrier(
                    cmd_buffer,
                    generating_stages,
                    consuming_stages, 
                    0, 
                    0, 
                    std::ptr::null(),
                    buffers_mem_barriers.len() as u32,
                    buffers_mem_barriers.as_ptr(),
                    0,
                    std::ptr::null()
                );
            }
        }
        Ok(())
    }

    pub fn create_buffer_view(&mut self, format: vulkan_bindings::VkFormat) -> Result<(), VulkanMemError>
    {
        unsafe
        {
            let fn_vkCreateBufferView = vulkan_init::vkCreateBufferView.unwrap();
            let ref logical_device = *self.logical_device;
            let view_create_info = vulkan_bindings::VkBufferViewCreateInfo
            {
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO,
                pNext: std::ptr::null(),
                flags: 0,
                buffer: self.handle,
                format,
                offset: 0,
                range: self.size
            };
            let result = fn_vkCreateBufferView(logical_device.device, &view_create_info, std::ptr::null(), &mut self.buffer_view);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            || self.buffer_view == std::ptr::null_mut()
            {
                return Err(VulkanMemError::FAILED_CREATING_BUFFER_VIEW);
            }
            Ok(())
        }
    }
}


pub struct VulkanImageMem
{
    logical_device: *const vulkan_init::VulkanLogicalDevice,
    handle : vulkan_bindings::VkImage,
    img_type : vulkan_bindings::VkImageType,
    format : vulkan_bindings::VkFormat,
    dimensions: vulkan_bindings::VkExtent3D,
    mipmap_lvl : u32,
    layer_num : u32,
    sample_count: vulkan_bindings::VkSampleCountFlagBits,
    usage: vulkan_bindings::VkImageUsageFlags,
}


impl VulkanImageMem
{
    pub fn new(
        logical_device: &vulkan_init::VulkanLogicalDevice,
        img_type : vulkan_bindings::VkImageType,
        format : vulkan_bindings::VkFormat,
        dimensions: vulkan_bindings::VkExtent3D,
        mipmap_lvl : u32,
        layer_num : u32,
        sample_count: vulkan_bindings::VkSampleCountFlagBits,
        usage: vulkan_bindings::VkImageUsageFlags,
    ) -> Result<Self, VulkanMemError>
    {
        unsafe
        {
            let fn_vkCreateImage = vulkan_init::vkCreateImage.unwrap();
            let mut new_image = VulkanImageMem {
                logical_device,
                handle: std::ptr::null_mut(),
                img_type,
                format,
                dimensions,
                mipmap_lvl,
                layer_num,
                sample_count,
                usage
            };
            let image_creation_info = vulkan_bindings::VkImageCreateInfo {
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
                pNext: std::ptr::null(),
                flags: 0,
                imageType: new_image.img_type,
                format: new_image.format,
                extent: new_image.dimensions,
                mipLevels: new_image.mipmap_lvl,
                arrayLayers: new_image.layer_num,
                samples: new_image.sample_count,
                tiling: vulkan_bindings::VkImageTiling_VK_IMAGE_TILING_OPTIMAL,
                usage: new_image.usage,
                sharingMode: vulkan_bindings::VkSharingMode_VK_SHARING_MODE_EXCLUSIVE,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: std::ptr::null(),
                initialLayout: vulkan_bindings::VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED
            };
            let result = fn_vkCreateImage(logical_device.device, &image_creation_info, std::ptr::null(), &mut new_image.handle);
            if result !=  vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanMemError::COULDNT_ALLOCATE_IMAGE);
            }
            Ok(new_image)
        }
    }
}

