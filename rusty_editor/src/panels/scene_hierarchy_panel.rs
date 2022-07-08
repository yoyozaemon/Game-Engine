#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// std
use __core::ops::RangeInclusive;

// Win32
use directx_math::XMFLOAT3;

// imgui
use imgui::*;

// legion
use legion::IntoQuery;

// Rusty Engine
use rusty_engine::core::utils::*;
use rusty_engine::scene::component::*;
use rusty_engine::scene::scene::*;
use rusty_engine::scene::entity::*;
use rusty_engine::core::asset_manager::*;

pub struct SceneHierarchyPanel
{
    m_Scene:                RustyRef<Scene>,
    m_SelectedEntity:       Entity,
}

impl SceneHierarchyPanel
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create() -> SceneHierarchyPanel
    {
        return SceneHierarchyPanel {
            m_Scene: RustyRef::CreateEmpty(),
            m_SelectedEntity: Entity::CreateEmpty(),
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnImGuiRender(&mut self, ui: &Ui)
    {
        // Scene hierarchy window
        if let Some(sceneHierarchy) = Window::new(im_str!("Scene Hierarchy"))
                                                .size([500.0, 400.0], imgui::Condition::FirstUseEver)
                                                .begin(&ui)
        {
            // Deselect entity when left clicking on blank space in the window
            if ui.is_mouse_clicked(MouseButton::Left) && ui.is_window_hovered()
            {
                self.m_SelectedEntity = Entity::CreateEmpty();
            }

            // Open context menu when right clicking on blank space in the window
            if ui.is_mouse_clicked(MouseButton::Right) && ui.is_window_hovered()
            {
                ui.open_popup(im_str!("##EntityPopup"));
            }

            if let Some(popup) = ui.begin_popup(im_str!("##EntityPopup"))
            {
                if imgui::MenuItem::new(im_str!("New Entity")).build(&ui)
                {
                    self.m_Scene.GetRefMut().CreateEntity("Unnamed");
                }

                popup.end()
            }
            
            // Display all entity nodes
            unsafe
            {
                let mut query = legion::Entity::query();
                let scene: *mut Scene = self.m_Scene.GetRawMut();

                for entity in query.iter(scene.as_ref().unwrap().GetWorld())
                {
                    let entity: Entity = Entity::Create(*entity, scene);
                    let tag: &String = &entity.GetComponent::<TagComponent>().Tag;

                    let mut flags = TreeNodeFlags::OPEN_ON_ARROW | TreeNodeFlags::SPAN_AVAIL_WIDTH | TreeNodeFlags::FRAME_PADDING;
                    if self.m_SelectedEntity.IsValid() && self.m_SelectedEntity.GetID() == entity.GetID() 
                    { 
                        flags |= TreeNodeFlags::SELECTED;
                    };

                    let entityIDStr = im_str!("{:?}", entity.GetID());
                    let nodeID: &ImStr = entityIDStr.as_ref();
                    TreeNode::new(TreeNodeId::from(nodeID))
                                    .flags(flags)
                                    .label(&im_str!("{}", tag)).build(ui, || {});

                    let idToken = ui.push_id(nodeID);

                    // Select when left clicking on the entity
                    if ui.is_item_clicked()
                    {
                        self.m_SelectedEntity = entity;
                    }

                    // Open entity pop-up and also select entity on right click
                    if ui.is_item_clicked_with_button(MouseButton::Right)
                    {
                        self.m_SelectedEntity = entity;
                        ui.open_popup(im_str!("##Entity"));
                    }
                
                    if let Some(popup) = ui.begin_popup(im_str!("##Entity"))
                    {
                        if imgui::MenuItem::new(im_str!("Delete")).build(&ui)
                        {
                            if self.m_SelectedEntity.GetID() == entity.GetID()
                            {
                                self.m_SelectedEntity = Entity::CreateEmpty();
                            }
                            self.m_Scene.GetRefMut().DeleteEntity(entity);
                        }
                    
                        popup.end()
                    }

                    idToken.pop();
                }
            }

            sceneHierarchy.end();
        }

        // Entity inspector window
        if let Some(properties) = Window::new(im_str!("Properties"))
                                                .size([500.0, 400.0], imgui::Condition::FirstUseEver)
                                                .begin(&ui)
        {
            if self.m_SelectedEntity.IsValid()
            {
                // Tag component
                SceneHierarchyPanel::DrawComponent::<TagComponent>(&ui, "Tag", false, self.m_SelectedEntity, &|component|
                {
                    ui.columns(2, im_str!("Tag"), true);
                    ui.set_column_width(0, 100.0);
                    ui.text("Tag");
                    ui.next_column();

                    // Create big enough input buffer for the tag
                    let mut buffer = ImString::with_capacity(40);
                    let mut tagChars = component.Tag.chars();
                    while let Some(character) = tagChars.next()
                    {
                        buffer.push(character);
                    }

                    if ui.input_text(im_str!("##Tag"), &mut buffer).build()
                    {
                        component.Tag = buffer.to_string();
                    }

                    ui.columns(1, im_str!("Tag"), true);
                });

                // Transform component
                SceneHierarchyPanel::DrawComponent::<TransformComponent>(&ui, "Transform", false, self.m_SelectedEntity, &|component|
                {
                    SceneHierarchyPanel::DrawVec3Control(ui, "Position", &mut component.Position, 0.0, 100.0);
                    SceneHierarchyPanel::DrawVec3Control(ui, "Rotation", &mut component.Rotation, 0.0, 100.0);
                    SceneHierarchyPanel::DrawVec3Control(ui, "Scale", &mut component.Scale, 1.0, 100.0);
                });

                // Mesh component
                if self.m_SelectedEntity.HasComponent::<MeshComponent>()
                {
                    SceneHierarchyPanel::DrawComponent::<MeshComponent>(&ui, "Mesh", true, self.m_SelectedEntity, &|component|
                    {
                        ui.columns(2, im_str!("MeshName"), true);
                        ui.set_column_width(0, 100.0);
                        ui.text("Mesh");
                        ui.next_column();

                        let mut meshName = ImString::from(component.MeshPath.clone());
                        ui.input_text(im_str!("##Mesh"), &mut meshName).read_only(true).build();

                        // If drag drop payload is accepted load the mesh with the sent path 
                        if let Some(target) = imgui::DragDropTarget::new(&ui) 
                        {
                            if let Some(Ok(payloadData)) = target
                                .accept_payload::<*const str>(im_str!("DragMeshPath"), imgui::DragDropFlags::empty())
                            {
                                // We know it is safe to dereference the pointer since it points to a string owned by the content browser panel
                                // and it lives through the whole application
                                let meshPath = unsafe { &*payloadData.data };
                                AssetManager::LoadMesh(meshPath);
                                component.MeshPath = String::from(meshPath);
                            }

                            target.pop();
                        }

                        ui.columns(1, im_str!("MeshName"), true);

                        let mesh = AssetManager::GetMesh(&component.MeshPath);

                        if mesh.IsValid()
                        {
                            let meshRef = mesh.GetRef();
                            let materials = meshRef.GetMaterials();

                            let flags = TreeNodeFlags::SPAN_AVAIL_WIDTH | TreeNodeFlags::FRAME_PADDING | TreeNodeFlags::DEFAULT_OPEN;
                            let nodeID: &ImStr = meshName.as_ref();
                            TreeNode::new(TreeNodeId::from(nodeID))
                                        .flags(flags)
                                        .label(im_str!("Submeshes")).build(ui, || 
                            {

                                for submesh in meshRef.GetSubmeshes()
                                {
                                    let mut materialName = ImString::from(materials[submesh.MaterialIndex as usize].GetRef().GetName().clone());

                                    let id = im_str!("Submeshes##{}{}", &submesh.Name, &materialName);
                                    let idToken = ui.push_id(&id);

                                    ui.columns(2, &id, true);
                                    ui.set_column_width(0, 100.0);
                                    ui.text(&submesh.Name);
                                    ui.next_column();

                                    ui.input_text(im_str!(""), &mut materialName).read_only(true).build();

                                    // TODO: Add material overriding when we have material files

                                    ui.columns(1, &id, true);

                                    idToken.pop();
                                }
                            });
                        }
                    });
                }

                // Sky Light component
                if self.m_SelectedEntity.HasComponent::<SkyLightComponent>()
                {
                    SceneHierarchyPanel::DrawComponent::<SkyLightComponent>(&ui, "Sky Light", true, self.m_SelectedEntity, &|component|
                    {
                        ui.columns(2, im_str!("SkyLight"), true);
                        ui.set_column_width(0, 150.0);
                        ui.text("Environment Map");
                        ui.next_column();

                        let mut environmentMapName = ImString::from(component.EnvironmentMapPath.clone());
                        ui.input_text(im_str!("##EnvironmentMap"), &mut environmentMapName).read_only(true).build();

                        // If drag drop payload is accepted load the environment map with the sent path 
                        if let Some(target) = imgui::DragDropTarget::new(&ui) 
                        {
                            if let Some(Ok(payloadData)) = target
                                .accept_payload::<*const str>(im_str!("DragEnvironmentMapPath"), imgui::DragDropFlags::empty())
                            {
                                // We know it is safe to dereference the pointer since it points to a string owned by the content browser panel
                                // and it lives through the whole application
                                let envMapPath: &str = unsafe { &*payloadData.data };
                                AssetManager::LoadEnvironmentMap(envMapPath);
                                component.EnvironmentMapPath = String::from(envMapPath);
                            }

                            target.pop();
                        }

                        ui.columns(1, im_str!("EnvironmentMap"), true);

                        ui.columns(2, im_str!("EnvironmentMap"), true);
                        ui.set_column_width(0, 150.0);
                        ui.text("Intensity");
                        ui.next_column();

                        let mut value: f32 = component.Intensity;
                        if imgui::Drag::new(im_str!("##SkyLightIntensity")).range(RangeInclusive::new(0.0, 15.0)).speed(0.01).build(ui, &mut value)
                        {
                            component.Intensity = value;
                        }

                        ui.columns(1, im_str!("SkyLight"), true);
                    });
                }

                // Directional Light component
                if self.m_SelectedEntity.HasComponent::<DirectionalLightComponent>()
                {
                    SceneHierarchyPanel::DrawComponent::<DirectionalLightComponent>(&ui, "Directional Light", true, self.m_SelectedEntity, &|component|
                    {
                        ui.columns(2, im_str!("DirectionalLight"), true);
                        ui.set_column_width(0, 100.0);
                        ui.text("Color");
                        ui.next_column();
                        imgui::ColorEdit::new(im_str!("##DirLightColor"), &mut component.Color).build(&ui);
                        ui.columns(1, im_str!("EnvironmentMap"), true);

                        ui.columns(2, im_str!("DirectionalLight"), true);
                        ui.set_column_width(0, 100.0);
                        ui.text("Intensity");
                        ui.next_column();
                        let mut value: f32 = component.Intensity;
                        if imgui::Drag::new(im_str!("##DirLightIntensity")).range(RangeInclusive::new(0.0, 15.0)).speed(0.01).build(ui, &mut value)
                        {
                            component.Intensity = value;
                        }
                        ui.columns(1, im_str!("DirectionalLight"), true);
                    });
                }

                // Point Light component
                if self.m_SelectedEntity.HasComponent::<PointLightComponent>()
                {
                    SceneHierarchyPanel::DrawComponent::<PointLightComponent>(&ui, "Point Light", true, self.m_SelectedEntity, &|component|
                    {
                        ui.columns(2, im_str!("PointLight"), true);
                        ui.set_column_width(0, 100.0);
                        ui.text("Color");
                        ui.next_column();
                        imgui::ColorEdit::new(im_str!("##PointLightColor"), &mut component.Color).build(&ui);
                        ui.columns(1, im_str!("EnvironmentMap"), true);

                        ui.columns(2, im_str!("PointLight"), true);
                        ui.set_column_width(0, 100.0);
                        ui.text("Intensity");
                        ui.next_column();
                        let mut value: f32 = component.Intensity;
                        if imgui::Drag::new(im_str!("##PointLightIntensity")).range(RangeInclusive::new(0.0, 15.0)).speed(0.01).build(ui, &mut value)
                        {
                            component.Intensity = value;
                        }
                        ui.columns(1, im_str!("PointLight"), true);
                    });
                }

                // Add Components
                if ui.button(im_str!("Add Component"))
                {
                    ui.open_popup(im_str!("AddComponentPopup"))
                }

                if let Some(popup) = ui.begin_popup(im_str!("AddComponentPopup"))
                {
                    self.AddComponentMenuItem::<MeshComponent>(ui, "Mesh");
                    self.AddComponentMenuItem::<SkyLightComponent>(ui, "Sky Light");
                    self.AddComponentMenuItem::<DirectionalLightComponent>(ui, "Directional Light");
                    self.AddComponentMenuItem::<PointLightComponent>(ui, "Point Light");

                    popup.end();
                }
            }
            
            properties.end();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetContext(&mut self, scene: RustyRef<Scene>)
    {
        self.m_Scene = scene;
        self.m_SelectedEntity = Entity::CreateEmpty();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetSelectedEntity(&mut self, entity: Entity)
    {
        self.m_SelectedEntity = entity;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSelectedEntity(&self) -> Entity
    {
        return self.m_SelectedEntity;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn DrawComponent<ComponentType: legion::storage::Component>(ui: &Ui, componentName: &str, removeable: bool, mut entity: Entity, buildFunction: &dyn Fn(&mut ComponentType))
    {
        let component: &mut ComponentType = entity.GetComponentMut::<ComponentType>();

        let flags = TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::FRAMED | TreeNodeFlags::SPAN_AVAIL_WIDTH |
                                 TreeNodeFlags::FRAME_PADDING | TreeNodeFlags::ALLOW_ITEM_OVERLAP;
        
        let contentRegion = ui.content_region_avail();
        let lineHeight = ui.current_font_size() + 5.0 * 2.0;

        let isOpened: bool = CollapsingHeader::new(&im_str!("{}", componentName)).flags(flags).build(ui);

        ui.same_line_with_pos(contentRegion[0] - lineHeight * 0.5);

        if ui.button_with_size(&im_str!("+##{}", componentName), [lineHeight, lineHeight])
        {
            ui.open_popup(&im_str!("Component Options##{}", componentName));
        }

        let mut removeComponent: bool = false;
        if removeable
        {
            if let Some(popup) = ui.begin_popup(&im_str!("Component Options##{}", componentName))
            {
                if imgui::MenuItem::new(im_str!("Remove")).build(&ui)
                {
                    removeComponent = true;
                }
            
                popup.end()
            }
        }

        // Render the node
        if isOpened
        {
            buildFunction(component);
        }

        // Remove after we draw everything so that we draw everything before the data of the component is released
        if removeComponent
        {
            entity.RemoveComponent::<ComponentType>();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn DrawVec3Control(ui: &Ui, label: &str, values: &mut [f32; 3], resetValue: f32, columnWidth: f32)
    {
        let labelStr: ImString = im_str!("{}", label);
        let id: &ImStr = labelStr.as_ref();

        let idToken = ui.push_id(id);

        ui.columns(2, id, true);
        ui.set_column_width(0, columnWidth);
        ui.text(label);
        ui.next_column();

        let sliderWidth: f32 = ui.column_width(1) * 0.2;
        let spacingToken = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));

        let lineHeight: f32 = ui.current_font_size() + 5.0 * 2.0;
        let buttonSize: [f32; 2] = [lineHeight + 3.0, lineHeight];
        
        // X Component
        let itemWidthToken = ui.push_item_width(sliderWidth);
        let buttonColorToken = ui.push_style_color(StyleColor::Button, [0.8, 0.1, 0.15, 1.0]);
        let buttonHoveredColorToken = ui.push_style_color(StyleColor::ButtonHovered, [0.9, 0.2, 0.2, 1.0]);
        let buttonActiveColorToken = ui.push_style_color(StyleColor::ButtonActive, [0.8, 0.1, 0.15, 1.0]);

        if ui.button_with_size(im_str!("X"), buttonSize)
        {
            values[0] = resetValue;
        }

        buttonActiveColorToken.pop();
        buttonHoveredColorToken.pop();
        buttonColorToken.pop();

        ui.same_line();
        imgui::Drag::new(im_str!("##X")).speed(0.1).build(ui, &mut values[0]);
        itemWidthToken.pop(ui);

        // Y Component
        ui.same_line();
        let itemWidthToken = ui.push_item_width(sliderWidth);
        let buttonColorToken = ui.push_style_color(StyleColor::Button, [0.2, 0.7, 0.2, 1.0]);
        let buttonHoveredColorToken = ui.push_style_color(StyleColor::ButtonHovered, [0.3, 0.8, 0.3, 1.0]);
        let buttonActiveColorToken = ui.push_style_color(StyleColor::ButtonActive, [0.2, 0.7, 0.2, 1.0]);

        if ui.button_with_size(im_str!("Y"), buttonSize)
        {
            values[1] = resetValue;
        }

        buttonActiveColorToken.pop();
        buttonHoveredColorToken.pop();
        buttonColorToken.pop();

        ui.same_line();
        imgui::Drag::new(im_str!("##Y")).speed(0.1).build(ui, &mut values[1]);
        itemWidthToken.pop(ui);

        // Z Component
        ui.same_line();
        let itemWidthToken = ui.push_item_width(sliderWidth);
        let buttonColorToken = ui.push_style_color(StyleColor::Button, [0.1, 0.25, 0.8, 1.0]);
        let buttonHoveredColorToken = ui.push_style_color(StyleColor::ButtonHovered, [0.2, 0.35, 0.9, 1.0]);
        let buttonActiveColorToken = ui.push_style_color(StyleColor::ButtonActive, [0.1, 0.25, 0.8, 1.0]);

        if ui.button_with_size(im_str!("Z"), buttonSize)
        {
            values[2] = resetValue;
        }

        buttonActiveColorToken.pop();
        buttonHoveredColorToken.pop();
        buttonColorToken.pop();

        ui.same_line();
        imgui::Drag::new(im_str!("##Z")).speed(0.1).build(ui, &mut values[2]);
        itemWidthToken.pop(ui);

        spacingToken.pop();

        ui.columns(1, id, true);
        idToken.pop();
    }

    fn AddComponentMenuItem<ComponentType: legion::storage::Component + Default>(&mut self, ui: &Ui, displayName: &str)
    {
        if !self.m_SelectedEntity.HasComponent::<ComponentType>()
        {
            let id = ui.push_id(format!("Component##{}", displayName).as_str());

            if imgui::MenuItem::new(im_str!("{}", displayName).as_ref()).build(ui)
            {
                let component = ComponentType::default();
                self.m_SelectedEntity.AddComponent(component);
            }

            id.pop();
        }
    }
}
