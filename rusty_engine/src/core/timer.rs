#![allow(non_snake_case)]

// std
use std::time::SystemTime;

// Core
use crate::core::timestep::*;

pub struct Timer
{
    m_Start:       SystemTime,
    m_ElapsedTime: f32          // in seconds
}

impl Timer
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create() -> Timer
    {
        return Timer { 
            m_Start: SystemTime::now(), 
            m_ElapsedTime: 0.0 
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Reset(&mut self)
    {
        self.m_Start = SystemTime::now();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Stop(&mut self)
    {
        self.m_ElapsedTime = self.m_Start.elapsed().unwrap().as_secs_f32();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetElapsedTime(&mut self) -> Timestep
    {
        return Timestep::Create(self.m_ElapsedTime);
    }
}