#![allow(non_snake_case)]

// Renderer
use crate::renderer::shader::ShaderType;
use crate::renderer::environment::*;

// --------------------------------------------------------------- ShaderUniform ----------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShaderUniformType
{
    None = 0,
    Float, Int, Uint,
    Vec2, Vec3, Vec4,
    Bool,
    Matrix,
    Light
}

impl ShaderUniformType 
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn ShaderUniformFromHLSLType(hlslType: &str) -> ShaderUniformType 
    {
        match hlslType 
        {
            "float"               => ShaderUniformType::Float,
            "float2"              => ShaderUniformType::Vec2,
            "float3"              => ShaderUniformType::Vec3,
            "float4"              => ShaderUniformType::Vec4,
            "int"                 => ShaderUniformType::Int,
            "uint"                => ShaderUniformType::Uint,
            "bool"                => ShaderUniformType::Bool,
            "matrix" | "float4x4" => ShaderUniformType::Matrix,
            "Light"               => ShaderUniformType::Light,
            _                     => { debug_assert!(false, "Unsupported shader type!"); ShaderUniformType::None }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn ShaderTypeSize(uniformType: ShaderUniformType) -> u32 
    {
        match uniformType 
        {
            ShaderUniformType::Float      => return 4,
            ShaderUniformType::Vec2       => return 4 * 2,
            ShaderUniformType::Vec3       => return 4 * 3,
            ShaderUniformType::Vec4       => return 4 * 4,
            ShaderUniformType::Int        => return 4,
            ShaderUniformType::Uint       => return 4,
            ShaderUniformType::Bool       => return 4,
            ShaderUniformType::Matrix     => return 4 * 4 * 4,
            ShaderUniformType::Light      => return (std::mem::size_of::<Light>() * MAX_LIGHT_COUNT) as u32,
            _                             => { debug_assert!(false, "Unsupported shader type!"); 0 }
        }
    }
}

#[derive(Clone)]
pub struct ShaderUniformDeclaration
{
    m_Name:           String,
    m_Type:           ShaderUniformType,
    m_Size:           u32,
    m_ByteOffset:     u32,
    m_BufferRegister: u32,
    m_ShaderType:     ShaderType
}

impl ShaderUniformDeclaration
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(name: &str, uniformType: ShaderUniformType, byteOffset: u32, bufferRegister: u32, shaderType: ShaderType) -> ShaderUniformDeclaration
    {
        return ShaderUniformDeclaration {
            m_Name: String::from(name),
            m_Type: uniformType,
            m_Size: ShaderUniformType::ShaderTypeSize(uniformType),
            m_ByteOffset: byteOffset,
            m_BufferRegister: bufferRegister,
            m_ShaderType: shaderType
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetName(&self) -> &String
    {
        return &self.m_Name;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetType(&self) -> ShaderUniformType
    {
        return self.m_Type;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSize(&self) -> u32
    {
        return self.m_Size;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetOffset(&self) -> u32
    {
        return self.m_ByteOffset;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetBufferRegister(&self) -> u32
    {
        return self.m_BufferRegister;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetShaderType(&self) -> ShaderType
    {
        return self.m_ShaderType;
    }
}

// --------------------------------------------------------------- ConstantBufferDeclaration ----------------------------------------------------------------
pub struct ConstantBufferDeclaration
{
    m_Name:         String,
    m_Register:     u32,
    m_Size:         u32,
    m_Uniforms:     Vec<ShaderUniformDeclaration>,
    m_ShaderType:   ShaderType,
    m_IsFinalized:  bool,
}

impl ConstantBufferDeclaration
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(name: &str, register: u32, shaderType: ShaderType) -> ConstantBufferDeclaration
    {
        return ConstantBufferDeclaration {
            m_Name: String::from(name),
            m_Register: register,
            m_Size: 0,
            m_Uniforms: vec![],
            m_ShaderType: shaderType,
            m_IsFinalized: false
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn AddUniform(&mut self, uniform: ShaderUniformDeclaration)
    {
        debug_assert!(!self.m_IsFinalized, "Cannot add uniform declaration to finalized layout!");
        self.m_Size += uniform.GetSize();
        self.m_Uniforms.push(uniform);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn FindUniform(&self, name: &str) -> Option<&ShaderUniformDeclaration>
    {
        let nameStr = String::from(name);

        for uniform in self.m_Uniforms.iter()
        {
            if uniform.m_Name == nameStr
            {
                return Some(uniform);
            }
        }

        return None;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Finalize(&mut self)
    {
        self.m_IsFinalized = true;

        // Add padding if necessary
        self.m_Size = ((self.m_Size + 15) / 16) * 16;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetBufferName(&self) -> &String
    {
        return &self.m_Name;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRegister(&self) -> u32
    {
        return self.m_Register;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSize(&self) -> u32
    {
        return self.m_Size;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetShaderType(&self) -> ShaderType
    {
        return self.m_ShaderType;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetUniforms(&self) -> &Vec<ShaderUniformDeclaration>
    {
        return &self.m_Uniforms;
    }
}