#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// std
use std::sync::Mutex;

// Win32
use directx_math::XMFLOAT2;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// other
use lazy_static::*;

pub struct Input
{
    m_WindowHandle:  HWND,
    m_CursorEnabled: bool
}

lazy_static! 
{ 
    static ref s_Instance: Mutex<Input> = Mutex::new(Input { 
        m_WindowHandle: HWND::default(), 
        m_CursorEnabled: true 
    });
}

impl Input
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Initialize(windowHandle: HWND)
    {
        s_Instance.lock().unwrap().m_WindowHandle = windowHandle;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn IsKeyPressed(key: VIRTUAL_KEY) -> bool
    {
        unsafe
        {
            return (GetAsyncKeyState(key.0 as i32) & (1 << 15)) != 0;
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn IsMouseButtonPressed(button: VIRTUAL_KEY) -> bool
    {
        unsafe
        {
            return (GetAsyncKeyState(button.0 as i32) & (1 << 15)) != 0;
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn IsCursorEnabled() -> bool
    {
        return s_Instance.lock().unwrap().m_CursorEnabled;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetMousePosition() -> XMFLOAT2
    {
        unsafe
        {
            let mut point = POINT::default();
            
            let mut result: BOOL = GetCursorPos(&mut point);
            debug_assert!(result.as_bool(), "Failed to get mouse position!");

            result = ScreenToClient(s_Instance.lock().unwrap().m_WindowHandle, &mut point);
            debug_assert!(result.as_bool(), "Could not convert from screen coords to client coords!");

            return XMFLOAT2::set(point.x as f32, point.y as f32);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetCursorState(enabled: bool)
    {
        unsafe
        {
            s_Instance.lock().unwrap().m_CursorEnabled = enabled;

            if enabled
            {
                while ShowCursor(true) < 0 {}
            }
            else
            {
                while ShowCursor(false) >= 0 {}
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetMousePosition(xPos: i32, yPos: i32)
    {
        unsafe
        {
            let mut point: POINT = POINT { x: xPos, y: yPos };
            let mut result: BOOL = ClientToScreen(s_Instance.lock().unwrap().m_WindowHandle, &mut point);
            debug_assert!(result.as_bool(), "Could not convert from client coords to screen coords!");

            result = SetCursorPos(point.x, point.y);
            debug_assert!(result.as_bool(), "Failed setting the mouse position to x:{}, y:{}", point.x, point.y);
        }
    }
}