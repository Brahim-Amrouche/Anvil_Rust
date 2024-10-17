use crate::vulkan_bindings;
use crate::vulkan_init;

#[derive(Debug)]
pub enum VulkanMemError
{
    COULDNT_ALLOCATE_BUFFER,
    COULDNT_ALLOCATE_DEVICE_MEMORY,
    FAILED_CREATING_BUFFER_VIEW,
    COULDNT_ALLOCATE_IMAGE,
    COULDNT_BIND_IMAGE_MEMORY,
    FAILED_CREATING_IMAGE_VIEW,
    FAILED_GETTING_MEMORY_POINTER,
}

impl std::fmt::Display for VulkanMemError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanMemError::COULDNT_ALLOCATE_BUFFER => write!(f,"Couldn't allocate a buffer memory"),
            VulkanMemError::COULDNT_ALLOCATE_DEVICE_MEMORY => write!(f, "Couldn't allocate device memory for buffer"),
            VulkanMemError::FAILED_CREATING_BUFFER_VIEW => write!(f, "Failed creating buffer view"),
            VulkanMemError::COULDNT_ALLOCATE_IMAGE => write!(f, "Couldn't allocate an image"),
            VulkanMemError::COULDNT_BIND_IMAGE_MEMORY => write!(f, "Couldn't bind image memory"),
            VulkanMemError::FAILED_CREATING_IMAGE_VIEW => write!(f, "Failed creating image view"),
            VulkanMemError::FAILED_GETTING_MEMORY_POINTER => write!(f, "Failed getting memory pointer")
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

pub struct VulkanImageTransition
{
    pub image: vulkan_bindings::VkImage,
    pub current_access : vulkan_bindings::VkAccessFlags,
    pub new_access: vulkan_bindings::VkAccessFlags,
    pub current_layout: vulkan_bindings::VkImageLayout,
    pub new_layout : vulkan_bindings::VkImageLayout,
    pub current_fam_queue : u32,
    pub new_fam_queue: u32,
    pub aspect : vulkan_bindings::VkImageAspectFlags
}

pub struct VulkanDeviceMemory
{
    pub logical_device : *const vulkan_init::VulkanLogicalDevice,
    pub handle: vulkan_bindings::VkDeviceMemory,
    pub size: u64,
    pub properties:vulkan_bindings::VkMemoryPropertyFlagBits,
    pub data_region: *mut std::ffi::c_void
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
                logical_device,
                handle : std::ptr::null_mut(),
                size: mem_req.size,
                properties: mem_props,
                data_region: std::ptr::null_mut()
            };
            let ref ph_device = *logical_device.physical_device;
            let ref memory_properties = ph_device.mem_properties;
            let mut mem_type = 0;
            while mem_type < memory_properties.memoryTypeCount
            {
                let has_property = memory_properties.memoryTypes[mem_type as usize].propertyFlags & (new_memory.properties as u32);
                // println!("values of first condition {} & {} ", mem_req.memoryTypeBits , 1 << mem_type);
                // println!("first conditions {}", mem_req.memoryTypeBits & (1 << mem_type) != 0);
                // println!("values of second condition {} & {}", memory_properties.memoryTypes[mem_type as usize].propertyFlags, new_memory.properties as u32);
                // println!("second condition {}", (memory_properties.memoryTypes[mem_type as usize].propertyFlags & (new_memory.properties as u32)) == (new_memory.properties as u32));
                if mem_req.memoryTypeBits & (1 << mem_type) != 0 && has_property == (new_memory.properties as u32)
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

    pub fn load_data_region(&mut self) -> Result<(), VulkanMemError>
    {
        unsafe
        {
            let fn_vkMapMemory = vulkan_init::vkMapMemory.unwrap();
            let logical_device = (*self.logical_device).device;
            let result = fn_vkMapMemory(logical_device, self.handle, 0, self.size, 0, &mut self.data_region);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanMemError::FAILED_GETTING_MEMORY_POINTER);
            }
            Ok(())
        }
    }

    // pub fn map_data(&self, data: *mut std::ffi::c_void, size: usize)
    // {
    // }

    pub fn destroy(self)
    {
        unsafe
        {
            let fn_vkUnmapMemory = vulkan_init::vkUnmapMemory.unwrap();
            let logical_device = (*self.logical_device).device;
            fn_vkUnmapMemory(logical_device, self.handle);
        }
    }
}

pub struct VulkanBufferMem
{
    pub logical_device: *const vulkan_init::VulkanLogicalDevice,
    pub handle : vulkan_bindings::VkBuffer,
    pub size: u64,
    pub device_memory: Option<VulkanDeviceMemory>,
    pub buffer_view: vulkan_bindings::VkBufferView
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
            new_buffer.allocate_memory(&mem_req, vulkan_bindings::VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT)?;
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

