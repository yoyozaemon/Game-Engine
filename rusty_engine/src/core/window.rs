#![allow(non_snake_case)]

use std::borrow::Borrow;
use std::os::windows::prelude::OsStrExt;

// Core
use crate::core::event::*;
use crate::core::window_event::*;
use crate::core::keyboard_event::*;
use crate::core::mouse_event::*;
use crate::core::utils::*;

// Win32
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Dwm::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::*;

// Renderer
use crate::renderer::graphics_context::*;


pub struct WindowData
{
    pub Title:        String,
    pub Width:        u32,
    pub Height:       u32,
    pub IsVSyncOn:    bool,
    pub IsFullscreen: bool,
    pub IsMinimized:  bool,
}

pub struct Window
{
    m_Data:            WindowData,
    m_Handle:          HWND,
    m_WindowClass:     WNDCLASSA,
    m_GraphicsContext: RustyRef<GraphicsContext>,
    m_EventBuffer:     Vec<Box<dyn Event>>,
}

impl Window
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(title: &str, width: u32, height: u32, vsync: bool) -> Window
    {
        unsafe
        {
            // Create window class
            let wndClass = WNDCLASSA {
                style: CS_OWNDC,
                lpfnWndProc: Some( Window::WindowProcSetup ),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleA(None),
                hIcon: HICON(LoadImageA(GetModuleHandleA(None), "resources/icons/editor_icon.ico\0", IMAGE_ICON, 0, 0, LR_LOADFROMFILE | LR_DEFAULTSIZE | LR_SHARED).0),
                lpszClassName: PSTR("Rusty Engine Window\0".as_ptr() as _),
                ..Default::default()
            };

            // Create window data
            let windowData = WindowData {
                Title: String::from(title),
                Width: width,
                Height: height,
                IsVSyncOn: vsync,
                IsFullscreen: false,
                IsMinimized: false,
            };

            return Window {
                m_Data: windowData,
                m_WindowClass: wndClass,
                m_Handle: HWND::default(),
                m_GraphicsContext: RustyRef::CreateEmpty(),
                m_EventBuffer: vec![],
            };
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Initialize(&mut self)
    {
        unsafe
        {
            // Initialize the window instance
            let result: u16 = RegisterClassA(&self.m_WindowClass);
            debug_assert!(result != 0);

            // Adjust the window rect
            let mut rect = RECT {
                left: 100,
                right: self.m_Data.Width as i32 + 100 ,
                top: 100,
                bottom: self.m_Data.Height as i32 + 100
            };

            debug_assert!(AdjustWindowRect(&mut rect, WS_CAPTION | WS_MINIMIZEBOX | WS_SYSMENU | WS_SIZEBOX, false) != false);

            // Get a pointer to the window data so we can pass it as a creation parameter when creating HWND handle
            let pData: *mut Window = self;

            self.m_Handle = CreateWindowExA(WINDOW_EX_STYLE::default(), 
                                            self.m_WindowClass.lpszClassName, 
                                            PSTR(self.m_Data.Title.as_mut_ptr()),
                                            WS_CAPTION | WS_MINIMIZEBOX | WS_SYSMENU | WS_MAXIMIZEBOX | WS_SIZEBOX,
                                            CW_USEDEFAULT,
                                            CW_USEDEFAULT, 
                                            rect.right - rect.left,
                                            rect.bottom - rect.top,
                                            None, None, GetModuleHandleA(None), pData as *mut _ as _);

            // Check for correct inizializations
            debug_assert!(self.m_Handle.0 != 0);
            
            // Set dark theme
            let useDarkMode = BOOL::from(true);
            SetWindowTheme(self.m_Handle, "DarkMode_Explorer\0", PWSTR(std::ptr::null_mut() as _)).expect("");
            DwmSetWindowAttribute(self.m_Handle, DWMWA_USE_IMMERSIVE_DARK_MODE, &useDarkMode as *const _ as _, std::mem::size_of_val(&useDarkMode) as _).expect("Failed to set dark theme!");

            // Create graphics context
            self.m_GraphicsContext = GraphicsContext::Create(self);

            // Open the window
            ShowWindow(self.m_Handle, SW_MAXIMIZE);

        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn ProcessMessages(&mut self)
    {
        unsafe
        {
            let mut msg = MSG::default();

            while PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE).into()
            {
                if msg.message == WM_QUIT
                {
                    return;
                }

                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetFullscreen(&mut self, state: bool)
    {
        unsafe        
        {
            if self.m_Data.IsFullscreen != state
            {
                self.m_Data.IsFullscreen = state;

                let mut windowRect = RECT::default();
                GetWindowRect(self.m_Handle, &mut windowRect);

                if self.m_Data.IsFullscreen
                {
                    // Switching to fullscreen
                    let windowStyle =  WS_OVERLAPPEDWINDOW & !(WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX);
                    SetWindowLongA(self.m_Handle, GWL_STYLE, windowStyle.0 as i32);

                    let hMonitor: HMONITOR = MonitorFromWindow(self.m_Handle, MONITOR_DEFAULTTONEAREST);

                    let mut monitorInfo = MONITORINFO::default();
                    monitorInfo.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

                    GetMonitorInfoA(hMonitor, &mut monitorInfo);

                    SetWindowPos(self.m_Handle, 
                                 HWND_TOP,
                                 monitorInfo.rcMonitor.left,
                                 monitorInfo.rcMonitor.top,
                                 monitorInfo.rcMonitor.right - monitorInfo.rcMonitor.left,
                                 monitorInfo.rcMonitor.bottom - monitorInfo.rcMonitor.top,
                                 SWP_FRAMECHANGED | SWP_NOACTIVATE);

                    ShowWindow(self.m_Handle, SW_MAXIMIZE);
                }
                else
                {
                    // Exiting fullscreen
                    SetWindowLongA(self.m_Handle, GWL_STYLE, WS_OVERLAPPEDWINDOW.0 as i32);

                    SetWindowPos(self.m_Handle, 
                                 HWND_NOTOPMOST, 
                                 windowRect.left,
                                 windowRect.top,
                                 windowRect.right - windowRect.left,
                                 windowRect.bottom - windowRect.top,
                                 SWP_FRAMECHANGED | SWP_NOACTIVATE);

                    ShowWindow(self.m_Handle, SW_NORMAL);
                }
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetMinimized(&mut self, state: bool)
    {
        self.m_Data.IsMinimized = state;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetVSync(&mut self, state: bool)
    {
        self.m_Data.IsVSyncOn = state;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetData(&mut self) -> &mut WindowData
    {
        return &mut self.m_Data;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetHandle(&self) -> HWND
    {
        return self.m_Handle;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetTitle(&self) -> &String
    {
        return &self.m_Data.Title;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetWidth(&self) -> u32
    {
        return self.m_Data.Width;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetHeight(&self) -> u32
    {
        return self.m_Data.Height;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVSync(&self) -> bool
    {
        return self.m_Data.IsVSyncOn;
    }
    
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn IsFullscreen(&self) -> bool
    {
        return self.m_Data.IsFullscreen;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn IsMinimized(&self) -> bool
    {
        return self.m_Data.IsMinimized;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetGfxContext(&self) -> RustyRef<GraphicsContext>
    {
        return self.m_GraphicsContext.clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetEventBuffer(&mut self) -> &mut Vec<Box<dyn Event>>
    {
        return &mut self.m_EventBuffer;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub unsafe extern "system" fn WindowProcSetup(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT
    {
        if Msg == WM_CREATE
        {
            let pCreate = lParam.0 as *const CREATESTRUCTA;
            let windowData = (*pCreate).lpCreateParams as *const WindowData;
            SetWindowLongPtrA(hWnd, GWLP_USERDATA, windowData as _);
            SetWindowLongPtrA(hWnd, GWLP_WNDPROC, Window::WindowProc as _);

            return Window::WindowProc(hWnd, Msg, wParam, lParam);
        }

        return DefWindowProcA(hWnd, Msg, wParam, lParam);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub unsafe extern "system" fn WindowProc(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT
    {
        let window = GetWindowLongPtrA(hWnd, GWLP_USERDATA) as *mut Window;
        
        if Msg == WM_DESTROY || Msg == WM_CLOSE
        {
            let event = WindowClosedEvent::Create();
            (*window).m_EventBuffer.push(Box::new(event));
            
            PostQuitMessage(0);
            return LRESULT(0);
        }
        else if Msg == WM_KEYDOWN || Msg == WM_SYSKEYDOWN
        {
            let event = KeyPressedEvent::Create(VIRTUAL_KEY(wParam.0 as u16), (lParam.0 & 0xffff) as u32);
            (*window).m_EventBuffer.push(Box::new(event));
        }
        else if Msg == WM_KEYUP || Msg == WM_SYSKEYUP
        {
            let event = KeyReleasedEvent::Create(VIRTUAL_KEY(wParam.0 as u16));
            (*window).m_EventBuffer.push(Box::new(event));
        }
        else if Msg == WM_CHAR
        {
            let event = KeyTypedEvent::Create(char::from_u32(wParam.0 as u32).unwrap());
            (*window).m_EventBuffer.push(Box::new(event));
        }
        else if Msg == WM_LBUTTONDOWN
        {
            let event = MouseButtonPressedEvent::Create(VK_LBUTTON);
            (*window).m_EventBuffer.push(Box::new(event));
        }
        else if Msg == WM_RBUTTONDOWN
        {
            let event = MouseButtonPressedEvent::Create(VK_RBUTTON);
            (*window).m_EventBuffer.push(Box::new(event));
        }
        else if Msg == WM_LBUTTONUP
        {
            let event = MouseButtonReleasedEvent::Create(VK_LBUTTON);
            (*window).m_EventBuffer.push(Box::new(event));
        }
        else if Msg == WM_RBUTTONUP
        {
            let event = MouseButtonReleasedEvent::Create(VK_RBUTTON);
            (*window).m_EventBuffer.push(Box::new(event));
        }
        else if Msg == WM_MOUSEMOVE
        {
            let point = POINT{x: (lParam.0 & 0xffff) as i32, y: ((lParam.0 >> 16) & 0xffff) as i32};
            let event = MouseMovedEvent::Create(point.x as u32, point.y as u32);
            (*window).m_EventBuffer.push(Box::new(event));
        }
        else if Msg == WM_MOUSEWHEEL
        {
            let event = MouseScrolledEvent::Create(0, (((wParam.0 >> 16) & 0xffff) as i16) as i32 / 120);
            (*window).m_EventBuffer.push(Box::new(event));
        }
        else if Msg == WM_SIZE
        {
            if wParam.0 == SIZE_MINIMIZED as usize
            {
                (*window).SetMinimized(true);
            }
            else if wParam.0 == SIZE_RESTORED as usize
            {
                (*window).SetMinimized(false);
            }

            (*window).m_Data.Width = (lParam.0 & 0xffff) as u32;
            (*window).m_Data.Height = ((lParam.0 >> 16) & 0xffff) as u32;

            (*window).m_GraphicsContext.GetRefMut().ResizeSwapChain((*window).m_Data.Width, (*window).m_Data.Height);

            let event = WindowResizedEvent::Create((*window).m_Data.Width, (*window).m_Data.Height);
            (*window).m_EventBuffer.push(Box::new(event));
        }

        return DefWindowProcA(hWnd, Msg, wParam, lParam);
    }
}

impl Drop for Window
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn drop(&mut self) 
    {
        unsafe
        {
            UnregisterClassA(self.m_WindowClass.lpszClassName, GetModuleHandleA(None));
            DestroyWindow(self.m_Handle);
        }
    }
}