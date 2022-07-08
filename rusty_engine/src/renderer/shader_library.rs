#![allow(non_snake_case, non_upper_case_globals)]

// std
use std::cell::Ref;
use std::collections::HashMap;

// Core
use crate::core::utils::*;

// Renderer
use crate::renderer::shader::*;

pub struct ShaderLibrary
{
    m_Shaders:        HashMap<String, RustyRef<Shader>>,
    m_ComputeShaders: HashMap<String, RustyRef<ComputeShader>>
}

thread_local! 
{ 
    static s_Instance: RustyRef<ShaderLibrary> = RustyRef::CreateRef(ShaderLibrary { 
        m_Shaders: HashMap::new(), 
        m_ComputeShaders: HashMap::new() 
    });
}

impl ShaderLibrary
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LoadShader(filepath: &str) -> RustyRef<Shader>
    {
        let shader = Shader::Create(filepath);
        let shaderName = String::from(shader.GetRef().GetName());

        return s_Instance.try_with(|shaderLib|
        {
            shaderLib.GetRefMut().m_Shaders.insert(shaderName, shader.clone());
            return shader;
        }).expect(format!("Failed loading a shader \"{}\" to the shader library!", filepath).as_str());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LoadComputeShader(filepath: &str) -> RustyRef<ComputeShader>
    {
        let shader = ComputeShader::Create(filepath);
        let shaderName = String::from(shader.GetRef().GetName());

        return s_Instance.try_with(|shaderLib|
        {
            shaderLib.GetRefMut().m_ComputeShaders.insert(shaderName, shader.clone());
            return shader;
        }).expect(format!("Failed loading a compute shader \"{}\" to the shader library!", filepath).as_str());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn AddShader(shader: RustyRef<Shader>)
    {
        let shaderName = String::from(shader.GetRef().GetName());
        s_Instance.try_with(|shaderLib|
        {
            shaderLib.GetRefMut().m_Shaders.insert(shaderName, shader.clone());
        }).expect(format!("Failed adding a shader \"{}\" to the shader library!", shader.GetRef().GetFilepath()).as_str());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn AddComputeShader(shader: RustyRef<ComputeShader>)
    {
        let shaderName = String::from(shader.GetRef().GetName());
        s_Instance.try_with(|shaderLib|
        {
            shaderLib.GetRefMut().m_ComputeShaders.insert(shaderName, shader.clone());
        }).expect(format!("Failed adding a compute shader \"{}\" to the shader library!", shader.GetRef().GetFilepath()).as_str());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetShader(name: &str) -> RustyRef<Shader>
    {
        return s_Instance.try_with(|shaderLib|
        {
            let shaderLibRef: Ref<ShaderLibrary> = shaderLib.GetRef();
            let shader: &RustyRef<Shader> = shaderLibRef.m_Shaders.get(name).expect(format!("Shader with name \"{}\" not found!", name).as_str());
            return shader.clone();
        }).expect(format!("Failed getting a shader with name \"{}\"!", name).as_str());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetComputeShader(name: &str) -> RustyRef<ComputeShader>
    {
        return s_Instance.try_with(|shaderLib|
        {
            let shaderLibRef: Ref<ShaderLibrary> = shaderLib.GetRef();
            let shader: &RustyRef<ComputeShader> = shaderLibRef.m_ComputeShaders.get(name).expect(format!("Compute shader with name \"{}\" not found!", name).as_str());
            return shader.clone();
        }).expect(format!("Failed getting a compute shader with name \"{}\"!", name).as_str());
    }
}