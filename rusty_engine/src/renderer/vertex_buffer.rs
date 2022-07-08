#![allow(non_snake_case)]

// std
use std::ptr::null;

// Win32
use windows::Win32::Graphics::Direct3D11::*;

// Renderer
use crate::renderer::renderer::*;

pub struct VertexBufferDescription
{
    pub VertexCount: u32,
    pub Stride:      u32,
    pub Size:        u32,
    pub Usage:       D3D11_USAGE
}

pub struct VertexBuffer
{
    m_BufferHandle: Option<ID3D11Buffer>,
    m_VertexCount:  u32,
    m_Stride:       u32,
    m_Size:         u32
}

impl VertexBuffer
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(data: *const u8, description: &VertexBufferDescription) -> VertexBuffer
    {
        unsafe
        {
            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();

            let mut bufferDesc = D3D11_BUFFER_DESC::default();
            bufferDesc.Usage          = description.Usage;
            bufferDesc.BindFlags      = D3D11_BIND_VERTEX_BUFFER.0;
            bufferDesc.CPUAccessFlags = if description.Usage == D3D11_USAGE_DYNAMIC { D3D11_CPU_ACCESS_WRITE.0 } else { 0 };
            bufferDesc.ByteWidth      = description.Size;
            bufferDesc.MiscFlags      = 0;

            let mut vertexBuffer: Option<ID3D11Buffer> = None;
            if data != null()
            {
                let mut bufferData = D3D11_SUBRESOURCE_DATA::default();
                bufferData.pSysMem = data as _;

                vertexBuffer = Some(DXCall!(device.CreateBuffer(&bufferDesc, &bufferData)));
            }
            else
            {
                vertexBuffer = Some(DXCall!(device.CreateBuffer(&bufferDesc, null())));
            }

            return VertexBuffer {
                m_BufferHandle: vertexBuffer,
                m_VertexCount: description.VertexCount,
                m_Stride: description.Stride,
                m_Size: description.Size
            };
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVertexCount(&self) -> u32
    {
        return self.m_VertexCount;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetStride(&self) -> u32
    {
        return self.m_Stride;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSize(&self) -> u32
    {
        return self.m_Size;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetHandle(&self) -> &Option<ID3D11Buffer>
    {
        return &self.m_BufferHandle;
    }
}