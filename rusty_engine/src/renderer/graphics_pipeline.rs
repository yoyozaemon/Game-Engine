#![allow(non_snake_case)]

// Win32
use windows::Win32::Graphics::Direct3D11::*;

// Core
use crate::core::RustyRef;

// Renderer
use crate::renderer::shader::*;
use crate::renderer::render_target::*;
use crate::renderer::renderer::*;

pub struct GraphicsPipelineDescription
{
    pub RasterizerStateDesc:   D3D11_RASTERIZER_DESC,
    pub BlendStateDesc:        D3D11_BLEND_DESC,
    pub DepthStencilStateDesc: D3D11_DEPTH_STENCIL_DESC,
    pub Shader:                RustyRef<Shader>,
    pub InputLayoutDesc:       Vec<D3D11_INPUT_ELEMENT_DESC>,
    pub Topology:              D3D_PRIMITIVE_TOPOLOGY,
    pub RenderTarget:          RustyRef<RenderTarget>
}

pub struct GraphicsPipeline
{
    m_Device:            ID3D11Device,
    m_RaserizerState:    ID3D11RasterizerState,
    m_BlendState:        ID3D11BlendState,
    m_DepthStencilState: ID3D11DepthStencilState,
    m_Shader:            RustyRef<Shader>,
    m_InputLayout:       ID3D11InputLayout,
    m_Topology:          D3D_PRIMITIVE_TOPOLOGY,
    m_RenderTarget:      RustyRef<RenderTarget>
}

impl GraphicsPipeline
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(description: GraphicsPipelineDescription) -> RustyRef<GraphicsPipeline>
    {
        unsafe
        {
            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();

            // Create states
            let rasterizerState: ID3D11RasterizerState = DXCall!(device.CreateRasterizerState(&description.RasterizerStateDesc));
            let blendState: ID3D11BlendState = DXCall!(device.CreateBlendState(&description.BlendStateDesc));
            let depthStencilState: ID3D11DepthStencilState = DXCall!(device.CreateDepthStencilState(&description.DepthStencilStateDesc));

            
            // Create input layout
            let shader = description.Shader.GetRef();
            let vsData: &ID3DBlob = shader.GetVSDataBlob();
            let layoutDesc: Vec<D3D11_INPUT_ELEMENT_DESC> = description.InputLayoutDesc;
            let layout: ID3D11InputLayout = DXCall!(device.CreateInputLayout(layoutDesc.as_ptr(), layoutDesc.len() as u32, vsData.GetBufferPointer(), vsData.GetBufferSize()));

            drop(shader);

            return RustyRef::CreateRef(GraphicsPipeline {
                m_Device: device.clone(),
                m_RaserizerState: rasterizerState,
                m_BlendState: blendState,
                m_DepthStencilState: depthStencilState,
                m_Shader: description.Shader,
                m_InputLayout: layout,
                m_Topology: description.Topology,
                m_RenderTarget: description.RenderTarget
            });
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetRasterizerState(&mut self, desc: &D3D11_RASTERIZER_DESC)
    {
        unsafe { self.m_RaserizerState = DXCall!(self.m_Device.CreateRasterizerState(desc)); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetBlendState(&mut self, desc: &D3D11_BLEND_DESC)
    {
        unsafe { self.m_BlendState = DXCall!(self.m_Device.CreateBlendState(desc)); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetDepthStencilState(&mut self, desc: &D3D11_DEPTH_STENCIL_DESC)
    {
        unsafe { self.m_DepthStencilState = DXCall!(self.m_Device.CreateDepthStencilState(desc)); }
    }
    
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetShader(&self) -> RustyRef<Shader>
    {
        return self.m_Shader.clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRasterizerState(&self) -> &ID3D11RasterizerState
    {
        return &self.m_RaserizerState;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetBlendState(&self) -> &ID3D11BlendState
    {
        return &self.m_BlendState;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetDepthStencilState(&self) -> &ID3D11DepthStencilState
    {
        return &self.m_DepthStencilState;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetInputLayout(&self) -> &ID3D11InputLayout
    {
        return &self.m_InputLayout;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetPrimitiveTopology(&self) -> D3D_PRIMITIVE_TOPOLOGY
    {
        return self.m_Topology;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRenderTarget(&self) -> RustyRef<RenderTarget>
    {
        return self.m_RenderTarget.clone();
    }
}