{
    LOAD_EXPORTED_VULKAN_FUNCTION!(vkGetInstanceProcAddr);
    
    
    LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION!(vkEnumerateInstanceExtensionProperties);
    LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION!(vkEnumerateInstanceLayerProperties);
    LOAD_GLOBAL_LEVEL_VULKAN_FUNCTION!(vkCreateInstance);
    
    LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION!(vkEnumeratePhysicalDevices);
    LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION!(vkEnumerateDeviceExtensionProperties);
    LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION!(vkGetPhysicalDeviceProperties);
    LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION!(vkGetPhysicalDeviceFeatures);
    LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION!(vkCreateDevice);
    LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION!(vkGetDeviceProcAddr);

    LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION_FROM_EXTENSIONS!(vkGetPhysicalDeviceSurfaceSupportKHR , VK_KHR_SURFACE_EXTENSION_NAME);
    LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION_FROM_EXTENSIONS!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR, VK_KHR_SURFACE_EXTENSION_NAME);
    LOAD_INSTANCE_LEVEL_VULKAN_FUNCTION_FROM_EXTENSIONS!(vkGetPhysicalDeviceSurfaceFormatsKHR , VK_KHR_SURFACE_EXTENSION_NAME);
}