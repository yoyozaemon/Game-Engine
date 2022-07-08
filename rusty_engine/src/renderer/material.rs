#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// Serialization
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;

// Win32
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D11::*;

// Core
use crate::core::utils::*;

// Renderer
use crate::renderer::shader::*;
use crate::renderer::shader_uniform::*;
use crate::renderer::shader_resource::*;
use crate::renderer::texture::*;
use crate::renderer::renderer::*;

bitflags::bitflags!
{
    pub struct MaterialFlags : u32
    {
        const None             = 0;
        const TwoSided         = 1;
        const DisableDepthTest = 2;
        const Wireframe        = 4;
        const Transparent      = 8;
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////// Standard Material //////////////////////////////////////////////////////////////////
pub struct Material
{
    m_Name:            String,
    m_Shader:          RustyRef<Shader>,
    m_Flags:           MaterialFlags,
    m_Uniforms:        Vec<ShaderUniformDeclaration>,
    m_Resources:       Vec<ShaderResourceDeclaration>,
    m_VSUniformBuffer: Vec<u8>,
    m_PSUniformBuffer: Vec<u8>,
    m_Textures:        Vec<RustyRef<Texture>>,

    m_RasterizerState: D3D11_RASTERIZER_DESC,
    m_BlendState:      D3D11_BLEND_DESC,
    m_DepthState:      D3D11_DEPTH_STENCIL_DESC
}

impl Material
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(name: &str, shader: RustyRef<Shader>, flags: MaterialFlags) -> RustyRef<Material>
    {
        debug_assert!(shader.IsValid(), "Shader is null!");

        let mut uniforms: Vec<ShaderUniformDeclaration> = Vec::new();
        let mut resources: Vec<ShaderResourceDeclaration> = Vec::new();
        let mut vsUniformBuffer: Vec<u8> = Vec::new();
        let mut psUniformBuffer: Vec<u8> = Vec::new();

        if let Some(vsMaterialBuffer) = shader.GetRef().GetVSMaterialConstantBuffer()
        {
            vsUniformBuffer.resize(vsMaterialBuffer.GetSize() as usize, 0);

            for uniform in vsMaterialBuffer.GetUniforms().iter()
            {
                uniforms.push(uniform.clone());
            }
        }

        if let Some(psMaterialBuffer) = shader.GetRef().GetPSMaterialConstantBuffer()
        {
            psUniformBuffer.resize(psMaterialBuffer.GetSize() as usize, 0);
            
            for uniform in psMaterialBuffer.GetUniforms().iter()
            {
                uniforms.push(uniform.clone());
            }
        }

        let mut highestRegisterValue: usize = 0;
        for resource in shader.GetRef().GetMaterialResources().iter()
        {
            highestRegisterValue = std::cmp::max(highestRegisterValue, resource.GetRegister() as usize);
            resources.push(resource.clone());
        }

        let mut material = Material {
            m_Name: String::from(name),
            m_Shader: shader.clone(),
            m_Flags: flags,
            m_Uniforms: uniforms,
            m_Resources: resources,
            m_VSUniformBuffer: vsUniformBuffer,
            m_PSUniformBuffer: psUniformBuffer,
            m_Textures: vec![Renderer::GetBlackTexture(); highestRegisterValue + 1],
            m_RasterizerState: D3D11_RASTERIZER_DESC::default(),
            m_BlendState: D3D11_BLEND_DESC::default(),
            m_DepthState: D3D11_DEPTH_STENCIL_DESC::default()
        };

        material.RecreateStates();

        return RustyRef::CreateRef(material);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetUniform<T>(&mut self, name: &str, value: T)
    {
        let uniform: Option<&ShaderUniformDeclaration> = self.FindUniform(name);
        debug_assert!(uniform.is_some(), "Uniform with name {} does not exist!", name);

        let uniform: &ShaderUniformDeclaration = uniform.unwrap();
        debug_assert!(uniform.GetSize() as usize == std::mem::size_of::<T>(), "Uniform size does not match the value size!");

        let byteOffset: isize = uniform.GetOffset() as isize;

        if uniform.GetShaderType() == ShaderType::VS
        {
            unsafe { std::ptr::copy_nonoverlapping(&value as *const T, self.m_VSUniformBuffer.as_mut_ptr().offset(byteOffset) as _, 1); }
        }
        else if uniform.GetShaderType() == ShaderType::PS
        {
            unsafe { std::ptr::copy_nonoverlapping(&value as *const T, self.m_PSUniformBuffer.as_mut_ptr().offset(byteOffset) as _, 1); }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetUniform<T>(&self, name: &str) -> &T
    {
        let uniform: Option<&ShaderUniformDeclaration> = self.FindUniform(name);
        debug_assert!(uniform.is_some(), "Uniform with name {} does not exist!", name);
        let uniform: &ShaderUniformDeclaration = uniform.unwrap();

        let byteOffset: isize = uniform.GetOffset() as isize;

        if uniform.GetShaderType() == ShaderType::VS
        {
            unsafe { return std::mem::transmute::<*const u8, &T>(self.m_VSUniformBuffer.as_ptr().offset(byteOffset)); }
        }
        else
        {
            unsafe { return std::mem::transmute::<*const u8, &T>(self.m_PSUniformBuffer.as_ptr().offset(byteOffset)); }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetTexture(&mut self, name: &str, texture: RustyRef<Texture>)
    {
        let resource: Option<&ShaderResourceDeclaration> = self.FindResource(name);
        debug_assert!(resource.is_some(), "Resource with name {} does not exist!", name);
        let register = resource.unwrap().GetRegister();
        self.m_Textures[register as usize] = texture;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetName(&mut self, name: &str)
    {
        self.m_Name = String::from(name);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetRenderFlags(&mut self, flags: MaterialFlags)
    {
        if self.m_Flags != flags
        {
            self.m_Flags = flags;
            self.RecreateStates();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRenderFlags(&self) -> MaterialFlags
    {
        return self.m_Flags;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetName(&self) -> &String
    {
        return &self.m_Name;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetShader(&self) -> RustyRef<Shader>
    {
        return self.m_Shader.clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetUniforms(&self) -> &Vec<ShaderUniformDeclaration>
    {
        return &self.m_Uniforms;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetResources(&self) -> &Vec<ShaderResourceDeclaration>
    {
        return &self.m_Resources;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVSUniformBuffer(&self) -> &Vec<u8>
    {
        return &self.m_VSUniformBuffer;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetPSUniformBuffer(&self) -> &Vec<u8>
    {
        return &self.m_PSUniformBuffer;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetTextures(&self) -> &Vec<RustyRef<Texture>>
    {
        return &self.m_Textures;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRasterizerState(&self) -> &D3D11_RASTERIZER_DESC
    {
        return &self.m_RasterizerState;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetBlendState(&self) -> &D3D11_BLEND_DESC
    {
        return &self.m_BlendState;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetDepthState(&self) -> &D3D11_DEPTH_STENCIL_DESC
    {
        return &self.m_DepthState;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn FindUniform(&self, name: &str) -> Option<&ShaderUniformDeclaration>
    {
        for uniform in self.m_Uniforms.iter()
        {
            if uniform.GetName() == name
            {
                return Some(uniform);
            }
        }

        return None;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn FindResource(&self, name: &str) -> Option<&ShaderResourceDeclaration>
    {
        for resource in self.m_Resources.iter()
        {
            if resource.GetName() == name
            {
                return Some(resource);
            }
        }

        return None;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn RecreateStates(&mut self)
    {
        let isDepthTested: bool = self.m_Flags & MaterialFlags::DisableDepthTest == MaterialFlags::None;
        let isTwoSided: bool = self.m_Flags & MaterialFlags::TwoSided != MaterialFlags::None;
        let isWireframe: bool = self.m_Flags & MaterialFlags::Wireframe != MaterialFlags::None;
        let isTransparent: bool = self.m_Flags & MaterialFlags::Transparent != MaterialFlags::None;

        // Rasterizer
        self.m_RasterizerState.CullMode = if isTwoSided { D3D11_CULL_NONE } else { D3D11_CULL_BACK };
        self.m_RasterizerState.FillMode = if isWireframe { D3D11_FILL_WIREFRAME } else { D3D11_FILL_SOLID };
        self.m_RasterizerState.AntialiasedLineEnable = BOOL::from(false);
        self.m_RasterizerState.FrontCounterClockwise = BOOL::from(true);
        self.m_RasterizerState.DepthBias = D3D11_DEFAULT_DEPTH_BIAS as i32; 
        self.m_RasterizerState.DepthBiasClamp = D3D11_DEFAULT_DEPTH_BIAS_CLAMP; 
        self.m_RasterizerState.DepthClipEnable = BOOL::from(true); 
        self.m_RasterizerState.MultisampleEnable = BOOL::from(false); 
        self.m_RasterizerState.ScissorEnable = BOOL::from(false); 
        self.m_RasterizerState.SlopeScaledDepthBias = D3D11_DEFAULT_SLOPE_SCALED_DEPTH_BIAS; 

        // Blending
        if isTransparent
        {
            self.m_BlendState.AlphaToCoverageEnable = BOOL::from(false);
            self.m_BlendState.IndependentBlendEnable = BOOL::from(false);
            self.m_BlendState.RenderTarget[0].BlendEnable = BOOL::from(true);
            self.m_BlendState.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8;
            self.m_BlendState.RenderTarget[0].SrcBlend = D3D11_BLEND_SRC_ALPHA;
            self.m_BlendState.RenderTarget[0].DestBlend = D3D11_BLEND_INV_SRC_ALPHA;
            self.m_BlendState.RenderTarget[0].SrcBlendAlpha = D3D11_BLEND_SRC_ALPHA;
            self.m_BlendState.RenderTarget[0].DestBlendAlpha = D3D11_BLEND_INV_SRC_ALPHA;
            self.m_BlendState.RenderTarget[0].BlendOp = D3D11_BLEND_OP_ADD;
            self.m_BlendState.RenderTarget[0].BlendOpAlpha = D3D11_BLEND_OP_ADD;
        }
        else
        {
            self.m_BlendState.RenderTarget[0].BlendEnable = BOOL::from(false);
            self.m_BlendState.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8;
        }

        // Depth-stencil
        if isDepthTested
        {
            self.m_DepthState.DepthEnable = BOOL::from(true);
            self.m_DepthState.DepthFunc = D3D11_COMPARISON_LESS_EQUAL;
            self.m_DepthState.DepthWriteMask = D3D11_DEPTH_WRITE_MASK_ALL;
        }
        else
        {
            self.m_DepthState.DepthEnable = BOOL::from(false);
            self.m_DepthState.DepthFunc = D3D11_COMPARISON_ALWAYS;
            self.m_DepthState.DepthWriteMask = D3D11_DEPTH_WRITE_MASK_ZERO;
        }

        self.m_DepthState.StencilEnable = BOOL::from(true);
        self.m_DepthState.StencilReadMask = 0xff;
        self.m_DepthState.StencilWriteMask = 0xff;
        self.m_DepthState.FrontFace.StencilFunc = D3D11_COMPARISON_ALWAYS;
        self.m_DepthState.FrontFace.StencilFailOp = D3D11_STENCIL_OP_KEEP;
        self.m_DepthState.FrontFace.StencilPassOp = D3D11_STENCIL_OP_INCR_SAT;
        self.m_DepthState.FrontFace.StencilDepthFailOp = D3D11_STENCIL_OP_KEEP;
        self.m_DepthState.BackFace = self.m_DepthState.FrontFace;
    }
}

impl Serialize for RustyRef<Material> 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let matRef = self.GetRef();
        let mut materialStruct = serializer.serialize_struct("Material", 11)?;
        materialStruct.serialize_field("Name", matRef.GetName())?;
        materialStruct.serialize_field("AlbedoColor", matRef.GetUniform::<directx_math::XMFLOAT3>("AlbedoColor").as_ref())?;
        materialStruct.serialize_field("UseAlbedoMap", &matRef.GetUniform::<bool>("UseAlbedoMap"))?;
        materialStruct.serialize_field("UseNormalMap", &matRef.GetUniform::<bool>("UseNormalMap"))?;
        materialStruct.serialize_field("Metalness", &matRef.GetUniform::<f32>("Metalness"))?;
        materialStruct.serialize_field("UseMetalnessMap", &matRef.GetUniform::<bool>("UseMetalnessMap"))?;
        materialStruct.serialize_field("Roughness", &matRef.GetUniform::<f32>("Roughness"))?;
        materialStruct.serialize_field("UseRoughnessMap", &matRef.GetUniform::<bool>("UseRoughnessMap"))?;
        materialStruct.serialize_field("DepthTested", &((matRef.GetRenderFlags() & MaterialFlags::DisableDepthTest) == MaterialFlags::None))?;
        materialStruct.serialize_field("Transparent", &((matRef.GetRenderFlags() & MaterialFlags::Transparent) != MaterialFlags::None))?;
        materialStruct.serialize_field("TwoSided", &((matRef.GetRenderFlags() & MaterialFlags::TwoSided) != MaterialFlags::None))?;
        materialStruct.serialize_field("Wireframe", &((matRef.GetRenderFlags() & MaterialFlags::Wireframe) != MaterialFlags::None))?;
        materialStruct.end()
    }
}