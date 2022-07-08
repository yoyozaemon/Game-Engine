#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShaderResourceType
{
    None = 0,
    Texture2D, TextureCube,
    RWTexture2D, RWTexture2DArray,
    Sampler
}

impl ShaderResourceType
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn ShaderResourceFromHLSLResource(hlslResource: &str) -> ShaderResourceType
    {
        match hlslResource
        {
            "Texture2D"         => ShaderResourceType::Texture2D,
            "TextureCube"       => ShaderResourceType::TextureCube,
            "RWTexture2D"       => ShaderResourceType::RWTexture2D,
            "RWTexture2DArray"  => ShaderResourceType::RWTexture2DArray,
            "SamplerState"      => ShaderResourceType::Sampler,
            _                   => { debug_assert!(false, "Unsupported shader resource type!"); ShaderResourceType::None }
        }
    }
}

#[derive(Clone)]
pub struct ShaderResourceDeclaration
{
    m_Name:     String,
    m_Type:     ShaderResourceType,
    m_Register: u32,
}

impl ShaderResourceDeclaration
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(name: &str, resourceType: ShaderResourceType, register: u32) -> ShaderResourceDeclaration
    {
        return ShaderResourceDeclaration {
            m_Name: String::from(name),
            m_Type: resourceType,
            m_Register: register
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetName(&self) -> &String
    {
        return &self.m_Name;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetType(&self) -> ShaderResourceType
    {
        return self.m_Type;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRegister(&self) -> u32
    {
        return self.m_Register;
    }
}