    pub fn destroy(mut self)
    {
        match self.device_memory.take()
        {
            Some(d) => d.destroy(),
            None => ()
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
    device_memory: Option<VulkanDeviceMemory>,
    view: vulkan_bindings::VkImageView
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
                usage,
                device_memory:None,
                view: std::ptr::null_mut()
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
            let mem_req = new_image.load_memory_requirements();
            new_image.allocate_memory(mem_req, vulkan_bindings::VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT)?;
            Ok(new_image)
        }
    }

    pub fn load_memory_requirements(&self) -> vulkan_bindings::VkMemoryRequirements
    {
        unsafe{
            let fn_vkGetImageMemoryRequirements = vulkan_init::vkGetImageMemoryRequirements.unwrap();
            let mut mem_reqs : vulkan_bindings::VkMemoryRequirements= std::mem::zeroed();
            let logical_device = (*self.logical_device).device;
            fn_vkGetImageMemoryRequirements(logical_device, self.handle, &mut mem_reqs);
            mem_reqs
        }
    }

    pub fn allocate_memory(&mut self,
        mem_req : vulkan_bindings::VkMemoryRequirements,
        mem_props: vulkan_bindings::VkMemoryPropertyFlagBits
    ) -> Result<(), VulkanMemError>
    {
        unsafe
        {
            let ref logical_device = *self.logical_device;
            let device_memory = VulkanDeviceMemory::new(logical_device, &mem_req, mem_props)?;
            let fn_vkBindImageMemory = vulkan_init::vkBindImageMemory.unwrap();
            let result = fn_vkBindImageMemory(logical_device.device, self.handle, device_memory.handle, 0);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanMemError::COULDNT_BIND_IMAGE_MEMORY);
            }
            self.device_memory = Some(device_memory);
            Ok(())
        }
    }

    pub fn create_image_barrier(&mut self,
        transitions : Vec<VulkanImageTransition>,
        cmd_buffer: vulkan_bindings::VkCommandBuffer,
        generating_stages: vulkan_bindings::VkPipelineStageFlags,
        consuming_stages: vulkan_bindings::VkPipelineStageFlags
    )
    {
        let mut image_barriers : Vec<vulkan_bindings::VkImageMemoryBarrier> = Vec::with_capacity(transitions.len());
        for transition in transitions
        {
            image_barriers.push(vulkan_bindings::VkImageMemoryBarrier {
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
                pNext: std::ptr::null(),
                srcAccessMask: transition.current_access,
                dstAccessMask: transition.new_access,
                oldLayout: transition.current_layout,
                newLayout: transition.new_layout,
                srcQueueFamilyIndex: transition.current_fam_queue,
                dstQueueFamilyIndex: transition.new_fam_queue,
                image: transition.image,
                subresourceRange: vulkan_bindings::VkImageSubresourceRange{
                    aspectMask: transition.aspect,
                    baseMipLevel: 0,
                    levelCount: vulkan_bindings::VK_REMAINING_MIP_LEVELS as u32,
                    baseArrayLayer: 0,
                    layerCount: vulkan_bindings::VK_REMAINING_ARRAY_LAYERS as u32,
                }
            });
        }
        if image_barriers.len() > 0
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
                    0,
                    std::ptr::null(),
                    image_barriers.len() as u32,
                    image_barriers.as_ptr()
                );
            }
        }
    }

    pub fn create_image_view(&mut self, view_type:  vulkan_bindings::VkImageViewType, aspect: vulkan_bindings::VkImageAspectFlags) -> Result<(), VulkanMemError>
    {
        unsafe
        {
            let fn_vkCreateImageView = vulkan_init::vkCreateImageView.unwrap();
            let image_view_create_info = vulkan_bindings::VkImageViewCreateInfo{
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                pNext: std::ptr::null(),
                flags: 0,
                image: self.handle,
                viewType: view_type,
                format: self.format,
                components: std::mem::zeroed(),
                subresourceRange : vulkan_bindings::VkImageSubresourceRange { 
                    aspectMask: aspect,
                    baseMipLevel: 0,
                    levelCount: vulkan_bindings::VK_REMAINING_MIP_LEVELS as u32,
                    baseArrayLayer: 0,
                    layerCount: vulkan_bindings::VK_REMAINING_ARRAY_LAYERS as u32
                }
            };
            let logical_device = (*self.logical_device).device;
            let result = fn_vkCreateImageView(logical_device, &image_view_create_info, std::ptr::null(), &mut self.view);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanMemError::FAILED_CREATING_IMAGE_VIEW);
            }
            Ok(())
        }
    }

    pub fn destroy(mut self)
    {
        match self.device_memory.take()
        {
            Some(d) => d.destroy(),
            None => ()
        }
    }
}

