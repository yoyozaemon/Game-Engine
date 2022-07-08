#![allow(non_snake_case)]

// std
use std::ptr::null;

// Win32
use windows::Win32::Graphics::{Direct3D11::*, Dxgi::DXGI_FORMAT};

// Renderer
use crate::renderer::renderer::*;

pub struct IndexBufferDescription
{
    pub IndexCount: u32,
    pub Size:       u32,
    pub Format:     DXGI_FORMAT,
    pub Usage:      D3D11_USAGE
}

pub struct IndexBuffer
{
    m_BufferHandle: Option<ID3D11Buffer>,
    m_IndexCount:   u32,
    m_Size:         u32,
    m_Format:       DXGI_FORMAT
}

impl IndexBuffer
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(data: *const u32, description: &IndexBufferDescription) -> IndexBuffer
    {
        unsafe
        {
            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();

            let mut bufferDesc = D3D11_BUFFER_DESC::default();
            bufferDesc.Usage          = description.Usage;
            bufferDesc.BindFlags      = D3D11_BIND_INDEX_BUFFER.0;
            bufferDesc.CPUAccessFlags = if description.Usage == D3D11_USAGE_DYNAMIC { D3D11_CPU_ACCESS_WRITE.0 } else { 0 };
            bufferDesc.ByteWidth      = description.Size;
            bufferDesc.MiscFlags      = 0;

            let mut indexBuffer: Option<ID3D11Buffer> = None;
            if data != null()
            {
                let mut bufferData = D3D11_SUBRESOURCE_DATA::default();
                bufferData.pSysMem = data as _;

                indexBuffer = Some(DXCall!(device.CreateBuffer(&bufferDesc, &bufferData)));
            }
            else
            {
                indexBuffer = Some(DXCall!(device.CreateBuffer(&bufferDesc, null())));
            }

            return IndexBuffer {
                m_BufferHandle: indexBuffer,
                m_IndexCount: description.IndexCount,
                m_Size: description.Size,
                m_Format: description.Format
            };
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetIndexCount(&self) -> u32
    {
        return self.m_IndexCount;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSize(&self) -> u32
    {
        return self.m_Size;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetFormat(&self) -> DXGI_FORMAT
    {
        return self.m_Format;
    }
    
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetHandle(&self) -> &ID3D11Buffer
    {
        return &self.m_BufferHandle.as_ref().unwrap();
    }
}