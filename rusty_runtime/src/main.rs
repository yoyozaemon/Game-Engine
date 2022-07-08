#![allow(non_snake_case)]

extern crate rusty_engine;

// rusty_engine
use rusty_engine::renderer::scene_renderer::*;
use rusty_engine::renderer::renderer::*;
use rusty_engine::core::input::*;
use rusty_engine::core::timestep::*;
use rusty_engine::core::timer::*;
use rusty_engine::core::event::*;
use rusty_engine::core::window_event::*;
use rusty_engine::core::window::*;
use rusty_engine::core::utils::*;
use rusty_engine::scene::scene::*;

pub struct RustyRuntimeApp
{
    m_IsRunning:       bool,
    m_Timer:           Timer,
    m_Window:          Window,
    m_SceneRenderer:   SceneRenderer,
    m_SceneFilePath:   String,
    m_Scene:           RustyRef<Scene>
}

impl RustyRuntimeApp
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(args: Vec<String>, windowWidth: u32, windowHeight: u32) -> RustyRuntimeApp
    {
        return RustyRuntimeApp {
            m_IsRunning: false,
            m_Timer: Timer::Create(),
            m_Window: Window::Create("Rusty Runtime\0", windowWidth, windowHeight, true),
            m_SceneRenderer: SceneRenderer::Create(true),
            m_SceneFilePath: args[1].clone(),
            m_Scene: RustyRef::CreateEmpty(),
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Initialize(&mut self)
    {
        // Initialize systems
        self.m_Window.Initialize();
        Input::Initialize(self.m_Window.GetHandle());
        Renderer::Initialize(self.m_Window.GetGfxContext());
        self.m_SceneRenderer.Initialize();

        // Load scene scene
        self.m_Scene = Scene::Create();
        self.m_Scene.GetRefMut().Deserialize(&self.m_SceneFilePath);

        self.m_IsRunning = true;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Run(&mut self)
    {
        while self.m_IsRunning
        {
            let timestep = self.m_Timer.GetElapsedTime();
            self.m_Timer.Reset();

            self.OnUpdate(timestep);
            self.OnRender();

            self.m_Timer.Stop();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnUpdate(&mut self, timestep: Timestep)
    {
        // Process events
        self.m_Window.ProcessMessages();

        while !self.m_Window.GetEventBuffer().is_empty()
        {
            let event = self.m_Window.GetEventBuffer().remove(0);
            self.OnEvent(event.as_ref());
        }

        self.m_Scene.GetRefMut().OnUpdate(timestep);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnRender(&mut self)
    {
        self.m_Scene.GetRefMut().OnRuntimeRender(&mut self.m_SceneRenderer);
        self.m_Window.GetGfxContext().GetRef().Present(self.m_Window.GetVSync());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnEvent(&mut self, event: &dyn Event)
    {
        if event.GetType() == EventType::WindowClosed
        {
            let windowClosedEvent: &WindowClosedEvent = event.AsAny().downcast_ref::<WindowClosedEvent>().expect("Event is now a WindowClosedEvent");
            self.OnWindowClosed(windowClosedEvent);
        }
        self.m_Scene.GetRefMut().GetEditorCamera().OnEvent(event);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnWindowClosed(&mut self, event: &WindowClosedEvent)
    {
        self.m_IsRunning = false;
    }
}

fn main() 
{
    let args: Vec<String> = std::env::args().collect();
    let mut app = RustyRuntimeApp::Create(args, 1600, 900);
    app.Initialize();
    app.Run();
}