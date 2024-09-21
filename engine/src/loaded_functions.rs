{
    LOAD_EXPORTED_VULKAN_FUNCTION!(vkGetInstanceProcAddr);

    #[allow(unused_macros)]
    macro_rules! LOAD_EXPORTED_VULKAN_FUNCTION {() => {}}

    LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION!(vkEnumerateInstanceExtensionProperties);
    LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION!(vkEnumerateInstanceLayerProperties);
    LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION!(vkCreateInstance);

    #[allow(unused_macros)]
    macro_rules!  LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION {() => {}}
}