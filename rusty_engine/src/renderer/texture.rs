#![allow(non_snake_case)]

// std
use std::ptr::null;

// freeimage
use freeimage::*;

// Win32
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;

// Core
use crate::core::utils::*;

// Renderer
use crate::renderer::renderer::*;

// -------------------------------------------------------------- Image -------------------------------------------------------------------------------------
pub struct Image
{
    pub Width:         i32,
    pub Height:        i32,
    pub BytesPerPixel: i32,
    pub IsHDR:         bool,
    pub PixelBuffer:   Box<u8>,
    pub Filepath:      String
}

impl Image
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(width: u32, height: u32, isHDR: bool, data: *const u8) -> Image
    {
        unsafe 
        {
            let bytesPerPixel = if isHDR { 4 * 4 } else { 4 };
            let bufferSize: usize = (width * height * bytesPerPixel) as usize;
            let buffer: *mut u8 =  std::alloc::alloc(std::alloc::Layout::array::<u8>(bufferSize).unwrap()) as _;
            std::ptr::copy_nonoverlapping(data, buffer, bufferSize);

            return Image {
                Width: width as i32,
                Height: height as i32,
                BytesPerPixel: bytesPerPixel as i32,
                IsHDR: isHDR,
                PixelBuffer: Box::from_raw(buffer),
                Filepath: String::new()
            };
        }
    }
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LoadFromFile(filepath: &str, flipY: bool) -> Image
    {
        unsafe
        {
            let mut image = Image {
                Width: 0,
                Height: 0,
                BytesPerPixel: 0,
                IsHDR: false,
                PixelBuffer: Box::new(0),
                Filepath: String::from(filepath)
            };

            let mut bitmap = Bitmap::load(filepath).expect(format!("Failed loading image file: {}", filepath).as_str());

            if flipY
            {
                bitmap = bitmap.flip_vertical().expect(format!("Failed flipping image file: {}", filepath).as_str());
            }

            image.Width = bitmap.width() as i32;
            image.Height = bitmap.height() as i32;
            image.BytesPerPixel = bitmap.bpp() as i32 / 8;
            image.IsHDR = bitmap.ty() == Type::RGBF || bitmap.ty() == Type::RGBAF;

            if bitmap.ty() == Type::BITMAP && bitmap.bpp() < 32
            {
                // If the image is not 32 bit then add alpha channel manually since DirectX requires 32 bit textures
                bitmap = bitmap.to_32bits().expect(format!("Failed to convert \"{}\" to 32-bit image!", filepath).as_str());

                let buffer: *mut u8 = std::alloc::alloc(std::alloc::Layout::array::<u8>(bitmap.pixels::<u8>().len()).unwrap()) as _;
                std::ptr::copy_nonoverlapping(bitmap.pixels::<u8>().as_ptr(), buffer, bitmap.pixels::<u8>().len());
                
                image.BytesPerPixel = 4;
                image.PixelBuffer = Box::from_raw(buffer);
            }
            else if bitmap.ty() == Type::RGBF
            {
                // Add padding if the hdr file does not have alpha channel
                let buffer: *mut f32 = std::alloc::alloc(std::alloc::Layout::array::<f32>((image.Width * image.Height * 4) as usize).unwrap()) as _;
                let pixels = bitmap.pixels::<f32>();
                let mut pixelIndex = 0;

                for i in 0..image.Width * image.Height * 4
                {
                    if (i + 1) % 4 == 0
                    {
                        *buffer.offset(i as isize) = 1.0;
                    }
                    else
                    {
                        *buffer.offset(i as isize) = pixels[pixelIndex];
                        pixelIndex += 1;
                    }
                }

                image.BytesPerPixel = 16;
                image.PixelBuffer = Box::from_raw(buffer as _);
            }
            else
            {
                // The texture doesn't need any padding
                let buffer: *mut u8 = std::alloc::alloc(std::alloc::Layout::array::<u8>(bitmap.pixels::<u8>().len()).unwrap()) as _;
                std::ptr::copy_nonoverlapping(bitmap.pixels::<u8>().as_ptr(), buffer, bitmap.pixels::<u8>().len());
                image.PixelBuffer = Box::from_raw(buffer);
            }
            
            return image;
        }
    }
}

// -------------------------------------------------------------- Texture -----------------------------------------------------------------------------------
pub struct TextureDescription
{
    pub Name:      String,
    pub Format:    DXGI_FORMAT,
    pub Width:     u32,
    pub Height:    u32,
    pub MipCount:  u32,
    pub BindFlags: D3D11_BIND_FLAG,
    pub ImageData: Vec<Option<Image>>
}

