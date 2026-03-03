//! Integration and thread safety tests for the schedule module.

use std::any::TypeId;
use std::collections::HashMap;

use crate::ecs::schedule::*;
use crate::ecs::World;

// ====================================================================
// Thread Safety Tests
// ====================================================================

mod thread_safety {
    use super::*;

    #[test]
    fn test_core_stage_send() {
        fn assert_send<T: Send>() {}
        assert_send::<CoreStage>();
    }

    #[test]
    fn test_core_stage_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<CoreStage>();
    }

    #[test]
    fn test_stage_label_id_send() {
        fn assert_send<T: Send>() {}
        assert_send::<StageLabelId>();
    }

    #[test]
    fn test_stage_label_id_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<StageLabelId>();
    }

    #[test]
    fn test_stage_position_send() {
        fn assert_send<T: Send>() {}
        assert_send::<StagePosition>();
    }

    #[test]
    fn test_stage_position_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<StagePosition>();
    }
}

// ====================================================================
// Stage Trait Tests
// ====================================================================

mod stage_trait {
    use super::*;

    struct EmptyStage {
        name: String,
        run_count: u32,
    }

    impl Stage for EmptyStage {
        fn name(&self) -> &str {
            &self.name
        }
        fn run(&mut self, _world: &mut World) {
            self.run_count += 1;
        }
        fn system_count(&self) -> usize {
            0
        }
    }

    #[test]
    fn test_stage_trait_name() {
        let stage = EmptyStage {
            name: "TestStage".to_string(),
            run_count: 0,
        };
        assert_eq!(stage.name(), "TestStage");
    }

    #[test]
    fn test_stage_trait_run() {
        let mut stage = EmptyStage {
            name: "TestStage".to_string(),
            run_count: 0,
        };
        let mut world = World::new();
        assert_eq!(stage.run_count, 0);
        stage.run(&mut world);
        assert_eq!(stage.run_count, 1);
        stage.run(&mut world);
        assert_eq!(stage.run_count, 2);
    }

    #[test]
    fn test_stage_trait_is_empty() {
        let stage = EmptyStage {
            name: "TestStage".to_string(),
            run_count: 0,
        };
        assert!(stage.is_empty());
        assert_eq!(stage.system_count(), 0);
    }

    #[test]
    fn test_stage_trait_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EmptyStage>();
    }
}

// ====================================================================
// Integration Tests
// ====================================================================

mod integration {
    use super::*;

    #[test]
    fn test_stage_label_in_hashmap() {
        let mut stages: HashMap<StageLabelId, Vec<&str>> = HashMap::new();
        stages.insert(CoreStage::PreUpdate.into(), vec!["input_system"]);
        stages.insert(CoreStage::Update.into(), vec!["movement", "ai"]);
        stages.insert(CoreStage::Render.into(), vec!["sprite_render"]);
        assert_eq!(stages.len(), 3);
        let update_systems = stages.get(&CoreStage::Update.into()).unwrap();
        assert_eq!(update_systems, &vec!["movement", "ai"]);
    }

    #[test]
    fn test_custom_and_core_stages_together() {
        #[derive(Debug, Clone, Copy)]
        struct NetworkStage;

        impl StageLabel for NetworkStage {
            fn label_id(&self) -> TypeId {
                TypeId::of::<Self>()
            }
            fn label_name(&self) -> &'static str {
                "NetworkStage"
            }
            fn dyn_clone(&self) -> Box<dyn StageLabel> {
                Box::new(*self)
            }
        }

        let stages: Vec<Box<dyn StageLabel>> = vec![
            Box::new(CoreStage::PreUpdate),
            Box::new(NetworkStage),
            Box::new(CoreStage::Update),
        ];
        assert_eq!(stages.len(), 3);
        assert_eq!(stages[0].label_name(), "PreUpdate");
        assert_eq!(stages[1].label_name(), "NetworkStage");
        assert_eq!(stages[2].label_name(), "Update");
        assert_ne!(stages[0].label_id(), stages[1].label_id());
        assert_ne!(stages[1].label_id(), stages[2].label_id());
    }

    #[test]
    fn test_stage_iteration_order() {
        let execution_order: Vec<CoreStage> = CoreStage::all().to_vec();
        for window in execution_order.windows(2) {
            let current = window[0];
            let next = window[1];
            assert!(current < next);
            assert_eq!(current.next(), Some(next));
            assert_eq!(next.previous(), Some(current));
        }
    }

    #[test]
    fn test_stage_filtering() {
        let logic_stages: Vec<_> = CoreStage::all()
            .into_iter()
            .filter(|s| s.is_logic())
            .collect();
        assert_eq!(logic_stages.len(), 3);
        assert!(logic_stages.contains(&CoreStage::PreUpdate));
        assert!(logic_stages.contains(&CoreStage::Update));
        assert!(logic_stages.contains(&CoreStage::PostUpdate));

        let render_stages: Vec<_> = CoreStage::all()
            .into_iter()
            .filter(|s| s.is_render())
            .collect();
        assert_eq!(render_stages.len(), 3);
        assert!(render_stages.contains(&CoreStage::PreRender));
        assert!(render_stages.contains(&CoreStage::Render));
        assert!(render_stages.contains(&CoreStage::PostRender));
    }
}
