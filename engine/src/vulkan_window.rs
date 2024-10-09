use crate::vulkan_init;
use crate::vulkan_bindings;
use crate::system_window;

#[derive(Debug)]
pub enum VulkanWindowError
{
    DEFAULT_ERROR(String),
    CANT_LOAD_VULKAN_SURFACE,
    CANT_LOAD_SURFACE_CAPABILITIES,
    UNSUPPORTED_IMAGE_USAGE,
    CANT_LOAD_SURFACE_FORMATS,
    FAILED_CREATING_SWAPCHAIN,
    CANT_LOAD_SWAPCHAIN_IMAGE,
    UNUSABLE_SWAPCHAIN
}

impl std::fmt::Display for VulkanWindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            VulkanWindowError::DEFAULT_ERROR(s) => write!(f, "{}", s),
            VulkanWindowError::CANT_LOAD_VULKAN_SURFACE => write!(f, "Couldn't a vulkan surface"),
            VulkanWindowError::CANT_LOAD_SURFACE_CAPABILITIES => write!(f, "Couldn't load surface Capabilities"),
            VulkanWindowError::UNSUPPORTED_IMAGE_USAGE => write!(f, "Unsupported swapchain image usage"),
            VulkanWindowError::CANT_LOAD_SURFACE_FORMATS => write!(f, "Couldn't load surface formats"),
            VulkanWindowError::FAILED_CREATING_SWAPCHAIN => write!(f, "Couldn't create swapchain"),
            VulkanWindowError::CANT_LOAD_SWAPCHAIN_IMAGE => write!(f, "Couldn't load swapchain image"),
            VulkanWindowError::UNUSABLE_SWAPCHAIN => write!(f, "")
        }
    }
}

impl From<vulkan_init::VulkanInitError> for VulkanWindowError
{
    fn from(value: vulkan_init::VulkanInitError) -> Self {
        VulkanWindowError::DEFAULT_ERROR(value.to_string())
    }
}

impl std::error::Error for VulkanWindowError {}


pub fn load_extension_names(extensions: &[&[u8]]) -> Vec<String>
{
    let mut desired_extensions :Vec<String> = Vec::with_capacity(extensions.len());
    for ext in extensions
    {
        desired_extensions.push(String::from_utf8(ext.to_vec()).unwrap().trim_end_matches('\0').to_string());
    }
    desired_extensions
}

pub struct VulkanSurface {
    pub window : system_window::WindowParameters,
    pub surface : vulkan_bindings::VkSurfaceKHR,
    pub logical_device: *const vulkan_init::VulkanLogicalDevice,
    pub capabilites : vulkan_bindings::VkSurfaceCapabilitiesKHR,
    pub swapchain_images_count : u32,
    pub swapchain_image_size: vulkan_bindings::VkExtent2D,
    pub swapchain_image_usage: vulkan_bindings::VkImageUsageFlags,
    pub swapchain_image_transform: vulkan_bindings::VkSurfaceTransformFlagsKHR,
    pub surface_format: vulkan_bindings::VkSurfaceFormatKHR,
    pub swapchain: Option<VulkanSwapchain>
}

impl VulkanSurface {
    pub fn new(vk_instance: &vulkan_init::VulkanInstance) -> Result<Self, VulkanWindowError>
    {
        unsafe {
            let mut vk_surface = VulkanSurface {
                window : system_window::WindowParameters::new("Anvil".to_string()),
                surface: std::ptr::null_mut(),
                logical_device: std::ptr::null(),
                capabilites: std::mem::zeroed(),
                swapchain_images_count : 0,
                swapchain_image_size: std::mem::zeroed(),
                swapchain_image_usage: 0,
                swapchain_image_transform : 0,
                surface_format: std::mem::zeroed(),
                swapchain: None
            };
            let vk_surface_create_info = vulkan_bindings::VkWin32SurfaceCreateInfoKHR {
                sType : vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
                pNext : std::ptr::null(),
                flags: 0,
                hinstance: vk_surface.window.Hinstance,
                hwnd: vk_surface.window.Hwnd
            };
            let fn_vkCreateWin32SurfaceKHR = vulkan_init::vkCreateWin32SurfaceKHR.unwrap();
            let result = fn_vkCreateWin32SurfaceKHR(vk_instance.instance, &vk_surface_create_info, std::ptr::null(), &mut vk_surface.surface);
            if result != vulkan_bindings::VkResult_VK_SUCCESS || vk_surface.surface == std::ptr::null_mut()
            {
                return Err(VulkanWindowError::CANT_LOAD_VULKAN_SURFACE);
            }
            Ok(vk_surface)
        }
    }

