use paste::paste;
use std::ffi::CString;
use crate::vulkan_bindings;

static mut VULKAN_INSTANCE:Option<VulkanInstance>= None;

macro_rules! EXPORTED_VULKAN_FUNCTION {
    ($name: ident) => {
        paste! {
        #[allow(non_upper_case_globals)]
        pub static mut $name :  vulkan_bindings::[<PFN_$name>] = None;
        }
    };
}

macro_rules! VK_MAKE_API_VERSION {
    ($variant:expr, $major:expr, $minor:expr, $patch:expr) => {
        ((($variant as u32) << 29) |
            (($major as u32) << 22) |
            (($minor as u32) << 12) |
            ($patch as u32))
    };
}

macro_rules! DEFAULTED_IMPORT_MACROS {
    () => {
        #[allow(unused_macros)]
        macro_rules! LOAD_EXPORTED_VULKAN_FUNCTION {($func: ident) => {}}
        #[allow(unused_macros)]
        macro_rules!  LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION {($func: ident) => {}}
        #[allow(unused_macros)]
        macro_rules!  LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION {($func: ident) => {}}
        #[allow(unused_macros)]
        macro_rules! LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION_FROM_EXTENSIONS {($function: ident, $ext: ident) => {}}
        #[allow(unused_macros)]
        macro_rules! LOAD_DEVICE_LEVEL_VULKAN_FUNCTION {($func: ident) => {}}
        #[allow(unused_macros)]
        macro_rules!  LOAD_DEVICE_LEVEL_VULKAN_FUNCTION_FROM_EXTENSION {($func: ident, $ext:ident) => {}}
    };
}

include!("./imported_functions.rs");

#[derive(Debug)]
pub enum VulkanInitError {
    UNLOADABLE_LIBRARY,
    EXPORTED_VK_FUNCTION_ERROR(String),
    GLOBAL_VK_FUNCTION_ERROR(String),
    UNLOADABLE_EXTENSIONS,
    UNAVAILABLE_EXTENSION(String),
    FAILED_INSTANTIATING_VULKAN,
    INSTANCE_VK_FUNCTION_ERROR(String),
    INSTANCE_VK_EXT_FUNCTION_ERROR(String),
    UNAVAILABLE_VULKAN_PHYSICAL_DEVICES,
    UNAVAILABLE_PHYSICAL_DEVICE_EXTENSIONS,
    FAILED_TO_LIST_FAMILY_QUEUES,
    NO_VALID_DESIRED_FAMILY_QUEUE,
    UNAVAILABLE_DESIRED_PHYSICAL_DEVICE_EXTENSION(String),
    FAILED_INSTANTIATING_LOGICAL_DEVICE,
    DEVICE_LEVEL_FUNCTION_ERROR(String),
    WRONG_DEVICE_QUEUE_INDEX,
    NO_CAPABLE_PHYSICAL_DEVICE,
    UNAVAILABLE_PRESENTATION_MODE
}

impl std::fmt::Display for VulkanInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VulkanInitError::UNLOADABLE_LIBRARY => write!(f, "Couldn't load vulkan library"),
            VulkanInitError::UNLOADABLE_EXTENSIONS => write!(f, "Couldn't load vulkan available extensions"),
            VulkanInitError::UNAVAILABLE_EXTENSION(exts) => write!(f, "Can't initiate this unavailable extensions: {}", exts),
            VulkanInitError::FAILED_INSTANTIATING_VULKAN => write!(f, "An Error Occured During Vulkan Instantiation"),
            VulkanInitError::UNAVAILABLE_VULKAN_PHYSICAL_DEVICES => write!(f, "Couldn't load any vulkan capable physical device"),
            VulkanInitError::UNAVAILABLE_PHYSICAL_DEVICE_EXTENSIONS => write!(f, "Couldn't load any physical device extension"),
            VulkanInitError::FAILED_TO_LIST_FAMILY_QUEUES => write!(f, "Couldn't list any family queues on physical device"),
            VulkanInitError::NO_VALID_DESIRED_FAMILY_QUEUE => write!(f, "Couldn't find valid desired family queue"),
            VulkanInitError::UNAVAILABLE_DESIRED_PHYSICAL_DEVICE_EXTENSION(ext) => write!(f, "Couldn't find this physical device extension: {}", ext),
            VulkanInitError::FAILED_INSTANTIATING_LOGICAL_DEVICE => write!(f, "Couldn't initiate Logical Device"),
            VulkanInitError::WRONG_DEVICE_QUEUE_INDEX => write!(f, "Wrong Device queue index given"),
            VulkanInitError::NO_CAPABLE_PHYSICAL_DEVICE => write!(f, "No Capable Physical Device in this machine"),
            VulkanInitError::UNAVAILABLE_PRESENTATION_MODE => write!(f, "Couldn't load any Presentation Mode"),
            VulkanInitError::EXPORTED_VK_FUNCTION_ERROR(msg) 
            | VulkanInitError::GLOBAL_VK_FUNCTION_ERROR(msg)
            | VulkanInitError::INSTANCE_VK_FUNCTION_ERROR(msg)
            | VulkanInitError::INSTANCE_VK_EXT_FUNCTION_ERROR(msg)
            | VulkanInitError::DEVICE_LEVEL_FUNCTION_ERROR(msg) => write!(f, "{}" ,msg),
        }
    }
}

