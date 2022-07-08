#![allow(non_snake_case)]

// std
use std::cell::RefMut;
use std::ptr::null;
use std::ptr::null_mut;

// Win32
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::runtime::Interface;
use windows::runtime::Result;
use windows::Win32::Foundation::*;

// Core
use crate::core::utils::*;
use crate::core::window::Window;

// Renderer
use crate::renderer::command_buffer::*;

pub struct GraphicsContext
{
    m_Adapter:                IDXGIAdapter4,
    m_Device:                 ID3D11Device,
    m_DeviceContext:          ID3D11DeviceContext,
    m_SwapChain:              IDXGISwapChain4,
    m_SwapChainRenderTargets: Vec<Option<ID3D11RenderTargetView>>,
    m_SwapChainDepthStencil:  Option<ID3D11DepthStencilView>,
    m_Viewport:               D3D11_VIEWPORT
}

impl GraphicsContext
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(window: &Window) -> RustyRef<GraphicsContext>
    {
        unsafe
        {
            // Create a factory
            let flags: u32 = if cfg!(debug_assertions) { DXGI_CREATE_FACTORY_DEBUG } else { 0 };
            let factory: IDXGIFactory6 = DXCall!(CreateDXGIFactory2(flags));

            // Enumarate all adapters by their performance and pick the most powerfull one
            let mut adapterIndex: u32 = 0;
            let mut currentMaxVideoMemory: usize = 0;
            let mut adapter: IDXGIAdapter4 = factory.EnumAdapterByGpuPreference(adapterIndex, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE).unwrap();

            loop 
            {
                let result: Result<IDXGIAdapter4> = factory.EnumAdapterByGpuPreference(adapterIndex, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE);

                if result.is_err()
                {
                    if result.as_ref().unwrap_err().code() == DXGI_ERROR_NOT_FOUND
                    {
                        // DXGI_ERROR_NOT_FOUND is acceptable and marks the end of the loop
                        break;
                    }
                    else 
                    {
                        // For all other errors, panic
                        panic!("Enumerate adapters failed with HRESULT: {}\nError: {}", result.as_ref().unwrap_err().code().0, result.as_ref().unwrap_err().message());
                    }
                }

                let tempAdapter: IDXGIAdapter4 = result.unwrap();
                let tempAdapterDesc: DXGI_ADAPTER_DESC1 = tempAdapter.GetDesc1().unwrap();

                if tempAdapterDesc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 != DXGI_ADAPTER_FLAG_NONE.0
                {
                    // We do not want software adapter
                    adapterIndex += 1;
                    continue;
                }

                // Check if the current adapter has more video memory than last one
                if tempAdapterDesc.DedicatedVideoMemory > currentMaxVideoMemory
                {
                    if D3D11CreateDevice(&tempAdapter, D3D_DRIVER_TYPE_UNKNOWN, None, D3D11_CREATE_DEVICE_FLAG(0), null(), 0, D3D11_SDK_VERSION, null_mut(), null_mut(), null_mut()).is_ok()
                    {
                        // If D3D11 supports that adapter than pick it
                        currentMaxVideoMemory = tempAdapterDesc.DedicatedVideoMemory;
                        adapter = tempAdapter;
                    }
                }

                adapterIndex += 1;
            }

            // Create the device
            let deviceFlags: D3D11_CREATE_DEVICE_FLAG = if cfg!(debug_assertions) { D3D11_CREATE_DEVICE_DEBUG } else { D3D11_CREATE_DEVICE_FLAG(0) };
            let mut device: Option<ID3D11Device> = None;
            let mut deviceContext: Option<ID3D11DeviceContext> = None;

            DXCall!(D3D11CreateDevice(&adapter, D3D_DRIVER_TYPE_UNKNOWN, None, deviceFlags, null(), 0, D3D11_SDK_VERSION, &mut device, null_mut(), &mut deviceContext));

            let device: ID3D11Device = device.unwrap();
            let deviceContext: ID3D11DeviceContext = deviceContext.unwrap();

            let debugInterface: ID3D11Debug = DXCall!(device.cast());
            DXCall!(debugInterface.ReportLiveDeviceObjects(D3D11_RLDO_SUMMARY));

            let infoQueue: ID3D11InfoQueue = DXCall!(device.cast());
            DXCall!(infoQueue.SetBreakOnSeverity(D3D11_MESSAGE_SEVERITY_CORRUPTION, true));
            DXCall!(infoQueue.SetBreakOnSeverity(D3D11_MESSAGE_SEVERITY_ERROR, true));
            DXCall!(infoQueue.SetBreakOnSeverity(D3D11_MESSAGE_SEVERITY_WARNING, true));

            // Create the swap chain
            let mut swapChainDesc = DXGI_SWAP_CHAIN_DESC1::default();
            swapChainDesc.BufferCount        = 3;
		    swapChainDesc.Width              = window.GetWidth();
		    swapChainDesc.Height             = window.GetHeight();
		    swapChainDesc.Format             = DXGI_FORMAT_R8G8B8A8_UNORM;
            swapChainDesc.BufferUsage        = DXGI_USAGE_RENDER_TARGET_OUTPUT;
		    swapChainDesc.SampleDesc.Count   = 1;
		    swapChainDesc.SampleDesc.Quality = 0;
		    swapChainDesc.SwapEffect         = DXGI_SWAP_EFFECT_FLIP_DISCARD;
		    swapChainDesc.Flags              = DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH.0 as u32;
            swapChainDesc.AlphaMode          = DXGI_ALPHA_MODE_UNSPECIFIED;
            swapChainDesc.Stereo             = BOOL::from(false);

            let tempSwapChain: IDXGISwapChain1 = DXCall!(factory.CreateSwapChainForHwnd(&device, window.GetHandle(), &swapChainDesc, null(), None));
            let swapChain: IDXGISwapChain4 = DXCall!(tempSwapChain.cast());

            // Disable Alt+Enter fullscreen mode
            DXCall!(factory.MakeWindowAssociation(window.GetHandle(), DXGI_MWA_NO_ALT_ENTER));

            // Crete swap chain render targets
            let backBuffer: ID3D11Resource = DXCall!(swapChain.GetBuffer(0));
            let swapChainRenderTarget: Option<ID3D11RenderTargetView> = Some(DXCall!(device.CreateRenderTargetView(&backBuffer, null())));

            let mut depthStencilDesc = D3D11_TEXTURE2D_DESC::default();
		    depthStencilDesc.Width              = window.GetWidth();
		    depthStencilDesc.Height             = window.GetHeight();
		    depthStencilDesc.MipLevels          = 1;
		    depthStencilDesc.ArraySize          = 1;
		    depthStencilDesc.SampleDesc.Count   = 1;
		    depthStencilDesc.SampleDesc.Quality = 0;
		    depthStencilDesc.Format             = DXGI_FORMAT_D24_UNORM_S8_UINT;
		    depthStencilDesc.BindFlags          = D3D11_BIND_DEPTH_STENCIL;
		    depthStencilDesc.Usage              = D3D11_USAGE_DEFAULT;

            let depthBuffer: ID3D11Texture2D = DXCall!(device.CreateTexture2D(&depthStencilDesc, null()));
            let swapChainDepthStencil: Option<ID3D11DepthStencilView> = Some(DXCall!(device.CreateDepthStencilView(&depthBuffer, null())));

            // Create the viewport
            let mut viewport = D3D11_VIEWPORT::default();
            viewport.TopLeftX = 0.0;
            viewport.TopLeftY = 0.0;
            viewport.Width = window.GetWidth() as f32;
            viewport.Height = window.GetHeight() as f32;
            viewport.MinDepth = 0.0;
            viewport.MaxDepth = 1.0;
            
            return RustyRef::CreateRef(GraphicsContext {
                m_Adapter: adapter,
                m_Device: device,
                m_DeviceContext: deviceContext,
                m_SwapChain: swapChain,
                m_SwapChainRenderTargets: vec![swapChainRenderTarget],
                m_SwapChainDepthStencil: swapChainDepthStencil,
                m_Viewport: viewport
            });
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Present(&self, vsync: bool)
    {
        unsafe
        {
            DXCall!(self.m_SwapChain.Present(if vsync { 1 } else { 0 }, 0));
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn ResizeSwapChain(&mut self, width: u32, height: u32)
    {
        unsafe
        {
            let swapChainDesc: DXGI_SWAP_CHAIN_DESC1 = DXCall!(self.m_SwapChain.GetDesc1());
            
            if swapChainDesc.Width != width || swapChainDesc.Height != height
            {
                // Release the render target for the current back buffer
                self.m_SwapChainRenderTargets.clear();
                
                // Resize the buffers
                DXCall!(self.m_SwapChain.ResizeBuffers(swapChainDesc.BufferCount, width, height, swapChainDesc.Format, swapChainDesc.Flags));

                // Resize the viewport
                self.m_Viewport.Width = width as f32;
                self.m_Viewport.Height = height as f32;

                // Update the render target
                self.RecreateRenderTarget();
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn ExecuteCommandBuffer(&self, commandBuffer: RustyRef<CommandBuffer>)
    {
        unsafe
        {
            let mut commandBuffer: RefMut<CommandBuffer> = commandBuffer.GetRefMut();
            self.m_DeviceContext.ExecuteCommandList(commandBuffer.GetCommandList(), false);
            commandBuffer.Reset();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetDevice(&self) -> ID3D11Device
    {
        return self.m_Device.clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetImmediateContext(&self) -> ID3D11DeviceContext
    {
        return self.m_DeviceContext.clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSwapChainRenderTargets(&self) -> &Vec<Option<ID3D11RenderTargetView>>
    {
        return &self.m_SwapChainRenderTargets;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSwapChainDepthStencil(&self) -> &Option<ID3D11DepthStencilView>
    {
        return &self.m_SwapChainDepthStencil;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSwapChainViewport(&self) -> &D3D11_VIEWPORT
    {
        return &self.m_Viewport;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetGPUDescription(&self) -> DXGI_ADAPTER_DESC1
    {
        unsafe
        {
            return DXCall!(self.m_Adapter.GetDesc1());
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn RecreateRenderTarget(&mut self)
    {
        unsafe
        {
            // Render target
            let backBuffer: ID3D11Resource = DXCall!(self.m_SwapChain.GetBuffer(0));
            self.m_SwapChainRenderTargets = vec![Some(DXCall!(self.m_Device.CreateRenderTargetView(&backBuffer, null())))];

            let swapChainDesc: DXGI_SWAP_CHAIN_DESC1 = DXCall!(self.m_SwapChain.GetDesc1());

            // Depth buffer
            let mut depthStencilDesc = D3D11_TEXTURE2D_DESC::default();
		    depthStencilDesc.Width              = swapChainDesc.Width;
		    depthStencilDesc.Height             = swapChainDesc.Height;
		    depthStencilDesc.MipLevels          = 1;
		    depthStencilDesc.ArraySize          = 1;
		    depthStencilDesc.SampleDesc.Count   = 1;
		    depthStencilDesc.SampleDesc.Quality = 0;
		    depthStencilDesc.Format             = DXGI_FORMAT_D24_UNORM_S8_UINT;
		    depthStencilDesc.BindFlags          = D3D11_BIND_DEPTH_STENCIL;
		    depthStencilDesc.Usage              = D3D11_USAGE_DEFAULT;

            let depthBuffer: ID3D11Texture2D = DXCall!(self.m_Device.CreateTexture2D(&depthStencilDesc, null()));
            self.m_SwapChainDepthStencil = Some(DXCall!(self.m_Device.CreateDepthStencilView(&depthBuffer, null())));
        }
    }
}