    pub fn load_surface_capabilities(&mut self) -> Result<(), VulkanWindowError>
    {
        unsafe
        {
            let fn_vkGetPhysicalDeviceSurfaceCapabilitiesKHR = vulkan_init::vkGetPhysicalDeviceSurfaceCapabilitiesKHR.unwrap();
            let ref physical_device =  *(*self.logical_device).physical_device;
            let physical_device = physical_device.ph_device;
            let result = fn_vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, self.surface, &mut self.capabilites);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanWindowError::CANT_LOAD_SURFACE_CAPABILITIES)
            }
        }
        Ok(())
    }

    pub fn set_swapchain_image_count(&mut self, desired_count: u32)
    {
        if self.capabilites.maxImageCount > 0
        && desired_count > self.capabilites.maxImageCount
        {
            self.swapchain_images_count = self.capabilites.maxImageCount;
        }
        else
        {
            self.swapchain_images_count = desired_count;
        }
    }

    pub fn set_swapchain_image_size(&mut self)
    {
        let ref capabilities = self.capabilites;
        if capabilities.currentExtent.width == (0xFFFFFFFF as u32)
        {
            let (width, height) = (system_window::DISPLAY_WIDTH as u32, system_window::DISPLAY_HEIGHT as u32);
            self.swapchain_image_size.width = match width
            {
                w if w < capabilities.minImageExtent.width => capabilities.minImageExtent.width,
                w if w > capabilities.maxImageExtent.width => capabilities.maxImageExtent.width,
                _ => width
            };
            self.swapchain_image_size.height = match height 
            {
                h if h < capabilities.minImageExtent.height => capabilities.minImageExtent.height,
                h if h > capabilities.maxImageExtent.height => capabilities.maxImageExtent.height,
                _ => height
            };
        }
        else
        {
            self.swapchain_image_size = capabilities.currentExtent;
        }
    }


    pub fn set_swapchain_image_usage(&mut self, desired_usage: vulkan_bindings::VkImageUsageFlags) -> Result<(), VulkanWindowError>
    {
        match desired_usage & self.capabilites.supportedUsageFlags
        {
            usage if usage == desired_usage => 
            {
                self.swapchain_image_usage = usage;
                Ok(())
            },
            _ => Err(VulkanWindowError::UNSUPPORTED_IMAGE_USAGE)
        }
    }

    pub fn set_swapchain_image_transform(&mut self, transform : vulkan_bindings::VkSurfaceTransformFlagsKHR)
    {
        if (transform & self.capabilites.supportedTransforms) == transform 
        {
            self.swapchain_image_transform = transform;
        }
        else
        {
            self.swapchain_image_transform = self.capabilites.currentTransform as  u32;
        }
    }

    pub fn load_surface_formats(& self) -> Result<Vec<vulkan_bindings::VkSurfaceFormatKHR>, VulkanWindowError>
    {
        unsafe
        {
            let mut formats_count:u32 = 0;
            let fn_vkGetPhysicalDeviceSurfaceFormatsKHR = vulkan_init::vkGetPhysicalDeviceSurfaceFormatsKHR.unwrap();
            let physical_device = (*(*self.logical_device).physical_device).ph_device;
            let result = fn_vkGetPhysicalDeviceSurfaceFormatsKHR(physical_device, self.surface, &mut formats_count, std::ptr::null_mut());
            if result != vulkan_bindings::VkResult_VK_SUCCESS || formats_count == 0
            {
                return Err(VulkanWindowError::CANT_LOAD_SURFACE_FORMATS);
            }
            let mut available_surface_format: Vec<vulkan_bindings::VkSurfaceFormatKHR> = vec![std::mem::zeroed(); formats_count as usize]; 
            let result = fn_vkGetPhysicalDeviceSurfaceFormatsKHR(physical_device, self.surface, &mut formats_count , available_surface_format.as_mut_ptr());
            if result != vulkan_bindings::VkResult_VK_SUCCESS || formats_count == 0
            {
                return Err(VulkanWindowError::CANT_LOAD_SURFACE_FORMATS);
            }
            Ok(available_surface_format)
        }
    }

    pub fn set_surface_format(&mut self, desired_format: &vulkan_bindings::VkSurfaceFormatKHR) -> Result<() ,VulkanWindowError>
    {
        let available_surface_formats = self.load_surface_formats()?;
        if available_surface_formats.len() == 1 
            && available_surface_formats[0].format == vulkan_bindings::VkFormat_VK_FORMAT_UNDEFINED
        {
            self.surface_format = *desired_format;
            return Ok(());
        }
        for surface_format in available_surface_formats.iter()
        {
            if surface_format.format == desired_format.format
            && surface_format.colorSpace == desired_format.colorSpace
            {
                self.surface_format = *desired_format;
                println!("{:?}", self.surface_format);
                return Ok(())
            }
        }
        for surface_format in available_surface_formats.iter()
        {
            if surface_format.format == desired_format.format
            {
                self.surface_format = *surface_format;
                println!("Couldnt choose the desired color space ...");
                return Ok(());
            }
        }
        self.surface_format = available_surface_formats[0];
        println!("Couldnt choose the desired format and color space; defaulting ...");
        Ok(())
    }

    pub fn configure_swapchain(&mut self,
        logical_device: &vulkan_init::VulkanLogicalDevice,
        image_count : u32,
        image_usage: vulkan_bindings::VkImageUsageFlags,
        image_transform : vulkan_bindings::VkSurfaceTransformFlagsKHR,
        image_format: &vulkan_bindings::VkSurfaceFormatKHR,
    ) -> Result<(), VulkanWindowError>
    {
        self.logical_device = logical_device;
        self.load_surface_capabilities()?;
        self.set_swapchain_image_count(image_count);
        self.set_swapchain_image_size();
        self.set_swapchain_image_usage(image_usage)?;
        self.set_swapchain_image_transform(image_transform);
        self.set_surface_format( image_format)?;
        self.create_swapchain()?;
        Ok(())
    }

    pub fn create_swapchain(&mut self) -> Result<(), VulkanWindowError>
    {
        println!("creating a swapchain");
        match self.swapchain.take() {
            Some(s) => 
            {
                s.destroy();
                self.swapchain = Some(VulkanSwapchain::new(self)?);
            },
            None =>{
                self.swapchain = Some(VulkanSwapchain::new(self)?);
            }
        };
        Ok(())
    }

    pub fn acquire_next_image(&mut self) -> Result<(), VulkanWindowError>
    {
        if let Some(swapchain_ref) = self.swapchain.as_mut()
        {
            match swapchain_ref.get_next_image()
            {
                Ok(_) => return Ok(()),
                Err(_) => ()
            };
        }
        self.create_swapchain()?;
        self.acquire_next_image()
    }

    pub fn destroy(mut self)
    {
        self.window.destroy();
        match self.swapchain.take()
        {
            Some(s) => s.destroy(),
            None => {}
        }
        unsafe{
            let fn_vkDestroySurfaceKHR = vulkan_init::vkDestroySurfaceKHR.unwrap();
            let vk_instance = vulkan_init::VULKAN_INSTANCE.as_ref().unwrap();
            fn_vkDestroySurfaceKHR( vk_instance.instance, self.surface, std::ptr::null());
        }
    }
}

