#![allow(non_snake_case)]

// std
use std::alloc::Layout;
use std::alloc::alloc;
use std::ptr::null;
use std::fs;
use std::path::Path;

// other
use regex::Regex;

// Win32
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Hlsl::*;

// Core
use crate::core::utils::*;

// Renderer
use crate::renderer::shader_uniform::*;
use crate::renderer::shader_resource::*;
use crate::renderer::renderer::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShaderType
{
    None = 0,
    VS, PS, CS
}

impl ShaderType
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn ShaderTypeFromString(typeString: &str) -> ShaderType
    {
        match typeString
        {
            "VS" => ShaderType::VS,
            "PS" => ShaderType::PS,
            _    => { debug_assert!(false, "Unknown shader type!"); ShaderType::None }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////// Shader ////////////////////////////////////////////////////////////////////////
pub struct Shader
{
    m_Device:                              ID3D11Device,
    m_Filepath:                            String,
    m_ShaderName:                          String,
 
    m_VSBlob:                              ID3DBlob,
    m_VSHandle:                            ID3D11VertexShader,
    m_VSMaterialConstantBufferDeclaration: Option<ConstantBufferDeclaration>,
    m_VSSystemConstantBufferDeclarations:  Vec<ConstantBufferDeclaration>,
    m_VSConstantBuffers:                   Vec<Option<ID3D11Buffer>>,
 
    m_PSBlob:                              ID3DBlob,
    m_PSHandle:                            ID3D11PixelShader,
    m_PSMaterialConstantBufferDeclaration: Option<ConstantBufferDeclaration>,
    m_PSSystemConstantBufferDeclarations:  Vec<ConstantBufferDeclaration>,
    m_PSConstantBuffers:                   Vec<Option<ID3D11Buffer>>,
 
    m_MaterialResources:                   Vec<ShaderResourceDeclaration>,
    m_SystemResources:                     Vec<ShaderResourceDeclaration>
}

impl Shader
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(filepath: &str) -> RustyRef<Shader>
    {
        unsafe
        {
            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();

            let shaderSource: String = fs::read_to_string(filepath).expect("Failed reading file");
            let shaderName: &str = Path::new(filepath).file_stem().unwrap().to_str().unwrap();

            let mut errorBlob: Option<ID3DBlob> = None;

            // Compile vertex shader
            let mut vsBlob: Option<ID3DBlob> = None;
            D3DCompile(shaderSource.as_ptr() as _, shaderSource.len(), None, null(), None, "VSMain", "vs_4_0", D3DCOMPILE_DEBUG, 0, &mut vsBlob, &mut errorBlob);

            if errorBlob.is_some() && errorBlob.as_ref().unwrap().GetBufferSize() > 0
            {
                // Error was found when compiling vertex shader
                let errorBlob = errorBlob.unwrap();
                let message = String::from_raw_parts(errorBlob.GetBufferPointer() as *mut u8, errorBlob.GetBufferSize(), errorBlob.GetBufferSize());
                panic!("Vertex shader compilation error: {}", message);
            }

            let vsBlob: ID3DBlob = vsBlob.unwrap();
            let vertexShader: ID3D11VertexShader = DXCall!(device.CreateVertexShader(vsBlob.GetBufferPointer(), vsBlob.GetBufferSize(), None));

            // Compile pixel shader
            let mut psBlob: Option<ID3DBlob> = None;
            D3DCompile(shaderSource.as_ptr() as _, shaderSource.len(), None, null(), None, "PSMain", "ps_4_0", D3DCOMPILE_DEBUG, 0, &mut psBlob, &mut errorBlob);

            if errorBlob.is_some() && errorBlob.as_ref().unwrap().GetBufferSize() > 0
            {
                // Error was found when compiling pixel shader
                let errorBlob = errorBlob.unwrap();
                let message = String::from_raw_parts(errorBlob.GetBufferPointer() as *mut u8, errorBlob.GetBufferSize(), errorBlob.GetBufferSize());
                panic!("Pixel shader compilation error: {}", message);
            }

            let psBlob: ID3DBlob = psBlob.unwrap();
            let pixelShader: ID3D11PixelShader = DXCall!(device.CreatePixelShader(psBlob.GetBufferPointer(), psBlob.GetBufferSize(), None));

            // Create the shader and do a reflection to get all uniforms and resources
            let mut shader = Shader { 
                m_Device: device.clone(),
                m_Filepath: String::from(filepath),
                m_ShaderName: String::from(shaderName),

                m_VSBlob: vsBlob,
                m_VSHandle: vertexShader,
                m_VSMaterialConstantBufferDeclaration: None,
                m_VSSystemConstantBufferDeclarations: vec![],
                m_VSConstantBuffers: vec![],

                m_PSBlob: psBlob,
                m_PSHandle: pixelShader,
                m_PSMaterialConstantBufferDeclaration: None,
                m_PSSystemConstantBufferDeclarations: vec![],
                m_PSConstantBuffers: vec![],

                m_MaterialResources: vec![],
                m_SystemResources: vec![],
            };

            shader.Reflect(shaderSource.as_str());
            shader.CreateConstantBuffers();

            return RustyRef::CreateRef(shader);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetFilepath(&self) -> &str
    {
        return self.m_Filepath.as_str();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetName(&self) -> &str
    {
        return self.m_ShaderName.as_str();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVSMaterialConstantBuffer(&self) -> &Option<ConstantBufferDeclaration>
    {
        return &self.m_VSMaterialConstantBufferDeclaration;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVSSystemConstantBuffers(&self) -> &Vec<ConstantBufferDeclaration>
    {
        return &self.m_VSSystemConstantBufferDeclarations;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVSDataBlob(&self) -> &ID3DBlob
    {
        return &self.m_VSBlob;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVSHandle(&self) -> &ID3D11VertexShader
    {
        return &self.m_VSHandle;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVSConstantBufferHandles(&self) -> &Vec<Option<ID3D11Buffer>>
    {
        return &self.m_VSConstantBuffers;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetPSMaterialConstantBuffer(&self) -> &Option<ConstantBufferDeclaration>
    {
        return &self.m_PSMaterialConstantBufferDeclaration;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetPSSystemConstantBuffers(&self) -> &Vec<ConstantBufferDeclaration>
    {
        return &self.m_PSSystemConstantBufferDeclarations;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetPSDataBlob(&self) -> &ID3DBlob
    {
        return &self.m_PSBlob;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetPSHandle(&self) -> &ID3D11PixelShader
    {
        return &self.m_PSHandle;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetPSConstantBufferHandles(&self) -> &Vec<Option<ID3D11Buffer>>
    {
        return &self.m_PSConstantBuffers;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetMaterialResources(&self) -> &Vec<ShaderResourceDeclaration>
    {
        return &self.m_MaterialResources;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetSystemResources(&self) -> &Vec<ShaderResourceDeclaration>
    {
        return &self.m_SystemResources;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn Reflect(&mut self, shaderSource: &str)
    {
        let constantBufferRegex = Regex::new(r"\s*cbuffer\s+([^{\s]+)\s*:\s*register\(b([0-9]+)\)\s*\{([^{]+)\}").unwrap();
        let variableRegex = Regex::new(r"([^;/\s]+)\s+([^;]+);").unwrap();
        let resourceRegex = Regex::new(r"\s*(Texture2D|TextureCube)\s+([^;\s]+)\s*:\s*register\([st]([0-9]+)\)").unwrap();
        
        // Find all constant buffers
        for bufferMatch in constantBufferRegex.captures_iter(shaderSource)
        {
            let name: &str = bufferMatch.get(1).unwrap().as_str();
            let register: u32 = bufferMatch.get(2).unwrap().as_str().parse::<u32>().unwrap();
            let variables: &str = bufferMatch.get(3).unwrap().as_str();

            let shaderType = ShaderType::ShaderTypeFromString(&name[0..2]);
            let mut constantBuffer = ConstantBufferDeclaration::Create(name, register, shaderType);

            // Process uniforms
            for varMatch in variableRegex.captures_iter(variables)
            {
                let varType: &str = varMatch.get(1).unwrap().as_str();
                let varName: &str = varMatch.get(2).unwrap().as_str();

                let variable = ShaderUniformDeclaration::Create(varName, ShaderUniformType::ShaderUniformFromHLSLType(varType), 
                                                                                        constantBuffer.GetSize(), constantBuffer.GetRegister(), shaderType);
                constantBuffer.AddUniform(variable);
            }

            constantBuffer.Finalize();

            // Set the correct shader constant buffer declaration to the newly created one, based on the buffer name
            if name.contains("VSSystem")
            {
                self.m_VSSystemConstantBufferDeclarations.push(constantBuffer);
            }
            else if name.contains("PSSystem")
            {
                self.m_PSSystemConstantBufferDeclarations.push(constantBuffer);
            }
            else if name.contains("VSMaterial")
            {
                debug_assert!(self.m_VSMaterialConstantBufferDeclaration.is_none(), "Vertex shader material constant buffer already set! Only 1 VS material constant buffer is supported.");
                self.m_VSMaterialConstantBufferDeclaration = Some(constantBuffer);
            }
            else if name.contains("PSMaterial")
            {
                debug_assert!(self.m_PSMaterialConstantBufferDeclaration.is_none(), "Pixel shader material constant buffer already set! Only 1 PS material constant buffer is supported.");
                self.m_PSMaterialConstantBufferDeclaration = Some(constantBuffer);
            }
            else
            {
                debug_assert!(false, "Unknown constant buffer signature!");    
            }
        }

        // Shader resources
        for resMatch in resourceRegex.captures_iter(shaderSource)
        {
            let resourceType: &str = resMatch.get(1).unwrap().as_str();
            let resourceName: &str = resMatch.get(2).unwrap().as_str();
            let register: u32 = resMatch.get(3).unwrap().as_str().parse::<u32>().unwrap();

            let shaderResource = ShaderResourceDeclaration::Create(resourceName, ShaderResourceType::ShaderResourceFromHLSLResource(resourceType), register);
            if resourceName.contains("sys_")
            {
                self.m_SystemResources.push(shaderResource);
            }
            else
            {
                self.m_MaterialResources.push(shaderResource);
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn CreateConstantBuffers(&mut self)
    {
        unsafe
        {
            let vsBufferCount: usize = if self.m_VSMaterialConstantBufferDeclaration.is_some() 
                                       { 
                                           self.m_VSSystemConstantBufferDeclarations.len() + 1
                                       } 
                                       else 
                                       { 
                                           self.m_VSSystemConstantBufferDeclarations.len() 
                                       };

            let psBufferCount: usize = if self.m_PSMaterialConstantBufferDeclaration.is_some() 
                                       { 
                                           self.m_PSSystemConstantBufferDeclarations.len() + 1
                                       } 
                                       else 
                                       { 
                                           self.m_PSSystemConstantBufferDeclarations.len() 
                                       };

            self.m_VSConstantBuffers = vec![None; vsBufferCount];        
            self.m_PSConstantBuffers = vec![None; psBufferCount];        
                      
            // Set up common parameters
            let mut desc = D3D11_BUFFER_DESC::default();
            desc.Usage          = D3D11_USAGE_DYNAMIC;
            desc.BindFlags      = D3D11_BIND_CONSTANT_BUFFER.0;
            desc.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE.0;

            // Create the VS and PS user uniform buffers if they exist
            if self.m_VSMaterialConstantBufferDeclaration.is_some()
            {
                let bufferSize = self.m_VSMaterialConstantBufferDeclaration.as_ref().unwrap().GetSize();
                let register = self.m_VSMaterialConstantBufferDeclaration.as_ref().unwrap().GetRegister();
                desc.ByteWidth = bufferSize;
            
                let mut data = D3D11_SUBRESOURCE_DATA::default();
                data.pSysMem = alloc(Layout::array::<u8>(bufferSize as usize).unwrap()) as _;

                let constantBuffer: ID3D11Buffer = DXCall!(self.m_Device.CreateBuffer(&desc, &data));
                self.m_VSConstantBuffers[register as usize] = Some(constantBuffer);
            }

            if self.m_PSMaterialConstantBufferDeclaration.is_some()
            {
                let bufferSize = self.m_PSMaterialConstantBufferDeclaration.as_ref().unwrap().GetSize();
                let register = self.m_PSMaterialConstantBufferDeclaration.as_ref().unwrap().GetRegister();
                desc.ByteWidth = bufferSize;
            
                let mut data = D3D11_SUBRESOURCE_DATA::default();
                data.pSysMem = alloc(Layout::array::<u8>(bufferSize as usize).unwrap()) as _;

                let constantBuffer: ID3D11Buffer = DXCall!(self.m_Device.CreateBuffer(&desc, &data));
                self.m_PSConstantBuffers[register as usize] = Some(constantBuffer);
            }

            // Create the VS and PS system uniform buffers
            for buffer in self.m_VSSystemConstantBufferDeclarations.iter()
            {
                desc.ByteWidth = buffer.GetSize();
            
                let mut data = D3D11_SUBRESOURCE_DATA::default();
                data.pSysMem = alloc(Layout::array::<u8>(buffer.GetSize() as usize).unwrap()) as _;

                let constantBuffer: ID3D11Buffer = DXCall!(self.m_Device.CreateBuffer(&desc, &data));
                self.m_VSConstantBuffers[buffer.GetRegister() as usize] = Some(constantBuffer);
            }

            for buffer in self.m_PSSystemConstantBufferDeclarations.iter()
            {
                desc.ByteWidth = buffer.GetSize();
            
                let mut data = D3D11_SUBRESOURCE_DATA::default();
                data.pSysMem = alloc(Layout::array::<u8>(buffer.GetSize() as usize).unwrap()) as _;

                let constantBuffer: ID3D11Buffer = DXCall!(self.m_Device.CreateBuffer(&desc, &data));
                self.m_PSConstantBuffers[buffer.GetRegister() as usize] = Some(constantBuffer);
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////// Compute Shader ///////////////////////////////////////////////////////////////////
pub struct ComputeShader
{
    m_Device:                     ID3D11Device,
    m_Filepath:                   String,
    m_ShaderName:                 String,
    m_Blob:                       ID3DBlob,
    m_Handle:                     ID3D11ComputeShader,
    m_ConstantBufferDeclarations: Vec<ConstantBufferDeclaration>,
    m_ConstantBuffers:            Vec<Option<ID3D11Buffer>>,
    m_Resources:                  Vec<ShaderResourceDeclaration>
}

impl ComputeShader
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(filepath: &str) -> RustyRef<ComputeShader>
    {
        unsafe
        {
            let device: ID3D11Device = Renderer::GetGfxContext().GetRef().GetDevice();
            
            let shaderSource: String = fs::read_to_string(filepath).expect("Failed reading file");
            let shaderName: &str = Path::new(filepath).file_stem().unwrap().to_str().unwrap();

            let mut errorBlob: Option<ID3DBlob> = None;

            // Compile compute shader
            let mut csBlob: Option<ID3DBlob> = None;
            D3DCompile(shaderSource.as_ptr() as _, shaderSource.len(), None, null(), None, "CSMain", "cs_5_0", D3DCOMPILE_DEBUG, 0, &mut csBlob, &mut errorBlob);

            if errorBlob.is_some() && errorBlob.as_ref().unwrap().GetBufferSize() > 0
            {
                // Error was found when compiling compute shader
                let errorBlob = errorBlob.unwrap();
                let message = String::from_raw_parts(errorBlob.GetBufferPointer() as *mut u8, errorBlob.GetBufferSize(), errorBlob.GetBufferSize());
                panic!("Compute shader compilation error: {}", message);
            }

            let csBlob: ID3DBlob = csBlob.unwrap();
            let computeShader: ID3D11ComputeShader = DXCall!(device.CreateComputeShader(csBlob.GetBufferPointer(), csBlob.GetBufferSize(), None));

            // Create the shader and do a reflection to get all uniforms and resources
            let mut shader = ComputeShader { 
                m_Device: device.clone(),
                m_Filepath: String::from(filepath),
                m_ShaderName: String::from(shaderName),
                m_Blob: csBlob,
                m_Handle: computeShader,
                m_ConstantBufferDeclarations: vec![],
                m_ConstantBuffers: vec![],
                m_Resources: vec![]
            };

            shader.Reflect(shaderSource.as_str());
            shader.CreateConstantBuffers();

            return RustyRef::CreateRef(shader);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetFilepath(&self) -> &str
    {
        return self.m_Filepath.as_str();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetName(&self) -> &str
    {
        return self.m_ShaderName.as_str();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetVSDataBlob(&self) -> &ID3DBlob
    {
        return &self.m_Blob;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetHandle(&self) -> &ID3D11ComputeShader
    {
        return &self.m_Handle;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetConstantBufferDeclarations(&self) -> &Vec<ConstantBufferDeclaration>
    {
        return &self.m_ConstantBufferDeclarations;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetConstantBufferHandles(&self) -> &Vec<Option<ID3D11Buffer>>
    {
        return &self.m_ConstantBuffers;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetResources(&self) -> &Vec<ShaderResourceDeclaration>
    {
        return &self.m_Resources;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn Reflect(&mut self, shaderSource: &str)
    {
        let constantBufferRegex = Regex::new(r"\s*cbuffer\s+([^{\s]+)\s*:\s*register\(b([0-9]+)\)\s*\{([^{]+)\}").unwrap();
        let variableRegex = Regex::new(r"([^;/\s]+)\s+([^;]+);").unwrap();
        let resourceRegex = Regex::new(r"\s*(RWTexture2D|RWTexture2DArray|Texture2D|TextureCube|SamplerState)(<[^{\s]+>)*\s+([^;\s]+)\s*:\s*register\([stu]([0-9]+)\)").unwrap();
        
        // Find all constant buffers
        for bufferMatch in constantBufferRegex.captures_iter(shaderSource)
        {
            let name: &str = bufferMatch.get(1).unwrap().as_str();
            let register: u32 = bufferMatch.get(2).unwrap().as_str().parse::<u32>().unwrap();
            let variables: &str = bufferMatch.get(3).unwrap().as_str();

            let mut constantBuffer = ConstantBufferDeclaration::Create(name, register, ShaderType::CS);

            // Process uniforms
            for varMatch in variableRegex.captures_iter(variables)
            {
                let varType: &str = varMatch.get(1).unwrap().as_str();
                let varName: &str = varMatch.get(2).unwrap().as_str();

                let variable = ShaderUniformDeclaration::Create(varName, ShaderUniformType::ShaderUniformFromHLSLType(varType), 
                                                                                        constantBuffer.GetSize(), constantBuffer.GetRegister(), ShaderType::CS);
                constantBuffer.AddUniform(variable);
            }

            constantBuffer.Finalize();
            self.m_ConstantBufferDeclarations.push(constantBuffer);
        }

        // Shader resources
        for resMatch in resourceRegex.captures_iter(shaderSource)
        {
            let resourceType: &str = resMatch.get(1).unwrap().as_str();
            let resourceName: &str = resMatch.get(3).unwrap().as_str();
            let register: u32 = resMatch.get(4).unwrap().as_str().parse::<u32>().unwrap();

            let shaderResource = ShaderResourceDeclaration::Create(resourceName, ShaderResourceType::ShaderResourceFromHLSLResource(resourceType), register);
            self.m_Resources.push(shaderResource);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn CreateConstantBuffers(&mut self)
    {
        unsafe
        {
            self.m_ConstantBuffers = vec![None; self.m_ConstantBufferDeclarations.len()];        
                      
            // Set up common parameters
            let mut desc = D3D11_BUFFER_DESC::default();
            desc.Usage          = D3D11_USAGE_DYNAMIC;
            desc.BindFlags      = D3D11_BIND_CONSTANT_BUFFER.0;
            desc.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE.0;

            // Create buffers
            for buffer in self.m_ConstantBufferDeclarations.iter()
            {
                desc.ByteWidth = buffer.GetSize();
            
                let mut data = D3D11_SUBRESOURCE_DATA::default();
                data.pSysMem = alloc(Layout::array::<u8>(buffer.GetSize() as usize).unwrap()) as _;

                let constantBuffer: ID3D11Buffer = DXCall!(self.m_Device.CreateBuffer(&desc, &data));
                self.m_ConstantBuffers[buffer.GetRegister() as usize] = Some(constantBuffer);
            }
        }
    }
}