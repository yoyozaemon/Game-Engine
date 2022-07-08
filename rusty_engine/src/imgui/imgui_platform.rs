#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// core
use crate::core::window::*;
use crate::core::events::event::*;
use crate::core::events::keyboard_event::*;
use crate::core::events::mouse_event::*;
use crate::core::input::*;

// Win32
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct ImGuiPlatform
{
    m_WindowHandle: HWND,
    m_Context:      imgui::Context
}

impl ImGuiPlatform
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create() -> ImGuiPlatform
    {
        return ImGuiPlatform {
            m_WindowHandle: HWND::default(),
            m_Context: imgui::Context::create()
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Initialize(&mut self, window: &Window)
    {
        self.m_WindowHandle = window.GetHandle();
        
        let io: &mut imgui::Io = self.m_Context.io_mut();
        io.backend_flags.insert(imgui::BackendFlags::HAS_MOUSE_CURSORS);
        io.backend_flags.insert(imgui::BackendFlags::HAS_SET_MOUSE_POS);
        io.config_flags.insert(imgui::ConfigFlags::DOCKING_ENABLE);

        io[imgui::Key::Tab]         = VK_TAB.0 as _;
        io[imgui::Key::LeftArrow]   = VK_LEFT.0 as _;
        io[imgui::Key::RightArrow]  = VK_RIGHT.0 as _;
        io[imgui::Key::UpArrow]     = VK_UP.0 as _;
        io[imgui::Key::DownArrow]   = VK_DOWN.0 as _;
        io[imgui::Key::PageUp]      = VK_PRIOR.0 as _;
        io[imgui::Key::PageDown]    = VK_NEXT.0 as _;
        io[imgui::Key::Home]        = VK_HOME.0 as _;
        io[imgui::Key::End]         = VK_END.0 as _;
        io[imgui::Key::Insert]      = VK_INSERT.0 as _;
        io[imgui::Key::Delete]      = VK_DELETE.0 as _;
        io[imgui::Key::Backspace]   = VK_BACK.0 as _;
        io[imgui::Key::Space]       = VK_SPACE.0 as _;
        io[imgui::Key::Enter]       = VK_RETURN.0 as _;
        io[imgui::Key::Escape]      = VK_ESCAPE.0 as _;
        io[imgui::Key::KeyPadEnter] = VK_RETURN .0 as _;
        io[imgui::Key::A]           = VK_A.0 as _;
        io[imgui::Key::C]           = VK_C.0 as _;
        io[imgui::Key::V]           = VK_V.0 as _;
        io[imgui::Key::X]           = VK_X.0 as _;
        io[imgui::Key::Y]           = VK_Y.0 as _;
        io[imgui::Key::Z]           = VK_Z.0 as _;

        io.display_size = [window.GetWidth() as f32, window.GetHeight() as f32];
        
        self.m_Context.set_platform_name(Some(imgui::im_str!("ImGuiPlatformWin32 {}", env!("CARGO_PKG_VERSION"))));

        self.SetDarkTheme();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn BeginFrame(&mut self) -> imgui::Ui
    {
        let io: &mut imgui::Io = self.m_Context.io_mut();

        let mut rect: RECT = RECT::default();
        unsafe { GetClientRect(self.m_WindowHandle, &mut rect) };
        io.display_size = [(rect.right - rect.left) as f32, (rect.bottom - rect.top) as f32];

        io.key_shift = Input::IsKeyPressed(VK_SHIFT);
        io.key_alt = Input::IsKeyPressed(VK_MENU);
        io.key_ctrl = Input::IsKeyPressed(VK_CONTROL);
        io.key_super = false;

        let foregroundWindow: HWND = unsafe { GetForegroundWindow() };
        if foregroundWindow == self.m_WindowHandle
        {
            let mousePos = Input::GetMousePosition();
            io.mouse_pos = [mousePos.x, mousePos.y];
        }

        return self.m_Context.frame();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn EndFrame(&mut self)
    {

    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnEvent(&mut self, event: &dyn Event)
    {
        if event.GetType() == EventType::KeyPressed
        {
            let keyPressedEvent: &KeyPressedEvent = event.AsAny().downcast_ref::<KeyPressedEvent>().expect("Event is now a KeyPressedEvent");
            self.OnKeyPressed(keyPressedEvent);
        }
        else if event.GetType() == EventType::KeyReleased
        {
            let keyReleasedEvent: &KeyReleasedEvent = event.AsAny().downcast_ref::<KeyReleasedEvent>().expect("Event is now a KeyReleasedEvent");
            self.OnKeyReleased(keyReleasedEvent);
        }
        else if event.GetType() == EventType::KeyTyped
        {
            let keyTypedEvent: &KeyTypedEvent = event.AsAny().downcast_ref::<KeyTypedEvent>().expect("Event is now a KeyTypedEvent");
            self.OnKeyTyped(keyTypedEvent);
        }
        else if event.GetType() == EventType::MouseButtonPressed
        {
            let mouseButtonPressedEvent: &MouseButtonPressedEvent = event.AsAny().downcast_ref::<MouseButtonPressedEvent>().expect("Event is now a MouseButtonPressedEvent");
            self.OnMouseButtonPressed(mouseButtonPressedEvent);
        }
        else if event.GetType() == EventType::MouseButtonReleased
        {
            let mouseButtonReleasedEvent: &MouseButtonReleasedEvent = event.AsAny().downcast_ref::<MouseButtonReleasedEvent>().expect("Event is now a MouseButtonReleasedEvent");
            self.OnMouseButtonReleased(mouseButtonReleasedEvent);
        }
        else if event.GetType() == EventType::MouseScrolled
        {
            let mouseScrolledEvent: &MouseScrolledEvent = event.AsAny().downcast_ref::<MouseScrolledEvent>().expect("Event is now a MouseScrolledEvent");
            self.OnMouseScrolled(mouseScrolledEvent);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn UpdateDeltaTime(&mut self, deltaTime: f32)
    {
        let io: &mut imgui::Io = self.m_Context.io_mut();
        io.delta_time = deltaTime;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetContext(&mut self) -> &mut imgui::Context
    {
        return &mut self.m_Context;
    } 

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnKeyPressed(&mut self, event: &KeyPressedEvent)
    {
        let io: &mut imgui::Io = self.m_Context.io_mut();

        if event.GetKeyCode().0 < 256
        {
            io.keys_down[event.GetKeyCode().0 as usize] = true;
        }    
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnKeyReleased(&mut self, event: &KeyReleasedEvent)
    {
        let io: &mut imgui::Io = self.m_Context.io_mut();
        
        if event.GetKeyCode().0 < 256
        {
            io.keys_down[event.GetKeyCode().0 as usize] = false;
        } 
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnKeyTyped(&mut self, event: &KeyTypedEvent)
    {
        let io: &mut imgui::Io = self.m_Context.io_mut();
        io.add_input_character(event.GetChar());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnMouseButtonPressed(&mut self, event: &MouseButtonPressedEvent)
    {
        let io: &mut imgui::Io = self.m_Context.io_mut();

        if event.GetMouseButton() == VK_LBUTTON
        {
            io.mouse_down[0] = true;
        }
        else if event.GetMouseButton() == VK_RBUTTON
        {
            io.mouse_down[1] = true;
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnMouseButtonReleased(&mut self, event: &MouseButtonReleasedEvent)
    {
        let io: &mut imgui::Io = self.m_Context.io_mut();
        
        if event.GetMouseButton() == VK_LBUTTON
        {
            io.mouse_down[0] = false;
        }
        else if event.GetMouseButton() == VK_RBUTTON
        {
            io.mouse_down[1] = false;
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnMouseScrolled(&mut self, event: &MouseScrolledEvent)
    {
        let io: &mut imgui::Io = self.m_Context.io_mut();
        io.mouse_wheel += event.GetYOffset() as f32;
        io.mouse_wheel_h += event.GetXOffset() as f32;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn SetDarkTheme(&mut self)
    {
        let style = self.m_Context.style_mut();
        style.window_min_size[0] = 370.0;
        style.frame_padding = [5.0, 5.0];

        style.colors[imgui::StyleColor::Header as usize]             = [0.2, 0.205, 0.21, 1.0];
		style.colors[imgui::StyleColor::HeaderHovered as usize]      = [0.3, 0.305, 0.31, 1.0];
		style.colors[imgui::StyleColor::HeaderActive as usize]       = [0.15, 0.1505, 0.151, 1.0];
        
        // Window BG
        style.colors[imgui::StyleColor::WindowBg as usize]           = [0.1, 0.1, 0.1, 1.0];

		// Buttons   
		style.colors[imgui::StyleColor::Button as usize]             = [0.2, 0.205, 0.21, 1.0];
		style.colors[imgui::StyleColor::ButtonHovered as usize]      = [0.3, 0.305, 0.31, 1.0];
		style.colors[imgui::StyleColor::ButtonActive as usize]       = [0.15, 0.1505, 0.151, 1.0];

		// Frame BG
		style.colors[imgui::StyleColor::FrameBg as usize]            = [0.2, 0.205, 0.21, 1.0];
		style.colors[imgui::StyleColor::FrameBgHovered as usize]     = [0.3, 0.305, 0.31, 1.0];
		style.colors[imgui::StyleColor::FrameBgActive as usize]      = [0.15, 0.1505, 0.151, 1.0];

		// Tabs
		style.colors[imgui::StyleColor::Tab as usize]                = [0.15, 0.1505, 0.151, 1.0];
		style.colors[imgui::StyleColor::TabHovered as usize]         = [0.38, 0.3805, 0.381, 1.0];
		style.colors[imgui::StyleColor::TabActive as usize]          = [0.28, 0.2805, 0.281, 1.0];
		style.colors[imgui::StyleColor::TabUnfocused as usize]       = [0.15, 0.1505, 0.151, 1.0];
		style.colors[imgui::StyleColor::TabUnfocusedActive as usize] = [0.2, 0.205, 0.21, 1.0];

		// Title
		style.colors[imgui::StyleColor::TitleBg as usize]            = [0.15, 0.1505, 0.151, 1.0];
		style.colors[imgui::StyleColor::TitleBgActive as usize]      = [0.15, 0.1505, 0.151, 1.0];
		style.colors[imgui::StyleColor::TitleBgCollapsed as usize]   = [0.15, 0.1505, 0.151, 1.0];

    }
}