pub struct VulkanSemaphore
{
    pub sem : vulkan_bindings::VkSemaphore,

}
//swapchain instance
pub struct VulkanSwapchain
{
    pub surface: *const VulkanSurface,
    pub swapchain_handle : vulkan_bindings::VkSwapchainKHR,
    pub swapchain_images : Vec<vulkan_bindings::VkImage>,
    pub images_sem: vulkan_bindings::VkSemaphore,
    pub images_fence: vulkan_bindings::VkFence,
    pub presentable_img_idx: u32,
}

impl VulkanSwapchain
{
    pub fn new(vk_surface:& VulkanSurface) -> Result<Self, VulkanWindowError>
    {
        let mut swapchain = VulkanSwapchain {
            surface : vk_surface,
            swapchain_handle: std::ptr::null_mut(),
            swapchain_images: Vec::new(),
            images_sem: std::ptr::null_mut(),
            images_fence: std::ptr::null_mut(),
            presentable_img_idx: 0
        };
        unsafe
        {
            let swapchain_create_info = vulkan_bindings::VkSwapchainCreateInfoKHR {
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
                pNext: std::ptr::null(),
                flags:0,
                surface: vk_surface.surface,
                minImageCount: vk_surface.swapchain_images_count,
                imageFormat:  vk_surface.surface_format.format,
                imageColorSpace : vk_surface.surface_format.colorSpace,
                imageExtent: vk_surface.swapchain_image_size,
                imageArrayLayers: 1,
                imageUsage: vk_surface.swapchain_image_usage,
                imageSharingMode: vulkan_bindings::VkSharingMode_VK_SHARING_MODE_EXCLUSIVE,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: std::ptr::null(),
                preTransform: vk_surface.swapchain_image_transform as i32,
                compositeAlpha: vulkan_bindings::VkCompositeAlphaFlagBitsKHR_VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
                presentMode: (*vk_surface.logical_device).presentation_mode,
                clipped: vulkan_bindings::VK_TRUE,
                oldSwapchain: match &vk_surface.swapchain { Some(s) => s.swapchain_handle , None => std::ptr::null_mut()}
            };
            let fn_vkCreateSwapchainKHR =  vulkan_init::vkCreateSwapchainKHR.unwrap();
            let result = fn_vkCreateSwapchainKHR((*vk_surface.logical_device).device,
                &swapchain_create_info, std::ptr::null(), &mut swapchain.swapchain_handle);
            if result != vulkan_bindings::VkResult_VK_SUCCESS
            {
                return Err(VulkanWindowError::FAILED_CREATING_SWAPCHAIN);
            }
            swapchain.load_swap_chain_images()?;
        }
        Ok(swapchain)
    }

