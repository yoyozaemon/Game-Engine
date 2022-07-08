#![allow(non_snake_case)]

// std
use std::ptr::null_mut;

// legion
use legion::EntityStore;

// Scene
use crate::scene::scene::*;

#[derive(Clone, Copy)]
pub struct Entity
{
    m_ID: Option<legion::Entity>,
    m_Scene: *mut Scene
}

impl Entity
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn Create(id: legion::Entity, scene: *mut Scene) -> Entity
    {
        return Entity {
            m_ID: Some(id),
            m_Scene: scene
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn CreateEmpty() -> Entity
    {
        return Entity {
            m_ID: None,
            m_Scene: null_mut()
        };
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn AddComponent<ComponentType: legion::storage::Component>(&mut self, component: ComponentType)
    {
        unsafe
        {
            let entry = self.m_Scene.as_mut().unwrap().GetWorldMut().entry(self.m_ID.unwrap());
            debug_assert!(entry.is_some(), "Entity does not exist in the scene!");
            let mut entry = entry.unwrap();

            debug_assert!(!entry.archetype().layout().has_component::<ComponentType>(), "Entity already has that component!");
            entry.add_component(component);
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn RemoveComponent<ComponentType: legion::storage::Component>(&mut self)
    {
        unsafe
        {
            let entry = self.m_Scene.as_mut().unwrap().GetWorldMut().entry(self.m_ID.unwrap());
            debug_assert!(entry.is_some(), "Entity does not exist in the scene!");
            let mut entry = entry.unwrap();

            debug_assert!(entry.archetype().layout().has_component::<ComponentType>(), "Entity does not have that component!");
            entry.remove_component::<ComponentType>();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn HasComponent<ComponentType: legion::storage::Component>(&self) -> bool
    {
        unsafe
        {
            let entry = self.m_Scene.as_mut().unwrap().GetWorld().entry_ref(self.m_ID.unwrap());
            debug_assert!(entry.is_ok(), "Entity does not exist in the scene!");
            
            return entry.unwrap().archetype().layout().has_component::<ComponentType>()
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetComponent<ComponentType: legion::storage::Component>(&self) -> &ComponentType
    {
        unsafe
        {
            let entry = self.m_Scene.as_mut().unwrap().GetWorld().entry_ref(self.m_ID.unwrap());
            debug_assert!(entry.is_ok(), "Entity does not exist in the scene!");
            
            let component = entry.unwrap().into_component::<ComponentType>();
            debug_assert!(component.is_ok(), "Component does not exist!");

            return component.unwrap();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetComponentMut<ComponentType: legion::storage::Component>(&mut self) -> &mut ComponentType
    {
        unsafe
        {
            let entry = self.m_Scene.as_mut().unwrap().GetWorldMut().entry_mut(self.m_ID.unwrap());
            debug_assert!(entry.is_ok(), "Entity does not exist in the scene!");
            
            let component = entry.unwrap().into_component_mut::<ComponentType>();
            debug_assert!(component.is_ok(), "Component does not exist!");

            return component.unwrap();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn IsValid(&self) -> bool
    {
        return self.m_ID.is_some() && self.m_Scene != null_mut();
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn GetID(&self) -> legion::Entity
    {
        return self.m_ID.unwrap();
    }
}