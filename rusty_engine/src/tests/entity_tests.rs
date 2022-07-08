#![allow(non_snake_case)]

#[cfg(test)]
mod entity_tests
{
    use crate::scene::entity::*;
    use crate::scene::scene::*;
    use crate::scene::component::*;

    #[test]
    fn EntityAddComponentTest()
    {
        let scene = Scene::Create();
        let mut entity = scene.GetRefMut().CreateEntity("TestEntity");
        entity.AddComponent(PointLightComponent{
            Color: [1.0, 1.0, 1.0],
            Intensity: 1.0
        });

        assert!(entity.HasComponent::<PointLightComponent>());
    }

    #[test]
    fn EntityRemoveComponentTest()
    {
        let scene = Scene::Create();
        let mut entity = scene.GetRefMut().CreateEntity("TestEntity");
        entity.AddComponent(PointLightComponent{
            Color: [1.0, 1.0, 1.0],
            Intensity: 1.0
        });

        assert!(entity.HasComponent::<PointLightComponent>());
        entity.RemoveComponent::<PointLightComponent>();
        assert!(!entity.HasComponent::<PointLightComponent>());
    }
}