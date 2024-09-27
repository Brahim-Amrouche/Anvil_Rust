use paste::paste;
use std::ffi::CString;
use crate::vulkan_bindings;

struct QueueInfo {
    familyIndex : usize,
    priorities : Vec<f32>
}

static mut VULKAN_LIBRARY:Option<libloading::Library>= None;
static mut AVAILABLE_EXTENSIONS: Vec<vulkan_bindings::VkExtensionProperties> = Vec::new();
static mut ENABLED_EXTENSIONS : Vec<&str> = Vec::new();
static mut VULKAN_INSTANCE: vulkan_bindings::VkInstance = std::ptr::null_mut();
static mut AVAILABLE_PHYSICAL_DEVICES: Vec<vulkan_bindings::VkPhysicalDevice> = Vec::new();
static mut PHYSICAL_DEVICE_EXTENSIONS: Vec<vulkan_bindings::VkExtensionProperties> = Vec::new();
static mut ENABLED_PHYSICAL_DEVICE_EXTENSIONS : Vec<&str> = Vec::new();
static mut PHYSICAL_DEVICE_FEATURES: Option<vulkan_bindings::VkPhysicalDeviceFeatures> = None;
static mut PHYSICAL_DEVICE_PROPERTIES : Option<vulkan_bindings::VkPhysicalDeviceProperties> = None;
static mut AVAILABLE_FAMILY_QUEUES: Vec<vulkan_bindings::VkQueueFamilyProperties> = Vec::new();
static mut DESIRED_FAMILY_QUEUES: Vec<QueueInfo> = Vec::new();
static mut QUEUE_CREATE_INFO : Vec<vulkan_bindings::VkDeviceQueueCreateInfo> = Vec::new();
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
    DEVICE_LEVEL_FUNCTION_ERROR(String)
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
        AVAILABLE_PHYSICAL_DEVICES.resize(devices_count as usize, std::ptr::null_mut());
        let result = fn_vkEnumeratePhysicalDevices(VULKAN_INSTANCE, &mut devices_count, AVAILABLE_PHYSICAL_DEVICES.as_mut_ptr());
        if result != vulkan_bindings::VkResult_VK_SUCCESS || devices_count == 0
        {
            return  Err(VulkanInitError::UNAVAILABLE_VULKAN_PHYSICAL_DEVICES);
        }
    }
    Ok(())
}

pub fn get_available_physical_device_extensions(physical_device: vulkan_bindings::VkPhysicalDevice) -> Result<(), VulkanInitError>
{
    unsafe {
        let fn_vkEnumerateDeviceExtensionProperties = vkEnumerateDeviceExtensionProperties.unwrap();
        let mut extension_count = 0;
        let result = fn_vkEnumerateDeviceExtensionProperties(physical_device, std::ptr::null(), &mut extension_count, std::ptr::null_mut());
        if result != vulkan_bindings::VkResult_VK_SUCCESS || extension_count == 0
        {
            return  Err(VulkanInitError::UNAVAILABLE_PHYSICAL_DEVICE_EXTENSIONS);
        }
        PHYSICAL_DEVICE_EXTENSIONS.resize(extension_count as usize, vulkan_bindings::VkExtensionProperties { extensionName: [0;256], specVersion: 0 });
        let result = fn_vkEnumerateDeviceExtensionProperties(physical_device, std::ptr::null(), &mut extension_count, PHYSICAL_DEVICE_EXTENSIONS.as_mut_ptr());
        if result != vulkan_bindings::VkResult_VK_SUCCESS || extension_count == 0
        {
            return  Err(VulkanInitError::UNAVAILABLE_PHYSICAL_DEVICE_EXTENSIONS);
        }
    }
    Ok(())
}

pub fn list_available_physical_device_extensions()
{
    unsafe {
        let ref extensions = *std::ptr::addr_of!(PHYSICAL_DEVICE_EXTENSIONS);
        for extension in  extensions{
            let x:[u8;256] = std::mem::transmute::<[i8;256], [u8;256]>(extension.extensionName);
            println!("{}", String::from_utf8(x.to_vec()).unwrap());
        }
    }
}

