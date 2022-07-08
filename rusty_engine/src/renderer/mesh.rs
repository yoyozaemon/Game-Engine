#![allow(non_snake_case)]

// std
use std::mem::size_of;
use std::path::Path;

// Win32
use directx_math::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;

// Renderer
use crate::renderer::vertex_buffer::*;
use crate::renderer::index_buffer::*;
use crate::renderer::material::*;
use crate::renderer::shader_library::*;

// Core
use crate::core::utils::*;
use crate::core::asset_manager::*;

// Other
use russimp::scene::*;

#[derive(Clone, Default)]
pub struct Vertex
{
    pub Position:  XMFLOAT3,
    pub UV:        XMFLOAT2,
    pub Normal:    XMFLOAT3,
    pub Tangent:   XMFLOAT3,
    pub Bitangent: XMFLOAT3,
}

pub struct Submesh
{
    pub Name: String,
    pub VertexOffset: u32,
    pub VertexCount: u32,
    pub IndexOffset: u32,
    pub IndexCount: u32,
    pub MaterialIndex: u32
}

pub struct Mesh
{
    m_Name:         String,
    m_VertexBuffer: VertexBuffer,
    m_IndexBuffer:  IndexBuffer,
    m_Submeshes:    Vec<Submesh>,
    m_Materials:    Vec<RustyRef<Material>>,
    m_Filepath:     String
}

