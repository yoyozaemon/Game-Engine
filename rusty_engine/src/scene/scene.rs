#![allow(non_snake_case)]

use std::io::Read;
use std::io::Write;

// Win32
use directx_math::*;
use serde::de::DeserializeSeed;

// Renderer
use crate::renderer::editor_camera::*;
use crate::renderer::scene_renderer::*;
use crate::renderer::mesh::*;
use crate::renderer::environment::*;

// Core
use crate::core::utils::*;
use crate::core::timestep::*;
use crate::core::asset_manager::*;

// Scene
use crate::scene::component::*;

// legion
use legion::*;
use legion::serialize::Canon;

#[derive(Copy, Clone, Debug)]
pub enum SceneState
{
    Edit, Running
}

pub struct Scene
{
    m_World:        legion::World,
    m_State:        SceneState,
    m_ViewportSize: XMFLOAT2,
    m_EditorCamera: EditorCamera
}

impl Scene
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create() -> RustyRef<Scene>
    {
        return RustyRef::CreateRef(Scene{
            m_World: legion::World::default(),
            m_State: SceneState::Edit,
            m_ViewportSize: XMFLOAT2::default(),
            m_EditorCamera: EditorCamera::Create(45.0, 16.0 / 9.0, 0.1, 10000.0)
        });
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateEntity(&mut self, name: &str) -> super::entity::Entity
    {
        // Every entity must have a transform and tag components
        let transformComponent = TransformComponent::default();
        let tagComponent = TagComponent { Tag: String::from(name) };
        let entityID = self.m_World.push((tagComponent, transformComponent));

        return super::entity::Entity::Create(entityID, self);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn DeleteEntity(&mut self, entity: super::entity::Entity)
    {
        let result: bool = self.m_World.remove(entity.GetID());
        debug_assert!(result, "Failed to delete entity!");
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn FindEntity(&mut self, name: &str) -> super::entity::Entity
    {
        let mut query = legion::Entity::query();
        
        for entity in query.iter(&self.m_World)
        {
            if let Ok(entry) = self.m_World.entry_ref(*entity)
            {
                if entry.get_component::<TagComponent>().unwrap().Tag == String::from(name)
                {
                    return super::entity::Entity::Create(*entity, self);
                }
            }
        }

        return super::entity::Entity::CreateEmpty();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnSceneStart(&mut self)
    {
        
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnUpdate(&mut self, ts: Timestep)
    {
        self.m_EditorCamera.OnUpdate(ts);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnSceneEnd(&mut self)
    {
        
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnEditRender(&self, renderer: &mut SceneRenderer)
    {
        // Setup the environment
        let mut environment = Environment::Create();

        // Sky light
        let mut query = <&SkyLightComponent>::query();
        for slc in query.iter(&self.m_World)
        {
            let envMap = AssetManager::GetEnvironmentMap(&slc.EnvironmentMapPath);
            if envMap.0.IsValid() && envMap.1.IsValid()
            {
                environment.SetEnvironmentMap(envMap.0.clone(), envMap.1.clone());
            }
            break;
        }

        // Directional lights
        let mut query = <(&DirectionalLightComponent, &TransformComponent)>::query();
        for (dlc, tc) in query.iter(&self.m_World)
        {
            let light = Light::CreateDirectionalLight(XMFLOAT3::from(dlc.Color), dlc.Intensity, XMFLOAT3::set(-tc.Position[0], -tc.Position[1], -tc.Position[2]));
            environment.AddLight(light);
        }

        // Point lights
        let mut query = <(&PointLightComponent, &TransformComponent)>::query();
        for (plc, tc) in query.iter(&self.m_World)
        {
            let light = Light::CreatePointLight(XMFLOAT3::from(plc.Color), plc.Intensity, XMFLOAT3::from(tc.Position));
            environment.AddLight(light);
        }

        renderer.BeginScene(&self.m_EditorCamera, environment);

        let mut query = <(&MeshComponent, &TransformComponent)>::query();
        for (mc, tc) in query.iter(&self.m_World)
        {
            let mesh: RustyRef<Mesh> = AssetManager::GetMesh(&mc.MeshPath);
            renderer.SubmitMesh(mesh.clone(), XMMatrixTranspose(tc.Transform()), &Vec::new());
        }

        renderer.Flush();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnRuntimeRender(&self, renderer: &mut SceneRenderer)
    {
        // Render the same way as the OnEditRender() for now

        // Setup the environment
        let mut environment = Environment::Create();

        // Sky light
        let mut query = <&SkyLightComponent>::query();
        for slc in query.iter(&self.m_World)
        {
            let envMap = AssetManager::GetEnvironmentMap(&slc.EnvironmentMapPath);
            if envMap.0.IsValid() && envMap.1.IsValid()
            {
                environment.SetEnvironmentMap(envMap.0.clone(), envMap.1.clone());
            }
            break;
        }

        // Directional lights
        let mut query = <(&DirectionalLightComponent, &TransformComponent)>::query();
        for (dlc, tc) in query.iter(&self.m_World)
        {
            let light = Light::CreateDirectionalLight(XMFLOAT3::from(dlc.Color), dlc.Intensity, XMFLOAT3::set(-tc.Position[0], -tc.Position[1], -tc.Position[2]));
            environment.AddLight(light);
        }

        // Point lights
        let mut query = <(&PointLightComponent, &TransformComponent)>::query();
        for (plc, tc) in query.iter(&self.m_World)
        {
            let light = Light::CreatePointLight(XMFLOAT3::from(plc.Color), plc.Intensity, XMFLOAT3::from(tc.Position));
            environment.AddLight(light);
        }

        renderer.BeginScene(&self.m_EditorCamera, environment);

        let mut query = <(&MeshComponent, &TransformComponent)>::query();
        for (mc, tc) in query.iter(&self.m_World)
        {
            let mesh: RustyRef<Mesh> = AssetManager::GetMesh(&mc.MeshPath);
            renderer.SubmitMesh(mesh.clone(), XMMatrixTranspose(tc.Transform()), &Vec::new());
        }

        renderer.Flush();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnViewportResize(&mut self, viewportWidth: f32, viewportHeight: f32)
    {
        self.m_ViewportSize.x = viewportWidth;
        self.m_ViewportSize.y = viewportHeight;

        self.m_EditorCamera.SetViewportSize(viewportWidth, viewportHeight);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Serialize(&self, filename: &str)
    {
        // Create a registry which uses strings as the external type ID
        let mut registry = Registry::<String>::default();
        registry.register::<TagComponent>("TagComponent".to_string());
        registry.register::<TransformComponent>("TransformComponent".to_string());
        registry.register::<MeshComponent>("MeshComponent".to_string());
        registry.register::<SkyLightComponent>("SkyLightComponent".to_string());
        registry.register::<DirectionalLightComponent>("DirectionalLightComponent".to_string());
        registry.register::<PointLightComponent>("PointLightComponent".to_string());

        let entitySerializer = Canon::default();
        let sceneFilestring = serde_json::to_string(&self.m_World.as_serializable(legion::any(), &registry, &entitySerializer)).expect("Failed to serialize world!");
        let mut sceneFile = std::fs::File::create(filename).expect("Failed to create scene file!");
        sceneFile.write_all(sceneFilestring.as_bytes()).expect("Writing to the scene file failed!");
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Deserialize(&mut self, filename: &str)
    {
        // Read the file
        let mut sceneFile = std::fs::File::open(filename).expect("Failed to open scene file");
        let mut fileContent = String::new();
        sceneFile.read_to_string(&mut fileContent).expect("Failed reading the scene file!");

        
        // Create a registry which uses strings as the external type ID
        let mut registry = Registry::<String>::default();
        registry.register::<TagComponent>("TagComponent".to_string());
        registry.register::<TransformComponent>("TransformComponent".to_string());
        registry.register::<MeshComponent>("MeshComponent".to_string());
        registry.register::<SkyLightComponent>("SkyLightComponent".to_string());
        registry.register::<DirectionalLightComponent>("DirectionalLightComponent".to_string());
        registry.register::<PointLightComponent>("PointLightComponent".to_string());
        
        let json: serde_json::Value = serde_json::from_str(&fileContent).expect("");
        let entityDeserializer = Canon::default();
        self.m_World = registry.as_deserialize(&entityDeserializer).deserialize(&json).expect("Failed to deserialize the scene!");

        // Load assets
        let mut query = <&SkyLightComponent>::query();
        for slc in query.iter(&self.m_World)
        {
            AssetManager::LoadEnvironmentMap(&slc.EnvironmentMapPath);
        }

        let mut query = <&MeshComponent>::query();
        for mc in query.iter(&self.m_World)
        {
            AssetManager::LoadMesh(&mc.MeshPath);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetSceneState(&mut self, state: SceneState)
    {
        self.m_State = state;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSceneState(&self) -> SceneState
    {
        return self.m_State;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetEditorCamera(&mut self) -> &mut EditorCamera
    {
        return &mut self.m_EditorCamera;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetWorld(&self) -> &legion::World
    {
        return &self.m_World;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetWorldMut(&mut self) -> &mut legion::World
    {
        return &mut self.m_World;
    }
}
