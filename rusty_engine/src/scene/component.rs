#![allow(non_snake_case)]

// Win32
use directx_math::*;

// Serialization
use serde::Deserialize;
use serde::Serialize;

// ---------------------------------------------------------------- Tag Component ---------------------------------------------------------------- //
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagComponent
{
    pub Tag: String
}

impl Default for TagComponent
{
    fn default() -> TagComponent
    {
        return TagComponent {
            Tag: String::from("Unnamed")
        }
    }
}

unsafe impl Send for TagComponent {}
unsafe impl Sync for TagComponent {}

// ---------------------------------------------------------------- Transform Component ---------------------------------------------------------- //
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformComponent
{
    pub Position: [f32; 3],
    pub Rotation: [f32; 3],
    pub Scale:    [f32; 3],
}

impl TransformComponent
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Transform(&self) -> XMMATRIX
    {
        let translation: XMMATRIX = XMMatrixTranslation(self.Position[0], self.Position[1], self.Position[2]);
        let rotationX: XMMATRIX = XMMatrixRotationX(XMConvertToRadians(self.Rotation[0]));
        let rotationY: XMMATRIX = XMMatrixRotationY(XMConvertToRadians(self.Rotation[1]));
        let rotationZ: XMMATRIX = XMMatrixRotationZ(XMConvertToRadians(self.Rotation[2]));
        let scale: XMMATRIX = XMMatrixScaling(self.Scale[0], self.Scale[1], self.Scale[2]);

        let mut transform: XMMATRIX = XMMatrixIdentity();
        transform = XMMatrixMultiply(translation, &transform);
        transform = XMMatrixMultiply(rotationZ, &transform);
        transform = XMMatrixMultiply(rotationY, &transform);
        transform = XMMatrixMultiply(rotationX, &transform);
        transform = XMMatrixMultiply(scale, &transform);

        return transform;
    }
}

impl Default for TransformComponent
{
    fn default() -> TransformComponent
    {
        return TransformComponent {
            Position: [0.0, 0.0, 0.0],
            Rotation: [0.0, 0.0, 0.0],
            Scale:    [1.0, 1.0, 1.0]
        }
    }
}

unsafe impl Send for TransformComponent {}
unsafe impl Sync for TransformComponent {}

// ------------------------------------------------------------------- Mesh Component ------------------------------------------------------------ //
#[derive(Clone, Serialize, Deserialize)]
pub struct MeshComponent
{
    pub MeshPath:  String,
}

impl Default for MeshComponent
{
    fn default() -> MeshComponent
    {
        return MeshComponent {
            MeshPath: String::new(),
        }
    }
}

unsafe impl Send for MeshComponent {}
unsafe impl Sync for MeshComponent {}

// ------------------------------------------------------------------- Sky Light Component ------------------------------------------------------------ //
#[derive(Clone, Serialize, Deserialize)]
pub struct SkyLightComponent
{
    pub EnvironmentMapPath: String,
    pub Intensity:          f32
}

impl Default for SkyLightComponent
{
    fn default() -> SkyLightComponent
    {
        return SkyLightComponent {
            EnvironmentMapPath: String::new(),
            Intensity: 1.0     
        }
    }
}

unsafe impl Send for SkyLightComponent {}
unsafe impl Sync for SkyLightComponent {}

// ------------------------------------------------------------------- Directional Light Component ------------------------------------------------------------ //
#[derive(Clone, Serialize, Deserialize)]
pub struct DirectionalLightComponent
{
    pub Color:     [f32; 3],
    pub Intensity: f32
}

impl Default for DirectionalLightComponent
{
    fn default() -> DirectionalLightComponent
    {
        return DirectionalLightComponent {
            Color: [1.0, 1.0, 1.0],
            Intensity: 1.0
        }
    }
}

unsafe impl Send for DirectionalLightComponent {}
unsafe impl Sync for DirectionalLightComponent {}

// ------------------------------------------------------------------- Point Light Component ------------------------------------------------------------ //
#[derive(Clone, Serialize, Deserialize)]
pub struct PointLightComponent
{
    pub Color:     [f32; 3],
    pub Intensity: f32
}

impl Default for PointLightComponent
{
    fn default() -> PointLightComponent
    {
        return PointLightComponent {
            Color: [1.0, 1.0, 1.0],
            Intensity: 1.0
        }
    }
}

unsafe impl Send for PointLightComponent {}
unsafe impl Sync for PointLightComponent {}