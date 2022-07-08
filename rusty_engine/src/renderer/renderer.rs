#![allow(non_snake_case, non_upper_case_globals)]

// std
use std::cell::Ref;
use std::mem::size_of;

// Win32
use directx_math::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;

// Core
use crate::core::utils::*;

// Renderer
use crate::renderer::vertex_buffer::*;
use crate::renderer::index_buffer::*;
use crate::renderer::texture::*;
use crate::renderer::graphics_context::*;
use crate::renderer::command_buffer::*;
use crate::renderer::graphics_pipeline::*;
use crate::renderer::material::*;
use crate::renderer::mesh::*;
use crate::renderer::shader::*;
use crate::renderer::shader_library::*;
use crate::renderer::render_target::*;

pub struct RendererData
{
    pub FullscreenQuadVB: VertexBuffer,
    pub FullscreenQuadIB: IndexBuffer,
    pub BRDFTexture:      RustyRef<Texture>,
    pub BlackTexture:     RustyRef<Texture>,
    pub BlackTextureCube: RustyRef<Texture>
}

pub struct Renderer
{
    m_RendererData: RustyRef<RendererData>,
    m_GfxContext:   RustyRef<GraphicsContext>
}

thread_local!
{
    static s_Instance: RustyRef<Renderer> = RustyRef::CreateRef(Renderer{
        m_RendererData: RustyRef::CreateEmpty(),
        m_GfxContext: RustyRef::CreateEmpty()
    });
}