impl std::error::Error for VulkanInitError {}

pub struct QueueInfo {
    familyIndex : usize,
    capability : vulkan_bindings::VkQueueFlags,
    priorities : Vec<f32>
}

pub struct VulkanInstance {
    vulkan_library: libloading::Library,
    available_extensions : Vec<vulkan_bindings::VkExtensionProperties>,
    enabled_extensions : Vec<String>,
    pub instance : vulkan_bindings::VkInstance,
    physical_devices : Vec<VulkanPhysicalDevice>
}

impl VulkanInstance {

    pub fn new(desired_extensions : Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let mut vulkan_instance = VulkanInstance {
                vulkan_library : if cfg!(target_os = "linux") { libloading::Library::new("libvulkan.so.1")? } else { libloading::Library::new("vulkan-1.dll")?},
                available_extensions : Vec::new(),
                enabled_extensions: desired_extensions,
                instance : std::ptr::null_mut(),
                physical_devices: Vec::new()
            };
            vulkan_instance.load_global_functions()?;
            vulkan_instance.load_available_extensions()?;
            vulkan_instance.extensions_are_valid()?;
            vulkan_instance.instantiate_vulkan()?;
            vulkan_instance.load_vulkan_instance_functions()?;
            vulkan_instance.load_physical_devices()?;
            Ok(vulkan_instance)
        }
    }

    pub fn load_global_functions(&mut self) -> Result<(), Box<dyn std::error::Error>>
    {
        unsafe 
        {
            DEFAULTED_IMPORT_MACROS!();
            macro_rules! LOAD_EXPORTED_VULKAN_FUNCTION {
                ($function:ident) => {
                    $function = self.vulkan_library.get(stringify!($function).as_bytes())
                        .ok()
                        .map(|symbol| *symbol);

                    match $function {
                        Some(_) => (),
                        None => {
                            let err = VulkanInitError::EXPORTED_VK_FUNCTION_ERROR(
                                format!("Couldn't load exported Vulkan function: {}", stringify!($function)),
                            );
                            return Err(Box::new(err));
                        }
                    };
                };
            }

            macro_rules! LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION {
                ($function: ident) => {
                    paste! {
                        let fn_vkGetInstanceProcAddr = vkGetInstanceProcAddr.unwrap();
                        let func_name = CString::new(stringify!($function)).unwrap();
                        let temp = fn_vkGetInstanceProcAddr(std::ptr::null_mut(), func_name.as_ptr());
                        $function = std::mem::transmute::<vulkan_bindings::PFN_vkVoidFunction, vulkan_bindings::[<PFN_$function>]>(temp);
                        match $function {
                            Some(_) => { 
                            },
                            None => {
                                let err = VulkanInitError::GLOBAL_VK_FUNCTION_ERROR(
                                    format!("Couldn't load global vulkan function : {}", stringify!($function))
                                );
                                return Err(Box::new(err));
                            }
                        };
                    }
                }
            }
            include!("loaded_functions.rs");
            Ok(())
        }
    }

    pub fn load_available_extensions(&mut self) -> Result<(), Box<VulkanInitError> >
    {
        unsafe {
            let mut extensions_count:u32 = 0;
            let result = vkEnumerateInstanceExtensionProperties.unwrap()(std::ptr::null(), &mut extensions_count, std::ptr::null_mut());
            if result != vulkan_bindings::VkResult_VK_SUCCESS || extensions_count == 0
            {
                return Err(Box::new(VulkanInitError::UNLOADABLE_EXTENSIONS));
            }
    
            self.available_extensions.resize(extensions_count as usize
                , vulkan_bindings::VkExtensionProperties { 
                    extensionName : [0;256],
                    specVersion: 0
                }
            );
            let result = vkEnumerateInstanceExtensionProperties.unwrap()(std::ptr::null(), &mut extensions_count, self.available_extensions.as_mut_ptr());
            if result != vulkan_bindings::VkResult_VK_SUCCESS ||  extensions_count == 0
            {
                return Err(Box::new(VulkanInitError::UNLOADABLE_EXTENSIONS));
            }
            Ok(())
        }
    }

    pub fn list_available_extensions(& self)
    {
        unsafe {
            for extension in  &self.available_extensions{
                let x:[u8;256] = std::mem::transmute::<[i8;256], [u8;256]>(extension.extensionName);
                println!("{}", String::from_utf8(x.to_vec()).unwrap());
            }
        }
    }

    pub fn extensions_are_valid(& self)-> Result<(), Box<VulkanInitError>>
    {
        unsafe {
            let mut found;
            for desired in &self.enabled_extensions{
                found = false;
                for extension in &self.available_extensions {
                    let extension = std::mem::transmute::<[i8;256], [u8;256]>(extension.extensionName);
                    let extension = String::from_utf8(extension.to_vec()).unwrap().trim_end_matches('\0').to_string();
                    if extension == *desired
                    {
                        found = true;
                        break;
                    }
                }
                if found == false
                {
                    return Err(Box::new(VulkanInitError::UNAVAILABLE_EXTENSION(desired.to_string())));
                }
            }
        }
        Ok(())
    }

    pub fn instantiate_vulkan(&mut self) -> Result<(), Box<VulkanInitError>>
    {
        let app_name = CString::new("anvil").unwrap();     
        let app_info  = vulkan_bindings::VkApplicationInfo {
            sType : vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_APPLICATION_INFO,
            pNext : std::ptr::null(),
            pApplicationName : app_name.as_ptr(),
            applicationVersion : VK_MAKE_API_VERSION!(0, 1 , 0, 0),
            pEngineName : app_name.as_ptr(),
            engineVersion : VK_MAKE_API_VERSION!(0, 1 , 0, 0),
            apiVersion: VK_MAKE_API_VERSION!(0, 1 , 0, 0)
        };
        
        unsafe {
            let desired_extensions_cstr: Vec<CString> = self.enabled_extensions.iter()
            .map(|s| CString::new(s.as_bytes()).unwrap())
            .collect();

            let desired_extensions_ptrs : Vec<* const i8> = desired_extensions_cstr.iter()
            .map(|s| s.as_ptr())
            .collect();
            let instance_creation_info = vulkan_bindings::VkInstanceCreateInfo {
                sType : vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                pNext : std::ptr::null(),
                flags: 0,
                pApplicationInfo : &app_info,
                enabledLayerCount: 0,
                ppEnabledLayerNames: std::ptr::null(),
                enabledExtensionCount: desired_extensions_ptrs.len() as u32,
                ppEnabledExtensionNames : if desired_extensions_ptrs.len() > 0 { desired_extensions_ptrs.as_ptr() } else { std::ptr::null() }
            };
            let  result = vkCreateInstance.unwrap()(&instance_creation_info, std::ptr::null(), &mut self.instance);
            if result != vulkan_bindings::VkResult_VK_SUCCESS || self.instance == std::ptr::null_mut() 
            {
                return  Err(Box::new(VulkanInitError::FAILED_INSTANTIATING_VULKAN));
            }
        }
        return  Ok(());
    }

    pub fn load_vulkan_instance_functions(&self) -> Result<(), Box<VulkanInitError>>
    {
        unsafe {
            DEFAULTED_IMPORT_MACROS!();
            macro_rules! LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION {
                ($function: ident) => {
                    paste! {
                        let proc_addr = vkGetInstanceProcAddr.unwrap();
                        let func_name = CString::new(stringify!($function)).unwrap();
                        let func = proc_addr(self.instance, func_name.as_ptr());
                        $function = std::mem::transmute::<vulkan_bindings::PFN_vkVoidFunction, vulkan_bindings::[<PFN_$function>]>(func);
                        match $function {
                            Some(_) => (),
                            None => {
                                let err = VulkanInitError::INSTANCE_VK_FUNCTION_ERROR(format!("Couldn't load instance level vulkan function: {}", stringify!($func) ) );
                                return Err(Box::new(err)) 
                            }
                        }
                    }
                };
            }
    
            macro_rules! LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION_FROM_EXTENSIONS {
                ($function: ident, $ext: ident) => {
                    paste! {
                        let proc_addr = vkGetInstanceProcAddr.unwrap();
                        let func_extension_name = String::from_utf8(vulkan_bindings::[<$ext>].into()).unwrap().trim_end_matches('\0').to_string();
                        let cstr_func_name = CString::new(stringify!($function)).unwrap();
                        for extension in &self.enabled_extensions
                        {
                            if func_extension_name == *extension
                            {
                                let func = proc_addr(self.instance, cstr_func_name.as_ptr());
                                $function = std::mem::transmute::<vulkan_bindings::PFN_vkVoidFunction, vulkan_bindings::[<PFN_$function>]>(func);
                                match $function {
                                    Some(_) => (),
                                    None => {
                                        let err = VulkanInitError::INSTANCE_VK_EXT_FUNCTION_ERROR(
                                            format!("Couldn't load instance level extension vulkan function: {}", stringify!($function))
                                        );
                                        return  Err(Box::new(err));
                                    }
                                };
                            }
                        }
                        if $function.is_none() {
                            let err = VulkanInitError::INSTANCE_VK_EXT_FUNCTION_ERROR(
                                format!("Couldn't load instance level extension vulkan function: {}", stringify!($function))
                            );
                            return  Err(Box::new(err));
                        }
                    }
                };
            }
    
            include!("loaded_functions.rs");  
        }
        Ok(())
    }

    pub fn load_physical_devices(&mut self) -> Result<(), VulkanInitError>
    {
        unsafe {
            let fn_vkEnumeratePhysicalDevices = vkEnumeratePhysicalDevices.unwrap();
            let mut devices_count = 0;
            let result = fn_vkEnumeratePhysicalDevices(self.instance, &mut devices_count, std::ptr::null_mut());
            if result != vulkan_bindings::VkResult_VK_SUCCESS || devices_count == 0
            {
                return  Err(VulkanInitError::UNAVAILABLE_VULKAN_PHYSICAL_DEVICES);
            }
            let mut physical_devices : Vec<vulkan_bindings::VkPhysicalDevice> = vec![std::ptr::null_mut(); devices_count as usize];
            let result = fn_vkEnumeratePhysicalDevices(self.instance, &mut devices_count, physical_devices.as_mut_ptr());
            if result != vulkan_bindings::VkResult_VK_SUCCESS || devices_count == 0
            {
                return  Err(VulkanInitError::UNAVAILABLE_VULKAN_PHYSICAL_DEVICES);
            }
            self.physical_devices.reserve(physical_devices.len());
            for (idx,  ph_device) in physical_devices.into_iter().enumerate()
            {
                self.physical_devices.push(VulkanPhysicalDevice::new(ph_device));
                self.physical_devices[idx].load_infos()?;
            }
        }
        Ok(())
    }

    pub fn create_logical_device(&mut self,
        desired_extensions : Vec<String>,
        desired_capabilites: &[vulkan_bindings::VkQueueFlags],
        surface : &vulkan_bindings::VkSurfaceKHR) 
        -> Result<VulkanLogicalDevice, VulkanInitError >
    {
        VulkanLogicalDevice::new(self, desired_extensions, desired_capabilites, surface)
    }

    pub fn destroy(mut self)
    {
        unsafe {
            let fn_vkDestroyInstance = vkDestroyInstance.unwrap();
            fn_vkDestroyInstance(self.instance, std::ptr::null());
            self.instance = std::ptr::null_mut();
            VULKAN_INSTANCE = None;
        }
    }
}

