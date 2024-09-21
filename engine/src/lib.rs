#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod vulkan_mod {
    use paste::paste;

    pub mod vulkan_bindings {
        include!("../bindings/binding.rs");
    }

    static mut VULKAN_LIBRARY:Option<libloading::Library>= None;

    macro_rules! EXPORTED_VULKAN_FUNCTION {
        ($name: ident) => {
           paste! {
            #[allow(non_upper_case_globals)]
            static mut $name :  vulkan_bindings::[<PFN_$name>] = None;
           }
        };
    }

    include!("./imported_functions.rs");
    
    #[derive(Debug)]
    pub enum VulkanInitError {
        UNLOADABLE_LIBRARY,
        EXPORTED_VK_FUNCTION_ERROR(String),
        GLOBAL_VK_FUNCTION_ERROR(String),
        UNLOADABLE_EXTENSIONS,
    }

    impl std::fmt::Display for VulkanInitError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                VulkanInitError::UNLOADABLE_LIBRARY => write!(f, "Couldn't load vulkan library"),
                VulkanInitError::UNLOADABLE_EXTENSIONS => write!(f, "Couldn't load vulkan available extensions"),
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

            use std::ffi::CString;
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


    static mut AVAILABLE_EXTENSIONS: Vec<vulkan_bindings::VkExtensionProperties> = Vec::new();

    pub fn get_vulkan_available_extensions() -> Result<(), VulkanInitError>
    {
        unsafe {
            let mut extensions_count:u32 = 0;
            let result = vkEnumerateInstanceExtensionProperties.unwrap()(std::ptr::null(), &mut extensions_count, std::ptr::null_mut());
            if result == vulkan_bindings::VkResult_VK_SUCCESS || extensions_count == 0
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
            if result == vulkan_bindings::VkResult_VK_SUCCESS ||  extensions_count == 0
            {
                return Err(VulkanInitError::UNLOADABLE_EXTENSIONS);
            }
            Ok(())
        }
    }

}
