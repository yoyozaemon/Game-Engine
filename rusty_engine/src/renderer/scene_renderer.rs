#![allow(non_snake_case)]

// std
use std::cell::*;

// Win32
use directx_math::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Foundation::*;

// Core
use crate::core::RustyRef;

// Renderer
use crate::renderer::graphics_pipeline::*;
use crate::renderer::editor_camera::*;
use crate::renderer::command_buffer::*;
use crate::renderer::texture::*;
use crate::renderer::render_target::*;
use crate::renderer::mesh::*;
use crate::renderer::material::*;
use crate::renderer::shader_library::*;
use crate::renderer::renderer::*;
use crate::renderer::environment::*;

struct TransformCB
{
    pub ProjectionMatrix: XMMATRIX,
    pub ViewMatrix:       XMMATRIX,
    pub ModelMatrix:      XMMATRIX,
    pub CameraPosition:   XMFLOAT3
}

struct DrawCommand
{
    pub Mesh:         RustyRef<Mesh>,
    pub SubmeshIndex: u32,
    pub Transform:    XMMATRIX,
    pub Material:     RustyRef<Material>
}

pub struct SceneRenderer
{
    m_CommandBuffer:          RustyRef<CommandBuffer>,
    m_TransformCB:            RustyRef<TransformCB>,
    m_SwapChainTarget:        bool,
    m_DrawList:               Vec<DrawCommand>,

    // Geometry pass
    m_GeometryPipeline:       RustyRef<GraphicsPipeline>,
    m_GridPipeline:           RustyRef<GraphicsPipeline>,
    m_Grid:                   RustyRef<Mesh>,
    m_GridMaterial:           RustyRef<Material>,

    // Composite pass
    m_CompositePipeline:      RustyRef<GraphicsPipeline>,
    m_CompositeMaterial:      RustyRef<Material>,

    // Swap chain pass
    m_FullScreenQuadPipeline: RustyRef<GraphicsPipeline>,
    m_FullScreenQuadMaterial: RustyRef<Material>,

    // Skybox pass
    m_SkyBoxPipeline:         RustyRef<GraphicsPipeline>,
    m_SkyBoxMaterial:         RustyRef<Material>,

    m_Exposure:               f32
}

