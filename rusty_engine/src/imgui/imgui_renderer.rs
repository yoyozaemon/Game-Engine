#![allow(non_snake_case)]

// std
use std::cell::RefMut;
use std::mem::size_of;
use std::ptr::null;
use std::alloc::alloc;
use std::alloc::dealloc;
use std::alloc::Layout;

// Win32
use imgui::FontAtlasTexture;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;

// core
use crate::core::utils::*;

// renderer
use crate::renderer::vertex_buffer::*;
use crate::renderer::index_buffer::*;
use crate::renderer::graphics_pipeline::*;
use crate::renderer::shader_library::*;
use crate::renderer::command_buffer::*;
use crate::renderer::texture::*;
use crate::renderer::renderer::*;


const FONT_TEX_ID: usize = !0;

const VERTEX_BUF_ADD_CAPACITY: u32 = 5000;
const INDEX_BUF_ADD_CAPACITY: u32 = 10000;

pub struct ImGuiRenderer
{
    m_VertexBuffer:      Option<VertexBuffer>,
    m_IndexBuffer:       Option<IndexBuffer>,
    m_Pipeline:          RustyRef<GraphicsPipeline>,
    m_CommandBuffer:     RustyRef<CommandBuffer>,
    m_ImGuiTextures:     imgui::Textures<Option<ID3D11ShaderResourceView>>,
    m_FontTexture:       RustyRef<Texture>
}

