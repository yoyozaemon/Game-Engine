#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// std
use std::collections::HashMap;

// Win32
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Direct3D11::*;

// Core
use crate::core::utils::*;

// Renderer
use crate::renderer::mesh::*;
use crate::renderer::texture::*;
use crate::renderer::command_buffer::*;
use crate::renderer::renderer::*;

pub struct AssetManager
{
    m_Meshes:          HashMap<String, RustyRef<Mesh>>,
    m_Textures:        HashMap<String, RustyRef<Texture>>,
    m_EnvironmentMaps: HashMap<String, (RustyRef<Texture>,RustyRef<Texture>)>,
}

thread_local!
{
    static s_Instance: RustyRef<AssetManager> = RustyRef::CreateRef(AssetManager{
        m_Meshes: HashMap::new(),
        m_Textures: HashMap::new(),
        m_EnvironmentMaps: HashMap::new()
    });
}

impl AssetManager
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LoadMesh(filepath: &str) -> RustyRef<Mesh>
    {
        if AssetManager::GetMesh(filepath).IsValid()
        {
            return AssetManager::GetMesh(filepath);
        }

        let mesh = Mesh::LoadFromFile(filepath);
        
        s_Instance.try_with(|assetManager|
        {
            assetManager.GetRefMut().m_Meshes.insert(String::from(filepath), mesh.clone());

        }).expect("Failed getting asset manager instance!");

        return mesh;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LoadTexture(filepath: &str, flipY: bool) -> RustyRef<Texture>
    {
        if AssetManager::GetTexture(filepath).IsValid()
        {
            return AssetManager::GetTexture(filepath);
        }

        let filename: &str = std::path::Path::new(filepath).file_stem().unwrap().to_str().unwrap();

        let textureDesc = TextureDescription {
            Name: String::from(filename),
            Width: 0,
            Height: 0,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            MipCount: 6,
            ImageData: vec![Some(Image::LoadFromFile(filepath, flipY))]
        };
    
        let samplerDesc = SamplerDescription {
            Wrap: D3D11_TEXTURE_ADDRESS_WRAP,
            Filter: D3D11_FILTER_ANISOTROPIC
        };

        let texture = Texture::CreateTexture2D(&textureDesc, &samplerDesc);
        
        s_Instance.try_with(|assetManager|
        {
            assetManager.GetRefMut().m_Textures.insert(String::from(filepath), texture.clone());

        }).expect("Failed getting asset manager instance!");

        return texture;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LoadEnvironmentMap(filepath: &str) -> (RustyRef<Texture>, RustyRef<Texture>)
    {
        if AssetManager::GetEnvironmentMap(filepath).0.IsValid()
        {
            return AssetManager::GetEnvironmentMap(filepath);
        }

        let commandBuffer: RustyRef<CommandBuffer> = CommandBuffer::Create();
        let envMap: (RustyRef<Texture>, RustyRef<Texture>) = Renderer::CreateEnvironmentMap(commandBuffer.clone(), filepath);
        commandBuffer.GetRefMut().Finish();
        Renderer::GetGfxContext().GetRefMut().ExecuteCommandBuffer(commandBuffer);

        s_Instance.try_with(|assetManager|
        {
            assetManager.GetRefMut().m_EnvironmentMaps.insert(String::from(filepath), (envMap.0.clone(), envMap.1.clone()));

        }).expect("Failed getting asset manager instance!");

        return envMap;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetMesh(filepath: &str) -> RustyRef<Mesh>
    {
        s_Instance.try_with(|assetManager|
        {
            if let Some(mesh) = assetManager.GetRefMut().m_Meshes.get(&String::from(filepath))
            {
                return mesh.clone();
            }
            else
            {
                return RustyRef::CreateEmpty();
            }

        }).expect("Failed getting asset manager instance!")
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetTexture(filepath: &str) -> RustyRef<Texture>
    {
        s_Instance.try_with(|assetManager|
        {
            if let Some(texture) = assetManager.GetRefMut().m_Textures.get(&String::from(filepath))
            {
                return texture.clone();
            }
            else
            {
                return RustyRef::CreateEmpty();
            }

        }).expect("Failed getting asset manager instance!")
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetEnvironmentMap(filepath: &str) -> (RustyRef<Texture>, RustyRef<Texture>) 
    {
        s_Instance.try_with(|assetManager|
        {
            if let Some(envMap) = assetManager.GetRefMut().m_EnvironmentMaps.get(&String::from(filepath))
            {
                return (envMap.0.clone(), envMap.1.clone());
            }
            else
            {
                return (RustyRef::CreateEmpty(), RustyRef::CreateEmpty());
            }

        }).expect("Failed getting asset manager instance!")
    }
}