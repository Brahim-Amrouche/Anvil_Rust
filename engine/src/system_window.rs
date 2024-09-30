use crate::vulkan_bindings;
// use windows::core::*;
// use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
// use windows::Win32::System::LibraryLoader::*;
// use windows::Win32::Graphics::Gdi::*;

pub struct WindowParameters {
    Hinstance: vulkan_bindings::HINSTANCE,
    Hwnd : vulkan_bindings::HWND,
}

impl WindowParameters {
    pub fn new() {
        unsafe {
            // let window_cls_name = w!("ANVIL_WINDOW_CLASS");
            // let h_instance = GetModuleHandleW(None).unwrap();

            // let wnd_class = WNDCLASSW {
            //     hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            //     hInstance : h_instance,
            //     lpszClassName 
            // }
            let h_instance = vulkan_bindings::GetModuleHandleW(std::ptr::null());
            let mut class_name : Vec<vulkan_bindings::wchar_t> = "Anvil_Window_Class".encode_utf16().collect();
            class_name.push(0);
            let mut wndClass : vulkan_bindings::WNDCLASSW = std::mem::zeroed();
            wndClass.lpszClassName = class_name.as_ptr();
            wndClass.hInstance = h_instance;
            wndClass.hIcon = vulkan_bindings::LoadIconW(std::ptr::null_mut(), IDI_WINLOGO.0);
            wndClass.hCursor = vulkan_bindings::LoadCursorW(std::ptr::null_mut(), IDC_ARROW.0);

            vulkan_bindings::RegisterClassW(& wndClass);
            let style : vulkan_bindings::DWORD = vulkan_bindings::WS_CAPTION | vulkan_bindings::WS_MINIMIZEBOX | vulkan_bindings::WS_SYSMENU;
            
            let width = 720;
            let height = 1366;

            let mut rect :vulkan_bindings::RECT = std::mem::zeroed();
            rect.left = 100;
            rect.top = 100;
            rect.right = rect.left + width;
            rect.bottom = rect.top + height;

            vulkan_bindings::AdjustWindowRect(&mut rect, style, 0);

            let mut window_title : Vec<vulkan_bindings::wchar_t> = "Anvil".encode_utf16().collect();
            window_title.push(0);
            let m_hwn = vulkan_bindings::CreateWindowExW(
                0, 
                class_name.as_ptr(), 
                window_title.as_ptr(),
                style,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                h_instance,
                std::ptr::null_mut()
            );

            // vulkan_bindings::ShowWindow(m_hwn, vulkan_bindings::SW_SHOW as i32);
        }
    }
}