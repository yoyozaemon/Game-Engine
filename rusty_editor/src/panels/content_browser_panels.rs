#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// imgui
use imgui::*;

// std
use std::path::PathBuf;

// Rusty Engine
use rusty_engine::core::utils::*;
use rusty_engine::renderer::texture::*;

pub struct ContentBrowserPanel
{
    m_AssetsDirectory: PathBuf,
    m_CurrentDirectory: PathBuf,
    m_CurrentFilepath: String,
    m_FolderIcon: RustyRef<Texture>,
    m_PNGIcon:    RustyRef<Texture>,
    m_JPGIcon:    RustyRef<Texture>,
    m_TGAIcon:    RustyRef<Texture>,
    m_HDRIcon:    RustyRef<Texture>,
    m_FBXIcon:    RustyRef<Texture>,
    m_RSceneIcon: RustyRef<Texture>,
}

impl ContentBrowserPanel
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create() -> ContentBrowserPanel
    {
        return ContentBrowserPanel {
            m_AssetsDirectory:  PathBuf::new(),
            m_CurrentDirectory: PathBuf::new(),
            m_CurrentFilepath:  String::new(),
            m_FolderIcon:       RustyRef::CreateEmpty(),
            m_PNGIcon:          RustyRef::CreateEmpty(),
            m_JPGIcon:          RustyRef::CreateEmpty(),
            m_TGAIcon:          RustyRef::CreateEmpty(),
            m_HDRIcon:          RustyRef::CreateEmpty(),
            m_FBXIcon:          RustyRef::CreateEmpty(),
            m_RSceneIcon:       RustyRef::CreateEmpty(),
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnImGuiRender(&mut self, ui: &Ui)
    {
        if let Some(contentBrowser) = Window::new(im_str!("Content Browser"))
                                                .size([500.0, 400.0], imgui::Condition::FirstUseEver)
                                                .begin(&ui)
        {

            if self.m_CurrentDirectory != self.m_AssetsDirectory
            {
                if ui.button(im_str!("<-"))
                {
                    self.m_CurrentDirectory = self.m_CurrentDirectory.as_path().parent().unwrap().to_path_buf();
                }
            }

            let windowVisibleX2: f32 = ui.window_pos()[0] + ui.content_region_max()[0];
            let buttonSize: [f32; 2] = [124.0, 124.0];

            for filepath in std::fs::read_dir(&self.m_CurrentDirectory).unwrap()
            {
                if filepath.is_ok()
                {
                    let path = filepath.unwrap().path();
                    let fileName = path.file_name().unwrap().to_str().unwrap();
                    
                    let groupToken = ui.begin_group();

                    if path.is_dir()
                    {
                        let idToken = ui.push_id(fileName);
                        let colorToken = ui.push_style_color(StyleColor::Button, [0.0, 0.0, 0.0, 0.0]);

                        imgui::ImageButton::new(TextureId::from(self.m_FolderIcon.GetRaw()), buttonSize).build(ui);

                        if ui.is_item_hovered() && ui.is_mouse_double_clicked(MouseButton::Left)
                        {
                            self.m_CurrentDirectory.push(path.file_name().unwrap());
                        }

                        ui.text_wrapped(im_str!("{}", fileName).as_ref());

                        colorToken.pop();
                        idToken.pop();
                    }
                    else
                    {
                        let idToken = ui.push_id(fileName);
                        let colorToken = ui.push_style_color(StyleColor::Button, [0.0, 0.0, 0.0, 0.0]);
                        if fileName.contains(".png")
                        {
                            imgui::ImageButton::new(TextureId::from(self.m_PNGIcon.GetRaw()), buttonSize).build(ui);

                            if let Some(dragDropSource) = DragDropSource::new(im_str!("DragTexturePath")).begin_source()
                            {
                                self.m_CurrentFilepath = String::from(path.to_str().unwrap());
                                let tooltip = dragDropSource.begin_payload(ui, self.m_CurrentFilepath.as_str() as *const str);
                                ui.text(format!("{}", fileName));
                                tooltip.end();
                            }

                            ui.text_wrapped(im_str!("{}", fileName).as_ref());
                        }
                        else if fileName.contains(".jpg") || fileName.contains(".jpeg")
                        {
                            imgui::ImageButton::new(TextureId::from(self.m_JPGIcon.GetRaw()), buttonSize).build(ui);

                            if let Some(dragDropSource) = DragDropSource::new(im_str!("DragTexturePath")).begin_source()
                            {
                                self.m_CurrentFilepath = String::from(path.to_str().unwrap());
                                let tooltip = dragDropSource.begin_payload(ui, self.m_CurrentFilepath.as_str() as *const str);
                                ui.text(format!("{}", fileName));
                                tooltip.end();
                            }

                            ui.text_wrapped(im_str!("{}", fileName).as_ref());
                        }
                        else if fileName.contains(".tga")
                        {
                            imgui::ImageButton::new(TextureId::from(self.m_TGAIcon.GetRaw()), buttonSize).build(ui);

                            if let Some(dragDropSource) = DragDropSource::new(im_str!("DragTexturePath")).begin_source()
                            {
                                self.m_CurrentFilepath = String::from(path.to_str().unwrap());
                                let tooltip = dragDropSource.begin_payload(ui, self.m_CurrentFilepath.as_str() as *const str);
                                ui.text(format!("{}", fileName));
                                tooltip.end();
                            }

                            ui.text_wrapped(im_str!("{}", fileName).as_ref());
                        }
                        else if fileName.contains(".hdr")
                        {
                            imgui::ImageButton::new(TextureId::from(self.m_HDRIcon.GetRaw()), buttonSize).build(ui);

                            if let Some(dragDropSource) = DragDropSource::new(im_str!("DragEnvironmentMapPath")).begin_source()
                            {
                                self.m_CurrentFilepath = String::from(path.to_str().unwrap());
                                let tooltip = dragDropSource.begin_payload(ui, self.m_CurrentFilepath.as_str() as *const str);
                                ui.text(format!("{}", fileName));
                                tooltip.end();
                            }

                            ui.text_wrapped(im_str!("{}", fileName).as_ref());
                        }
                        else if fileName.contains(".fbx") || fileName.contains(".FBX")
                        {
                            imgui::ImageButton::new(TextureId::from(self.m_FBXIcon.GetRaw()), buttonSize).build(ui);

                            if let Some(dragDropSource) = DragDropSource::new(im_str!("DragMeshPath")).begin_source()
                            {
                                self.m_CurrentFilepath = String::from(path.to_str().unwrap());
                                let tooltip = dragDropSource.begin_payload(ui, self.m_CurrentFilepath.as_str() as *const str);
                                ui.text(format!("{}", fileName));
                                tooltip.end();
                            }

                            ui.text_wrapped(im_str!("{}", fileName).as_ref());
                        }
                        else if fileName.contains(".rscene")
                        {
                            imgui::ImageButton::new(TextureId::from(self.m_RSceneIcon.GetRaw()), buttonSize).build(ui);

                            if let Some(dragDropSource) = DragDropSource::new(im_str!("DragScenePath")).begin_source()
                            {
                                self.m_CurrentFilepath = String::from(path.to_str().unwrap());
                                let tooltip = dragDropSource.begin_payload(ui, self.m_CurrentFilepath.as_str() as *const str);
                                ui.text(format!("{}", fileName));
                                tooltip.end();
                            }

                            ui.text_wrapped(im_str!("{}", fileName).as_ref());
                        }
                        colorToken.pop();
                        idToken.pop();
                    }

                    groupToken.end();

                    let lastButtonX2: f32 = ui.item_rect_max()[0];
                    let nextButtonX2: f32 = lastButtonX2 + buttonSize[0] + 8.0;
                    if nextButtonX2 < windowVisibleX2
                    {
                        ui.same_line_with_spacing(0.0, 20.0);
                    }
                }
            }

            contentBrowser.end();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetAssetsDirectory(&mut self, directory: &str)
    {
        self.m_AssetsDirectory = PathBuf::from(directory);
        self.m_CurrentDirectory = PathBuf::from(directory);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetFolderIcon(&mut self, icon: RustyRef<Texture>)
    {
        self.m_FolderIcon = icon;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetPNGIcon(&mut self, icon: RustyRef<Texture>)
    {
        self.m_PNGIcon = icon;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetJPGIcon(&mut self, icon: RustyRef<Texture>)
    {
        self.m_JPGIcon = icon;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetTGAIcon(&mut self, icon: RustyRef<Texture>)
    {
        self.m_TGAIcon = icon;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetHDRIcon(&mut self, icon: RustyRef<Texture>)
    {
        self.m_HDRIcon = icon;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetFBXIcon(&mut self, icon: RustyRef<Texture>)
    {
        self.m_FBXIcon = icon;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetRSceneIcon(&mut self, icon: RustyRef<Texture>)
    {
        self.m_RSceneIcon = icon;
    }
}
