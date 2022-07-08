#![allow(non_snake_case)]

// Win32
use directx_math::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;

// Core
use crate::core::utils::*;

// Renderer
use crate::renderer::texture::*;

pub struct RenderTargetAttachment
{
    pub Format:   DXGI_FORMAT,
    pub MipCount: u32,
    pub Wrap:     D3D11_TEXTURE_ADDRESS_MODE,
    pub Filter:   D3D11_FILTER
}

pub struct RenderTargetDescription
{
    pub Width:       u32,
    pub Height:      u32,
    pub ClearColor:  XMFLOAT4,
    pub Attachments: Vec<RenderTargetAttachment>
}

pub struct RenderTarget
{
    m_Description:       RenderTargetDescription,
    m_ColorAttachments:  Vec<RustyRef<Texture>>,
    m_DepthAttachment:   RustyRef<Texture>,
    m_RenderTargetViews: Vec<Option<ID3D11RenderTargetView>>,
    m_DepthStencilView:  Option<ID3D11DepthStencilView>,
    m_Viewport:          D3D11_VIEWPORT
}

impl RenderTarget
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(description: RenderTargetDescription) -> RustyRef<RenderTarget>
    {
        let mut renderTarget = RenderTarget {
            m_Description: description,
            m_ColorAttachments: vec![],
            m_DepthAttachment: RustyRef::CreateEmpty(),
            m_RenderTargetViews: vec![],
            m_DepthStencilView: None,
            m_Viewport: D3D11_VIEWPORT::default()
        };

        renderTarget.Invalidate();

        return RustyRef::CreateRef(renderTarget); 
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Resize(&mut self, width: u32, height: u32)
    {
        if width != self.m_Description.Width || height != self.m_Description.Height
        {
            self.m_Description.Width = width;
            self.m_Description.Height = height;

            self.Invalidate();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetColorAttachment(&self, slot: usize) -> RustyRef<Texture>
    {
        debug_assert!(slot < self.m_ColorAttachments.len(), "Invalid slot index!");
        return self.m_ColorAttachments[slot].clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetDepthAttachment(&self) -> RustyRef<Texture>
    {
        return self.m_DepthAttachment.clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetWidth(&self) -> u32
    {
        return self.m_Description.Width;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetHeight(&self) -> u32
    {
        return self.m_Description.Height;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetClearColor(&self) -> &XMFLOAT4
    {
        return &self.m_Description.ClearColor;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRTVs(&self) -> &Vec<Option<ID3D11RenderTargetView>>
    {
        return &self.m_RenderTargetViews;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetDSV(&self) -> &Option<ID3D11DepthStencilView>
    {
        return &self.m_DepthStencilView;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetViewport(&self) -> &D3D11_VIEWPORT
    {
        return &self.m_Viewport;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn Invalidate(&mut self)
    {
        // Reset everything
        self.m_ColorAttachments.clear();
        self.m_DepthAttachment = RustyRef::CreateEmpty();
        self.m_RenderTargetViews.clear();
        self.m_DepthStencilView = None;

        for attachment in self.m_Description.Attachments.iter()
        {
            let isDepthAttachment: bool = attachment.Format == DXGI_FORMAT_D24_UNORM_S8_UINT || attachment.Format == DXGI_FORMAT_D32_FLOAT;

            let textureDesc = TextureDescription {
                Name: String::new(),
                Format: attachment.Format,
                Width: self.m_Description.Width,
                Height: self.m_Description.Height,
                BindFlags: D3D11_BIND_SHADER_RESOURCE | if isDepthAttachment { D3D11_BIND_DEPTH_STENCIL} else { D3D11_BIND_RENDER_TARGET },
                MipCount: attachment.MipCount,
                ImageData: vec![]
            };

            let samplerDesc = SamplerDescription {
                Wrap: attachment.Wrap,
                Filter: attachment.Filter
            };

            let texture = Texture::CreateTexture2D(&textureDesc, &samplerDesc);

            if isDepthAttachment
            {
                // Create the depth stencil view and set the depth attachment texture
                self.m_DepthStencilView = texture.GetRef().CreateDSV(0, 0);
                self.m_DepthAttachment = texture;
            }
            else
            {
                // Create a render target view and add a color attachment texture
                self.m_RenderTargetViews.push(texture.GetRef().CreateRTV(0, 0));
                self.m_ColorAttachments.push(texture);
            }
        }

        // Create viewport
        self.m_Viewport.Width = self.m_Description.Width as f32;
        self.m_Viewport.Height = self.m_Description.Height as f32;
        self.m_Viewport.TopLeftX = 0.0;
        self.m_Viewport.TopLeftY = 0.0;
        self.m_Viewport.MinDepth = 0.0;
        self.m_Viewport.MaxDepth = 1.0;
    }
}