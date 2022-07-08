#![allow(non_snake_case)]

// Win32
use directx_math::*;

// Core
use crate::core::utils::*;

// Renderer
use crate::renderer::texture::*;
use crate::renderer::renderer::*;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////// Light //////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Default)]
pub struct Light
{
    pub Position:             XMFLOAT4,
    pub Direction:            XMFLOAT4,
    pub Color:                XMFLOAT4,
    pub Intensity:            f32,
    pub ConeAngle:            f32,
    pub ConstAttenuation:     f32,
    pub LinearAttenuation:    f32,
    pub QuadraticAttenuation: f32,
    pub LightType:            u32,
    padding:                  XMFLOAT2,
}

impl Light 
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateDirectionalLight(color: XMFLOAT3, intensity: f32, direction: XMFLOAT3) -> Light
    {
        return Light {
            Position: XMFLOAT4::default(),
            Direction: XMFLOAT4::set(direction.x, direction.y, direction.z, 0.0),
            Color: XMFLOAT4::set(color.x, color.y, color.z, 1.0),
            Intensity: intensity,
            ConeAngle: 0.0,
            ConstAttenuation: 1.0,
            LinearAttenuation: 0.08,
            QuadraticAttenuation: 0.0,
            LightType: 0,
            padding: XMFLOAT2::default()
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreatePointLight(color: XMFLOAT3, intensity: f32, position: XMFLOAT3) -> Light
    {
        return Light {
            Position: XMFLOAT4::set(position.x, position.y, position.z, 1.0),
            Direction: XMFLOAT4::default(),
            Color: XMFLOAT4::set(color.x, color.y, color.z, 1.0),
            Intensity: intensity,
            ConeAngle: 0.0,
            ConstAttenuation: 1.0,
            LinearAttenuation: 0.08,
            QuadraticAttenuation: 0.0,
            LightType: 1,
            padding: XMFLOAT2::default()
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateSpotLight(color: XMFLOAT3, intensity: f32, position: XMFLOAT3, coneAngle: f32) -> Light
    {
        return Light {
            Position: XMFLOAT4::set(position.x, position.y, position.z, 1.0),
            Direction: XMFLOAT4::default(),
            Color: XMFLOAT4::set(color.x, color.y, color.z, 1.0),
            Intensity: intensity,
            ConeAngle: coneAngle,
            ConstAttenuation: 1.0,
            LinearAttenuation: 0.08,
            QuadraticAttenuation: 0.0,
            LightType: 2,
            padding: XMFLOAT2::default()
        };
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////// Environment ////////////////////////////////////////////////////////////////////////

pub const MAX_LIGHT_COUNT: usize = 10;
pub struct Environment
{
    m_EnvironmentMap: RustyRef<Texture>,
    m_IrradianceMap:  RustyRef<Texture>,
    m_Lights:         [Light; MAX_LIGHT_COUNT],
    m_LightIndex:     usize
}

impl Environment
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create() -> Environment
    {
        return Environment {
            m_EnvironmentMap: Renderer::GetBlackTextureCube(),
            m_IrradianceMap:  Renderer::GetBlackTextureCube(),
            m_Lights:         [Light::default(); MAX_LIGHT_COUNT],
            m_LightIndex:     0,
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetEnvironmentMap(&mut self, environmentMap: RustyRef<Texture>, irradianceMap: RustyRef<Texture>)
    {
        self.m_EnvironmentMap = environmentMap;
        self.m_IrradianceMap = irradianceMap;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn AddLight(&mut self, light: Light)
    {
        assert!(self.m_LightIndex < MAX_LIGHT_COUNT, "Too many lights in the scene! Max is {}", MAX_LIGHT_COUNT);
        self.m_Lights[self.m_LightIndex] = light;
        self.m_LightIndex += 1;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetEnvironmentMap(&self) -> RustyRef<Texture>
    {
        return self.m_EnvironmentMap.clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetIrradianceMap(&self) -> RustyRef<Texture>
    {
        return self.m_IrradianceMap.clone();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetLights(&self) -> &[Light; MAX_LIGHT_COUNT]
    {
        return &self.m_Lights;
    }
}