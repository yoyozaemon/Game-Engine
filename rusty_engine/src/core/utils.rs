#![allow(non_snake_case)]

// std
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;

pub struct RustyRef<T>
{
    m_Ptr: Option<Arc<RefCell<T>>>
}

impl<T> RustyRef<T>
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateRef(object: T) -> RustyRef<T>
    {
        return RustyRef {
            m_Ptr: Some(Arc::new(RefCell::new(object)))
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateEmpty() -> RustyRef<T>
    {
        return RustyRef { m_Ptr: None };    
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn IsValid(&self) -> bool
    {
        return self.m_Ptr.is_some();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRef(&self) -> Ref<'_, T>
    {
        if self.m_Ptr.is_none()
        {
            panic!("Reference is uninitialized!");
        }
        else 
        {
            return (*self.m_Ptr.as_ref().unwrap()).borrow();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRefMut(&self) -> RefMut<'_, T>
    {
        if self.m_Ptr.is_none()
        {
            panic!("Reference is uninitialized!");
        }
        else 
        {
            return (*self.m_Ptr.as_ref().unwrap()).borrow_mut();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRaw(&self) -> *const T
    {
        if self.m_Ptr.is_none()
        {
            return std::ptr::null();
        }
        else
        {
            return &*self.m_Ptr.as_ref().unwrap().borrow() as *const T;
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetRawMut(&self) -> *mut T
    {
        if self.m_Ptr.is_none()
        {
            return std::ptr::null_mut();
        }
        else
        {
            return &mut *self.m_Ptr.as_ref().unwrap().borrow_mut() as *mut T;
        }
    }
}

impl<T> Clone for RustyRef<T>
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn clone(&self) -> RustyRef<T>
    {
        let ptr: Option<Arc<RefCell<T>>> = if self.m_Ptr.is_some() { Some(Arc::clone(&self.m_Ptr.as_ref().unwrap())) } else { None };

        return RustyRef {
            m_Ptr: ptr
        };
    }
}

// ------------------------------------------------------------------- Macros --------------------------------------------------------------------------------
macro_rules! DXCall {
    ($hr:expr) => {
        match $hr
        {
            Ok(value) => value,
            Err(err) => panic!("Invalid HRESULT: {}\nError: {}", err.code().0, err.message())
        }
    }
}