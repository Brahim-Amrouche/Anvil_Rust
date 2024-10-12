
//Exported
EXPORTED_VULKAN_FUNCTION!(vkGetInstanceProcAddr);

//Global Vulkan Functions
EXPORTED_VULKAN_FUNCTION!(vkEnumerateInstanceExtensionProperties);
EXPORTED_VULKAN_FUNCTION!(vkEnumerateInstanceLayerProperties);
EXPORTED_VULKAN_FUNCTION!(vkCreateInstance);
EXPORTED_VULKAN_FUNCTION!(vkDestroyInstance);

// Instance Level Vulkan Functions
EXPORTED_VULKAN_FUNCTION!(vkEnumeratePhysicalDevices);
EXPORTED_VULKAN_FUNCTION!(vkEnumerateDeviceExtensionProperties);
EXPORTED_VULKAN_FUNCTION!(vkGetPhysicalDeviceProperties);
EXPORTED_VULKAN_FUNCTION!(vkGetPhysicalDeviceFeatures);
EXPORTED_VULKAN_FUNCTION!(vkCreateDevice);
EXPORTED_VULKAN_FUNCTION!(vkGetDeviceProcAddr);
EXPORTED_VULKAN_FUNCTION!(vkGetPhysicalDeviceQueueFamilyProperties);
EXPORTED_VULKAN_FUNCTION!(vkGetPhysicalDeviceSurfacePresentModesKHR);
EXPORTED_VULKAN_FUNCTION!(vkCreateCommandPool);
EXPORTED_VULKAN_FUNCTION!(vkResetCommandPool);
EXPORTED_VULKAN_FUNCTION!(vkDestroyCommandPool);
EXPORTED_VULKAN_FUNCTION!(vkAllocateCommandBuffers);
EXPORTED_VULKAN_FUNCTION!(vkBeginCommandBuffer);
EXPORTED_VULKAN_FUNCTION!(vkEndCommandBuffer);
EXPORTED_VULKAN_FUNCTION!(vkResetCommandBuffer);

// Instance Level Vulkan Extensions Functions
EXPORTED_VULKAN_FUNCTION!(vkGetPhysicalDeviceSurfaceSupportKHR);
EXPORTED_VULKAN_FUNCTION!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR);
EXPORTED_VULKAN_FUNCTION!(vkGetPhysicalDeviceSurfaceFormatsKHR);
EXPORTED_VULKAN_FUNCTION!(vkDestroySurfaceKHR);
EXPORTED_VULKAN_FUNCTION!(vkCreateWin32SurfaceKHR);


// Device Level Vulkan Function
EXPORTED_VULKAN_FUNCTION!(vkGetDeviceQueue);
EXPORTED_VULKAN_FUNCTION!(vkDeviceWaitIdle);
EXPORTED_VULKAN_FUNCTION!(vkDestroyDevice);
EXPORTED_VULKAN_FUNCTION!(vkCreateBuffer);
EXPORTED_VULKAN_FUNCTION!(vkFreeCommandBuffers);
EXPORTED_VULKAN_FUNCTION!(vkGetBufferMemoryRequirements);
EXPORTED_VULKAN_FUNCTION!(vkCreateSemaphore);
EXPORTED_VULKAN_FUNCTION!(vkDestroySemaphore);
EXPORTED_VULKAN_FUNCTION!(vkCreateFence);
EXPORTED_VULKAN_FUNCTION!(vkWaitForFences);
EXPORTED_VULKAN_FUNCTION!(vkResetFences);
EXPORTED_VULKAN_FUNCTION!(vkDestroyFence);
EXPORTED_VULKAN_FUNCTION!(vkQueueSubmit);
EXPORTED_VULKAN_FUNCTION!(vkQueueWaitIdle);

// Device Level Vulkan Extensions Functions
EXPORTED_VULKAN_FUNCTION!(vkCreateSwapchainKHR);
EXPORTED_VULKAN_FUNCTION!(vkGetSwapchainImagesKHR);
EXPORTED_VULKAN_FUNCTION!(vkDestroySwapchainKHR);
EXPORTED_VULKAN_FUNCTION!(vkAcquireNextImageKHR);
EXPORTED_VULKAN_FUNCTION!(vkQueuePresentKHR);