pub struct VulkanPhysicalDevice {
    pub ph_device : vulkan_bindings::VkPhysicalDevice,
    pub extensions : Vec<vulkan_bindings::VkExtensionProperties>,
    pub features : vulkan_bindings::VkPhysicalDeviceFeatures,
    pub properties : vulkan_bindings::VkPhysicalDeviceProperties,
    pub family_queues: Vec<vulkan_bindings::VkQueueFamilyProperties>,
    pub desired_queues : Vec<QueueInfo>,
    pub supports_presentation : bool,
    pub presentation_queue_idx : i32,
    pub supported_presentation_modes : Vec<vulkan_bindings::VkPresentModeKHR>
}

impl VulkanPhysicalDevice {
    pub fn new(ph_device : vulkan_bindings::VkPhysicalDevice) -> Self {
        unsafe  {
            Self {
                ph_device,
                extensions : Vec::new(),
                features : std::mem::zeroed(),
                properties: std::mem::zeroed(),
                family_queues : Vec::new(),
                desired_queues: Vec::new(),
                supports_presentation: false,
                presentation_queue_idx : -1,
                supported_presentation_modes: Vec::new()
            }
        }
    }

    pub fn list_extensions(& self)
    {
        unsafe {
            for extension in  &self.extensions{
                let x:[u8;256] = std::mem::transmute::<[i8;256], [u8;256]>(extension.extensionName);
                println!("{}", String::from_utf8(x.to_vec()).unwrap());
            }
        }
    }