pub fn get_physical_device_features(physical_device: vulkan_bindings::VkPhysicalDevice)
{
    unsafe {
        let mut ph_device_features: vulkan_bindings::VkPhysicalDeviceFeatures = std::mem::zeroed();
        let fn_vkGetPhysicalDeviceFeatures = vkGetPhysicalDeviceFeatures.unwrap();
        fn_vkGetPhysicalDeviceFeatures(physical_device, &mut ph_device_features);
        PHYSICAL_DEVICE_FEATURES  = Some(ph_device_features);
    }
}

pub fn get_physical_device_properties(physical_device: vulkan_bindings::VkPhysicalDevice)
{
    unsafe{
        let mut ph_device_properties: vulkan_bindings::VkPhysicalDeviceProperties = std::mem::zeroed();
        let fn_vkGetPhysicalDeviceProperties = vkGetPhysicalDeviceProperties.unwrap();
        fn_vkGetPhysicalDeviceProperties(physical_device, &mut ph_device_properties);
        PHYSICAL_DEVICE_PROPERTIES = Some(ph_device_properties);
    }
}

pub fn check_available_family_queues(physical_device: vulkan_bindings::VkPhysicalDevice) -> Result<(), VulkanInitError>
{
    unsafe{
        let fn_vkGetPhysicalDeviceQueueFamilyProperties = vkGetPhysicalDeviceQueueFamilyProperties.unwrap();
        let mut family_queues_count = 0;

        fn_vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut family_queues_count, std::ptr::null_mut());
        if family_queues_count == 0
        {
            return Err(VulkanInitError::FAILED_TO_LIST_FAMILY_QUEUES);
        }
        let default_fam_queue: vulkan_bindings::VkQueueFamilyProperties = std::mem::zeroed();
        AVAILABLE_FAMILY_QUEUES.resize(family_queues_count as usize, default_fam_queue);
        fn_vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut family_queues_count, AVAILABLE_FAMILY_QUEUES.as_mut_ptr());
        if family_queues_count == 0
        {
            return Err(VulkanInitError::FAILED_TO_LIST_FAMILY_QUEUES);
        }
    }
    Ok(())
}