impl SceneRenderer
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(swapChainTarget: bool) -> SceneRenderer
    {
        return SceneRenderer {
            m_CommandBuffer: RustyRef::CreateEmpty(),
            m_GeometryPipeline: RustyRef::CreateEmpty(),
            m_GridPipeline: RustyRef::CreateEmpty(),
            m_FullScreenQuadPipeline: RustyRef::CreateEmpty(),
            m_TransformCB: RustyRef::CreateEmpty(),
            m_FullScreenQuadMaterial: RustyRef::CreateEmpty(),
            m_Grid: RustyRef::CreateEmpty(),
            m_GridMaterial: RustyRef::CreateEmpty(),
            m_SwapChainTarget: swapChainTarget,
            m_DrawList: vec![],
            m_SkyBoxPipeline: RustyRef::CreateEmpty(),
            m_SkyBoxMaterial: RustyRef::CreateEmpty(),
            m_CompositePipeline: RustyRef::CreateEmpty(),
            m_CompositeMaterial: RustyRef::CreateEmpty(),
            m_Exposure: 0.3,
        };
    }

    pub fn GetExposure(&mut self) -> &mut f32
    {
        return &mut self.m_Exposure;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Initialize(&mut self)
    {
        //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //////////////////////////////////////////// Gfx context and command buffer //////////////////////////////////////////////////////////
        
        self.m_CommandBuffer = CommandBuffer::Create();

        //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////// Pipelines /////////////////////////////////////////////////////////
        
        self.CreatePipelines();

        //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////// Fullscreen quad ///////////////////////////////////////////////////////////////
        
        self.m_FullScreenQuadMaterial = Material::Create("FullScreenQuad", ShaderLibrary::GetShader("fullscreen_quad_shader"), MaterialFlags::None);

        //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////// Grid //////////////////////////////////////////////////////////////////////////
        
        self.m_Grid = Mesh::CreatePlane(1000.0, 1000.0);

        let textureDesc = TextureDescription {
            Name: String::from("GridTexture"),
            Width: 0,
            Height: 0,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            MipCount: 7,
            ImageData: vec![Some(Image::LoadFromFile("rusty_engine/assets/textures/grid.png", false))]
        };

        let samplerDesc = SamplerDescription {
            Wrap: D3D11_TEXTURE_ADDRESS_MIRROR,
            Filter: D3D11_FILTER_ANISOTROPIC
        };

        self.m_GridMaterial = Material::Create("Grid", ShaderLibrary::GetShader("grid_shader").clone(), MaterialFlags::TwoSided | MaterialFlags::Transparent);
        self.m_GridMaterial.GetRefMut().SetTexture("u_GridTexture", Texture::CreateTexture2D(&textureDesc, &samplerDesc));

        //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        ///////////////////////////////////////////////////// Constant buffers ///////////////////////////////////////////////////////////////
        
        self.m_TransformCB = RustyRef::CreateRef(TransformCB{
            ViewMatrix: XMMatrixIdentity(),
            ProjectionMatrix: XMMatrixIdentity(),
            ModelMatrix: XMMatrixIdentity(),
            CameraPosition: XMFLOAT3::default()
        });


        self.m_CompositeMaterial = Material::Create("CompositeMaterial", ShaderLibrary::GetShader("composite_shader").clone(), MaterialFlags::DisableDepthTest);

        //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////// Skybox ////////////////////////////////////////////////////////////////////////
        
        self.m_SkyBoxMaterial = Material::Create("SkyBoxMaterial", ShaderLibrary::GetShader("skybox_shader").clone(), MaterialFlags::DisableDepthTest);
        self.m_SkyBoxMaterial.GetRefMut().SetTexture("u_EnvironmentMap", Renderer::GetBlackTextureCube());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn BeginScene(&mut self, camera: &EditorCamera, environment: Environment)
    {
        let mut transformCB: RefMut<TransformCB> = self.m_TransformCB.GetRefMut();

        // Copy the camera matrices to the constant buffer
        transformCB.ViewMatrix = XMMatrixTranspose(*camera.GetViewMatrix());
        transformCB.ProjectionMatrix = XMMatrixTranspose(*camera.GetProjectionMatrix());
        transformCB.CameraPosition = camera.GetPosition();

        self.m_CommandBuffer.GetRef().SetPSConstantBufferData(ShaderLibrary::GetShader("mesh_pbr_shader").clone(), environment.GetLights(), 0);

        self.m_SkyBoxMaterial.GetRefMut().SetTexture("u_EnvironmentMap", environment.GetEnvironmentMap());
        self.m_CommandBuffer.GetRefMut().SetPSShaderResources(&environment.GetEnvironmentMap().GetRef().CreateSRV(), 1, 4);
        self.m_CommandBuffer.GetRefMut().SetPSSamplers(environment.GetEnvironmentMap().GetRef().GetSampler(), 1, 4);

        self.m_CommandBuffer.GetRefMut().SetPSShaderResources(&environment.GetIrradianceMap().GetRef().CreateSRV(), 1, 5);
        self.m_CommandBuffer.GetRefMut().SetPSSamplers(environment.GetIrradianceMap().GetRef().GetSampler(), 1, 5);

        self.m_CommandBuffer.GetRefMut().SetPSShaderResources(&Renderer::GetBRDF().GetRef().CreateSRV(), 1, 6);
        self.m_CommandBuffer.GetRefMut().SetPSSamplers(Renderer::GetBRDF().GetRef().GetSampler(), 1, 6);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SubmitMesh(&mut self, mesh: RustyRef<Mesh>, transform: XMMATRIX, materials: &Vec<RustyRef<Material>>)
    {
        if !mesh.IsValid()
        {
            return;
        }

        let meshRef: Ref<Mesh> = mesh.GetRef();
        let submeshes: &Vec<Submesh> = meshRef.GetSubmeshes();

        for index in 0..submeshes.len()
        {
            let materialIndex: usize = submeshes[index].MaterialIndex as usize;
            let mut submeshMaterial: RustyRef<Material> = RustyRef::CreateEmpty();

            if materials.len() > index && materials[index].IsValid()
            {
                submeshMaterial = materials[index].clone();
            }
            else
            {
                submeshMaterial = meshRef.GetMaterials()[materialIndex].clone()
            }

            let drawCmd = DrawCommand {
                Mesh: mesh.clone(),
                SubmeshIndex: index as u32,
                Transform: transform,
                Material: submeshMaterial
            };

            self.m_DrawList.push(drawCmd);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Flush(&mut self)
    {
        self.PreRender();
        self.GeometryPass();
        self.CompositePass();

        if self.m_SwapChainTarget
        {
            self.SwapChainPass();
        }

        self.m_CommandBuffer.GetRefMut().Finish();
        Renderer::GetGfxContext().GetRef().ExecuteCommandBuffer(self.m_CommandBuffer.clone());
        self.m_DrawList.clear();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetCompositePassTexture(&self) -> RustyRef<Texture>
    {
        return self.m_CompositePipeline.GetRef().GetRenderTarget().GetRef().GetColorAttachment(0).clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetCompositePassTarget(&mut self) -> RustyRef<RenderTarget>
    {
        return self.m_CompositePipeline.GetRef().GetRenderTarget().clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn PreRender(&mut self)
    {
        self.m_DrawList.sort_by_key(|cmd| { cmd.Material.GetRef().GetRenderFlags() & MaterialFlags::Transparent != MaterialFlags::None });
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn GeometryPass(&mut self)
    {
        Renderer::BeginRenderPass(self.m_CommandBuffer.clone(), self.m_GeometryPipeline.GetRef().GetRenderTarget().clone(), true);

        // Render skybox
        self.m_TransformCB.GetRefMut().ModelMatrix = XMMatrixTranspose(XMMatrixIdentity());
        let invViewProjMatrix: XMMATRIX = XMMatrixInverse(None, XMMatrixMultiply(self.m_TransformCB.GetRef().ProjectionMatrix, &self.m_TransformCB.GetRef().ViewMatrix));
        self.m_SkyBoxMaterial.GetRefMut().SetUniform("u_InvViewProjMatrix", invViewProjMatrix);
        Renderer::RenderFullscreenQuad(self.m_CommandBuffer.clone(), self.m_SkyBoxPipeline.clone(), self.m_SkyBoxMaterial.clone());

        // Render scene
        for drawCmd in self.m_DrawList.iter()
        {
            self.m_TransformCB.GetRefMut().ModelMatrix = drawCmd.Transform;
            Renderer::RenderMesh(self.m_CommandBuffer.clone(), self.m_GeometryPipeline.clone(), drawCmd.Mesh.clone(), drawCmd.SubmeshIndex, drawCmd.Material.clone(), self.m_TransformCB.clone());
        }

        // Render grid
        self.m_TransformCB.GetRefMut().ModelMatrix = XMMatrixTranspose(XMMatrixIdentity());
        Renderer::RenderMesh(self.m_CommandBuffer.clone(), self.m_GridPipeline.clone(), self.m_Grid.clone(), 0, self.m_GridMaterial.clone(), self.m_TransformCB.clone());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn CompositePass(&mut self)
    {
        Renderer::BeginRenderPass(self.m_CommandBuffer.clone(), self.m_CompositePipeline.GetRef().GetRenderTarget().clone(), true);

        self.m_CompositeMaterial.GetRefMut().SetTexture("u_SceneTexture", self.m_GeometryPipeline.GetRef().GetRenderTarget().GetRef().GetColorAttachment(0).clone());
        self.m_CompositeMaterial.GetRefMut().SetUniform("u_Exposure", self.m_Exposure);

        Renderer::RenderFullscreenQuad(self.m_CommandBuffer.clone(), self.m_CompositePipeline.clone(), self.m_CompositeMaterial.clone());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn SwapChainPass(&mut self)
    {
        Renderer::BeginSwapChainRenderPass(self.m_CommandBuffer.clone(), true);

        self.m_FullScreenQuadMaterial.GetRefMut().SetTexture("u_SceneTexture", self.m_CompositePipeline.GetRef().GetRenderTarget().GetRef().GetColorAttachment(0).clone());

        Renderer::RenderFullscreenQuad(self.m_CommandBuffer.clone(), self.m_FullScreenQuadPipeline.clone(), self.m_FullScreenQuadMaterial.clone());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn CreatePipelines(&mut self)
    {
        let mut defaultRasterizer = D3D11_RASTERIZER_DESC::default();
        defaultRasterizer.FillMode = D3D11_FILL_SOLID;
        defaultRasterizer.CullMode = D3D11_CULL_BACK;
        defaultRasterizer.FrontCounterClockwise = BOOL::from(true);
        
        let rtDesc = RenderTargetDescription {
            Width: 1280,
            Height: 720,
            ClearColor: XMFLOAT4::set(0.1, 0.2, 0.3, 1.0),
            Attachments: vec![
                RenderTargetAttachment {
                    Format: DXGI_FORMAT_R16G16B16A16_FLOAT,
                    MipCount: 1,
                    Wrap: D3D11_TEXTURE_ADDRESS_WRAP,
                    Filter: D3D11_FILTER_ANISOTROPIC     
                },

                RenderTargetAttachment {
                    Format: DXGI_FORMAT_D24_UNORM_S8_UINT,
                    MipCount: 1,
                    Wrap: D3D11_TEXTURE_ADDRESS_WRAP,
                    Filter: D3D11_FILTER_ANISOTROPIC     
                }
            ]
        };

        let geometryRenderTarget = RenderTarget::Create(rtDesc); 

        // Geometry pipeline
        {
            let desc = GraphicsPipelineDescription {
                RasterizerStateDesc:   defaultRasterizer,
                BlendStateDesc:        D3D11_BLEND_DESC::default(),
                DepthStencilStateDesc: D3D11_DEPTH_STENCIL_DESC::default(),
                Topology:              D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
                InputLayoutDesc: vec![
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("POSITION\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("TEX_COORD\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT ,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("NORMAL\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("TANGENT\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("BITANGENT\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                ],
                Shader: ShaderLibrary::GetShader("mesh_pbr_shader").clone(),
                RenderTarget: geometryRenderTarget.clone()
            };

            self.m_GeometryPipeline = GraphicsPipeline::Create(desc);
        }

        // Grid pipeline
        {
            let desc = GraphicsPipelineDescription {
                RasterizerStateDesc:   defaultRasterizer,
                BlendStateDesc:        D3D11_BLEND_DESC::default(),
                DepthStencilStateDesc: D3D11_DEPTH_STENCIL_DESC::default(),
                Topology:              D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
                InputLayoutDesc: vec![
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("POSITION\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("TEX_COORD\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT ,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("NORMAL\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("TANGENT\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("BITANGENT\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                ],
                Shader: ShaderLibrary::GetShader("grid_shader").clone(),
                RenderTarget: geometryRenderTarget.clone()
            };

            self.m_GridPipeline = GraphicsPipeline::Create(desc);
        }

        // Composite pipeline
        {
            let rtDesc = RenderTargetDescription {
                Width: 1280,
                Height: 720,
                ClearColor: XMFLOAT4::set(0.1, 0.2, 0.3, 1.0),
                Attachments: vec![
                    RenderTargetAttachment {
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        MipCount: 1,
                        Wrap: D3D11_TEXTURE_ADDRESS_WRAP,
                        Filter: D3D11_FILTER_ANISOTROPIC     
                    }
                ]
            };
    
            let compositeRenderTarget = RenderTarget::Create(rtDesc); 

            let desc = GraphicsPipelineDescription {
                RasterizerStateDesc:   defaultRasterizer,
                BlendStateDesc:        D3D11_BLEND_DESC::default(),
                DepthStencilStateDesc: D3D11_DEPTH_STENCIL_DESC::default(),
                Topology:              D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
                InputLayoutDesc: vec![
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("POSITION\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("TEX_COORD\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT ,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    }
                ],
                Shader: ShaderLibrary::GetShader("composite_shader").clone(),
                RenderTarget: compositeRenderTarget
            };

            self.m_CompositePipeline = GraphicsPipeline::Create(desc);
        }

        // Fullscreen quad pipeline
        {
            let desc = GraphicsPipelineDescription {
                RasterizerStateDesc:   defaultRasterizer,
                BlendStateDesc:        D3D11_BLEND_DESC::default(),
                DepthStencilStateDesc: D3D11_DEPTH_STENCIL_DESC::default(),
                Topology:              D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
                InputLayoutDesc: vec![
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("POSITION\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: 0,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("TEX_COORD\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT ,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    }
                ],
                Shader: ShaderLibrary::GetShader("fullscreen_quad_shader").clone(),
                RenderTarget: RustyRef::CreateEmpty()
            };

            self.m_FullScreenQuadPipeline = GraphicsPipeline::Create(desc);
        }

        // Skybox pipeline
        {
            let desc = GraphicsPipelineDescription {
                RasterizerStateDesc:   defaultRasterizer,
                BlendStateDesc:        D3D11_BLEND_DESC::default(),
                DepthStencilStateDesc: D3D11_DEPTH_STENCIL_DESC::default(),
                Topology:              D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
                InputLayoutDesc: vec![
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("POSITION\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32B32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: 0,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    },
                    D3D11_INPUT_ELEMENT_DESC { 
                        SemanticName: PSTR("TEX_COORD\0".as_ptr() as _), 
                        SemanticIndex: 0, 
                        Format: DXGI_FORMAT_R32G32_FLOAT, 
                        InputSlot: 0,
                        AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT ,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0
                    }
                ],
                Shader: ShaderLibrary::GetShader("skybox_shader").clone(),
                RenderTarget: geometryRenderTarget.clone()
            };

            self.m_SkyBoxPipeline = GraphicsPipeline::Create(desc);
        }
    }
}