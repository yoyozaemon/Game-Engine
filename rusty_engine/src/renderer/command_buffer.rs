#![allow(non_snake_case)]

// std
use std::cell::Ref;
use std::ptr::null;

// Win32
use directx_math::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D11::*;

// Core
use crate::core::utils::*;

// Renderer
use crate::renderer::shader::*;
use crate::renderer::graphics_pipeline::*;
use crate::renderer::vertex_buffer::*;
use crate::renderer::index_buffer::*;
use crate::renderer::renderer::*;

pub struct CommandBuffer
{
    m_DeferredContext: ID3D11DeviceContext,
    m_CommandList:     Option<ID3D11CommandList>,
}

impl CommandBuffer
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create() -> RustyRef<CommandBuffer>
    {
        unsafe
        {
            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();
            let deferredContext: ID3D11DeviceContext = DXCall!(device.CreateDeferredContext(0));

            return RustyRef::CreateRef(CommandBuffer {
                m_DeferredContext: deferredContext,
                m_CommandList: None
            });
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Reset(&mut self)
    {
        self.m_CommandList = None;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Finish(&mut self)
    {
        unsafe { self.m_CommandList = Some(DXCall!(self.m_DeferredContext.FinishCommandList(false))); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn DispatchComputeShader(&self, shader: RustyRef<ComputeShader>, threadGroupCountX: u32, threadGroupCountY: u32, threadGroupCountZ: u32)
    {
        unsafe 
        { 
            self.m_DeferredContext.CSSetShader(shader.GetRef().GetHandle(), null(), 0);
            self.m_DeferredContext.Dispatch(threadGroupCountX, threadGroupCountY, threadGroupCountZ);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetVertexBuffer(&self, vertexBuffer: &VertexBuffer)
    {
        unsafe
        {
            let offset: u32 = 0;
            let stride: u32 = vertexBuffer.GetStride();
            self.m_DeferredContext.IASetVertexBuffers(0, 1, vertexBuffer.GetHandle(), &stride, &offset);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetIndexBuffer(&self, indexBuffer: &IndexBuffer)
    {
        unsafe { self.m_DeferredContext.IASetIndexBuffer(indexBuffer.GetHandle(), indexBuffer.GetFormat(), 0); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetVertexBufferData(&self, data: *const u8, size: usize, vertexBuffer: &VertexBuffer)
    {
        unsafe
        {
            debug_assert!(size <= vertexBuffer.GetSize() as usize, "Vertex buffer size is too small!");

            let msr: D3D11_MAPPED_SUBRESOURCE = DXCall!(self.m_DeferredContext.Map(vertexBuffer.GetHandle().as_ref().unwrap(), 0, D3D11_MAP_WRITE_DISCARD, 0));
            std::ptr::copy_nonoverlapping(data, msr.pData as _, size);
            self.m_DeferredContext.Unmap(vertexBuffer.GetHandle().as_ref().unwrap(), 0);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetIndexBufferData(&self, data: *const u8, size: usize, indexBuffer: &IndexBuffer)
    {
        unsafe
        {
            debug_assert!(size <= indexBuffer.GetSize() as usize, "Vertex buffer size is too small!");

            let msr: D3D11_MAPPED_SUBRESOURCE = DXCall!(self.m_DeferredContext.Map(indexBuffer.GetHandle(), 0, D3D11_MAP_WRITE_DISCARD, 0));
            std::ptr::copy_nonoverlapping(data, msr.pData as _, size);
            self.m_DeferredContext.Unmap(indexBuffer.GetHandle(), 0);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetGraphicsPipelineState(&self, pipeline: RustyRef<GraphicsPipeline>)
    {
        unsafe
        {
            let pipeline: Ref<GraphicsPipeline> = pipeline.GetRef();

            // Bind shaders
            let shader: RustyRef<Shader> = pipeline.GetShader();
            let shaderRef: Ref<Shader> = shader.GetRef();
            let vsConstantBuffers: &Vec<Option<ID3D11Buffer>> = shaderRef.GetVSConstantBufferHandles();
            let psConstantBuffers: &Vec<Option<ID3D11Buffer>> = shaderRef.GetPSConstantBufferHandles();

            self.m_DeferredContext.VSSetShader(shaderRef.GetVSHandle(), null(), 0);
            self.m_DeferredContext.VSSetConstantBuffers(0, vsConstantBuffers.len() as u32, vsConstantBuffers.as_ptr());
            self.m_DeferredContext.PSSetShader(shaderRef.GetPSHandle(), null(), 0);
            self.m_DeferredContext.PSSetConstantBuffers(0, psConstantBuffers.len() as u32, psConstantBuffers.as_ptr());

            // Bind states
            self.m_DeferredContext.RSSetState(pipeline.GetRasterizerState());
            self.m_DeferredContext.OMSetBlendState(pipeline.GetBlendState(), null(), 0xffffffff);
            self.m_DeferredContext.OMSetDepthStencilState(pipeline.GetDepthStencilState(), 0);

            // Set topology
            self.m_DeferredContext.IASetPrimitiveTopology(pipeline.GetPrimitiveTopology());

            // Set input layout
            self.m_DeferredContext.IASetInputLayout(pipeline.GetInputLayout());
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetVSConstantBufferData<T>(&self, shader: RustyRef<Shader>, data: &T, bufferRegister: u32)
    {
        unsafe
        {
            let shaderRef: Ref<Shader> = shader.GetRef();
            let vsConstantBuffers: &Vec<Option<ID3D11Buffer>> = shaderRef.GetVSConstantBufferHandles();

            debug_assert!((bufferRegister as usize) < vsConstantBuffers.len(), "Invalid VS system constant buffer register");

            let msr: D3D11_MAPPED_SUBRESOURCE = DXCall!(self.m_DeferredContext.Map(vsConstantBuffers[bufferRegister as usize].as_ref().unwrap(), 0, D3D11_MAP_WRITE_DISCARD, 0));
            std::ptr::copy_nonoverlapping(data, msr.pData as _, 1);
            self.m_DeferredContext.Unmap(vsConstantBuffers[bufferRegister as usize].as_ref().unwrap(), 0);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetVSMaterialConstantBufferData(&self, shader: RustyRef<Shader>, data: &Vec<u8>)
    {
        unsafe
        {
            let shaderRef: Ref<Shader> = shader.GetRef();
            let vsConstantBuffers: &Vec<Option<ID3D11Buffer>> = shaderRef.GetVSConstantBufferHandles();
            let vsMaterialConstantBuffer = shaderRef.GetVSMaterialConstantBuffer();
            debug_assert!(vsMaterialConstantBuffer.is_some(), "Shader does not have a VS material constant buffer!");

            let vsMaterialConstantBuffer = vsMaterialConstantBuffer.as_ref().unwrap();
            let bufferRegister = vsMaterialConstantBuffer.GetRegister();
            debug_assert!((bufferRegister as usize) < vsConstantBuffers.len(), "Invalid VS system constant buffer register");

            let msr: D3D11_MAPPED_SUBRESOURCE = DXCall!(self.m_DeferredContext.Map(vsConstantBuffers[bufferRegister as usize].as_ref().unwrap(), 0, D3D11_MAP_WRITE_DISCARD, 0));
            std::ptr::copy_nonoverlapping(data.as_ptr(), msr.pData as _, data.len());
            self.m_DeferredContext.Unmap(vsConstantBuffers[bufferRegister as usize].as_ref().unwrap(), 0);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetPSConstantBufferData<T>(&self, shader: RustyRef<Shader>, data: &T, bufferRegister: u32)
    {
        unsafe
        {
            let shaderRef: Ref<Shader> = shader.GetRef();
            let psConstantBuffers: &Vec<Option<ID3D11Buffer>> = shaderRef.GetPSConstantBufferHandles();

            debug_assert!((bufferRegister as usize) < psConstantBuffers.len(), "Invalid PS system constant buffer register");

            let msr: D3D11_MAPPED_SUBRESOURCE = DXCall!(self.m_DeferredContext.Map(psConstantBuffers[bufferRegister as usize].as_ref().unwrap(), 0, D3D11_MAP_WRITE_DISCARD, 0));
            std::ptr::copy_nonoverlapping(data, msr.pData as _, 1);
            self.m_DeferredContext.Unmap(psConstantBuffers[bufferRegister as usize].as_ref().unwrap(), 0);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetPSMaterialConstantBufferData(&self, shader: RustyRef<Shader>, data: &Vec<u8>)
    {
        unsafe
        {
            let shaderRef: Ref<Shader> = shader.GetRef();
            let psConstantBuffers: &Vec<Option<ID3D11Buffer>> = shaderRef.GetPSConstantBufferHandles();
            let psMaterialConstantBuffer = shaderRef.GetPSMaterialConstantBuffer();
            debug_assert!(psMaterialConstantBuffer.is_some(), "Shader does not have a PS material constant buffer!");

            let psMaterialConstantBuffer = psMaterialConstantBuffer.as_ref().unwrap();
            let bufferRegister = psMaterialConstantBuffer.GetRegister();
            debug_assert!((bufferRegister as usize) < psConstantBuffers.len(), "Invalid PS system constant buffer register");

            let msr: D3D11_MAPPED_SUBRESOURCE = DXCall!(self.m_DeferredContext.Map(psConstantBuffers[bufferRegister as usize].as_ref().unwrap(), 0, D3D11_MAP_WRITE_DISCARD, 0));
            std::ptr::copy_nonoverlapping(data.as_ptr(), msr.pData as _, data.len());
            self.m_DeferredContext.Unmap(psConstantBuffers[bufferRegister as usize].as_ref().unwrap(), 0);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetCSConstantBufferData<T>(&self, shader: RustyRef<ComputeShader>, data: &T, bufferRegister: u32)
    {
        unsafe
        {
            let shaderRef: Ref<ComputeShader> = shader.GetRef();
            let constantBuffers: &Vec<Option<ID3D11Buffer>> = shaderRef.GetConstantBufferHandles();

            debug_assert!((bufferRegister as usize) < constantBuffers.len(), "Invalid CS constant buffer register");

            let msr: D3D11_MAPPED_SUBRESOURCE = DXCall!(self.m_DeferredContext.Map(constantBuffers[bufferRegister as usize].as_ref().unwrap(), 0, D3D11_MAP_WRITE_DISCARD, 0));
            std::ptr::copy_nonoverlapping(data, msr.pData as _, 1);
            self.m_DeferredContext.Unmap(constantBuffers[bufferRegister as usize].as_ref().unwrap(), 0);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetPSShaderResources(&self, shaderResource: *const Option<ID3D11ShaderResourceView>, count: u32, startSlot: u32)
    {
        unsafe { self.m_DeferredContext.PSSetShaderResources(startSlot, count, shaderResource); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetPSSamplers(&self, samplers: *const Option<ID3D11SamplerState>, count: u32, startSlot: u32)
    {
        unsafe { self.m_DeferredContext.PSSetSamplers(startSlot, count, samplers); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetCSShaderResources(&self, shaderResource: *const Option<ID3D11ShaderResourceView>, count: u32, startSlot: u32)
    {
        unsafe { self.m_DeferredContext.CSSetShaderResources(startSlot, count, shaderResource); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetCSSamplers(&self, samplers: *const Option<ID3D11SamplerState>, count: u32, startSlot: u32)
    {
        unsafe { self.m_DeferredContext.CSSetSamplers(startSlot, count, samplers); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetCSUnorderedAccessViews(&self, unorderedAccessViews: *const Option<ID3D11UnorderedAccessView>, count: u32, startSlot: u32)
    {
        unsafe { self.m_DeferredContext.CSSetUnorderedAccessViews(startSlot, count, unorderedAccessViews, null()); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn ClearRenderTarget(&self, renderTarget: &Option<ID3D11RenderTargetView>, color: &XMFLOAT4)
    {
        unsafe { self.m_DeferredContext.ClearRenderTargetView(renderTarget, color.as_ref().as_ptr()); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn ClearDepthStencilBuffer(&self, depthBuffer: &Option<ID3D11DepthStencilView>, depthClearValue: f32, stencilClearValue: u8)
    {
        unsafe { self.m_DeferredContext.ClearDepthStencilView(depthBuffer, (D3D11_CLEAR_DEPTH.0 | D3D11_CLEAR_STENCIL.0) as u32, depthClearValue, stencilClearValue); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetRenderTargets(&self, renderTargets: *const Option<ID3D11RenderTargetView>, count: u32, depthBuffer: &Option<ID3D11DepthStencilView>)
    {
        unsafe { self.m_DeferredContext.OMSetRenderTargets(count, renderTargets, depthBuffer); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetViewports(&self, viewports: *const D3D11_VIEWPORT, count: u32)
    {
        unsafe { self.m_DeferredContext.RSSetViewports(count, viewports); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetScissorRect(&self, rect: &RECT)
    {
        unsafe { self.m_DeferredContext.RSSetScissorRects(1, rect); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn DrawIndexed(&self, indexCount: u32, startIndex: u32, startVertex: u32)
    {
        unsafe { self.m_DeferredContext.DrawIndexed(indexCount, startIndex, startVertex as i32); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GenerateMips(&self, srv: &Option<ID3D11ShaderResourceView>)
    {
        unsafe { self.m_DeferredContext.GenerateMips(srv); }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CopyTexture(&self, src: &ID3D11Texture2D, dest: &ID3D11Texture2D, subresourceIndex: u32)
    {
        unsafe { self.m_DeferredContext.CopySubresourceRegion(dest, subresourceIndex, 0, 0, 0, src, subresourceIndex, null());}
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetDeferredContext(&self) -> &ID3D11DeviceContext
    {
        return &self.m_DeferredContext;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetCommandList(&self) -> &Option<ID3D11CommandList>
    {
        return &self.m_CommandList;
    }
}