    pub fn load_extensions(&mut self) -> Result<(), VulkanInitError>
    {
        unsafe {
            let fn_vkEnumerateDeviceExtensionProperties = vkEnumerateDeviceExtensionProperties.unwrap();
            let mut extension_count = 0;
            let result = fn_vkEnumerateDeviceExtensionProperties(self.ph_device, std::ptr::null(), &mut extension_count, std::ptr::null_mut());
            if result != vulkan_bindings::VkResult_VK_SUCCESS || extension_count == 0
            {
                return  Err(VulkanInitError::UNAVAILABLE_PHYSICAL_DEVICE_EXTENSIONS);
            }
            self.extensions.resize(extension_count as usize, vulkan_bindings::VkExtensionProperties { extensionName: [0;256], specVersion: 0 });
            let result = fn_vkEnumerateDeviceExtensionProperties(self.ph_device, std::ptr::null(), &mut extension_count, self.extensions.as_mut_ptr());
            if result != vulkan_bindings::VkResult_VK_SUCCESS || extension_count == 0
            {
                return  Err(VulkanInitError::UNAVAILABLE_PHYSICAL_DEVICE_EXTENSIONS);
            }
        }
        Ok(())
    }

    pub fn load_features(&mut self)
    {
        unsafe {
            let fn_vkGetPhysicalDeviceFeatures = vkGetPhysicalDeviceFeatures.unwrap();
            fn_vkGetPhysicalDeviceFeatures(self.ph_device, &mut self.features);
        }
    }