    pub fn load_swap_chain_images(&mut self) -> Result< (), VulkanWindowError>
    {
        unsafe
        {
            let fn_vkGetSwapchainImagesKHR =  vulkan_init::vkGetSwapchainImagesKHR.unwrap();
            let mut image_count = 0;
            let ref logical_device = *(*self.surface).logical_device;
            let result = fn_vkGetSwapchainImagesKHR(logical_device.device, self.swapchain_handle,&mut image_count, std::ptr::null_mut());
            if result != vulkan_bindings::VkResult_VK_SUCCESS || image_count == 0
            {
                return Err(VulkanWindowError::CANT_LOAD_SWAPCHAIN_IMAGE);
            }
            self.swapchain_images.resize(image_count as usize, std::mem::zeroed());
            let result = fn_vkGetSwapchainImagesKHR(logical_device.device,
                self.swapchain_handle, &mut image_count, self.swapchain_images.as_mut_ptr());
            if result != vulkan_bindings::VkResult_VK_SUCCESS || image_count == 0
            {
                return Err(VulkanWindowError::CANT_LOAD_SWAPCHAIN_IMAGE);
            }
        }
        Ok(())
    }

    pub fn get_next_image(&mut self) -> Result<(), VulkanWindowError>
    {
        unsafe {
            let ref logical_device = *(*self.surface).logical_device;
            self.images_sem = vulkan_init::init_semaphore(logical_device)?;
            self.images_fence = vulkan_init::init_fence(logical_device)?;
            let fn_vkAcquireNextImageKHR = vulkan_init::vkAcquireNextImageKHR.unwrap();
            let logical_device = (*(*self.surface).logical_device).device;
            let result = fn_vkAcquireNextImageKHR(logical_device, self.swapchain_handle, 2000000000, self.images_sem, self.images_fence, &mut self.presentable_img_idx);
            match result
            {
                vulkan_bindings::VkResult_VK_SUCCESS | vulkan_bindings::VkResult_VK_SUBOPTIMAL_KHR => (),
                _ => return Err(VulkanWindowError::UNUSABLE_SWAPCHAIN)
            };
            // println!("the idx is {}", self.presentable_img_idx);
        }
        Ok(())
    }

    pub fn destroy(self)
    {
        unsafe
        {
            let fn_vkDestroySwapchainKHR = vulkan_init::vkDestroySwapchainKHR.unwrap();
            let ref logical_device = *(*self.surface).logical_device;
            fn_vkDestroySwapchainKHR(logical_device.device, self.swapchain_handle, std::ptr::null());
        }
    }
}

pub fn vulkan_init_window()
{
    let global_exts = load_extension_names(&[vulkan_bindings::VK_KHR_SURFACE_EXTENSION_NAME, vulkan_bindings::VK_KHR_WIN32_SURFACE_EXTENSION_NAME]);
    let mut vk_instance = vulkan_init::initialize_vulkan(global_exts);
    let mut vk_surface = VulkanSurface::new(vk_instance).unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    let device_exts = load_extension_names(&[vulkan_bindings::VK_KHR_SWAPCHAIN_EXTENSION_NAME]);
    let logical_device;
    logical_device = vulkan_init::VulkanLogicalDevice::new(
        &mut vk_instance,
        device_exts, 
        &[(vulkan_bindings::VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT | vulkan_bindings::VkQueueFlagBits_VK_QUEUE_COMPUTE_BIT) as u32],
        &vk_surface.surface,
        vulkan_bindings::VkPresentModeKHR_VK_PRESENT_MODE_MAILBOX_KHR
    ).unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    let desired_surface_format = vulkan_bindings::VkSurfaceFormatKHR {
        format : vulkan_bindings::VkFormat_VK_FORMAT_R8G8B8A8_UNORM,
        colorSpace: vulkan_bindings::VkColorSpaceKHR_VK_COLOR_SPACE_SRGB_NONLINEAR_KHR
    };
    vk_surface.configure_swapchain(&logical_device, 
        3,
        (vulkan_bindings::VkImageUsageFlagBits_VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT) as u32,
        vulkan_bindings::VkSurfaceTransformFlagBitsKHR_VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR as u32,
        &desired_surface_format
    ).unwrap_or_else(|e| {
            eprintln!("{}",e);
            std::process::exit(1);
    });
    vk_surface.acquire_next_image().unwrap_or_else(|e| {
        eprintln!("{}",e);
        std::process::exit(1);
    });
    vk_surface.destroy();
    logical_device.destroy();
    vk_instance.destroy();
}