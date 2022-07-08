#![allow(non_snake_case)]

// Win32
use directx_math::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

// Core
use crate::core::input::*;
use crate::core::event::*; 
use crate::core::mouse_event::*;
use crate::core::timestep::*;

pub struct EditorCamera
{
    m_ViewMatrix:      XMMATRIX,
    m_ProjectionMatix: XMMATRIX,

    m_Position:        XMVECTOR,
    m_FocalPoint:      XMVECTOR,

    m_Distance:        f32,
    m_PitchAngle:      f32,
    m_YawAngle:        f32,
    
    m_LastMousePos:    XMFLOAT2,
    m_ViewportSize:    XMFLOAT2,
    m_Fov:             f32,
    m_NearClip:        f32,
    m_FarClip:         f32
}

impl EditorCamera
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(fov: f32, aspectRatio: f32, near: f32, far: f32) -> EditorCamera
    {
        let mut camera = EditorCamera {
            m_ViewMatrix: XMMatrixIdentity(),
            m_ProjectionMatix: XMMatrixPerspectiveFovLH(XMConvertToRadians(fov), aspectRatio, near, far),
            m_Position: XMVectorZero(),
            m_FocalPoint: XMVectorZero(),
            m_Distance: 50.0,
            m_PitchAngle: directx_math::XM_PI / 6.0,
            m_YawAngle: directx_math::XM_PI / 4.0,
            m_LastMousePos: XMFLOAT2::set(0.0, 0.0),
            m_ViewportSize: XMFLOAT2::set(1280.0, 720.0),
            m_Fov: fov,
            m_NearClip: near,
            m_FarClip: far
        };

        camera.UpdateViewMatrix();

        return camera;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnUpdate(&mut self, timestep: Timestep)
    {
        // Get mouse delta
        let mousePos: XMFLOAT2 = Input::GetMousePosition();
        let mut delta: XMFLOAT2 = XMFLOAT2::set(0.0, 0.0);
        XMStoreFloat2(&mut delta, XMVectorSubtract(XMLoadFloat2(&mousePos), XMLoadFloat2(&self.m_LastMousePos)));
        delta.x *= timestep.GetSeconds();
        delta.y *= timestep.GetSeconds();
        self.m_LastMousePos = mousePos;

        // Poll input
        if Input::IsKeyPressed(VK_MENU)
        {
            if Input::IsKeyPressed(VK_LBUTTON)
            {
                self.Rotate(delta);
            }
            else if Input::IsKeyPressed(VK_RBUTTON)
            {
                self.Zoom(delta.y);
            }
            else if Input::IsKeyPressed(VK_MBUTTON)
            {
                self.Pan(delta);
            }
        }

        self.UpdateViewMatrix();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnEvent(&mut self, event: &dyn Event)
    {
        if event.GetType() == EventType::MouseScrolled
        {
            let mouseScrolledEvent: &MouseScrolledEvent = event.AsAny().downcast_ref::<MouseScrolledEvent>().expect("Event is now a MouseScrolledEvent");
            self.OnMouseScrolled(mouseScrolledEvent);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetDistance(&mut self, distance: f32)
    {
        self.m_Distance = distance;
    }
    
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetDistance(&self) -> f32
    {
        return self.m_Distance;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn SetViewportSize(&mut self, width: f32, height: f32)
    {
        self.m_ViewportSize = XMFLOAT2::set(width, height);
        self.UpdateProjectionMatrix();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetViewMatrix(&self) -> &XMMATRIX
    {
        return &self.m_ViewMatrix;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetProjectionMatrix(&self) -> &XMMATRIX
    {
        return &self.m_ProjectionMatix;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetViewProjectionMatrix(&self) -> XMMATRIX
    {
        return XMMatrixMultiply(self.m_ViewMatrix, &self.m_ProjectionMatix);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetPosition(&self) -> XMFLOAT3
    {
        let mut pos = XMFLOAT3::default();
        XMStoreFloat3(&mut pos, self.m_Position);
        return pos;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetOrientation(&self) -> XMVECTOR
    {
        return XMQuaternionRotationRollPitchYaw(self.m_PitchAngle, self.m_YawAngle, 0.0);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetCameraUp(&self) -> XMVECTOR
    {
        let worldUp = XMFLOAT3::set(0.0, 1.0, 0.0);
        return XMVector3Rotate(XMLoadFloat3(&worldUp), self.GetOrientation());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetCameraRight(&self) -> XMVECTOR
    {
        let worldRight = XMFLOAT3::set(1.0, 0.0, 0.0);
        return XMVector3Rotate(XMLoadFloat3(&worldRight), self.GetOrientation());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetCameraForward(&self) -> XMVECTOR
    {
        let worldForward = XMFLOAT3::set(0.0, 0.0, 1.0);
        return XMVector3Rotate(XMLoadFloat3(&worldForward), self.GetOrientation());
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetPitch(&self) -> f32
    {
        return self.m_PitchAngle;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetYaw(&self) -> f32
    {
        return self.m_YawAngle;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn OnMouseScrolled(&mut self, event: &MouseScrolledEvent)
    {
        self.Zoom(event.GetYOffset() as f32 * 0.3);
        self.UpdateViewMatrix();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn UpdateViewMatrix(&mut self)
    {
        self.m_Position = self.CalculatePosition();
        self.m_ViewMatrix = XMMatrixMultiply(XMMatrixRotationQuaternion(self.GetOrientation()), &XMMatrixTranslationFromVector(self.m_Position));
        self.m_ViewMatrix = XMMatrixInverse(None, self.m_ViewMatrix);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn UpdateProjectionMatrix(&mut self)
    {
        let aspectRatio: f32 = self.m_ViewportSize.x / self.m_ViewportSize.y;
        self.m_ProjectionMatix = XMMatrixPerspectiveFovLH(self.m_Fov, aspectRatio, self.m_NearClip, self.m_FarClip);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn Pan(&mut self, mouseDelta: XMFLOAT2)
    {
        let panSpeed: XMFLOAT2 = self.GetPanSpeed();
        let mut direction: XMVECTOR = XMVectorScale(self.GetCameraRight(), -mouseDelta.x * panSpeed.x * self.m_Distance);
        self.m_FocalPoint = XMVectorAdd(self.m_FocalPoint, direction);

        direction = XMVectorScale(self.GetCameraUp(), mouseDelta.y * panSpeed.y * self.m_Distance);
        self.m_FocalPoint = XMVectorAdd(self.m_FocalPoint, direction);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn Rotate(&mut self, mouseDelta: XMFLOAT2)
    {
        let upVector: XMVECTOR = self.GetCameraUp();
        let yawSign: f32 = if XMVectorGetY(upVector) < 0.0 { -1.0 } else { 1.0 };
        self.m_YawAngle += yawSign * mouseDelta.x * self.GetRotationSpeed();
        self.m_PitchAngle += mouseDelta.y * self.GetRotationSpeed();
        
        if self.m_PitchAngle > directx_math::XM_PI / 2.0
        {
            self.m_PitchAngle = directx_math::XM_PI / 2.0;
        }
        else if self.m_PitchAngle < -directx_math::XM_PI / 2.0
        {
            self.m_PitchAngle = -directx_math::XM_PI / 2.0;
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn Zoom(&mut self, mouseDelta: f32)
    {
        self.m_Distance -= mouseDelta * self.GetZoomSpeed();
        if self.m_Distance < 5.0
        {
            self.m_Distance = 5.0;
        }
        if self.m_Distance > 1000.0
        {
            self.m_Distance = 1000.0;
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn CalculatePosition(&mut self) -> XMVECTOR
    {
        let direction: XMVECTOR = XMVectorScale(self.GetCameraForward(), self.m_Distance);
        return XMVectorSubtract(self.m_FocalPoint, direction);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn GetPanSpeed(&self) -> XMFLOAT2
    {
        // Get the pan speed based on the current viewport size
        let x: f32 = XMMin(self.m_ViewportSize.x / 1000.0, 2.4);
        let xFactor = 0.0366 * (x * x) - 0.1778 * x + 0.3021;

        let y: f32 = XMMin(self.m_ViewportSize.y / 1000.0, 2.4);
        let yFactor = 0.0366 * (y * y) - 0.1778 * y + 0.3021;

        return XMFLOAT2::set(xFactor, yFactor);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn GetRotationSpeed(&self) -> f32
    {
        return 1.0;
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn GetZoomSpeed(&self) -> f32
    {
        // Get zoom speed based on the distance from the focal point
        let mut distance: f32 = self.m_Distance * 0.2;
        distance = XMMax(distance, 0.0);
        let mut speed: f32 = distance * distance;
        speed = XMMin(speed, 100.0);

        return speed;
    }
}