    pub fn load_properties(&mut self)
    {
        unsafe {
            let fn_vkGetPhysicalDeviceProperties = vkGetPhysicalDeviceProperties.unwrap();
            fn_vkGetPhysicalDeviceProperties(self.ph_device, &mut self.properties);
        }
    }

    pub fn load_family_queues(&mut self) -> Result<(), VulkanInitError>
    {
        unsafe{
            let fn_vkGetPhysicalDeviceQueueFamilyProperties = vkGetPhysicalDeviceQueueFamilyProperties.unwrap();
            let mut family_queues_count = 0;
    
            fn_vkGetPhysicalDeviceQueueFamilyProperties(self.ph_device, &mut family_queues_count, std::ptr::null_mut());
            if family_queues_count == 0
            {
                return Err(VulkanInitError::FAILED_TO_LIST_FAMILY_QUEUES);
            }
            let default_fam_queue: vulkan_bindings::VkQueueFamilyProperties = std::mem::zeroed();
            self.family_queues.resize(family_queues_count as usize, default_fam_queue);
            fn_vkGetPhysicalDeviceQueueFamilyProperties(self.ph_device, &mut family_queues_count, self.family_queues.as_mut_ptr());
            if family_queues_count == 0
            {
                return Err(VulkanInitError::FAILED_TO_LIST_FAMILY_QUEUES);
            }
        }
        Ok(())   
    }

    pub fn supports_presentation(& self , queue_idx: u32, surface : &vulkan_bindings::VkSurfaceKHR) -> bool
    {
        unsafe 
        {
            let mut supports: u32 = 0;
            let fn_vkGetPhysicalDeviceSurfaceSupportKHR = vkGetPhysicalDeviceSurfaceSupportKHR.unwrap();
            let result = fn_vkGetPhysicalDeviceSurfaceSupportKHR(self.ph_device, queue_idx, *surface, &mut supports);
            if result == vulkan_bindings::VkResult_VK_SUCCESS && supports == vulkan_bindings::VK_TRUE
            {
                return true;
            }
            false
        }
    }