pub struct SamplerDescription
{
    pub Wrap:   D3D11_TEXTURE_ADDRESS_MODE,
    pub Filter: D3D11_FILTER
}

pub struct Texture
{
    m_TextureDesc:   D3D11_TEXTURE2D_DESC,
    m_TextureHandle: ID3D11Texture2D,
    m_SamplerDesc:   D3D11_SAMPLER_DESC,
    m_Sampler:       Option<ID3D11SamplerState>,
    m_IsCubemap:     bool,
    m_Name:          String,
    m_Filepath:      String
}

impl Texture
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateTexture2D(description: &TextureDescription, samplerDescription: &SamplerDescription) -> RustyRef<Texture>
    {
        unsafe
        {
            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();

            // Create d3d11 texture
            let mut textureDesc = D3D11_TEXTURE2D_DESC::default();

            // If the user provided image data then pick the width and height of the loaded texture file
            let mut filepath = String::new();
            if !description.ImageData.is_empty() && description.ImageData[0].is_some()
            {
                filepath = description.ImageData[0].as_ref().unwrap().Filepath.clone();
                textureDesc.Width = description.ImageData[0].as_ref().unwrap().Width as u32;
                textureDesc.Height = description.ImageData[0].as_ref().unwrap().Height as u32;
            }
            else
            {
                textureDesc.Width = description.Width;
                textureDesc.Height = description.Height;
            }
            
            // For depth-stencil formats we have to create textures as typeless format in order to be able to use them as shader resources
            match description.Format
            {
                DXGI_FORMAT_D24_UNORM_S8_UINT => textureDesc.Format = DXGI_FORMAT_R24G8_TYPELESS,
                DXGI_FORMAT_D32_FLOAT         => textureDesc.Format = DXGI_FORMAT_R32_TYPELESS,
                _                                        => textureDesc.Format = description.Format
            }

		    textureDesc.MipLevels			= description.MipCount;
		    textureDesc.ArraySize			= 1;
		    textureDesc.Usage				= D3D11_USAGE_DEFAULT;
		    textureDesc.SampleDesc.Count	= 1;
		    textureDesc.SampleDesc.Quality	= 0;
		    textureDesc.BindFlags			= description.BindFlags;
		    textureDesc.CPUAccessFlags		= D3D11_CPU_ACCESS_FLAG(0);

            if description.MipCount > 1 
            {
                textureDesc.BindFlags       |= D3D11_BIND_RENDER_TARGET;
                textureDesc.MiscFlags       = D3D11_RESOURCE_MISC_GENERATE_MIPS 
            }

            let d3d11Texture: ID3D11Texture2D = DXCall!(device.CreateTexture2D(&textureDesc, null()));

            // Create sampler
            let mut samplerDesc = D3D11_SAMPLER_DESC::default();
            samplerDesc.AddressU       = samplerDescription.Wrap;
            samplerDesc.AddressV       = samplerDescription.Wrap;
            samplerDesc.AddressW       = samplerDescription.Wrap;
            samplerDesc.MinLOD         = 0.0;
            samplerDesc.MaxLOD         = D3D11_FLOAT32_MAX;
            samplerDesc.MaxAnisotropy  = if samplerDescription.Filter == D3D11_FILTER_ANISOTROPIC { D3D11_REQ_MAXANISOTROPY } else { 1 };
            samplerDesc.Filter         = samplerDescription.Filter;
            samplerDesc.ComparisonFunc = D3D11_COMPARISON_NEVER;

            let sampler: ID3D11SamplerState = DXCall!(device.CreateSamplerState(&samplerDesc));

            // Create Texture object
            let texture = Texture {
                m_TextureHandle: d3d11Texture, 
                m_TextureDesc: textureDesc,
                m_SamplerDesc: samplerDesc,
                m_Sampler: Some(sampler),
                m_IsCubemap: false,
                m_Name: description.Name.clone(),
                m_Filepath: filepath
            };

            if !description.ImageData.is_empty() && description.ImageData[0].is_some()
            {
                // If the user provided image data, copy the pixels
                let imageData: &Image = description.ImageData[0].as_ref().unwrap();

                let mut deviceContext: Option<ID3D11DeviceContext> = None;
                device.GetImmediateContext(&mut deviceContext);
                let deviceContext: ID3D11DeviceContext = deviceContext.unwrap();

                deviceContext.UpdateSubresource(&texture.m_TextureHandle, 0, null(), &*imageData.PixelBuffer as *const u8 as _, (imageData.BytesPerPixel * imageData.Width) as u32, 0);
            }

            return RustyRef::CreateRef(texture);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateTextureCube(description: &TextureDescription, samplerDescription: &SamplerDescription) -> RustyRef<Texture>
    {
        unsafe
        {
            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();
            
            // Create d3d11 texture
            let mut textureDesc = D3D11_TEXTURE2D_DESC::default();

            // If the user provided image data then pick the width and height of the loaded texture file
            let mut filepath = String::new();
            if !description.ImageData.is_empty() && description.ImageData[0].is_some()
            {
                filepath = description.ImageData[0].as_ref().unwrap().Filepath.clone();
                textureDesc.Width = description.ImageData[0].as_ref().unwrap().Width as u32;
                textureDesc.Height = description.ImageData[0].as_ref().unwrap().Height as u32;
            }
            else
            {
                textureDesc.Width = description.Width;
                textureDesc.Height = description.Height;
            }

		    // For depth-stencil formats we have to create textures as typeless format in order to be able to use them as shader resources
            match description.Format
            {
                DXGI_FORMAT_D24_UNORM_S8_UINT => textureDesc.Format = DXGI_FORMAT_R24G8_TYPELESS,
                DXGI_FORMAT_D32_FLOAT         => textureDesc.Format = DXGI_FORMAT_R32_TYPELESS,
                _                                        => textureDesc.Format = description.Format
            }

		    textureDesc.MipLevels			= description.MipCount;
		    textureDesc.ArraySize			= 6;
		    textureDesc.Usage				= D3D11_USAGE_DEFAULT;
		    textureDesc.SampleDesc.Count	= 1;
		    textureDesc.SampleDesc.Quality	= 0;
		    textureDesc.BindFlags			= D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS;
		    textureDesc.CPUAccessFlags		= D3D11_CPU_ACCESS_FLAG(0);
            textureDesc.MiscFlags           = D3D11_RESOURCE_MISC_TEXTURECUBE;

            if description.MipCount > 1 
            {
                textureDesc.BindFlags       |= D3D11_BIND_RENDER_TARGET;
                textureDesc.MiscFlags       |= D3D11_RESOURCE_MISC_GENERATE_MIPS 
            }

            let texture: ID3D11Texture2D = DXCall!(device.CreateTexture2D(&textureDesc, null()));

            // Create sampler
            let mut samplerDesc = D3D11_SAMPLER_DESC::default();
            samplerDesc.AddressU       = samplerDescription.Wrap;
            samplerDesc.AddressV       = samplerDescription.Wrap;
            samplerDesc.AddressW       = samplerDescription.Wrap;
            samplerDesc.MinLOD         = 0.0;
            samplerDesc.MaxLOD         = D3D11_FLOAT32_MAX;
            samplerDesc.MaxAnisotropy  = if samplerDescription.Filter == D3D11_FILTER_ANISOTROPIC { D3D11_REQ_MAXANISOTROPY} else { 1 };
            samplerDesc.Filter         = samplerDescription.Filter;
            samplerDesc.ComparisonFunc = D3D11_COMPARISON_NEVER;

            let sampler: ID3D11SamplerState = DXCall!(device.CreateSamplerState(&samplerDesc));

            // Create Texture object
            let texture = Texture {
                m_TextureHandle: texture, 
                m_TextureDesc: textureDesc,
                m_SamplerDesc: samplerDesc,
                m_Sampler: Some(sampler),
                m_IsCubemap: true,
                m_Name: description.Name.clone(),
                m_Filepath: filepath
            };

            let mut deviceContext: Option<ID3D11DeviceContext> = None;
            device.GetImmediateContext(&mut deviceContext);
            let deviceContext: ID3D11DeviceContext = deviceContext.unwrap();

            for i in 0..description.ImageData.len()
            {
                // If the user provided image data, copy the pixels
                if description.ImageData[i].is_some()
                {
                    let imageData: &Image = description.ImageData[i].as_ref().unwrap();
                    deviceContext.UpdateSubresource(&texture.m_TextureHandle, i as u32, null(), &*imageData.PixelBuffer as *const u8 as _, (imageData.BytesPerPixel * imageData.Width) as u32, 0);
                }
            }

            return RustyRef::CreateRef(texture);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetWidth(&self) -> u32
    {
        return self.m_TextureDesc.Width;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetHeight(&self) -> u32
    {
        return self.m_TextureDesc.Height;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetFormat(&self) -> DXGI_FORMAT
    {
        return self.m_TextureDesc.Format;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetWrapMode(&self) -> D3D11_TEXTURE_ADDRESS_MODE
    {
        return self.m_SamplerDesc.AddressU;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetFilter(&self) -> D3D11_FILTER
    {
        return self.m_SamplerDesc.Filter;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetHandle(&self) -> &ID3D11Texture2D
    {
        return &self.m_TextureHandle;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSampler(&self) -> &Option<ID3D11SamplerState>
    {
        return &self.m_Sampler;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetMipCount(&self) -> u32
    {
        return self.m_TextureDesc.MipLevels;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetName(&self) -> &String
    {
        return &self.m_Name;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetFilepath(&self) -> &String
    {
        return &self.m_Filepath;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateSRV(&self) -> Option<ID3D11ShaderResourceView>
    {
        unsafe
        {
            debug_assert!(self.m_TextureDesc.BindFlags & D3D11_BIND_SHADER_RESOURCE != D3D11_BIND_FLAG(0), "Texture is missing SRV bind flag!");

            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();
            let mut srvDesc = D3D11_SHADER_RESOURCE_VIEW_DESC::default();

            // Pick the correct format when creating a SRV for a depth texture
            match self.m_TextureDesc.Format
            {
                DXGI_FORMAT_R24G8_TYPELESS => srvDesc.Format = DXGI_FORMAT_R24_UNORM_X8_TYPELESS,
                DXGI_FORMAT_R32_TYPELESS   => srvDesc.Format = DXGI_FORMAT_R32_FLOAT,
                _                                     => srvDesc.Format = self.m_TextureDesc.Format
            }
            
            if self.m_IsCubemap
            {
                srvDesc.ViewDimension                         = D3D_SRV_DIMENSION_TEXTURECUBE;
                srvDesc.Anonymous.TextureCube.MostDetailedMip = 0;
                srvDesc.Anonymous.TextureCube.MipLevels       = self.m_TextureDesc.MipLevels;
            }
            else
            {
                srvDesc.ViewDimension                       = D3D_SRV_DIMENSION_TEXTURE2D;
                srvDesc.Anonymous.Texture2D.MostDetailedMip = 0;
                srvDesc.Anonymous.Texture2D.MipLevels       = self.m_TextureDesc.MipLevels;
            }

            let srv: Option<ID3D11ShaderResourceView> = Some(DXCall!(device.CreateShaderResourceView(&self.m_TextureHandle, &srvDesc)));

            // Generate mips
            if self.m_TextureDesc.MipLevels > 1
            {
                let mut deviceContext: Option<ID3D11DeviceContext> = None;
                device.GetImmediateContext(&mut deviceContext);
                let deviceContext: ID3D11DeviceContext = deviceContext.unwrap();
                deviceContext.GenerateMips(&srv);
            }

            return srv;
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateRTV(&self, slice: i32, mipLevel: u32) -> Option<ID3D11RenderTargetView>
    {
        unsafe
        {
            debug_assert!(self.m_TextureDesc.BindFlags & D3D11_BIND_RENDER_TARGET != D3D11_BIND_FLAG(0), "Texture is missing RTV bind flag!");
            debug_assert!(slice < self.m_TextureDesc.ArraySize as i32, "Invalid array slice index!");
            debug_assert!(mipLevel < self.m_TextureDesc.MipLevels, "Invalid mip level index!");

            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();
            let mut rtvDesc = D3D11_RENDER_TARGET_VIEW_DESC::default();

            // Pick the correct format when creating a RTV for a depth texture
            match self.m_TextureDesc.Format
            {
                DXGI_FORMAT_R24G8_TYPELESS => rtvDesc.Format = DXGI_FORMAT_R24_UNORM_X8_TYPELESS,
                DXGI_FORMAT_R32_TYPELESS   => rtvDesc.Format = DXGI_FORMAT_R32_FLOAT,
                _                                     => rtvDesc.Format = self.m_TextureDesc.Format
            }

            if self.m_IsCubemap
            {
                rtvDesc.ViewDimension                            = D3D11_RTV_DIMENSION_TEXTURE2DARRAY;
                rtvDesc.Anonymous.Texture2DArray.MipSlice        = mipLevel;

                if slice > -1
                {
                    rtvDesc.Anonymous.Texture2DArray.FirstArraySlice = slice as u32;
                    rtvDesc.Anonymous.Texture2DArray.ArraySize       = 1;
                }
                else
                {
                    rtvDesc.Anonymous.Texture2DArray.FirstArraySlice = 0;
                    rtvDesc.Anonymous.Texture2DArray.ArraySize       = 6;
                }
            }
            else
            {
                rtvDesc.ViewDimension                            = D3D11_RTV_DIMENSION_TEXTURE2D;
                rtvDesc.Anonymous.Texture2D.MipSlice             = mipLevel;
            }

            return Some(DXCall!(device.CreateRenderTargetView(&self.m_TextureHandle, &rtvDesc)));
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateDSV(&self, slice: i32, mipLevel: u32) -> Option<ID3D11DepthStencilView>
    {
        unsafe
        {
            debug_assert!(self.m_TextureDesc.BindFlags & D3D11_BIND_DEPTH_STENCIL != D3D11_BIND_FLAG(0), "Texture is missing DSV bind flag!");
            debug_assert!(slice < self.m_TextureDesc.ArraySize as i32, "Invalid array slice index!");
            debug_assert!(mipLevel < self.m_TextureDesc.MipLevels, "Invalid mip level index!");

            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();
            let mut dsvDesc = D3D11_DEPTH_STENCIL_VIEW_DESC::default();
            dsvDesc.Flags = 0;

            // Pick the correct format when creating a DSV for a depth texture
            match self.m_TextureDesc.Format
            {
                DXGI_FORMAT_R24G8_TYPELESS => dsvDesc.Format = DXGI_FORMAT_D24_UNORM_S8_UINT,
                DXGI_FORMAT_R32_TYPELESS   => dsvDesc.Format = DXGI_FORMAT_D32_FLOAT,
                _                                     => dsvDesc.Format = self.m_TextureDesc.Format
            }

            if self.m_IsCubemap
            {
                dsvDesc.ViewDimension                            = D3D11_DSV_DIMENSION_TEXTURE2DARRAY;
                dsvDesc.Anonymous.Texture2DArray.MipSlice        = mipLevel;

                if slice > -1
                {
                    dsvDesc.Anonymous.Texture2DArray.FirstArraySlice = slice as u32;
                    dsvDesc.Anonymous.Texture2DArray.ArraySize       = 1;
                }
                else
                {
                    dsvDesc.Anonymous.Texture2DArray.FirstArraySlice = 0;
                    dsvDesc.Anonymous.Texture2DArray.ArraySize       = 6;
                }
            }
            else
            {
                dsvDesc.ViewDimension                            = D3D11_DSV_DIMENSION_TEXTURE2D;
                dsvDesc.Anonymous.Texture2D.MipSlice             = mipLevel;
            }

            return Some(DXCall!(device.CreateDepthStencilView(&self.m_TextureHandle, &dsvDesc)));
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateUAV(&self, slice: i32, mipLevel: u32) -> Option<ID3D11UnorderedAccessView>
    {
        unsafe
        {
            debug_assert!(self.m_TextureDesc.BindFlags & D3D11_BIND_UNORDERED_ACCESS != D3D11_BIND_FLAG(0), "Texture is missing UAV bind flag!");
            debug_assert!(slice < self.m_TextureDesc.ArraySize as i32, "Invalid array slice index!");
            debug_assert!(mipLevel < self.m_TextureDesc.MipLevels, "Invalid mip level index!");

            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();
            let mut uavDesc = D3D11_UNORDERED_ACCESS_VIEW_DESC::default();

            // Pick the correct format when creating a UAV for a depth texture
            match self.m_TextureDesc.Format
            {
                DXGI_FORMAT_R24G8_TYPELESS => uavDesc.Format = DXGI_FORMAT_R24_UNORM_X8_TYPELESS,
                DXGI_FORMAT_R32_TYPELESS   => uavDesc.Format = DXGI_FORMAT_R32_FLOAT,
                _                                     => uavDesc.Format = self.m_TextureDesc.Format
            }
            
            if self.m_IsCubemap
            {
                uavDesc.ViewDimension                            = D3D11_UAV_DIMENSION_TEXTURE2DARRAY;
                uavDesc.Anonymous.Texture2DArray.MipSlice        = mipLevel;

                if slice > -1
                {
                    uavDesc.Anonymous.Texture2DArray.FirstArraySlice = slice as u32;
                    uavDesc.Anonymous.Texture2DArray.ArraySize       = 1;
                }
                else
                {
                    uavDesc.Anonymous.Texture2DArray.FirstArraySlice = 0;
                    uavDesc.Anonymous.Texture2DArray.ArraySize       = 6;
                }
            }
            else
            {
                uavDesc.ViewDimension                            = D3D11_UAV_DIMENSION_TEXTURE2D;
                uavDesc.Anonymous.Texture2D.MipSlice             = mipLevel;
            }

            return Some(DXCall!(device.CreateUnorderedAccessView(&self.m_TextureHandle, &uavDesc)));
        }
    }
}