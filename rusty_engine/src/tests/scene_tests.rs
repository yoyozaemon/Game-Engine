#![allow(non_snake_case)]

#[cfg(test)]
mod scene_tests
{
    use crate::scene::scene::*;
    use crate::scene::component::*;

    #[test]
    fn SceneSerializeTest()
    {
        std::env::set_current_dir("../").expect("Failed setting working dir");

        let scene = Scene::Create();
        scene.GetRefMut().CreateEntity("TestEntity1");
        scene.GetRefMut().CreateEntity("TestEntity2");
        scene.GetRef().Serialize("rusty_engine/test_scene.rscene");

        let fileExists = std::path::Path::new("rusty_engine/test_scene.rscene").exists();
        std::fs::remove_file("rusty_engine/test_scene.rscene").expect("Failed removing file");
        assert!(fileExists);
    }

    #[test]
    fn FindEntityInSceneTest()
    {
        let scene = Scene::Create();

        // Add test entities to the scene and serialize
        let mut testEntity1 = scene.GetRefMut().CreateEntity("TestEntity1");
        testEntity1.AddComponent(PointLightComponent{
            Color: [1.0, 1.0, 1.0],
            Intensity: 1.0
        });

        let mut testEntity2 = scene.GetRefMut().CreateEntity("TestEntity2");
        testEntity2.AddComponent(DirectionalLightComponent{
            Color: [1.0, 0.0, 1.0],
            Intensity: 2.0
        });

        assert!(scene.GetRefMut().FindEntity("TestEntity1").IsValid());
        assert!(scene.GetRefMut().FindEntity("TestEntity2").IsValid());
    }

    #[test]
    fn SceneDeserializeTest()
    {
        std::env::set_current_dir("../").expect("Failed setting working dir");
        println!("{}", std::env::current_dir().unwrap().as_os_str().to_str().unwrap());
        let scene = Scene::Create();

        // Add test entities to the scene and serialize
        let mut testEntity1 = scene.GetRefMut().CreateEntity("TestEntity1");
        testEntity1.AddComponent(PointLightComponent{
            Color: [1.0, 1.0, 1.0],
            Intensity: 1.0
        });

        let mut testEntity2 = scene.GetRefMut().CreateEntity("TestEntity2");
        testEntity2.AddComponent(DirectionalLightComponent{
            Color: [1.0, 0.0, 1.0],
            Intensity: 2.0
        });

        scene.GetRef().Serialize("test_scene.rscene");

        // Deserialize
        let deserializedScene = Scene::Create();
        deserializedScene.GetRefMut().Deserialize("test_scene.rscene");
        std::fs::remove_file("test_scene.rscene").expect("Failed removing file");

        // Test if entities are the same as in the serialized scene
        let entity1 = scene.GetRefMut().FindEntity("TestEntity1");
        assert!(entity1.IsValid());
        assert!(entity1.HasComponent::<PointLightComponent>());
        assert_eq!(entity1.GetComponent::<PointLightComponent>().Color, [1.0, 1.0, 1.0]);
        assert_eq!(entity1.GetComponent::<PointLightComponent>().Intensity, 1.0);

        let entity2 = scene.GetRefMut().FindEntity("TestEntity2");
        assert!(entity2.IsValid());
        assert!(entity2.HasComponent::<DirectionalLightComponent>());
        assert_eq!(entity2.GetComponent::<DirectionalLightComponent>().Color, [1.0, 0.0, 1.0]);
        assert_eq!(entity2.GetComponent::<DirectionalLightComponent>().Intensity, 2.0);
    }
}