    pub fn load_infos(&mut self) -> Result<(), VulkanInitError>
    {
        self.load_extensions()?;
        self.load_features();
        self.load_properties();
        self.load_family_queues()?;
        Ok(())
    }

    pub fn has_desired_family_queues(&mut self, desired_capabilities: &[vulkan_bindings::VkQueueFlags], surface : &vulkan_bindings::VkSurfaceKHR) -> bool
    {
        let mut found_capabilites = 0;
        let mut found_presentation_queue = false;
        let mut family_queues = self.family_queues.clone();
        'outer: for capability_idx in 0..desired_capabilities.len() {
            for (queue_idx,  queue )in family_queues.iter_mut().enumerate() {
                if queue.queueCount > 0 && (queue.queueFlags & desired_capabilities[capability_idx] != 0)
                {
                    if !found_presentation_queue && self.supports_presentation(queue_idx as u32, surface)
                    {
                        self.supports_presentation = true;
                        self.presentation_queue_idx = queue_idx as i32;
                        found_presentation_queue = true
                    }
                    self.desired_queues.push(QueueInfo { familyIndex: queue_idx, capability: desired_capabilities[capability_idx] ,priorities: vec![0.5f32; queue.queueCount as usize]});
                    queue.queueCount = 0;
                    found_capabilites += 1;
                    continue 'outer;
                }
            }
        }
        found_capabilites == desired_capabilities.len() && found_presentation_queue
    }

    pub fn has_desired_extensions(& self, desired_extensions: &Vec<String>) -> bool
    {
        unsafe {
            let mut found;
            for desired in desired_extensions{
                found = false;
                for available in &self.extensions{
                    let available = std::mem::transmute::<[i8;256], [u8;256]>(available.extensionName);
                    let available = String::from_utf8(available.to_vec()).unwrap().trim_end_matches('\0').to_string();
                    if *desired == available
                    {
                        found = true;
                        break;
                    }
                }
                if found == false
                {
                    return false;
                }
            }
        }
        true
    }

    pub fn load_presentation_modes(&mut self, presentation_surface: &vulkan_bindings::VkSurfaceKHR) -> Result<(), VulkanInitError>
    {
        let mut presentation_modes_count :u32 = 0;
        unsafe {
            let fn_vkGetPhysicalDeviceSurfacePresentModesKHR = vkGetPhysicalDeviceSurfacePresentModesKHR.unwrap();
            let result = fn_vkGetPhysicalDeviceSurfacePresentModesKHR(self.ph_device, 
                *presentation_surface, 
                &mut presentation_modes_count, 
                std::ptr::null_mut()
            );
            if result != vulkan_bindings::VkResult_VK_SUCCESS || presentation_modes_count == 0
            {
                return Err(VulkanInitError::UNAVAILABLE_PRESENTATION_MODE);
            }
            self.supported_presentation_modes.resize(presentation_modes_count as usize, 0);
            let result = fn_vkGetPhysicalDeviceSurfacePresentModesKHR(self.ph_device,
                *presentation_surface,
                &mut  presentation_modes_count,
                self.supported_presentation_modes.as_mut_ptr()
            );
            if result != vulkan_bindings::VkResult_VK_SUCCESS || presentation_modes_count == 0
            {
                return Err(VulkanInitError::UNAVAILABLE_PRESENTATION_MODE)   
            }
        }
        Ok(())
    }
    
}

enum PhysicalDeviceVendorsId {
    NVIDIA = 0x10DE,
    AMD = 0x1002,
    INTEL = 0x8086,
    UNDEFINED_VENDOR,
}

impl PhysicalDeviceVendorsId {
    pub fn new(id : u32) -> Self{
        match id{
            0x10DE => Self::NVIDIA,
            0x1002 => Self::AMD,
            0x8086 => Self::INTEL,
            _ => Self::UNDEFINED_VENDOR
        }
    }
}

