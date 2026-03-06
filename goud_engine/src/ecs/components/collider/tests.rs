//! Tests for ColliderShape and Collider component.

#[cfg(test)]
mod shape_tests {
    use crate::core::math::{Rect, Vec2};
    use crate::ecs::components::{Collider, ColliderShape};
    use crate::ecs::Component;

    // -------------------------------------------------------------------------
    // ColliderShape Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_collider_shape_type_names() {
        assert_eq!(ColliderShape::Circle { radius: 1.0 }.type_name(), "Circle");
        assert_eq!(
            ColliderShape::Aabb {
                half_extents: Vec2::one()
            }
            .type_name(),
            "AABB"
        );
        assert_eq!(
            ColliderShape::Obb {
                half_extents: Vec2::one()
            }
            .type_name(),
            "OBB"
        );
        assert_eq!(
            ColliderShape::Capsule {
                half_height: 1.0,
                radius: 0.5
            }
            .type_name(),
            "Capsule"
        );
        assert_eq!(
            ColliderShape::Polygon {
                vertices: vec![Vec2::zero(), Vec2::unit_x(), Vec2::unit_y()]
            }
            .type_name(),
            "Polygon"
        );
    }

    #[test]
    fn test_collider_shape_predicates() {
        let circle = ColliderShape::Circle { radius: 1.0 };
        assert!(circle.is_circle());
        assert!(!circle.is_aabb());
        assert!(!circle.is_obb());
        assert!(!circle.is_capsule());
        assert!(!circle.is_polygon());

        let aabb_shape = ColliderShape::Aabb {
            half_extents: Vec2::one(),
        };
        assert!(!aabb_shape.is_circle());
        assert!(aabb_shape.is_aabb());
        assert!(!aabb_shape.is_obb());
    }

    #[test]
    fn test_collider_shape_is_valid() {
        // Valid shapes
        assert!(ColliderShape::Circle { radius: 1.0 }.is_valid());
        assert!(ColliderShape::Aabb {
            half_extents: Vec2::one()
        }
        .is_valid());
        assert!(ColliderShape::Capsule {
            half_height: 1.0,
            radius: 0.5
        }
        .is_valid());
        assert!(ColliderShape::Polygon {
            vertices: vec![Vec2::zero(), Vec2::unit_x(), Vec2::unit_y()]
        }
        .is_valid());

        // Invalid shapes
        assert!(!ColliderShape::Circle { radius: 0.0 }.is_valid());
        assert!(!ColliderShape::Circle { radius: -1.0 }.is_valid());
        assert!(!ColliderShape::Aabb {
            half_extents: Vec2::zero()
        }
        .is_valid());
        assert!(!ColliderShape::Polygon {
            vertices: vec![Vec2::zero(), Vec2::unit_x()]
        }
        .is_valid());
    }

    #[test]
    fn test_collider_shape_compute_aabb_circle() {
        let shape = ColliderShape::Circle { radius: 2.0 };
        let aabb_rect = shape.compute_aabb();
        assert_eq!(aabb_rect.x, -2.0);
        assert_eq!(aabb_rect.y, -2.0);
        assert_eq!(aabb_rect.width, 4.0);
        assert_eq!(aabb_rect.height, 4.0);
    }

    #[test]
    fn test_collider_shape_compute_aabb_box() {
        let shape = ColliderShape::Aabb {
            half_extents: Vec2::new(3.0, 2.0),
        };
        let aabb_rect = shape.compute_aabb();
        assert_eq!(aabb_rect.x, -3.0);
        assert_eq!(aabb_rect.y, -2.0);
        assert_eq!(aabb_rect.width, 6.0);
        assert_eq!(aabb_rect.height, 4.0);
    }

    #[test]
    fn test_collider_shape_compute_aabb_capsule() {
        let shape = ColliderShape::Capsule {
            half_height: 1.0,
            radius: 0.5,
        };
        let aabb_rect = shape.compute_aabb();
        assert_eq!(aabb_rect.x, -0.5);
        assert_eq!(aabb_rect.y, -1.5);
        assert_eq!(aabb_rect.width, 1.0);
        assert_eq!(aabb_rect.height, 3.0);
    }

    #[test]
    fn test_collider_shape_compute_aabb_polygon() {
        let shape = ColliderShape::Polygon {
            vertices: vec![
                Vec2::new(-1.0, -1.0),
                Vec2::new(2.0, 0.0),
                Vec2::new(0.0, 3.0),
            ],
        };
        let aabb_rect = shape.compute_aabb();
        assert_eq!(aabb_rect.x, -1.0);
        assert_eq!(aabb_rect.y, -1.0);
        assert_eq!(aabb_rect.width, 3.0);
        assert_eq!(aabb_rect.height, 4.0);
    }

    #[test]
    fn test_collider_shape_compute_aabb_empty_polygon() {
        let shape = ColliderShape::Polygon { vertices: vec![] };
        let aabb_rect = shape.compute_aabb();
        // Should return unit rect for empty polygon
        assert_eq!(aabb_rect, Rect::unit());
    }

    #[test]
    fn test_collider_shape_default() {
        let shape = ColliderShape::default();
        assert!(shape.is_circle());
        if let ColliderShape::Circle { radius } = shape {
            assert_eq!(radius, 1.0);
        }
    }

    // -------------------------------------------------------------------------
    // Collider Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_collider_circle() {
        let collider = Collider::circle(1.5);
        assert!(collider.shape().is_circle());
        assert_eq!(collider.restitution(), 0.3);
        assert_eq!(collider.friction(), 0.5);
        assert!(!collider.is_sensor());
        assert!(collider.is_enabled());
    }

    #[test]
    fn test_collider_aabb() {
        let collider = Collider::aabb(Vec2::new(2.0, 3.0));
        assert!(collider.shape().is_aabb());
    }

    #[test]
    fn test_collider_obb() {
        let collider = Collider::obb(Vec2::new(2.0, 3.0));
        assert!(collider.shape().is_obb());
    }

    #[test]
    fn test_collider_capsule() {
        let collider = Collider::capsule(1.0, 0.5);
        assert!(collider.shape().is_capsule());
    }

    #[test]
    fn test_collider_polygon() {
        let collider = Collider::polygon(vec![Vec2::zero(), Vec2::unit_x(), Vec2::new(0.5, 1.0)]);
        assert!(collider.shape().is_polygon());
    }

    #[test]
    #[should_panic(expected = "must have at least 3 vertices")]
    fn test_collider_polygon_panics_with_too_few_vertices() {
        Collider::polygon(vec![Vec2::zero(), Vec2::unit_x()]);
    }

    #[test]
    fn test_collider_builder_pattern() {
        let collider = Collider::circle(1.0)
            .with_restitution(0.8)
            .with_friction(0.1)
            .with_density(2.5)
            .with_layer(0b0010)
            .with_mask(0b1100)
            .with_is_sensor(true)
            .with_enabled(false);

        assert_eq!(collider.restitution(), 0.8);
        assert_eq!(collider.friction(), 0.1);
        assert_eq!(collider.density(), Some(2.5));
        assert_eq!(collider.layer(), 0b0010);
        assert_eq!(collider.mask(), 0b1100);
        assert!(collider.is_sensor());
        assert!(!collider.is_enabled());
    }

    #[test]
    fn test_collider_restitution_clamping() {
        let collider = Collider::circle(1.0).with_restitution(1.5);
        assert_eq!(collider.restitution(), 1.0);

        let collider = Collider::circle(1.0).with_restitution(-0.5);
        assert_eq!(collider.restitution(), 0.0);
    }

    #[test]
    fn test_collider_friction_clamping() {
        let collider = Collider::circle(1.0).with_friction(-1.0);
        assert_eq!(collider.friction(), 0.0);

        // Friction can exceed 1.0
        let collider = Collider::circle(1.0).with_friction(2.0);
        assert_eq!(collider.friction(), 2.0);
    }

    #[test]
    fn test_collider_density_clamping() {
        let collider = Collider::circle(1.0).with_density(-1.0);
        assert_eq!(collider.density(), Some(0.0));
    }

    #[test]
    fn test_collider_mutators() {
        let mut collider = Collider::circle(1.0);

        collider.set_restitution(0.9);
        assert_eq!(collider.restitution(), 0.9);

        collider.set_friction(0.2);
        assert_eq!(collider.friction(), 0.2);

        collider.set_density(Some(3.0));
        assert_eq!(collider.density(), Some(3.0));

        collider.set_layer(0b0100);
        assert_eq!(collider.layer(), 0b0100);

        collider.set_mask(0b1000);
        assert_eq!(collider.mask(), 0b1000);

        collider.set_is_sensor(true);
        assert!(collider.is_sensor());

        collider.set_enabled(false);
        assert!(!collider.is_enabled());
    }

    #[test]
    fn test_collider_can_collide_with() {
        let collider_a = Collider::circle(1.0).with_layer(0b0001).with_mask(0b0010);
        let collider_b = Collider::circle(1.0).with_layer(0b0010).with_mask(0b0001);
        let collider_c = Collider::circle(1.0).with_layer(0b0100).with_mask(0b1000);

        // A and B should collide (mutual layer/mask match)
        assert!(collider_a.can_collide_with(&collider_b));
        assert!(collider_b.can_collide_with(&collider_a));

        // A and C should not collide (no layer/mask overlap)
        assert!(!collider_a.can_collide_with(&collider_c));
        assert!(!collider_c.can_collide_with(&collider_a));

        // B and C should not collide
        assert!(!collider_b.can_collide_with(&collider_c));
    }

    #[test]
    fn test_collider_compute_aabb() {
        let collider = Collider::circle(2.0);
        let aabb_rect = collider.compute_aabb();
        assert_eq!(aabb_rect.x, -2.0);
        assert_eq!(aabb_rect.y, -2.0);
        assert_eq!(aabb_rect.width, 4.0);
        assert_eq!(aabb_rect.height, 4.0);
    }

    #[test]
    fn test_collider_set_shape() {
        let mut collider = Collider::circle(1.0);
        assert!(collider.shape().is_circle());

        collider.set_shape(ColliderShape::Aabb {
            half_extents: Vec2::one(),
        });
        assert!(collider.shape().is_aabb());
    }

    #[test]
    fn test_collider_default() {
        let collider = Collider::default();
        assert!(collider.shape().is_circle());
        assert_eq!(collider.restitution(), 0.3);
        assert_eq!(collider.friction(), 0.5);
        assert!(!collider.is_sensor());
        assert!(collider.is_enabled());
    }

    #[test]
    fn test_collider_display() {
        let collider = Collider::circle(1.0);
        let display = format!("{}", collider);
        assert!(display.contains("Circle"));
        assert!(display.contains("restitution"));
        assert!(display.contains("friction"));

        let sensor = Collider::circle(1.0).with_is_sensor(true);
        let display = format!("{}", sensor);
        assert!(display.contains("sensor"));

        let disabled = Collider::circle(1.0).with_enabled(false);
        let display = format!("{}", disabled);
        assert!(display.contains("disabled"));
    }

    #[test]
    fn test_collider_is_component() {
        fn assert_component<T: Component>() {}
        assert_component::<Collider>();
    }

    #[test]
    fn test_collider_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Collider>();
    }

    #[test]
    fn test_collider_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Collider>();
    }

    #[test]
    fn test_collider_clone() {
        let collider = Collider::circle(1.0).with_restitution(0.8);
        let cloned = collider.clone();
        assert_eq!(collider, cloned);
    }

    #[test]
    fn test_collider_shape_clone() {
        let shape = ColliderShape::Circle { radius: 1.0 };
        let cloned = shape.clone();
        assert_eq!(shape, cloned);
    }
}
