use paste::paste;
use std::ffi::CString;
use crate::vulkan_bindings;

static mut VULKAN_LIBRARY:Option<libloading::Library>= None;
static mut AVAILABLE_EXTENSIONS: Vec<vulkan_bindings::VkExtensionProperties> = Vec::new();
static mut ENABLED_EXTENSIONS : Vec<&str> = Vec::new();
static mut VULKAN_INSTANCE: vulkan_bindings::VkInstance = std::ptr::null_mut();
static mut VULKAN_PHYSICAL_DEVICES : Vec<VulkanPhysicalDevice> = Vec::new();
static mut LOGICAL_DEVICE : vulkan_bindings::VkDevice = std::ptr::null_mut();

macro_rules! EXPORTED_VULKAN_FUNCTION {
    ($name: ident) => {
        paste! {
        #[allow(non_upper_case_globals)]
        static mut $name :  vulkan_bindings::[<PFN_$name>] = None;
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
            VulkanInitError::EXPORTED_VK_FUNCTION_ERROR(msg) 
            | VulkanInitError::GLOBAL_VK_FUNCTION_ERROR(msg)
            | VulkanInitError::INSTANCE_VK_FUNCTION_ERROR(msg)
            | VulkanInitError::INSTANCE_VK_EXT_FUNCTION_ERROR(msg)
            | VulkanInitError::DEVICE_LEVEL_FUNCTION_ERROR(msg) => write!(f, "{}" ,msg),
        }
    }
}

impl std::error::Error for VulkanInitError {}

pub type VulkanInitResult = Result<(), Box<dyn std::error::Error>>;

pub struct QueueInfo {
    familyIndex : usize,
    priorities : Vec<f32>
}

pub struct VulkanPhysicalDevice {
    pub ph_device : vulkan_bindings::VkPhysicalDevice,
    pub extensions : Vec<vulkan_bindings::VkExtensionProperties>,
    pub features : vulkan_bindings::VkPhysicalDeviceFeatures,
    pub properties : vulkan_bindings::VkPhysicalDeviceProperties,
    pub family_queues: Vec<vulkan_bindings::VkQueueFamilyProperties>,
    pub desired_queues : Vec<QueueInfo>
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
                desired_queues: Vec::new()
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

    pub fn load_infos(&mut self) -> Result<(), VulkanInitError>
    {
        self.load_extensions()?;
        self.load_features();
        self.load_properties();
        self.load_family_queues()?;
        Ok(())
    }

    pub fn has_desired_family_queues(&mut self, desired_capabilities: &[vulkan_bindings::VkQueueFlags]) -> bool
    {
        'outer: for desire_capability in desired_capabilities {
            for (idx,  queue )in self.family_queues.iter().enumerate() {
                if queue.queueCount > 0 && (queue.queueFlags & desire_capability != 0)
                {
                    self.desired_queues.push(QueueInfo { familyIndex: idx, priorities: vec![0.5f32; queue.queueCount as usize]});
                    self.family_queues.remove(idx);
                    continue 'outer;
                }
            }
            return false;
        }
        true
    }

    pub fn has_desired_extensions(& self, desired_extensions: &[&str]) -> bool
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

    
}

struct  VulkanLogicalDevice {
    pub device : vulkan_bindings::VkDevice,
    pub demanded_queues : Vec<vulkan_bindings::VkDeviceQueueCreateInfo>,
    pub enabled_extensions : Vec<&'static str>,
    pub enabled_capabilites : Vec<vulkan_bindings::VkQueueFlags>,
    pub physical_device : *const VulkanPhysicalDevice
}

impl  VulkanLogicalDevice {

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
            .map(|s| CString::new(*s).unwrap())
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