impl std::fmt::Display for VulkanPhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let device_type = match self.properties.deviceType {
            vulkan_bindings::VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU => "Discrete Gpu",
            vulkan_bindings::VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_CPU => "CPU",
            vulkan_bindings::VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU => "Integrated Gpu",
            _ => "Undefined Device Type"
        };
        let vendor = match PhysicalDeviceVendorsId::new(self.properties.vendorID) {
            PhysicalDeviceVendorsId::NVIDIA  => "Nvidia",
            PhysicalDeviceVendorsId::AMD => "Amd",
            PhysicalDeviceVendorsId::INTEL => "Intel",
            _ => "Undefined Vendor"
        };
        unsafe {
            let device_name = std::mem::transmute::<[i8;256], [u8;256]>(self.properties.deviceName);
            write!(f, "the physical device {} of type {} from {}", String::from_utf8(device_name.to_vec()).unwrap() , device_type, vendor)
        }
    }
}

pub struct  VulkanLogicalDevice {
    pub device : vulkan_bindings::VkDevice,
    pub demanded_queues : Vec<vulkan_bindings::VkDeviceQueueCreateInfo>,
    pub enabled_extensions : Vec<String>,
    pub physical_device : *const VulkanPhysicalDevice
}

impl  VulkanLogicalDevice {

    pub fn new(vulkan_instance: &mut VulkanInstance, desired_extensions: Vec<String>, 
        desired_capabilites: &[vulkan_bindings::VkQueueFlags],
        surface : &vulkan_bindings::VkSurfaceKHR) -> Result<Self, VulkanInitError> {
        let mut vulkan_logical_device = VulkanLogicalDevice {
            device : std::ptr::null_mut(),
            demanded_queues : Vec::new(),
            enabled_extensions : desired_extensions,
            physical_device : std::ptr::null()
        };
        let ref mut physical_devices = vulkan_instance.physical_devices;
        for ph_device in  physical_devices {
            if ph_device.has_desired_extensions(&vulkan_logical_device.enabled_extensions) 
                && ph_device.has_desired_family_queues(desired_capabilites, surface)
                && ph_device.supports_presentation
            {

                vulkan_logical_device.physical_device = ph_device;
                vulkan_logical_device.init_device_queue_info();
                vulkan_logical_device.create_logical_device()?;
                vulkan_logical_device.load_device_functions()?;
                return Ok(vulkan_logical_device); 
            }
        }
        Err(VulkanInitError::NO_CAPABLE_PHYSICAL_DEVICE)
    }

    pub fn init_device_queue_info(&mut self)
    {
        unsafe {
            for queue in &(*self.physical_device).desired_queues {
                self.demanded_queues.push(vulkan_bindings::VkDeviceQueueCreateInfo { 
                        sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO, 
                        pNext: std::ptr::null(), 
                        flags: 0, 
                        queueFamilyIndex: queue.familyIndex as u32, 
                        queueCount: queue.priorities.len() as u32, 
                        pQueuePriorities: queue.priorities.as_ptr()
                    }
                );
            }
        }
    }

    pub fn create_logical_device(&mut self) -> Result<(), VulkanInitError>
    {
        unsafe {
            let ref physical_device = *self.physical_device;
            let enabled_ph_device_exts_cstr: Vec<CString> = self.enabled_extensions.iter()
            .map(|s| CString::new(s.as_bytes()).unwrap())
            .collect();
            let enabled_ph_device_exts_ptrs: Vec<* const i8> = enabled_ph_device_exts_cstr.iter()
            .map(|cs| cs.as_ptr())
            .collect();
            let device_create_info = vulkan_bindings::VkDeviceCreateInfo{
                sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                pNext: std::ptr::null(),
                flags: 0,
                queueCreateInfoCount: self.demanded_queues.len() as u32,
                pQueueCreateInfos: if self.demanded_queues.len() > 0  {  self.demanded_queues.as_ptr() } else { std::ptr::null() },
                enabledLayerCount: 0,
                ppEnabledLayerNames: std::ptr::null(),
                enabledExtensionCount: enabled_ph_device_exts_ptrs.len() as u32,
                ppEnabledExtensionNames: if enabled_ph_device_exts_ptrs.len() > 0 { enabled_ph_device_exts_ptrs.as_ptr() } else { std::ptr::null() },
                pEnabledFeatures : std::ptr::null()
            };
            let fn_vkCreateDevice = vkCreateDevice.unwrap();
            let result = fn_vkCreateDevice(physical_device.ph_device, &device_create_info, std::ptr::null(), &mut self.device);
            if result != vulkan_bindings::VkResult_VK_SUCCESS || self.device == std::ptr::null_mut()
            {
                return Err(VulkanInitError::FAILED_INSTANTIATING_LOGICAL_DEVICE);
            }
        }
        Ok(())
    }