impl Mesh
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(name: &str, vertices: Vec<Vertex>, indices: Vec<u32>, submeshes: Vec<Submesh>, materials: Vec<RustyRef<Material>>) -> RustyRef<Mesh>
    {
        let vbDesc = VertexBufferDescription {
            VertexCount: vertices.len() as u32,
            Stride: size_of::<Vertex>() as u32,
            Size: (vertices.len() * size_of::<Vertex>()) as u32,
            Usage: D3D11_USAGE_DEFAULT
        };

        let vertexBuffer = VertexBuffer::Create(vertices.as_ptr() as _, &vbDesc);

        let ibDesc = IndexBufferDescription {
            IndexCount: indices.len() as u32,
            Size: (indices.len() * size_of::<u32>()) as u32,
            Format: DXGI_FORMAT_R32_UINT,
            Usage: D3D11_USAGE_DEFAULT
        };

        let indexBuffer = IndexBuffer::Create(indices.as_ptr(), &ibDesc);

        return RustyRef::CreateRef(Mesh {
            m_Name: String::from(name),
            m_VertexBuffer: vertexBuffer,
            m_IndexBuffer: indexBuffer,
            m_Submeshes: submeshes,
            m_Materials: materials,
            m_Filepath: String::new()
        });
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LoadFromFile(filepath: &str) -> RustyRef<Mesh>
    {
        let scene = Scene::from_file(filepath, vec![
            PostProcess::Triangulate,
            PostProcess::JoinIdenticalVertices,
            PostProcess::MakeLeftHanded,
            PostProcess::GenerateNormals,
            PostProcess::GenerateUVCoords,
            PostProcess::PreTransformVertices,
            PostProcess::ValidateDataStructure,
            PostProcess::CalculateTangentSpace]).unwrap();

        let meshName: &str = Path::new(filepath).file_stem().unwrap().to_str().unwrap();
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        let mut submeshes: Vec<Submesh> = Vec::with_capacity(scene.meshes.len());
        let mut materials: Vec<RustyRef<Material>> = Vec::with_capacity(scene.materials.len());

        for submesh in scene.meshes.iter()
        {
            let vertexOffset: u32 = vertices.len() as u32;
            let indexOffset: u32 = indices.len() as u32;

            let positions = &submesh.vertices;
            let UVs = &submesh.texture_coords;
            let normals = &submesh.normals;
            let tangents = &submesh.tangents;
            let bitangents = &submesh.bitangents;
            
            let mut submeshVertexCount: u32 = 0;

            for i in 0..submesh.vertices.len()
            {
                let vertex = Vertex {
                    Position: XMFLOAT3::set(positions[i].x, positions[i].y, positions[i].z),
                    UV: XMFLOAT2::set(UVs[0].as_ref().unwrap()[i].x, UVs[0].as_ref().unwrap()[i].y),
                    Normal: XMFLOAT3::set(normals[i].x, normals[i].y, normals[i].z),
                    Tangent: XMFLOAT3::set(tangents[i].x, tangents[i].y, tangents[i].z),
                    Bitangent: XMFLOAT3::set(bitangents[i].x, bitangents[i].y, bitangents[i].z)
                };

                vertices.push(vertex);
                submeshVertexCount += 1;
            }
            
            let mut submeshIndexCount: u32 = 0;

            for face in submesh.faces.iter()
            {
                for index in face.0.iter()
                {
                    indices.push(*index);
                }

                submeshIndexCount += face.0.len() as u32;
            }

            // Create the material for the submesh
            let material: RustyRef<Material> = Material::Create("UnnamedMaterial", ShaderLibrary::GetShader("mesh_pbr_shader"), MaterialFlags::None);
            let mut materialFlags: MaterialFlags = MaterialFlags::TwoSided;

            let parentDirectory = Path::new(filepath).parent().unwrap().to_str().unwrap();

            for property in scene.materials[submesh.material_index as usize].properties.iter()
            {
                // Name
                if property.key == "?mat.name"
                {
                    match &property.data
                    {
                        russimp::material::PropertyTypeInfo::String(name) => 
                        {
                            material.GetRefMut().SetName(&name);
                        }
                        _ => debug_assert!(false, "Invalid value type!")
                    }
                }

                // Albedo
                if property.key == "$clr.diffuse"
                {
                    match &property.data
                    {
                        russimp::material::PropertyTypeInfo::FloatArray(color) => 
                        {
                            material.GetRefMut().SetUniform("AlbedoColor", XMFLOAT4::set(color[0], color[1], color[2], 1.0));
                        }
                        _ => debug_assert!(false, "Invalid value type!")
                    }
                }

                // Metalness
                if property.key == "$mat.reflectivity"
                {
                    match &property.data
                    {
                        russimp::material::PropertyTypeInfo::FloatArray(metalness) => 
                        {
                            material.GetRefMut().SetUniform("Metalness", metalness[0]);
                        }
                        _ => debug_assert!(false, "Invalid value type!")
                    }
                }

                // Roughness
                if property.key == "$mat.shininess"
                {
                    match &property.data
                    {
                        russimp::material::PropertyTypeInfo::FloatArray(shininess) => 
                        {
                            material.GetRefMut().SetUniform("Roughness", 1.0 - shininess[0] / 100.0);
                        }
                        _ => debug_assert!(false, "Invalid value type!")
                    }
                }

                // Albedo Map
                if property.key == "$raw.DiffuseColor|file"
                {
                    match &property.data
                    {
                        russimp::material::PropertyTypeInfo::String(textureName) => 
                        {
                            let path = format!("{}/textures/{}", parentDirectory, textureName);
                            material.GetRefMut().SetTexture("AlbedoMap", AssetManager::LoadTexture(&path, false));
                            material.GetRefMut().SetUniform("UseAlbedoMap", 1);
                        }
                        _ => debug_assert!(false, "Invalid value type!")
                    }
                }

                // Normal Map
                if property.key == "$raw.NormalMap|file"
                {
                    match &property.data
                    {
                        russimp::material::PropertyTypeInfo::String(textureName) => 
                        {
                            let path = format!("{}/textures/{}", parentDirectory, textureName);
                            material.GetRefMut().SetTexture("NormalMap", AssetManager::LoadTexture(&path, false));
                            material.GetRefMut().SetUniform("UseNormalMap", 1);
                        }
                        _ => debug_assert!(false, "Invalid value type!")
                    }
                }

                // Metalness Map
                if property.key == "$raw.ReflectionFactor|file"
                {
                    match &property.data
                    {
                        russimp::material::PropertyTypeInfo::String(textureName) => 
                        {
                            let path = format!("{}/textures/{}", parentDirectory, textureName);
                            material.GetRefMut().SetTexture("MetalnessMap", AssetManager::LoadTexture(&path, false));
                            material.GetRefMut().SetUniform("UseMetalnessMap", 1);
                        }
                        _ => debug_assert!(false, "Invalid value type!")
                    }
                }

                // Roughness Map
                if property.key == "$raw.ShininessExponent|file"
                {
                    match &property.data
                    {
                        russimp::material::PropertyTypeInfo::String(textureName) => 
                        {
                            let path = format!("{}/textures/{}", parentDirectory, textureName);
                            material.GetRefMut().SetTexture("RoughnessMap", AssetManager::LoadTexture(&path, false));
                            material.GetRefMut().SetUniform("UseRoughnessMap", 1);
                        }
                        _ => debug_assert!(false, "Invalid value type!")
                    }
                }

                // Transparency flag
                if property.key == "$mat.opacity"
                {
                    match &property.data
                    {
                        russimp::material::PropertyTypeInfo::FloatArray(opacity) => 
                        {
                            if opacity[0] < 1.0
                            {
                                materialFlags |= MaterialFlags::Transparent;
                                
                                let mut albedoColor = *material.GetRef().GetUniform::<XMFLOAT4>("AlbedoColor");
                                albedoColor.w = opacity[0];
                                material.GetRefMut().SetUniform("AlbedoColor", albedoColor);
                            }
                        }
                        _ => debug_assert!(false, "Invalid value type!")
                    }
                }

                if property.key == "$mat.twosided"
                {
                    materialFlags |= MaterialFlags::TwoSided;
                }
            }

            material.GetRefMut().SetRenderFlags(materialFlags);
            materials.push(material);

            submeshes.push(Submesh{
                Name: submesh.name.clone(),
                VertexOffset: vertexOffset,
                VertexCount: submeshVertexCount,
                IndexOffset: indexOffset,
                IndexCount: submeshIndexCount,
                MaterialIndex: submesh.material_index
            });
        }

        let vbDesc = VertexBufferDescription {
            VertexCount: vertices.len() as u32,
            Stride: size_of::<Vertex>() as u32,
            Size: (vertices.len() * size_of::<Vertex>()) as u32,
            Usage: D3D11_USAGE_DEFAULT
        };

        let vertexBuffer = VertexBuffer::Create(vertices.as_ptr() as _, &vbDesc);

        let ibDesc = IndexBufferDescription {
            IndexCount: indices.len() as u32,
            Size: (indices.len() * size_of::<u32>()) as u32,
            Format: DXGI_FORMAT_R32_UINT,
            Usage: D3D11_USAGE_DEFAULT
        };

        let indexBuffer = IndexBuffer::Create(indices.as_ptr(), &ibDesc);

        return RustyRef::CreateRef(Mesh {
            m_Name: String::from(meshName),
            m_VertexBuffer: vertexBuffer,
            m_IndexBuffer: indexBuffer,
            m_Submeshes: submeshes,
            m_Materials: materials,
            m_Filepath: String::from(filepath)
        });
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetName(&self) -> &String
    {
        return &self.m_Name;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVertexBuffer(&self) -> &VertexBuffer
    {
        return &self.m_VertexBuffer;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetIndexBuffer(&self) -> &IndexBuffer
    {
        return &self.m_IndexBuffer;
    }
    
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSubmeshes(&self) -> &Vec<Submesh>
    {
        return &self.m_Submeshes;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetMaterials(&self) -> &Vec<RustyRef<Material>>
    {
        return &self.m_Materials;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetFilepath(&self) -> &String
    {
        return &self.m_Filepath;
    }

    // ------------------------------------------------------------------- Mesh Factory ---------------------------------------------------------------------
    pub fn CreatePlane(width: f32, height: f32) -> RustyRef<Mesh>
    {
        let mut vertices: Vec<Vertex> = vec![Vertex::default(); 4];

        vertices[0].Position = XMFLOAT3::set(-width / 2.0, 0.0, -height / 2.0);
        vertices[0].UV = XMFLOAT2::set(0.0, 0.0);
        vertices[0].Normal = XMFLOAT3::set(0.0, 1.0, 0.0);

        vertices[1].Position = XMFLOAT3::set(width / 2.0, 0.0, -height / 2.0);
        vertices[1].UV = XMFLOAT2::set(100.0, 0.0);
        vertices[1].Normal = XMFLOAT3::set(0.0, 1.0, 0.0);

        vertices[2].Position = XMFLOAT3::set(width / 2.0, 0.0, height / 2.0);
        vertices[2].UV = XMFLOAT2::set(100.0, 100.0);
        vertices[2].Normal = XMFLOAT3::set(0.0, 1.0, 0.0);

        vertices[3].Position = XMFLOAT3::set(-width / 2.0, 0.0, height / 2.0);
        vertices[3].UV = XMFLOAT2::set(0.0, 100.0);
        vertices[3].Normal = XMFLOAT3::set(0.0, 1.0, 0.0);

        let indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0];

        let submesh = Submesh {
            Name: String::from("Plane"),
            VertexOffset: 0,
            VertexCount: vertices.len() as u32,
            IndexOffset: 0,
            IndexCount: indices.len() as u32,
            MaterialIndex: 0
        };

        let material: RustyRef<Material> = Material::Create("PlaneMaterial", ShaderLibrary::GetShader("mesh_pbr_shader"), MaterialFlags::None);

        return Mesh::Create("Plane", vertices, indices, vec![submesh], vec![material]);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateCube(size: f32) -> RustyRef<Mesh>
    {
        let mut positions: Vec<XMFLOAT3> = vec![XMFLOAT3::default(); 8];
        positions[0] = XMFLOAT3::set( size / 2.0, -size / 2.0, -size / 2.0);
        positions[1] = XMFLOAT3::set( size / 2.0,  size / 2.0, -size / 2.0);
        positions[2] = XMFLOAT3::set(-size / 2.0,  size / 2.0, -size / 2.0);
        positions[3] = XMFLOAT3::set(-size / 2.0, -size / 2.0, -size / 2.0);
        positions[4] = XMFLOAT3::set( size / 2.0, -size / 2.0, size / 2.0);
        positions[5] = XMFLOAT3::set( size / 2.0,  size / 2.0, size / 2.0);
        positions[6] = XMFLOAT3::set(-size / 2.0,  size / 2.0, size / 2.0);
        positions[7] = XMFLOAT3::set(-size / 2.0, -size / 2.0, size / 2.0);


        let mut uv: Vec<XMFLOAT2> = vec![XMFLOAT2::default(); 4];
        uv[0] = XMFLOAT2::set(0.0, 0.0);
        uv[1] = XMFLOAT2::set(0.0, 1.0);
        uv[2] = XMFLOAT2::set(1.0, 0.0);
        uv[3] = XMFLOAT2::set(1.0, 1.0);

        let mut normals: Vec<XMFLOAT3> = vec![XMFLOAT3::default(); 6];
        normals[0] = XMFLOAT3::set( 1.0,  0.0,  0.0);
        normals[1] = XMFLOAT3::set(-1.0,  0.0,  0.0);
        normals[2] = XMFLOAT3::set( 0.0,  1.0,  0.0);
        normals[3] = XMFLOAT3::set( 0.0, -1.0,  0.0);
        normals[4] = XMFLOAT3::set( 0.0,  0.0,  1.0);
        normals[5] = XMFLOAT3::set( 0.0,  0.0, -1.0);

        let vertices: Vec<Vertex> = vec!
        [
            Vertex { Position: positions[0], UV: uv[0], Normal: normals[0], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[1], UV: uv[2], Normal: normals[0], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[4], UV: uv[1], Normal: normals[0], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[4], UV: uv[1], Normal: normals[0], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[1], UV: uv[2], Normal: normals[0], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[5], UV: uv[3], Normal: normals[0], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[6], UV: uv[1], Normal: normals[1], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[2], UV: uv[0], Normal: normals[1], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[7], UV: uv[3], Normal: normals[1], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[7], UV: uv[3], Normal: normals[1], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[2], UV: uv[0], Normal: normals[1], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[3], UV: uv[2], Normal: normals[1], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[5], UV: uv[1], Normal: normals[2], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[1], UV: uv[0], Normal: normals[2], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[6], UV: uv[3], Normal: normals[2], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[6], UV: uv[3], Normal: normals[2], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[1], UV: uv[0], Normal: normals[2], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[2], UV: uv[2], Normal: normals[2], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[4], UV: uv[3], Normal: normals[3], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[7], UV: uv[1], Normal: normals[3], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[0], UV: uv[2], Normal: normals[3], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[0], UV: uv[2], Normal: normals[3], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[7], UV: uv[1], Normal: normals[3], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[3], UV: uv[0], Normal: normals[3], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[4], UV: uv[0], Normal: normals[4], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[5], UV: uv[2], Normal: normals[4], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[7], UV: uv[1], Normal: normals[4], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[7], UV: uv[1], Normal: normals[4], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[5], UV: uv[2], Normal: normals[4], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[6], UV: uv[3], Normal: normals[4], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[0], UV: uv[0], Normal: normals[5], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[3], UV: uv[1], Normal: normals[5], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[1], UV: uv[2], Normal: normals[5], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() },
            Vertex { Position: positions[1], UV: uv[2], Normal: normals[5], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[3], UV: uv[1], Normal: normals[5], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }, 
            Vertex { Position: positions[2], UV: uv[3], Normal: normals[5], Tangent: XMFLOAT3::default(), Bitangent: XMFLOAT3::default() }
        ];

        let indices: Vec<u32> = vec!
        [
            35,34,33,
            32,31,30,
            29,28,27,
            26,25,24,
            23,22,21,
            20,19,18,
            17,16,15,
            14,13,12,
            11,10,9,
            8,7,6,
            5,4,3,
            2,1,0
        ];

        let submesh = Submesh {
            Name: String::from("Cube"),
            VertexOffset: 0,
            VertexCount: vertices.len() as u32,
            IndexOffset: 0,
            IndexCount: indices.len() as u32,
            MaterialIndex: 0
        };

        let material: RustyRef<Material> = Material::Create("PlaneMaterial", ShaderLibrary::GetShader("mesh_pbr_shader"), MaterialFlags::None);

        return Mesh::Create("Cube", vertices, indices, vec![submesh], vec![material]);
    }
}