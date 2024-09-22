#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
mod vulkan_bindings;

pub mod vulkan_mod {
    use paste::paste;
    use std::ffi::CString;

    use crate::vulkan_bindings;

    static mut VULKAN_LIBRARY:Option<libloading::Library>= None;
    static mut AVAILABLE_EXTENSIONS: Vec<vulkan_bindings::VkExtensionProperties> = Vec::new();
    static mut VULKAN_INSTANCE: vulkan_bindings::VkInstance = std::ptr::null_mut();

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

    include!("./imported_functions.rs");
    
    #[derive(Debug)]
    pub enum VulkanInitError {
        UNLOADABLE_LIBRARY,
        EXPORTED_VK_FUNCTION_ERROR(String),
        GLOBAL_VK_FUNCTION_ERROR(String),
        UNLOADABLE_EXTENSIONS,
        UNAVAILABLE_EXTENSION(String),
        FAILED_INSTANTIATING_VULKAN,
    }

    impl std::fmt::Display for VulkanInitError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                VulkanInitError::UNLOADABLE_LIBRARY => write!(f, "Couldn't load vulkan library"),
                VulkanInitError::UNLOADABLE_EXTENSIONS => write!(f, "Couldn't load vulkan available extensions"),
                VulkanInitError::UNAVAILABLE_EXTENSION(exts) => write!(f, "Can't initiate this unavailable extensions: {}", exts),
                VulkanInitError::FAILED_INSTANTIATING_VULKAN => write!(f, "An Error Occured During Vulkan Instantiation"),
                VulkanInitError::EXPORTED_VK_FUNCTION_ERROR(msg) 
                | VulkanInitError::GLOBAL_VK_FUNCTION_ERROR(msg) => write!(f, "{}" ,msg),
            }
        }
    }

    impl std::error::Error for VulkanInitError {}

    pub type VulkanInitResult = Result<(), Box<dyn std::error::Error>>;


    pub fn load_vulkan_lib() -> VulkanInitResult
    {
        unsafe {
            VULKAN_LIBRARY = Some(libloading::Library::new("libvulkan.so.1")?);

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
            let extensions = std::ptr::addr_of!(AVAILABLE_EXTENSIONS);
            let extensions = &*extensions;
            for extension in  extensions{
                let x:[u8;256] = std::mem::transmute::<[i8;256], [u8;256]>(extension.extensionName);
                println!("{}", String::from_utf8(x.to_vec()).unwrap());
            }
        }
    }

    pub fn check_desired_extensions(desired_extensions: &Vec<&str>)-> Result<(), VulkanInitError>
    {
        unsafe {
            let extensions = std::ptr::addr_of!(AVAILABLE_EXTENSIONS);
            let extensions = &*extensions;
            let mut found;
            for desired in desired_extensions {
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

    pub fn instantiate_vulkan(desired_extensions: &Vec<&str>) -> Result<(), VulkanInitError>
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

        unsafe {
            let vulkan_instance = std::ptr::addr_of_mut!(VULKAN_INSTANCE);
            let  result = vkCreateInstance.unwrap()(&instance_creation_info, std::ptr::null(), vulkan_instance);
            if result != vulkan_bindings::VkResult_VK_SUCCESS || vulkan_instance == std::ptr::null_mut() 
            {
                return  Err(VulkanInitError::FAILED_INSTANTIATING_VULKAN);
            }
        }
        return  Ok(());
    }

}
