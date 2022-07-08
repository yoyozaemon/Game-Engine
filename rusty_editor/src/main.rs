#![allow(non_snake_case)]
#[macro_use]

extern crate imgui;
extern crate imguizmo;
extern crate rusty_engine;

pub mod panels;

// Editor
use panels::console_panel::*;
use panels::scene_hierarchy_panel::*;
use panels::material_inspector_panel::*;
use panels::content_browser_panel::*;

// Win32
use directx_math::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

// rusty_engine
use rusty_engine::renderer::texture::*;
use rusty_engine::renderer::scene_renderer::*;
use rusty_engine::renderer::renderer::*;
use rusty_engine::imgui::imgui_renderer::*;
use rusty_engine::imgui::imgui_platform::*;
use rusty_engine::core::input::*;
use rusty_engine::core::timestep::*;
use rusty_engine::core::timer::*;
use rusty_engine::core::event::*;
use rusty_engine::core::window_event::*;
use rusty_engine::core::keyboard_event::*;
use rusty_engine::core::window::*;
use rusty_engine::core::utils::*;
use rusty_engine::core::file_dialog::*;
use rusty_engine::core::asset_manager::*;
use rusty_engine::scene::scene::*;
use rusty_engine::scene::entity::*;
use rusty_engine::scene::component::*;

pub struct RustyEditorApp
{
    m_IsRunning:              bool,
    m_Timer:                  Timer,
    m_Window:                 Window,
    m_ImGuiPlatform:          ImGuiPlatform,
    m_ImGuiRenderer:          ImGuiRenderer,
    
    m_SceneRenderer:          SceneRenderer,
    m_Scene:                  RustyRef<Scene>,
    
    m_ViewportFocused:        bool,
    m_ViewportHovered:        bool,
    m_ViewportSize:           XMFLOAT2,
    m_GizmoOperation:         imguizmo::Operation,

    m_FolderIcon:             RustyRef<Texture>,
    m_PNGIcon:                RustyRef<Texture>,
    m_JPGIcon:                RustyRef<Texture>,
    m_TGAIcon:                RustyRef<Texture>,
    m_HDRIcon:                RustyRef<Texture>,
    m_FBXIcon:                RustyRef<Texture>,
    m_RSceneIcon:             RustyRef<Texture>,

    // Panels
    m_SceneHierarchyPanel:    SceneHierarchyPanel,
    m_MaterialInspectorPanel: MaterialInspectorPanel,
    m_ContentBrowserPanel:    ContentBrowserPanel
}