pub fn get_desired_family_queues(desired_capabilities: &[vulkan_bindings::VkQueueFlags]) -> Result<(), VulkanInitError>
{
    unsafe {
        let available_fam_queues = std::ptr::addr_of_mut!(AVAILABLE_FAMILY_QUEUES);
        'outer: for desire_capability in desired_capabilities {
            for (idx,  queue )in (*available_fam_queues).iter().enumerate() {
                if queue.queueCount > 0 && queue.queueFlags & desire_capability != 0
                {
                    DESIRED_FAMILY_QUEUES.push(QueueInfo { familyIndex: idx, priorities: vec![0.5f32; queue.queueCount as usize]});
                    (*available_fam_queues).remove(idx);
                    continue 'outer;
                }
            }
            return Err(VulkanInitError::NO_VALID_DESIRED_FAMILY_QUEUE);
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

pub fn validate_desired_physical_device_extensions() -> Result<(), VulkanInitError>
{
    unsafe {
        let ref available_extensions = *std::ptr::addr_of!(PHYSICAL_DEVICE_EXTENSIONS);
        let ref desired_extensions = *std::ptr::addr_of!(ENABLED_PHYSICAL_DEVICE_EXTENSIONS);
        let mut found;
        for desired in desired_extensions{
            found = false;
            for available in available_extensions{
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
                return Err(VulkanInitError::UNAVAILABLE_DESIRED_PHYSICAL_DEVICE_EXTENSION(desired.to_string()));
            }
        }
    }
    Ok(())
}

pub fn init_device_queue_info()
{
    unsafe {
        let ref desired_fam_queues = *std::ptr::addr_of!(DESIRED_FAMILY_QUEUES);
        for queue in desired_fam_queues {
            QUEUE_CREATE_INFO.push(vulkan_bindings::VkDeviceQueueCreateInfo { 
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

pub fn create_logical_device(physical_device: vulkan_bindings::VkPhysicalDevice) -> Result<(), VulkanInitError>
{
    unsafe {
        let enabled_ph_device_exts_cstr: Vec<CString> = ENABLED_PHYSICAL_DEVICE_EXTENSIONS.iter()
        .map(|s| CString::new(*s).unwrap())
        .collect();
        let enabled_ph_device_exts_ptrs: Vec<* const i8> = enabled_ph_device_exts_cstr.iter()
        .map(|cs| cs.as_ptr())
        .collect();
        let device_create_info = vulkan_bindings::VkDeviceCreateInfo{
            sType: vulkan_bindings::VkStructureType_VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: std::ptr::null(),
            flags: 0,
            queueCreateInfoCount: QUEUE_CREATE_INFO.len() as u32,
            pQueueCreateInfos: if QUEUE_CREATE_INFO.len() > 0  {  QUEUE_CREATE_INFO.as_ptr() } else { std::ptr::null() },
            enabledLayerCount: 0,
            ppEnabledLayerNames: std::ptr::null(),
            enabledExtensionCount: enabled_ph_device_exts_ptrs.len() as u32,
            ppEnabledExtensionNames: if enabled_ph_device_exts_ptrs.len() > 0 { enabled_ph_device_exts_ptrs.as_ptr() } else { std::ptr::null() },
            pEnabledFeatures : std::ptr::null()
        };
        let fn_vkCreateDevice = vkCreateDevice.unwrap();
        let result = fn_vkCreateDevice(physical_device, &device_create_info, std::ptr::null(), std::ptr::addr_of_mut!(LOGICAL_DEVICE));
        if result != vulkan_bindings::VkResult_VK_SUCCESS || LOGICAL_DEVICE == std::ptr::null_mut()
        {
            return Err(VulkanInitError::FAILED_INSTANTIATING_LOGICAL_DEVICE);
        }
    }
    Ok(())
}

pub fn init_logical_device(physical_device: vulkan_bindings::VkPhysicalDevice) -> Result<(), VulkanInitError>
{   
    self::get_available_physical_device_extensions(physical_device)?;
    // self::list_available_physical_device_extensions();
    self::get_physical_device_features(physical_device);
    self::get_physical_device_properties(physical_device);
    self::check_available_family_queues(physical_device)?;
    self::get_desired_family_queues(&[vulkan_bindings::VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT | vulkan_bindings::VkQueueFlagBits_VK_QUEUE_COMPUTE_BIT])?;
    self::validate_desired_physical_device_extensions()?;
    self::init_device_queue_info();
    self::create_logical_device(physical_device)?;
    Ok(())   
}


pub fn load_device_level_functions() -> Result<(), VulkanInitError>
{
    unsafe {
        DEFAULTED_IMPORT_MACROS!();

        macro_rules! LOAD_DEVICE_LEVEL_VULKAN_FUNCTION {
            ($function: ident) => {
                paste! {
                    let fn_vkGetDeviceProcAddr = vkGetDeviceProcAddr.unwrap();
                    let func_name = CString::new(stringify!($function)).unwrap();
                    let func = fn_vkGetDeviceProcAddr(LOGICAL_DEVICE, func_name.as_ptr());
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
                    let ref ph_extensions = *std::ptr::addr_of!(ENABLED_PHYSICAL_DEVICE_EXTENSIONS);
                    let fn_vkGetDeviceProcAddr = vkGetDeviceProcAddr.unwrap();
                    let func_name = CString::new(stringify!($function)).unwrap();
                    let extension_name = String::from_utf8(vulkan_bindings::[<$ext>].into()).unwrap().trim_end_matches('\0').to_string();
                    for ph_ext in ph_extensions
                    {
                        if extension_name == *ph_ext
                        {
                            let func = fn_vkGetDeviceProcAddr(LOGICAL_DEVICE, func_name.as_ptr());
                            $function = std::mem::transmute::<vulkan_bindings::PFN_vkVoidFunction, vulkan_bindings::[<PFN_$function>] >(func);
                            if $function.is_none()
                            {
                                println!("Erro here");
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
    let physical_device;
    unsafe {
        physical_device = AVAILABLE_PHYSICAL_DEVICES[0];
        ENABLED_PHYSICAL_DEVICE_EXTENSIONS.push("VK_KHR_swapchain");
    }
    INIT_OPERATION_UNWRAP_RESULT!( self::init_logical_device(physical_device));
    INIT_OPERATION_UNWRAP_RESULT!( self::load_device_level_functions());
    println!("Done Loading");
}