impl ImGuiRenderer
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create() -> ImGuiRenderer
    {
        return ImGuiRenderer {
            m_VertexBuffer: None,
            m_IndexBuffer: None,
            m_Pipeline: RustyRef::CreateEmpty(),
            m_CommandBuffer: RustyRef::CreateEmpty(),
            m_FontTexture: RustyRef::CreateEmpty(),
            m_ImGuiTextures: imgui::Textures::new()
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Initialize(&mut self, imguiContext: &mut imgui::Context)
    {
        imguiContext.set_renderer_name(Some(imgui::im_str!("ImGuiRenderer_DirectX11 {}", env!("CARGO_PKG_VERSION"))));
        imguiContext.io_mut().backend_flags |= imgui::BackendFlags::RENDERER_HAS_VTX_OFFSET;

        // Create vertex buffer
        let vbDesc = VertexBufferDescription {
            VertexCount: VERTEX_BUF_ADD_CAPACITY,
            Stride: size_of::<imgui::DrawVert>() as u32,
            Size: VERTEX_BUF_ADD_CAPACITY * size_of::<imgui::DrawVert>() as u32,
            Usage: D3D11_USAGE_DYNAMIC
        };

        self.m_VertexBuffer = Some(VertexBuffer::Create(null(), &vbDesc));

        // Create index buffer
        let ibDesc = IndexBufferDescription {
            IndexCount: INDEX_BUF_ADD_CAPACITY,
            Size: INDEX_BUF_ADD_CAPACITY * size_of::<imgui::DrawIdx>() as u32,
            Format: DXGI_FORMAT_R16_UINT,
            Usage: D3D11_USAGE_DYNAMIC
        };

        self.m_IndexBuffer = Some(IndexBuffer::Create(null(), &ibDesc));

        // Create rasterizer state
        let mut rasterizerState = D3D11_RASTERIZER_DESC::default();
        rasterizerState.FillMode        = D3D11_FILL_SOLID;
        rasterizerState.CullMode        = D3D11_CULL_NONE;
        rasterizerState.ScissorEnable   = BOOL::from(true);
        rasterizerState.DepthClipEnable = BOOL::from(true);

        // Create blend state
        let mut blendState = D3D11_BLEND_DESC::default();
        blendState.AlphaToCoverageEnable                 = BOOL::from(false);
        blendState.RenderTarget[0].BlendEnable           = BOOL::from(true);
        blendState.RenderTarget[0].SrcBlend              = D3D11_BLEND_SRC_ALPHA;
        blendState.RenderTarget[0].DestBlend             = D3D11_BLEND_INV_SRC_ALPHA;
        blendState.RenderTarget[0].BlendOp               = D3D11_BLEND_OP_ADD;
        blendState.RenderTarget[0].SrcBlendAlpha         = D3D11_BLEND_INV_SRC_ALPHA;
        blendState.RenderTarget[0].DestBlendAlpha        = D3D11_BLEND_ZERO;
        blendState.RenderTarget[0].BlendOpAlpha          = D3D11_BLEND_OP_ADD;
        blendState.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8;

        // Create depth stencil state
        let mut depthStencilState = D3D11_DEPTH_STENCIL_DESC::default();
        depthStencilState.DepthEnable                  = BOOL::from(false);
        depthStencilState.DepthWriteMask               = D3D11_DEPTH_WRITE_MASK_ALL;
        depthStencilState.DepthFunc                    = D3D11_COMPARISON_ALWAYS;
        depthStencilState.StencilEnable                = BOOL::from(false);
        depthStencilState.FrontFace.StencilFailOp      = D3D11_STENCIL_OP_KEEP;
        depthStencilState.FrontFace.StencilDepthFailOp = D3D11_STENCIL_OP_KEEP;
        depthStencilState.FrontFace.StencilPassOp      = D3D11_STENCIL_OP_KEEP;
        depthStencilState.FrontFace.StencilFunc        = D3D11_COMPARISON_ALWAYS;
        depthStencilState.BackFace                     = depthStencilState.FrontFace;

        // Create pipeline
        let pipelineDesc = GraphicsPipelineDescription {
            RasterizerStateDesc:   rasterizerState,
            BlendStateDesc:        blendState,
            DepthStencilStateDesc: depthStencilState,
            Topology:              D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            InputLayoutDesc: vec![
                D3D11_INPUT_ELEMENT_DESC { 
                    SemanticName: PSTR("POSITION\0".as_ptr() as _), 
                    SemanticIndex: 0, 
                    Format: DXGI_FORMAT_R32G32_FLOAT, 
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
                    AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0
                },

                D3D11_INPUT_ELEMENT_DESC { 
                    SemanticName: PSTR("COLOR\0".as_ptr() as _), 
                    SemanticIndex: 0, 
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM, 
                    InputSlot: 0,
                    AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0
                }
            ],
            Shader: ShaderLibrary::GetShader("imgui_shader"),
            RenderTarget: RustyRef::CreateEmpty()
        };

        self.m_Pipeline = GraphicsPipeline::Create(pipelineDesc);

        // Create command buffer
        self.m_CommandBuffer = CommandBuffer::Create();

        // Create font texture
        let mut fontAtlas: imgui::FontAtlasRefMut = imguiContext.fonts();
        let fontTextureInfo: FontAtlasTexture = fontAtlas.build_rgba32_texture();

        let fontTextureDesc = TextureDescription {
            Name: String::from("ImGuiFontTexture"),
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            Width: fontTextureInfo.width,
            Height: fontTextureInfo.height,
            MipCount: 1,
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            ImageData: vec![Some(Image::Create(fontTextureInfo.width, fontTextureInfo.height, false, fontTextureInfo.data.as_ptr()))]
        };

        let samplerDesc = SamplerDescription {
            Wrap: D3D11_TEXTURE_ADDRESS_WRAP,
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR
        };
        
        self.m_FontTexture = Texture::CreateTexture2D(&fontTextureDesc, &samplerDesc);
        fontAtlas.tex_id = imgui::TextureId::from(FONT_TEX_ID);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn RenderDrawData(&mut self, drawData: &imgui::DrawData)
    {
        Renderer::BeginSwapChainRenderPass(self.m_CommandBuffer.clone(), false);
        let mut commandBuffer: RefMut<CommandBuffer> = self.m_CommandBuffer.GetRefMut();

        if drawData.display_size[0] > 0.0 && drawData.display_size[1] > 0.0
        {
            // Recreate buffers if needed
            if self.m_VertexBuffer.as_ref().unwrap().GetVertexCount() < drawData.total_vtx_count as u32
            {
                let vbDesc = VertexBufferDescription {
                    VertexCount: drawData.total_vtx_count as u32 + VERTEX_BUF_ADD_CAPACITY,
                    Stride: size_of::<imgui::DrawVert>() as u32,
                    Size: (drawData.total_vtx_count as u32 + VERTEX_BUF_ADD_CAPACITY) * size_of::<imgui::DrawVert>() as u32,
                    Usage: D3D11_USAGE_DYNAMIC
                };
            
                self.m_VertexBuffer = Some(VertexBuffer::Create(null(), &vbDesc));
            }

            if self.m_IndexBuffer.as_ref().unwrap().GetIndexCount() < drawData.total_idx_count as u32
            {
                let ibDesc = IndexBufferDescription {
                    IndexCount: drawData.total_idx_count as u32 + INDEX_BUF_ADD_CAPACITY,
                    Size: (drawData.total_idx_count as u32 + INDEX_BUF_ADD_CAPACITY) * size_of::<imgui::DrawIdx>() as u32,
                    Format: DXGI_FORMAT_R16_UINT,
                    Usage: D3D11_USAGE_DYNAMIC
                };
            
                self.m_IndexBuffer = Some(IndexBuffer::Create(null(), &ibDesc));
            }

            // Upload the vertex and index data
            unsafe 
            {
                let vertexData: *mut imgui::DrawVert = alloc(Layout::array::<imgui::DrawVert>(drawData.total_vtx_count as usize).unwrap()) as _;
                let mut vertexDataOffset = 0;

                let indexData: *mut imgui::DrawIdx = alloc(Layout::array::<imgui::DrawIdx>(drawData.total_idx_count as usize).unwrap()) as _;
                let mut indexDataOffset = 0;

                for drawList in drawData.draw_lists()
                {
                    std::ptr::copy_nonoverlapping(drawList.vtx_buffer().as_ptr(), vertexData.add(vertexDataOffset), drawList.vtx_buffer().len());
                    std::ptr::copy_nonoverlapping(drawList.idx_buffer().as_ptr(), indexData.add(indexDataOffset), drawList.idx_buffer().len());
                    vertexDataOffset += drawList.vtx_buffer().len();
                    indexDataOffset += drawList.idx_buffer().len();
                }

                commandBuffer.SetVertexBufferData(vertexData as _, drawData.total_vtx_count as usize * size_of::<imgui::DrawVert>(), self.m_VertexBuffer.as_ref().unwrap());
                commandBuffer.SetVertexBuffer(self.m_VertexBuffer.as_ref().unwrap());

                commandBuffer.SetIndexBufferData(indexData as _, drawData.total_idx_count as usize * size_of::<imgui::DrawIdx>(), self.m_IndexBuffer.as_ref().unwrap());
                commandBuffer.SetIndexBuffer(self.m_IndexBuffer.as_ref().unwrap());

                dealloc(vertexData as _, Layout::array::<imgui::DrawVert>(drawData.total_vtx_count as usize).unwrap());
                dealloc(indexData as _, Layout::array::<imgui::DrawIdx>(drawData.total_idx_count as usize).unwrap());
            }

            // Upload the constant buffer data
            let l = drawData.display_pos[0];
            let r = drawData.display_pos[0] + drawData.display_size[0];
            let t = drawData.display_pos[1];
            let b = drawData.display_pos[1] + drawData.display_size[1];
            let mvp = [
                [2.0 / (r - l), 0.0, 0.0, 0.0],
                [0.0, 2.0 / (t - b), 0.0, 0.0],
                [0.0, 0.0, 0.5, 0.0],
                [(r + l) / (l - r), (t + b) / (b - t), 0.5, 1.0],
            ];

            commandBuffer.SetVSConstantBufferData(self.m_Pipeline.GetRef().GetShader(), &mvp, 0);

            // Set the pipeline state
            commandBuffer.SetGraphicsPipelineState(self.m_Pipeline.clone());

            // Set the sampler state
            commandBuffer.SetPSSamplers(self.m_FontTexture.GetRef().GetSampler(), 1, 0);

            // Render
            let clipOff: [f32; 2] = drawData.display_pos;
            let mut vertexOffset: usize = 0;
            let mut indexOffset: usize = 0;

            for drawList in drawData.draw_lists()
            {
                for drawCmd in drawList.commands()
                {
                    match drawCmd
                    {
                        imgui::DrawCmd::Elements { 
                            count, 
                            cmd_params: imgui::DrawCmdParams { clip_rect, texture_id, vtx_offset, idx_offset }
                        } => {
                            if texture_id.id() == FONT_TEX_ID 
                            {
                                commandBuffer.SetPSShaderResources(&self.m_FontTexture.GetRef().CreateSRV(), 1, 0);
                            }
                            else 
                            {
                                let srv: &Option<ID3D11ShaderResourceView> = self.m_ImGuiTextures.get(texture_id).ok_or(DXGI_ERROR_INVALID_CALL).unwrap();
                                commandBuffer.SetPSShaderResources(srv, 1, 0);
                            };
                            
                            let mut rect = RECT::default();
                            rect.left   = (clip_rect[0] - clipOff[0]) as i32;
                            rect.top    = (clip_rect[1] - clipOff[1]) as i32;
                            rect.right  = (clip_rect[2] - clipOff[0]) as i32;
                            rect.bottom = (clip_rect[3] - clipOff[1]) as i32;

                            commandBuffer.SetScissorRect(&rect);
                            commandBuffer.DrawIndexed(count as u32, (idx_offset + indexOffset) as u32, (vtx_offset + vertexOffset) as u32);
                        }
                        _ => {}
                    }
                }
                vertexOffset += drawList.vtx_buffer().len();
                indexOffset += drawList.idx_buffer().len();
            }
        }

        commandBuffer.Finish();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Flush(&mut self)
    {
        // Execute all commands and present the frame
        Renderer::GetGfxContext().GetRef().ExecuteCommandBuffer(self.m_CommandBuffer.clone());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetCommandBuffer(&mut self) -> RustyRef<CommandBuffer>
    {
        return self.m_CommandBuffer.clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetTextures(&mut self) -> &mut imgui::Textures<Option<ID3D11ShaderResourceView>>
    {
        return &mut self.m_ImGuiTextures;
    }
}