impl RustyEditorApp
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(windowWidth: u32, windowHeight: u32) -> RustyEditorApp
    {
        return RustyEditorApp {
            m_IsRunning: false,
            m_Timer: Timer::Create(),
            m_Window: Window::Create("Rusty Editor\0", windowWidth, windowHeight, true),
            m_ImGuiPlatform: ImGuiPlatform::Create(),
            m_ImGuiRenderer: ImGuiRenderer::Create(),
            
            m_SceneRenderer: SceneRenderer::Create(false),
            m_Scene: RustyRef::CreateEmpty(),

            m_ViewportFocused: false,
            m_ViewportHovered: false,
            m_ViewportSize: XMFLOAT2::default(),
            m_GizmoOperation: imguizmo::Operation::Translate,

            m_FolderIcon: RustyRef::CreateEmpty(),
            m_PNGIcon:    RustyRef::CreateEmpty(),
            m_JPGIcon:    RustyRef::CreateEmpty(),
            m_TGAIcon:    RustyRef::CreateEmpty(),
            m_HDRIcon:    RustyRef::CreateEmpty(),
            m_FBXIcon:    RustyRef::CreateEmpty(),
            m_RSceneIcon: RustyRef::CreateEmpty(),

            m_SceneHierarchyPanel: SceneHierarchyPanel::Create(),
            m_MaterialInspectorPanel: MaterialInspectorPanel::Create(),
            m_ContentBrowserPanel: ContentBrowserPanel::Create()
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Initialize(&mut self)
    {
        // Initialize systems
        self.m_Window.Initialize();
        Input::Initialize(self.m_Window.GetHandle());
        Renderer::Initialize(self.m_Window.GetGfxContext());
        self.m_ImGuiPlatform.Initialize(&self.m_Window);
        self.m_ImGuiRenderer.Initialize(self.m_ImGuiPlatform.GetContext());
        self.m_SceneRenderer.Initialize();

        // Load editor textures and icons
        self.m_FolderIcon = AssetManager::LoadTexture("rusty_editor/assets/folder_icon.png", true);
        self.m_ImGuiRenderer.GetTextures().replace(imgui::TextureId::from(self.m_FolderIcon.GetRaw()), self.m_FolderIcon.GetRef().CreateSRV());

        self.m_PNGIcon = AssetManager::LoadTexture("rusty_editor/assets/png_icon.png", true);
        self.m_ImGuiRenderer.GetTextures().replace(imgui::TextureId::from(self.m_PNGIcon.GetRaw()), self.m_PNGIcon.GetRef().CreateSRV());

        self.m_JPGIcon = AssetManager::LoadTexture("rusty_editor/assets/jpg_icon.png", true);
        self.m_ImGuiRenderer.GetTextures().replace(imgui::TextureId::from(self.m_JPGIcon.GetRaw()), self.m_JPGIcon.GetRef().CreateSRV());

        self.m_TGAIcon = AssetManager::LoadTexture("rusty_editor/assets/tga_icon.png", true);
        self.m_ImGuiRenderer.GetTextures().replace(imgui::TextureId::from(self.m_TGAIcon.GetRaw()), self.m_TGAIcon.GetRef().CreateSRV());

        self.m_HDRIcon = AssetManager::LoadTexture("rusty_editor/assets/hdr_icon.png", true);
        self.m_ImGuiRenderer.GetTextures().replace(imgui::TextureId::from(self.m_HDRIcon.GetRaw()), self.m_HDRIcon.GetRef().CreateSRV());

        self.m_FBXIcon = AssetManager::LoadTexture("rusty_editor/assets/fbx_icon.png", true);
        self.m_ImGuiRenderer.GetTextures().replace(imgui::TextureId::from(self.m_FBXIcon.GetRaw()), self.m_FBXIcon.GetRef().CreateSRV());

        self.m_RSceneIcon = AssetManager::LoadTexture("rusty_editor/assets/rscene_icon.png", true);
        self.m_ImGuiRenderer.GetTextures().replace(imgui::TextureId::from(self.m_RSceneIcon.GetRaw()), self.m_RSceneIcon.GetRef().CreateSRV());

        // Create scene
        self.m_Scene = Scene::Create();

        // Set panel data
        self.m_SceneHierarchyPanel.SetContext(self.m_Scene.clone());
        self.m_ContentBrowserPanel.SetAssetsDirectory("assets");
        self.m_ContentBrowserPanel.SetFolderIcon(self.m_FolderIcon.clone());
        self.m_ContentBrowserPanel.SetPNGIcon(self.m_PNGIcon.clone());
        self.m_ContentBrowserPanel.SetJPGIcon(self.m_JPGIcon.clone());
        self.m_ContentBrowserPanel.SetTGAIcon(self.m_TGAIcon.clone());
        self.m_ContentBrowserPanel.SetHDRIcon(self.m_HDRIcon.clone());
        self.m_ContentBrowserPanel.SetFBXIcon(self.m_FBXIcon.clone());
        self.m_ContentBrowserPanel.SetRSceneIcon(self.m_RSceneIcon.clone());

        Console::LogInfo("Welcome to Rusty Engine!");

        self.m_IsRunning = true;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Run(&mut self)
    {
        while self.m_IsRunning
        {
            let timestep = self.m_Timer.GetElapsedTime();
            self.m_ImGuiPlatform.UpdateDeltaTime(timestep.GetSeconds());

            self.m_Timer.Reset();

            self.OnUpdate(timestep);
            self.OnRender();

            self.m_Timer.Stop();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetWindow(&self) -> &Window
    {
        return &self.m_Window;
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
        self.m_Scene.GetRefMut().OnEditRender(&mut self.m_SceneRenderer);

        self.OnImGuiRender();

        self.m_Window.GetGfxContext().GetRef().Present(self.m_Window.GetVSync());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnImGuiRender(&mut self)
    {
        let ui: imgui::Ui = self.m_ImGuiPlatform.BeginFrame();
        
        // Create dockspace
        {
            let windowFlags = imgui::WindowFlags::MENU_BAR | imgui::WindowFlags::NO_TITLE_BAR |
                                         imgui::WindowFlags::NO_COLLAPSE | imgui::WindowFlags::NO_RESIZE |
                                         imgui::WindowFlags::NO_MOVE | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS |
                                         imgui::WindowFlags::NO_NAV_FOCUS | imgui::WindowFlags::MENU_BAR;

            let style = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));

            if let Some(dockSpace) = imgui::Window::new(im_str!("DockSpace"))
                                                .flags(windowFlags)
                                                .position([0.0, 0.0], imgui::Condition::Always)
                                                .size([self.m_Window.GetWidth() as f32, self.m_Window.GetHeight() as f32], imgui::Condition::Always)
                                                .begin(&ui)
            {
                style.pop();
                ui.dock_space(imgui::Id::from("MyDockSpace"), [0.0, 0.0]);

                // Menu bar
                if let Some(menuBarToken) = ui.begin_menu_bar()
                {
                    if let Some(menuToken) = ui.begin_menu(im_str!("File"))
                    {
                        if imgui::MenuItem::new(im_str!("New Scene")).shortcut(im_str!("Ctrl+N")).build(&ui)
                        {
                            self.m_Scene = Scene::Create();
                            self.m_SceneHierarchyPanel.SetContext(self.m_Scene.clone());
                        }
                        if imgui::MenuItem::new(im_str!("Open Scene")).shortcut(im_str!("Ctrl+O")).build(&ui)
                        {
                            let scenePath = OpenFile(String::from("Rusty Scene (*.rscene)\0*.rscene\0"));
                            if std::path::Path::new(&scenePath).exists()
                            {
                                self.m_Scene = Scene::Create();
                                self.m_Scene.GetRefMut().Deserialize(&scenePath);
                                self.m_SceneHierarchyPanel.SetContext(self.m_Scene.clone());
                                Console::LogInfo("Scene opened successfully!");
                            }
                        }
                        if imgui::MenuItem::new(im_str!("Save As")).shortcut(im_str!("Ctrl+S")).build(&ui)
                        {
                            let scenePath = SaveFile(String::from("Rusty Scene (*.rscene)\0*.rscene\0"));
                            if !scenePath.is_empty()
                            {
                                self.m_Scene.GetRef().Serialize(&scenePath);
                                Console::LogInfo("Scene saved successfully!");
                            }
                        }
                        menuToken.end();
                    }
                    menuBarToken.end();
                }

                // Renderer Info
                if let Some(rendererInfo) = imgui::Window::new(im_str!("Renderer info")).begin(&ui)
                {
                    let gpuDesc = self.m_Window.GetGfxContext().GetRef().GetGPUDescription();
                    ui.text(im_str!("{:.1} fps, {:.3} ms", 1000.0 / self.m_Timer.GetElapsedTime().GetMilliSeconds(), self.m_Timer.GetElapsedTime().GetMilliSeconds()));
                    ui.text(im_str!("DirectX 11 Info:"));
                    ui.text(im_str!("  Graphics Card: {}", String::from_utf16(&gpuDesc.Description).unwrap()));
                    ui.text(im_str!("  Video Memory: {} MB", gpuDesc.DedicatedVideoMemory / (1000 * 1000)));
                    ui.text(im_str!("  System Memory: {} MB", gpuDesc.DedicatedSystemMemory / (1000 * 1000)));
                    ui.text(im_str!("  Shared Memory: {} MB", gpuDesc.SharedSystemMemory / (1000 * 1000)));

                    rendererInfo.end();
                }
                
                // Pannels
                Console::OnImGuiRender(&ui);
                self.m_SceneHierarchyPanel.OnImGuiRender(&ui);

                let selectedEntity: Entity = self.m_SceneHierarchyPanel.GetSelectedEntity();
                if  !selectedEntity.IsValid() ||
                    !self.m_MaterialInspectorPanel.GetSelectedEntity().IsValid() || 
                    selectedEntity.GetID() != self.m_MaterialInspectorPanel.GetSelectedEntity().GetID()
                {
                    self.m_MaterialInspectorPanel.SetSelectedEntity(selectedEntity);
                }

                self.m_MaterialInspectorPanel.OnImGuiRender(&ui, &mut self.m_ImGuiRenderer);
                self.m_ContentBrowserPanel.OnImGuiRender(&ui);

                if let Some(exposure) = imgui::Window::new(im_str!("Camera Settings")).begin(&ui)
                {
                    let slider: imgui::Slider<f32> = imgui::Slider::new(im_str!("Exposure"));
                    let range: std::ops::RangeInclusive<f32> = std::ops::RangeInclusive::new(0.1, 8.0);
                    slider.range(range).build(&ui, self.m_SceneRenderer.GetExposure());

                    exposure.end();
                }

                // Scene Viewport
                let viewportFlags = imgui::WindowFlags::NO_SCROLLBAR | imgui::WindowFlags::NO_MOVE | imgui::WindowFlags::NO_TITLE_BAR;
                let style: imgui::StyleStackToken = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));

                if let Some(viewport) = imgui::Window::new(im_str!("Viewport")).flags(viewportFlags).begin(&ui)
                {
                    style.pop();

                    self.m_ViewportHovered = ui.is_window_hovered();
                    self.m_ViewportFocused = ui.is_window_focused();

                    // Get the rendered scene image from the final render pass target
                    let finalPassImage: Option<ID3D11ShaderResourceView> = self.m_SceneRenderer.GetCompositePassTexture().GetRef().CreateSRV();
                    let finalPassImageID = imgui::TextureId::from(self.m_SceneRenderer.GetCompositePassTexture().GetRaw());
                    self.m_ImGuiRenderer.GetTextures().replace(finalPassImageID, finalPassImage);

                    // Resize the framebuffer and the camera viewport if needed
                    let viewportSize: [f32; 2] = ui.window_size();
                    if self.m_ViewportSize.x != viewportSize[0] || self.m_ViewportSize.y != viewportSize[1]
                    {
                        self.m_ViewportSize = XMFLOAT2::set(viewportSize[0], viewportSize[1]);
                        self.m_SceneRenderer.GetCompositePassTarget().GetRefMut().Resize(viewportSize[0] as u32, viewportSize[1] as u32);
                        self.m_Scene.GetRefMut().OnViewportResize(viewportSize[0], viewportSize[1]);
                    }

                    imgui::Image::new(finalPassImageID, [viewportSize[0], viewportSize[1]]).build(&ui);

                    // If drag drop payload is accepted load the mesh with the sent path 
                    if let Some(target) = imgui::DragDropTarget::new(&ui) 
                    {
                        if let Some(Ok(payloadData)) = target
                                .accept_payload::<*const str>(im_str!("DragMeshPath"), imgui::DragDropFlags::empty())
                        {
                            // We know it is safe to dereference the pointer since it points to a string owned by the content browser panel
                            // and it lives through the whole application
                            let meshPath = unsafe { &*payloadData.data };
                            let mesh = AssetManager::LoadMesh(meshPath);
                            let mut meshEntity: Entity = self.m_Scene.GetRefMut().CreateEntity(mesh.GetRef().GetName());

                            let mc: MeshComponent = MeshComponent { 
                                MeshPath: String::from(meshPath), 
                            };

                            meshEntity.AddComponent(mc);
                        }
                        else if let Some(Ok(payloadData)) = target
                                    .accept_payload::<*const str>(im_str!("DragScenePath"), imgui::DragDropFlags::empty())
                        {
                            let scenePath = unsafe { &*payloadData.data };
                            self.m_Scene = Scene::Create();
                            self.m_Scene.GetRefMut().Deserialize(scenePath);
                            self.m_SceneHierarchyPanel.SetContext(self.m_Scene.clone());
                            Console::LogInfo("Scene opened successfully!");
                        }

                        target.pop();
                    }

                    // Draw gizmos
                    let mut selectedEntity: Entity = self.m_SceneHierarchyPanel.GetSelectedEntity();
                    if selectedEntity.IsValid()
                    {
                        let mut viewMatrix = XMFLOAT4X4::default();
                        XMStoreFloat4x4(&mut viewMatrix, *self.m_Scene.GetRefMut().GetEditorCamera().GetViewMatrix());

                        let mut projMatrix = XMFLOAT4X4::default();
                        XMStoreFloat4x4(&mut projMatrix, *self.m_Scene.GetRefMut().GetEditorCamera().GetProjectionMatrix());

                        let mut tc = selectedEntity.GetComponentMut::<TransformComponent>();
                        let mut transform = XMFLOAT4X4::default();
                        XMStoreFloat4x4(&mut transform, tc.Transform());

                        let gizmo = imguizmo::Gizmo::begin_frame(&ui);
                        gizmo.set_draw_list();
                        gizmo.set_rect(ui.window_pos()[0], ui.window_pos()[1], self.m_ViewportSize.x, self.m_ViewportSize.y);
                        gizmo.set_orthographic(false);
                        gizmo.manipulate(&viewMatrix.m, &projMatrix.m, self.m_GizmoOperation, imguizmo::Mode::Local, &mut transform.m,
                                            None, None, None, None );

                        if gizmo.is_using()
                        {
                            let mut position = [0.0, 0.0, 0.0];
                            let mut rotation = [0.0, 0.0, 0.0];
                            let mut scale = [0.0, 0.0, 0.0];
                            imguizmo::decompose_matrix_to_components(&transform.m, &mut position, &mut rotation, &mut scale);

                            let deltaRotation = [rotation[0] - tc.Rotation[0], rotation[1] - tc.Rotation[1], rotation[2] - tc.Rotation[2]];

                            tc.Position = position;
                            tc.Rotation = [tc.Rotation[0] + deltaRotation[0], tc.Rotation[1] + deltaRotation[1], tc.Rotation[2] + deltaRotation[2]];
                            tc.Scale    = scale;
                        }
                    }

                    viewport.end();
                };

                dockSpace.end();
            };
        }
        
        // Render UI
        self.m_ImGuiRenderer.RenderDrawData(ui.render());
        self.m_ImGuiRenderer.Flush();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnEvent(&mut self, event: &dyn Event)
    {
        if event.GetType() == EventType::WindowClosed
        {
            let windowClosedEvent: &WindowClosedEvent = event.AsAny().downcast_ref::<WindowClosedEvent>().expect("Event is now a WindowClosedEvent");
            self.OnWindowClosed(windowClosedEvent);
        }

        self.m_ImGuiPlatform.OnEvent(event);

        if self.m_ViewportFocused && self.m_ViewportHovered
        {
            if event.GetType() == EventType::KeyPressed
            {
                let keyPressedEvent: &KeyPressedEvent = event.AsAny().downcast_ref::<KeyPressedEvent>().expect("Event is now a KeyPressedEvent");
                self.OnKeyPressed(keyPressedEvent);
            }
            
            self.m_Scene.GetRefMut().GetEditorCamera().OnEvent(event);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnWindowClosed(&mut self, event: &WindowClosedEvent)
    {
        self.m_IsRunning = false;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnKeyPressed(&mut self, event: &KeyPressedEvent)
    {
        if event.GetKeyCode() == VK_T
        {
            self.m_GizmoOperation = imguizmo::Operation::Translate;
            Console::LogInfo("Gizmo operation: Translate");
        }
        else if event.GetKeyCode() == VK_R
        {
            self.m_GizmoOperation = imguizmo::Operation::Rotate;
            Console::LogInfo("Gizmo operation: Rotate");
        }
        else if event.GetKeyCode() == VK_E
        {
            self.m_GizmoOperation = imguizmo::Operation::Scale;
            Console::LogInfo("Gizmo operation: Scale");
        }
        else if event.GetKeyCode() == VK_Q
        {
            self.m_GizmoOperation = imguizmo::Operation::Bounds;
            Console::LogInfo("Gizmo hidden");
        }

        if Input::IsKeyPressed(VK_LCONTROL) && event.GetKeyCode() == VK_N
        {
            self.m_Scene = Scene::Create();
            self.m_SceneHierarchyPanel.SetContext(self.m_Scene.clone());
        }
        else if Input::IsKeyPressed(VK_LCONTROL) && event.GetKeyCode() == VK_O
        {
            let scenePath = OpenFile(String::from("Rusty Scene (*.rscene)\0*.rscene\0"));
            self.m_Scene = Scene::Create();
            self.m_Scene.GetRefMut().Deserialize(&scenePath);
            self.m_SceneHierarchyPanel.SetContext(self.m_Scene.clone());
            Console::LogInfo("Scene opened successfully!");
        }
        else if Input::IsKeyPressed(VK_LCONTROL) && event.GetKeyCode() == VK_S
        {
            let scenePath = SaveFile(String::from("Rusty Scene (*.rscene)\0*.rscene\0"));
            self.m_Scene.GetRef().Serialize(&scenePath);
            Console::LogInfo("Scene saved successfully!");
        }
    }
}

fn main() 
{
    let mut app = RustyEditorApp::Create(1600, 900);
    app.Initialize();
    app.Run();
}
