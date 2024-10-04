use crate::vulkan_bindings;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const DISPLAY_WIDTH: i32 = 1920;
pub const DISPLAY_HEIGHT: i32 = 1080;

pub struct WindowParameters {
    pub Hinstance: vulkan_bindings::HINSTANCE,
    pub Hwnd : vulkan_bindings::HWND,
    pub Title : String
}

impl WindowParameters {

    pub extern "C" fn window_proc(
        h_wnd: vulkan_bindings::HWND,
        u_msg: vulkan_bindings::UINT,
        w_param: vulkan_bindings::WPARAM,
        l_param: vulkan_bindings::LPARAM,
    ) -> vulkan_bindings::LRESULT {
        unsafe {
            match u_msg {
                vulkan_bindings::WM_CLOSE => vulkan_bindings::DestroyWindow(h_wnd),
                _ => {0}
            };
            vulkan_bindings::DefWindowProcW(h_wnd, u_msg, w_param, l_param)
        }
    }

    pub fn new(window_title: String) -> Self {
        unsafe {
            let h_instance = vulkan_bindings::GetModuleHandleW(std::ptr::null());
            let class_name = window_title.clone() + "_Window_Class";
            let mut class_name : Vec<vulkan_bindings::wchar_t> = class_name.encode_utf16().collect();
            class_name.push(0);
            let mut wndClass : vulkan_bindings::WNDCLASSW = std::mem::zeroed();
            wndClass.lpszClassName = class_name.as_ptr();
            wndClass.hInstance = h_instance;
            wndClass.hIcon = vulkan_bindings::LoadIconW(std::ptr::null_mut(), IDI_WINLOGO.0);
            wndClass.hCursor = vulkan_bindings::LoadCursorW(std::ptr::null_mut(), IDC_ARROW.0);
            wndClass.lpfnWndProc = Some(WindowParameters::window_proc);

            vulkan_bindings::RegisterClassW(& wndClass);
            let style : vulkan_bindings::DWORD = vulkan_bindings::WS_CAPTION | vulkan_bindings::WS_MINIMIZEBOX | vulkan_bindings::WS_SYSMENU;
            
            let mut rect :vulkan_bindings::RECT = std::mem::zeroed();
            rect.left = 100;
            rect.top = 100;
            rect.right = rect.left +  DISPLAY_WIDTH;
            rect.bottom = rect.top + DISPLAY_HEIGHT;

            vulkan_bindings::AdjustWindowRect(&mut rect, style, 0);

            let mut title : Vec<vulkan_bindings::wchar_t> = window_title.clone().encode_utf16().collect();
            title.push(0);
            let m_hwnd = vulkan_bindings::CreateWindowExW(
                0, 
                class_name.as_ptr(), 
                title.as_ptr(),
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

            // vulkan_bindings::ShowWindow(m_hwnd, vulkan_bindings::SW_SHOW as i32);
            WindowParameters {
                Hinstance : h_instance,
                Hwnd: m_hwnd,
                Title: window_title
            }
        }
    }

    pub fn destroy(self)
    {
        let mut class_name : Vec<vulkan_bindings::wchar_t> = "Anvil_Window_Class".encode_utf16().collect();
        class_name.push(0);

        unsafe {
            vulkan_bindings::UnregisterClassW(class_name.as_ptr(), self.Hinstance);
        }
    }
}