    pub fn load_device_functions(& self) -> Result<(), VulkanInitError>
    {
        unsafe {
            DEFAULTED_IMPORT_MACROS!();

            macro_rules! LOAD_DEVICE_LEVEL_VULKAN_FUNCTION {
                ($function: ident) => {
                    paste! {
                        let fn_vkGetDeviceProcAddr = vkGetDeviceProcAddr.unwrap();
                        let func_name = CString::new(stringify!($function)).unwrap();
                        let func = fn_vkGetDeviceProcAddr(self.device , func_name.as_ptr());
                        $function = std::mem::transmute::<vulkan_bindings::PFN_vkVoidFunction, vulkan_bindings::[<PFN_$function>]>(func);
                        match $function {
                            Some(_) => (),
                            None => {
                                let err = VulkanInitError::INSTANCE_VK_EXT_FUNCTION_ERROR(
                                    format!("Couldn't load device level vulkan function: {}", stringify!($function))
                                );
                                return  Err(err);
                            }
                        };
                    }
                };
            }

            macro_rules!  LOAD_DEVICE_LEVEL_VULKAN_FUNCTION_FROM_EXTENSION {
                ($function: ident , $ext: ident) => {
                    paste!{
                        let fn_vkGetDeviceProcAddr = vkGetDeviceProcAddr.unwrap();
                        let func_name = CString::new(stringify!($function)).unwrap();
                        let extension_name = String::from_utf8(vulkan_bindings::[<$ext>].into()).unwrap().trim_end_matches('\0').to_string();
                        for ph_ext in &self.enabled_extensions
                        {
                            if extension_name == *ph_ext
                            {
                                let func = fn_vkGetDeviceProcAddr(self.device, func_name.as_ptr());
                                $function = std::mem::transmute::<vulkan_bindings::PFN_vkVoidFunction, vulkan_bindings::[<PFN_$function>] >(func);
                                if $function.is_none()
                                {
                                    let err = VulkanInitError::INSTANCE_VK_EXT_FUNCTION_ERROR(
                                        format!("Couldn't load device level extension vulkan function: {}", stringify!($function))
                                    );
                                    return  Err(err);
                                }
                            }
                        }
                        if $function.is_none() {
                            let err = VulkanInitError::INSTANCE_VK_EXT_FUNCTION_ERROR(
                                format!("Couldn't load device level extension vulkan function: {}", stringify!($function))
                            );
                            return  Err(err);
                        }
                    }
                };
            }
            
            include!("loaded_functions.rs");
        }
        Ok(())
    }


    pub fn get_device_queue(& self , desired_capability: vulkan_bindings::VkQueueFlags , queue_idx: usize) -> Option<vulkan_bindings::VkQueue>
    {
        unsafe 
        {
            let mut queue : vulkan_bindings::VkQueue = std::ptr::null_mut();
            let ref physical_device = *self.physical_device;
            let fn_vkGetDeviceQueue = vkGetDeviceQueue.unwrap();

            for fam_queue in  &physical_device.desired_queues {
                if (fam_queue.capability &  desired_capability != 0) && queue_idx < fam_queue.priorities.len() 
                {
                    fn_vkGetDeviceQueue(self.device, fam_queue.familyIndex as u32 , queue_idx as u32, &mut queue);
                    return Some(queue);
                }
            }
            None
        }
    }

    pub fn destroy(mut self)
    {
        unsafe {
            let fn_vkDestroyDevice = vkDestroyDevice.unwrap();
            fn_vkDestroyDevice(self.device, std::ptr::null());
            self.device = std::ptr::null_mut();
        }
    }

}

pub fn initialize_vulkan(desired_global_extensions : Vec<String>) -> &'static mut VulkanInstance
{
    unsafe {
        VULKAN_INSTANCE = match VulkanInstance::new(desired_global_extensions) {
            Ok(i) => Some(i),
            Err(e) =>{
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };
        let vk = (*std::ptr::addr_of_mut!(VULKAN_INSTANCE)).as_mut();
        let vk = vk.unwrap();
        vk
    }
}