impl Renderer
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Initialize(graphicsContext: RustyRef<GraphicsContext>)
    {
        s_Instance.try_with(|renderer|
        {
            // Set the graphics context
            renderer.GetRefMut().m_GfxContext = graphicsContext;

            // Load all shaders
            let shadersFolder = "rusty_engine/assets/shaders";
            let computeShadersFolder = "rusty_engine/assets/compute_shaders";

            // Regular shaders
            for shaderEntry in std::fs::read_dir(shadersFolder).unwrap()
            {
                if shaderEntry.is_ok()
                {
                    let shaderPath = shaderEntry.unwrap().path();
                    ShaderLibrary::LoadShader(shaderPath.to_str().unwrap());
                }
            }

            // Compute shaders
            for shaderEntry in std::fs::read_dir(computeShadersFolder).unwrap()
            {
                if shaderEntry.is_ok()
                {
                    let shaderPath = shaderEntry.unwrap().path();
                    ShaderLibrary::LoadComputeShader(shaderPath.to_str().unwrap());
                }
            }

            // Initialize fullscreen quad data
            let quadVertices: [f32; 20] = [
                -1.0, -1.0, 0.0, 0.0, 1.0,
                1.0, -1.0, 0.0, 1.0, 1.0,
                1.0, 1.0, 0.0, 1.0, 0.0,
                -1.0, 1.0, 0.0, 0.0, 0.0
            ];

            let vbDesc = VertexBufferDescription {
                VertexCount: 4,
                Stride: 20,
                Size: 80,
                Usage: D3D11_USAGE_DEFAULT
            };

            let quadVB = VertexBuffer::Create(quadVertices.as_ptr() as _, &vbDesc);

            let quadIndices: [u32; 6] = [0, 1, 2, 2, 3, 0];

            let ibDesc = IndexBufferDescription {
                IndexCount: 6,
                Size: (6 * size_of::<u32>()) as u32,
                Format: DXGI_FORMAT_R32_UINT,
                Usage: D3D11_USAGE_DEFAULT
            };
    
            let quadIB = IndexBuffer::Create(quadIndices.as_ptr() as _, &ibDesc);

            // Create 1x1 black texture
            let blackTextureDesc = TextureDescription {
                Name: String::from("BlackTexture"),
                Width: 0,
                Height: 0,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                BindFlags: D3D11_BIND_SHADER_RESOURCE,
                MipCount: 1,
                ImageData: vec![Some(Image::Create(1, 1, false, [0, 0, 0, 255].as_ptr()))]
            };

            let blackTextureSamplerDesc = SamplerDescription {
                Wrap: D3D11_TEXTURE_ADDRESS_WRAP,
                Filter: D3D11_FILTER_ANISOTROPIC
            };

            // Create 1x1 black texture cubemap
            let blackTextureCubeDesc = TextureDescription {
                Name: String::from("BlackTextureCube"),
                Width: 0,
                Height: 0,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                BindFlags: D3D11_BIND_SHADER_RESOURCE,
                MipCount: 1,
                ImageData: vec![Some(Image::Create(1, 1, false, [0, 0, 0, 255].as_ptr())),
                                Some(Image::Create(1, 1, false, [0, 0, 0, 255].as_ptr())),
                                Some(Image::Create(1, 1, false, [0, 0, 0, 255].as_ptr())),
                                Some(Image::Create(1, 1, false, [0, 0, 0, 255].as_ptr())),
                                Some(Image::Create(1, 1, false, [0, 0, 0, 255].as_ptr())),
                                Some(Image::Create(1, 1, false, [0, 0, 0, 255].as_ptr()))]
            };

            // Pre-compute BRDF Texture
            let brdfMapDesc = TextureDescription {
                Name: String::from("BRDFTexture"),
                Width: 256,
                Height: 256,
                Format: DXGI_FORMAT_R16G16_FLOAT,
                BindFlags: D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
                MipCount: 1,
                ImageData: vec![]
            };

            let brdfSamplerDesc = SamplerDescription {
                Wrap: D3D11_TEXTURE_ADDRESS_CLAMP,
                Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR
            };

            let brdfMap:    RustyRef<Texture> = Texture::CreateTexture2D(&brdfMapDesc, &brdfSamplerDesc);
            let brdfShader: RustyRef<ComputeShader> = ShaderLibrary::GetComputeShader("environment_brdf");

            let commandBuffer = CommandBuffer::Create();
            commandBuffer.GetRef().SetCSUnorderedAccessViews(&brdfMap.GetRef().CreateUAV(0, 0), 1, 0);
            commandBuffer.GetRef().DispatchComputeShader(brdfShader.clone(), brdfMapDesc.Width / 32, brdfMapDesc.Height / 32, 1);
            commandBuffer.GetRef().SetCSUnorderedAccessViews(vec![None; 1].as_ptr(), 1, 0);
            commandBuffer.GetRefMut().Finish();
            renderer.GetRefMut().m_GfxContext.GetRefMut().ExecuteCommandBuffer(commandBuffer);

            renderer.GetRefMut().m_RendererData = RustyRef::CreateRef(RendererData{
                FullscreenQuadVB: quadVB,
                FullscreenQuadIB: quadIB,
                BRDFTexture: brdfMap,
                BlackTexture: Texture::CreateTexture2D(&blackTextureDesc, &blackTextureSamplerDesc),
                BlackTextureCube: Texture::CreateTextureCube(&blackTextureCubeDesc, &blackTextureSamplerDesc)
            });

        }).expect("Failed to initialize the renderer!");
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateEnvironmentMap(commandBuffer: RustyRef<CommandBuffer>, filepath: &str) -> (RustyRef<Texture>, RustyRef<Texture>)
    {
        let commandBuffer: Ref<CommandBuffer> = commandBuffer.GetRef();

        // Load the equirectangular environment map
        let filename: &str = std::path::Path::new(filepath).file_stem().unwrap().to_str().unwrap();

        let hdrTextureDesc = TextureDescription {
            Name: String::from(filename),
            Width: 0,
            Height: 0,
            Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            MipCount: 1,
            ImageData: vec![Some(Image::LoadFromFile(filepath, false))]
        };

        let hdrSamplerDesc = SamplerDescription {
            Wrap: D3D11_TEXTURE_ADDRESS_WRAP,
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR
        };

        let hdrTexture: RustyRef<Texture> = Texture::CreateTexture2D(&hdrTextureDesc, &hdrSamplerDesc);
        
        let nullUAVs: Vec<Option<ID3D11UnorderedAccessView>> = vec![None; 1];

        /////////////////////////////////////////////////////////////////////////////////////////////////
        //                        Phase 1: Equirectangular map to cubemap                              //
        /////////////////////////////////////////////////////////////////////////////////////////////////
        let envMapUnfilteredDesc = TextureDescription {
            Name: String::new(),
            Width: 1024,
            Height: 1024,
            Format: DXGI_FORMAT_R16G16B16A16_FLOAT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
            MipCount: 11,
            ImageData: vec![]
        };

        let computeSamplerDesc = SamplerDescription {
            Wrap: D3D11_TEXTURE_ADDRESS_WRAP,
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR
        };

        let envMapUnfiltered: RustyRef<Texture> = Texture::CreateTextureCube(&envMapUnfilteredDesc, &computeSamplerDesc);
        let equirectToCubeShader: RustyRef<ComputeShader> = ShaderLibrary::GetComputeShader("environment_equirect_to_cube");

        commandBuffer.SetCSShaderResources(&hdrTexture.GetRef().CreateSRV(), 1, 0);
        commandBuffer.SetCSSamplers(hdrTexture.GetRef().GetSampler(), 1, 0);
        commandBuffer.SetCSUnorderedAccessViews(&envMapUnfiltered.GetRef().CreateUAV(-1, 0), 1, 0);
        commandBuffer.DispatchComputeShader(equirectToCubeShader.clone(), envMapUnfilteredDesc.Width / 32, envMapUnfilteredDesc.Height / 32, 6);
        commandBuffer.SetCSUnorderedAccessViews(nullUAVs.as_ptr(), 1, 0);

        let envMapUnfilteredSRV: Option<ID3D11ShaderResourceView> = envMapUnfiltered.GetRef().CreateSRV();

        /////////////////////////////////////////////////////////////////////////////////////////////////
        //                                    Phase 2: Pre-filtering                                   //
        /////////////////////////////////////////////////////////////////////////////////////////////////
        let envMapDesc = TextureDescription {
            Name: String::from(filename),
            Width: 1024,
            Height: 1024,
            Format: DXGI_FORMAT_R16G16B16A16_FLOAT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
            MipCount: 11,
            ImageData: vec![]
        };

        let environmentMap: RustyRef<Texture> = Texture::CreateTextureCube(&envMapDesc, &computeSamplerDesc);
        let prefilterShader: RustyRef<ComputeShader> = ShaderLibrary::GetComputeShader("environment_prefilter");

        // Copy the data from the first mip level of the unfiltered texture
        for slice in 0..6
        {
            let subresourceIndex: u32 = slice * envMapDesc.MipCount;
            commandBuffer.CopyTexture(envMapUnfiltered.GetRef().GetHandle(), environmentMap.GetRef().GetHandle(), subresourceIndex);
        }

        commandBuffer.SetCSShaderResources(&envMapUnfilteredSRV, 1, 0);
        commandBuffer.SetCSSamplers(envMapUnfiltered.GetRef().GetSampler(), 1, 0);

        // Pre-filter the mip chain
        let deltaRoughness: f32 = 1.0 / std::cmp::max(envMapDesc.MipCount - 1, 1) as f32;
        let mut size: u32 = envMapDesc.Width / 2;

        for mipLevel in 1..envMapDesc.MipCount
        {
            let numThreadGroups: u32 = std::cmp::max(1, size / 32);
            
            commandBuffer.SetCSConstantBufferData(prefilterShader.clone(), &((mipLevel as f32) * deltaRoughness), 0);
            commandBuffer.SetCSUnorderedAccessViews(&environmentMap.GetRef().CreateUAV(-1, mipLevel), 1, 0);
            commandBuffer.DispatchComputeShader(prefilterShader.clone(), numThreadGroups, numThreadGroups, 6);
            size /= 2;
        }

        commandBuffer.SetCSUnorderedAccessViews(nullUAVs.as_ptr(), 1, 0);

        /////////////////////////////////////////////////////////////////////////////////////////////////
        //                                    Phase 3: Irradiance map                                  //
        /////////////////////////////////////////////////////////////////////////////////////////////////
        let irradianceMapDesc = TextureDescription {
            Name: String::from(filename),
            Width: 32,
            Height: 32,
            Format: DXGI_FORMAT_R16G16B16A16_FLOAT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS,
            MipCount: 1,
            ImageData: vec![]
        };

        let irradianceMap: RustyRef<Texture> = Texture::CreateTextureCube(&irradianceMapDesc, &computeSamplerDesc);
        let irradianceShader: RustyRef<ComputeShader> = ShaderLibrary::GetComputeShader("environment_irradiance");

        commandBuffer.SetCSShaderResources(&environmentMap.GetRef().CreateSRV(), 1, 0);
        commandBuffer.SetCSSamplers(environmentMap.GetRef().GetSampler(), 1, 0);
        commandBuffer.SetCSUnorderedAccessViews(&irradianceMap.GetRef().CreateUAV(-1, 0), 1, 0);
        commandBuffer.DispatchComputeShader(irradianceShader.clone(), irradianceMapDesc.Width / 32, irradianceMapDesc.Height / 32, 6);
        commandBuffer.SetCSUnorderedAccessViews(nullUAVs.as_ptr(), 1, 0);

        return (environmentMap, irradianceMap);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn BeginRenderPass(commandBuffer: RustyRef<CommandBuffer>, renderTarget: RustyRef<RenderTarget>, clear: bool)
    {
        let commandBuffer: Ref<CommandBuffer> = commandBuffer.GetRef();
        let renderTarget: Ref<RenderTarget> = renderTarget.GetRef();

        let renderTargetViews: &Vec<Option<ID3D11RenderTargetView>> = renderTarget.GetRTVs();
        let depthStencilView: &Option<ID3D11DepthStencilView> = renderTarget.GetDSV();
        let viewport: &D3D11_VIEWPORT = renderTarget.GetViewport();
        let clearColor: &XMFLOAT4 = renderTarget.GetClearColor();
        
        if clear
        {
            for renderTarget in renderTargetViews.iter()
            {
                commandBuffer.ClearRenderTarget(renderTarget, &clearColor);
            }
        
            if depthStencilView.is_some()
            {
                commandBuffer.ClearDepthStencilBuffer(depthStencilView, 1.0, 0);
            }
        }

        commandBuffer.SetRenderTargets(renderTargetViews.as_ptr(), renderTargetViews.len() as u32, depthStencilView);
        commandBuffer.SetViewports(viewport, 1);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn BeginSwapChainRenderPass(commandBuffer: RustyRef<CommandBuffer>, clear: bool)
    {
        let gfxContext: RustyRef<GraphicsContext> = Renderer::GetGfxContext();
        let gfxContext: Ref<GraphicsContext> = gfxContext.GetRef();
        let commandBuffer: Ref<CommandBuffer> = commandBuffer.GetRef();
        let swapChainRenderTargetViews: &Vec<Option<ID3D11RenderTargetView>> = gfxContext.GetSwapChainRenderTargets();
        let swapChainDepthStencilView: &Option<ID3D11DepthStencilView> = gfxContext.GetSwapChainDepthStencil();
        let swapChainViewport: &D3D11_VIEWPORT = gfxContext.GetSwapChainViewport();

        if clear
        {
            for renderTarget in swapChainRenderTargetViews.iter()
            {
                commandBuffer.ClearRenderTarget(renderTarget, &XMFLOAT4::set(0.3, 0.3, 0.3, 1.0));
            }
        
            if swapChainDepthStencilView.is_some()
            {
                commandBuffer.ClearDepthStencilBuffer(swapChainDepthStencilView, 1.0, 0);
            }
        }
    
        commandBuffer.SetRenderTargets(swapChainRenderTargetViews.as_ptr(), swapChainRenderTargetViews.len() as u32, swapChainDepthStencilView);
        commandBuffer.SetViewports(swapChainViewport, 1);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn RenderMesh<T>(commandBuffer: RustyRef<CommandBuffer>, pipeline: RustyRef<GraphicsPipeline>, mesh: RustyRef<Mesh>, submeshIndex: u32, material: RustyRef<Material>, uniformBuffer: RustyRef<T>)
    {
        let commandBuffer: Ref<CommandBuffer> = commandBuffer.GetRef();
        let material: Ref<Material> = material.GetRef();
        let mesh: Ref<Mesh> = mesh.GetRef();
        
        // Set renderer uniform buffer
        if uniformBuffer.IsValid()
        {
            commandBuffer.SetVSConstantBufferData(pipeline.GetRefMut().GetShader(), &*uniformBuffer.GetRef(), 0);
        }

        // Set material data
        if !material.GetVSUniformBuffer().is_empty()
        {
            commandBuffer.SetVSMaterialConstantBufferData(material.GetShader().clone(), material.GetVSUniformBuffer());
        }
        if !material.GetPSUniformBuffer().is_empty()
        {  
            commandBuffer.SetPSMaterialConstantBufferData(material.GetShader().clone(), material.GetPSUniformBuffer());
        }

        let materialTextures = material.GetTextures();
        let mut textureSRVs: Vec<Option<ID3D11ShaderResourceView>> = Vec::with_capacity(materialTextures.len());
        let mut textureSamplers: Vec<Option<ID3D11SamplerState>> = Vec::with_capacity(materialTextures.len());
        for i in 0..materialTextures.len()
        {
            textureSRVs.push(materialTextures[i].GetRef().CreateSRV());
            textureSamplers.push(materialTextures[i].GetRef().GetSampler().clone());
        }
        commandBuffer.SetPSShaderResources(textureSRVs.as_ptr(), textureSRVs.len() as u32, 0);
        commandBuffer.SetPSSamplers(textureSamplers.as_ptr(), textureSamplers.len() as u32, 0);

        // Set pipeline state
        pipeline.GetRefMut().SetRasterizerState(material.GetRasterizerState());
        pipeline.GetRefMut().SetBlendState(material.GetBlendState());
        pipeline.GetRefMut().SetDepthStencilState(material.GetDepthState());
        commandBuffer.SetGraphicsPipelineState(pipeline.clone());

        // Set vertex/index buffers and render
        commandBuffer.SetVertexBuffer(mesh.GetVertexBuffer());
        commandBuffer.SetIndexBuffer(mesh.GetIndexBuffer());

        let submesh: &Submesh = &mesh.GetSubmeshes()[submeshIndex as usize];
        commandBuffer.DrawIndexed(submesh.IndexCount, submesh.IndexOffset, submesh.VertexOffset);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn RenderFullscreenQuad(commandBuffer: RustyRef<CommandBuffer>, pipeline: RustyRef<GraphicsPipeline>, material: RustyRef<Material>)
    {
        s_Instance.try_with(|renderer|
        {
            let renderer: Ref<Renderer> = renderer.GetRef();
            let material: Ref<Material> = material.GetRef();
            let commandBuffer: Ref<CommandBuffer> = commandBuffer.GetRef();
            
            // Set material data
            if !material.GetVSUniformBuffer().is_empty()
            {
                commandBuffer.SetVSMaterialConstantBufferData(material.GetShader().clone(), material.GetVSUniformBuffer());
            }
            if !material.GetPSUniformBuffer().is_empty()
            {  
                commandBuffer.SetPSMaterialConstantBufferData(material.GetShader().clone(), material.GetPSUniformBuffer());
            }
        
            let materialTextures = material.GetTextures();
            let mut textureSRVs: Vec<Option<ID3D11ShaderResourceView>> = Vec::with_capacity(materialTextures.len());
            let mut textureSamplers: Vec<Option<ID3D11SamplerState>> = Vec::with_capacity(materialTextures.len());
            for i in 0..materialTextures.len()
            {
                textureSRVs.push(materialTextures[i].GetRef().CreateSRV());
                textureSamplers.push(materialTextures[i].GetRef().GetSampler().clone());
            }
            commandBuffer.SetPSShaderResources(textureSRVs.as_ptr(), textureSRVs.len() as u32, 0);
            commandBuffer.SetPSSamplers(textureSamplers.as_ptr(), textureSamplers.len() as u32, 0);
        
            // Set pipeline state
            pipeline.GetRefMut().SetRasterizerState(material.GetRasterizerState());
            pipeline.GetRefMut().SetBlendState(material.GetBlendState());
            pipeline.GetRefMut().SetDepthStencilState(material.GetDepthState());
            commandBuffer.SetGraphicsPipelineState(pipeline.clone());
        
            // Set vertex/index buffers and render
            commandBuffer.SetVertexBuffer(&renderer.m_RendererData.GetRef().FullscreenQuadVB);
            commandBuffer.SetIndexBuffer(&renderer.m_RendererData.GetRef().FullscreenQuadIB);
            commandBuffer.DrawIndexed(renderer.m_RendererData.GetRef().FullscreenQuadIB.GetIndexCount(), 0, 0);

        }).expect("Rendering fullscreen quad failed!");
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetGfxContext() -> RustyRef<GraphicsContext>
    {
        return s_Instance.try_with(|renderer|
        {
            return renderer.GetRef().m_GfxContext.clone();

        }).expect("Failed retrieving graphics context!");
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetBRDF() -> RustyRef<Texture>
    {
        return s_Instance.try_with(|renderer|
        {
            return renderer.GetRef().m_RendererData.GetRef().BRDFTexture.clone();

        }).expect("Failed retrieving BRDF texture!");
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetBlackTexture() -> RustyRef<Texture>
    {
        return s_Instance.try_with(|renderer|
        {
            return renderer.GetRef().m_RendererData.GetRef().BlackTexture.clone();

        }).expect("Failed retrieving black texture!");
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetBlackTextureCube() -> RustyRef<Texture>
    {
        return s_Instance.try_with(|renderer|
        {
            return renderer.GetRef().m_RendererData.GetRef().BlackTextureCube.clone();

        }).expect("Failed retrieving black texture cube!");
    }
}