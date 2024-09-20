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
    enum VulkanInitError {
        UNLOADABLE_LIBRARY,
        EXPORTED_VK_FUNCTION_ERROR(String),
    }

    impl std::fmt::Display for VulkanInitError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                VulkanInitError::UNLOADABLE_LIBRARY => write!(f, "Couldn't load vulkan library"),
                VulkanInitError::EXPORTED_VK_FUNCTION_ERROR(msg) => write!(f, "{}" ,msg),
            }
        }
    }

    impl std::error::Error for VulkanInitError {}

    pub type VulkanInitResult = Result<(), Box<dyn std::error::Error>>;

   
    // macro_rules! EXPORTED_VULKAN_FUNCTION {
    //     ($function: ident ) => {
    //         if let Some(ref lib) = VULKAN_LIBRARY {
    //             $function = lib.get(stringify!($function).as_bytes())
    //             .ok()
    //             .map(|symbol| *symbol);
    //             match $function {
    //                 Some(_) => (),
    //                 None => {
    //                     let err = VulkanInitError::EXPORTED_VK_FUNCTION_ERROR(format!("Couldn't load exported vulkan function: {}", stringify!($function)));
    //                     return Err(Box::new(err))
    //                 }
    //             };
    //         }
    //         else {
    //             return Err(Box::new(VulkanInitError::UNLOADABLE_LIBRARY));
    //         }
    //     };
    // }

    pub fn load_vulkan_lib() -> VulkanInitResult
    {
        unsafe {
            VULKAN_LIBRARY = Some(libloading::Library::new("libvulkan.so.1")?);
            macro_rules! EXPORTED_VULKAN_FUNCTION {
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

            include!("imported_functions.rs");
            Ok(())
        }
    }
}