    pub fn new(desired_extensions: &[&'static str], desired_capabilites: &[vulkan_bindings::VkQueueFlags]) -> Result<Self, VulkanInitError> {
        let mut vulkan_logical_device = VulkanLogicalDevice {
            device : std::ptr::null_mut(),
            demanded_queues : Vec::new(),
            enabled_extensions : desired_extensions.to_vec(),
            enabled_capabilites : desired_capabilites.into(),
            physical_device : std::ptr::null()
        };
        unsafe {
            let ref mut physical_devices = *std::ptr::addr_of_mut!(VULKAN_PHYSICAL_DEVICES);
            for ph_device in  physical_devices {
                if ph_device.has_desired_extensions(desired_extensions) 
                    && ph_device.has_desired_family_queues(desired_capabilites)
                {
                    vulkan_logical_device.physical_device = ph_device;
                    vulkan_logical_device.init_device_queue_info();
                    vulkan_logical_device.create_logical_device()?;
                    vulkan_logical_device.load_device_functions()?;
                    return Ok(vulkan_logical_device); 
                }
            }
        }
        Err(VulkanInitError::NO_CAPABLE_PHYSICAL_DEVICE)
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


    pub fn get_device_queue(& self ,queue_fam_idx:usize ,  queue_idx: usize) -> Result<vulkan_bindings::VkQueue, VulkanInitError>
    {
        let mut queue : vulkan_bindings::VkQueue = std::ptr::null_mut();
        unsafe 
        {
            let ref physical_device = *self.physical_device;
            let fn_vkGetDeviceQueue = vkGetDeviceQueue.unwrap();
            if queue_fam_idx >= physical_device.desired_queues.len() || queue_idx >=  physical_device.desired_queues[queue_fam_idx].priorities.len()
            {
                return  Err(VulkanInitError::WRONG_DEVICE_QUEUE_INDEX);
            }
            fn_vkGetDeviceQueue(self.device, physical_device.desired_queues[queue_fam_idx].familyIndex as u32 , queue_idx as u32, &mut queue);
        }
        Ok(queue)
    }

}

pub fn load_vulkan_lib() -> VulkanInitResult
{
    unsafe {
        VULKAN_LIBRARY = Some(libloading::Library::new("libvulkan.so.1")?);
        DEFAULTED_IMPORT_MACROS!();
        macro_rules! LOAD_EXPORTED_VULKAN_FUNCTION {
            ($function:ident) => {
                if let Some(ref lib) = VULKAN_LIBRARY {
                    $function = lib.get(stringify!($function).as_bytes())
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
                } else {
                    return Err(Box::new(VulkanInitError::UNLOADABLE_LIBRARY));
                }
            };
        }

        macro_rules! LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION {
            ($function: ident) => {
                paste! {
                    if let Some(ref proc_addr) = vkGetInstanceProcAddr {
                        let func_name = CString::new(stringify!($function)).unwrap();
                        let temp = proc_addr(std::ptr::null_mut(), func_name.as_ptr());
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
        }
        include!("loaded_functions.rs");
        Ok(())
    }
}

pub fn get_vulkan_available_extensions() -> Result<(), VulkanInitError>
{
    unsafe {
        let mut extensions_count:u32 = 0;
        let result = vkEnumerateInstanceExtensionProperties.unwrap()(std::ptr::null(), &mut extensions_count, std::ptr::null_mut());
        if result != vulkan_bindings::VkResult_VK_SUCCESS || extensions_count == 0
        {
            return Err(VulkanInitError::UNLOADABLE_EXTENSIONS);
        }

        AVAILABLE_EXTENSIONS.resize(extensions_count as usize
            , vulkan_bindings::VkExtensionProperties { 
                extensionName : [0;256],
                specVersion: 0
            }
        );
        let result = vkEnumerateInstanceExtensionProperties.unwrap()(std::ptr::null(), &mut extensions_count, AVAILABLE_EXTENSIONS.as_mut_ptr());
        if result != vulkan_bindings::VkResult_VK_SUCCESS ||  extensions_count == 0
        {
            return Err(VulkanInitError::UNLOADABLE_EXTENSIONS);
        }
        Ok(())
    }
}

pub fn list_available_extensions()
{
    unsafe {
        let ref extensions = *std::ptr::addr_of!(AVAILABLE_EXTENSIONS);
        for extension in  extensions{
            let x:[u8;256] = std::mem::transmute::<[i8;256], [u8;256]>(extension.extensionName);
            println!("{}", String::from_utf8(x.to_vec()).unwrap());
        }
    }
}

pub fn check_desired_extensions()-> Result<(), VulkanInitError>
{
    unsafe {
        let ref extensions = *std::ptr::addr_of!(AVAILABLE_EXTENSIONS);
        let ref desired_extensions = *std::ptr::addr_of!(ENABLED_EXTENSIONS);
        let mut found;
        for desired in desired_extensions{
            found = false;
            for extension in extensions {
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
                return Err(VulkanInitError::UNAVAILABLE_EXTENSION(desired.to_string()));
            }
        }
    }
    Ok(())
}

pub fn instantiate_vulkan() -> Result<(), VulkanInitError>
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
        let ref desired_extensions = *std::ptr::addr_of!(ENABLED_EXTENSIONS);
        let desired_extensions_cstr: Vec<CString> = desired_extensions.iter()
        .map(|&s| CString::new(s).unwrap())
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
            enabledExtensionCount: desired_extensions.len() as u32,
            ppEnabledExtensionNames : if desired_extensions.len() > 0 { desired_extensions_ptrs.as_ptr() } else { std::ptr::null() }
        };
        let vulkan_instance = std::ptr::addr_of_mut!(VULKAN_INSTANCE);
        let  result = vkCreateInstance.unwrap()(&instance_creation_info, std::ptr::null(), vulkan_instance);
        if result != vulkan_bindings::VkResult_VK_SUCCESS || vulkan_instance == std::ptr::null_mut() 
        {
            return  Err(VulkanInitError::FAILED_INSTANTIATING_VULKAN);
        }
    }
    return  Ok(());
}

pub fn load_vulkan_instance_functions() -> Result<(), VulkanInitError>
{
    unsafe {
        DEFAULTED_IMPORT_MACROS!();
        macro_rules! LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION {
            ($function: ident) => {
                paste! {
                    let proc_addr = vkGetInstanceProcAddr.unwrap();
                    let func_name = CString::new(stringify!($function)).unwrap();
                    let func = proc_addr(VULKAN_INSTANCE, func_name.as_ptr());
                    $function = std::mem::transmute::<vulkan_bindings::PFN_vkVoidFunction, vulkan_bindings::[<PFN_$function>]>(func);
                    match $function {
                        Some(_) => (),
                        None => {
                            let err = VulkanInitError::INSTANCE_VK_FUNCTION_ERROR(format!("Couldn't load instance level vulkan function: {}", stringify!($func) ) );
                            return Err(err) 
                        }
                    }
                }
            };
        }

        macro_rules! LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION_FROM_EXTENSIONS {
            ($function: ident, $ext: ident) => {
                paste! {
                    let ref enabled_extensions = *std::ptr::addr_of!(ENABLED_EXTENSIONS);
                    let proc_addr = vkGetInstanceProcAddr.unwrap();
                    let func_extension_name = String::from_utf8(vulkan_bindings::[<$ext>].into()).unwrap().trim_end_matches('\0').to_string();
                    let cstr_func_name = CString::new(stringify!($function)).unwrap();
                    for extension in enabled_extensions
                    {
                        if func_extension_name == *extension
                        {
                            let func = proc_addr(VULKAN_INSTANCE, cstr_func_name.as_ptr());
                            $function = std::mem::transmute::<vulkan_bindings::PFN_vkVoidFunction, vulkan_bindings::[<PFN_$function>]>(func);
                            match $function {
                                Some(_) => (),
                                None => {
                                    let err = VulkanInitError::INSTANCE_VK_EXT_FUNCTION_ERROR(
                                        format!("Couldn't load instance level extension vulkan function: {}", stringify!($function))
                                    );
                                    return  Err(err);
                                }
                            };
                        }
                    }
                    if $function.is_none() {
                        let err = VulkanInitError::INSTANCE_VK_EXT_FUNCTION_ERROR(
                            format!("Couldn't load instance level extension vulkan function: {}", stringify!($function))
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



pub fn get_vulkan_available_physical_devices() -> Result<(), VulkanInitError>
{
    unsafe {
        let fn_vkEnumeratePhysicalDevices = vkEnumeratePhysicalDevices.unwrap();
        let mut devices_count = 0;
        let result = fn_vkEnumeratePhysicalDevices(VULKAN_INSTANCE, &mut devices_count, std::ptr::null_mut());
        if result != vulkan_bindings::VkResult_VK_SUCCESS || devices_count == 0
        {
            return  Err(VulkanInitError::UNAVAILABLE_VULKAN_PHYSICAL_DEVICES);
        }
        let mut physical_devices : Vec<vulkan_bindings::VkPhysicalDevice> = vec![std::ptr::null_mut(); devices_count as usize];
        let result = fn_vkEnumeratePhysicalDevices(VULKAN_INSTANCE, &mut devices_count, physical_devices.as_mut_ptr());
        if result != vulkan_bindings::VkResult_VK_SUCCESS || devices_count == 0
        {
            return  Err(VulkanInitError::UNAVAILABLE_VULKAN_PHYSICAL_DEVICES);
        }
        for (idx,  ph_device) in physical_devices.into_iter().enumerate()
        {
            VULKAN_PHYSICAL_DEVICES.push(VulkanPhysicalDevice::new(ph_device));
            VULKAN_PHYSICAL_DEVICES[idx].load_infos()?;
        }
    }
    Ok(())
}

macro_rules! INIT_OPERATION_UNWRAP_RESULT {
    ($func_res: expr) => {
        match $func_res {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    };
}

pub fn initialize_vulkan(){
    unsafe {
        ENABLED_EXTENSIONS.extend_from_slice(&["VK_KHR_surface", "VK_KHR_display"]);
    }
    INIT_OPERATION_UNWRAP_RESULT!( self::load_vulkan_lib() ); 
    INIT_OPERATION_UNWRAP_RESULT!( self::get_vulkan_available_extensions() );
    // list_available_extensions();
    INIT_OPERATION_UNWRAP_RESULT!( self::check_desired_extensions() );
    INIT_OPERATION_UNWRAP_RESULT!( self::instantiate_vulkan()) ;
    INIT_OPERATION_UNWRAP_RESULT!( self::load_vulkan_instance_functions());
    INIT_OPERATION_UNWRAP_RESULT!( self::get_vulkan_available_physical_devices());
    INIT_OPERATION_UNWRAP_RESULT!( VulkanLogicalDevice::new(&["VK_KHR_swapchain"], &[vulkan_bindings::VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT]));
    // INIT_OPERATION_UNWRAP_RESULT!( self::load_device_level_functions());
    // unsafe {
    //     if let Ok(queue) = get_device_queue(&DESIRED_FAMILY_QUEUES[0], 0)
    //     {
    //         println!("the given queue {:?}", queue);
    //     }
    // }
    println!("Done Loading");
}
