use cgmath::Vector3;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum LightType {
    Point = 0,
    Directional = 1,
    Spot = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub id: u32,
    pub light_type: LightType,
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub temperature: f32,
    pub range: f32,
    pub spot_angle: f32, // For spot lights
    pub enabled: bool,
}

impl Light {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u32,
        light_type: LightType,
        position: Vector3<f32>,
        direction: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
        temperature: f32,
        range: f32,
        spot_angle: f32,
    ) -> Self {
        Light {
            id,
            light_type,
            position,
            direction,
            color,
            intensity,
            temperature,
            range,
            spot_angle,
            enabled: true,
        }
    }

    pub fn get_color_with_temperature(&self) -> Vector3<f32> {
        // Convert temperature to RGB using approximation
        // Temperature should be in Kelvin (1000K - 40000K)
        let temp = self.temperature.clamp(1000.0, 40000.0) / 100.0;

        let mut red = 1.0;
        let green;
        let blue;

        if temp <= 66.0 {
            green = 0.390_081_58 * temp.ln() - 0.631_841_4;
            if temp <= 19.0 {
                blue = 0.0;
            } else {
                blue = 0.543_206_8 * (temp - 10.0).ln() - 1.196_254_1;
            }
        } else {
            red = 1.292_936_2 * (temp - 60.0).powf(-0.133_204_76);
            green = 1.129_890_9 * (temp - 60.0).powf(-0.075_514_846);
            blue = 1.0;
        }

        // Clamp values and multiply with the light's color
        let temperature_color = Vector3::new(
            red.clamp(0.0, 1.0),
            green.clamp(0.0, 1.0),
            blue.clamp(0.0, 1.0),
        );

        Vector3::new(
            self.color.x * temperature_color.x,
            self.color.y * temperature_color.y,
            self.color.z * temperature_color.z,
        )
    }
}

#[derive(Debug)]
pub struct LightManager {
    lights: Vec<Light>,
    next_light_id: u32,
}

impl LightManager {
    pub fn new() -> Self {
        LightManager {
            lights: Vec::new(),
            next_light_id: 1,
        }
    }

    pub fn add_light(&mut self, mut light: Light) -> u32 {
        let id = self.next_light_id;
        light.id = id;
        self.next_light_id += 1;
        self.lights.push(light);
        id
    }

    pub fn remove_light(&mut self, light_id: u32) {
        self.lights.retain(|light| light.id != light_id);
    }

    #[allow(dead_code)]
    pub fn get_light(&self, light_id: u32) -> Option<&Light> {
        self.lights.iter().find(|light| light.id == light_id)
    }

    pub fn get_light_mut(&mut self, light_id: u32) -> Option<&mut Light> {
        self.lights.iter_mut().find(|light| light.id == light_id)
    }

    pub fn get_all_lights(&self) -> &[Light] {
        &self.lights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_new() {
        let light = Light::new(
            1,
            LightType::Point,
            Vector3::new(1.0, 2.0, 3.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.5,
            5000.0,
            10.0,
            45.0,
        );

        assert_eq!(light.id, 1);
        assert!(matches!(light.light_type, LightType::Point));
        assert_eq!(light.position, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(light.direction, Vector3::new(0.0, -1.0, 0.0));
        assert_eq!(light.color, Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(light.intensity, 1.5);
        assert_eq!(light.temperature, 5000.0);
        assert_eq!(light.range, 10.0);
        assert_eq!(light.spot_angle, 45.0);
        assert!(light.enabled);
    }

    #[test]
    fn test_light_types() {
        let point_light = Light::new(
            1,
            LightType::Point,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            5000.0,
            10.0,
            0.0,
        );
        assert!(matches!(point_light.light_type, LightType::Point));

        let directional_light = Light::new(
            2,
            LightType::Directional,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            5000.0,
            0.0,
            0.0,
        );
        assert!(matches!(
            directional_light.light_type,
            LightType::Directional
        ));

        let spot_light = Light::new(
            3,
            LightType::Spot,
            Vector3::new(0.0, 5.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            5000.0,
            20.0,
            30.0,
        );
        assert!(matches!(spot_light.light_type, LightType::Spot));
    }

    #[test]
    fn test_light_color_temperature_low() {
        let light = Light::new(
            1,
            LightType::Point,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            1500.0, // Low temperature (warm/reddish)
            10.0,
            0.0,
        );

        let color = light.get_color_with_temperature();
        // Low temperatures should have higher red values
        assert!(
            color.x > color.z,
            "Red should be stronger than blue for low temperatures"
        );
    }

    #[test]
    fn test_light_color_temperature_medium() {
        let light = Light::new(
            1,
            LightType::Point,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            5000.0, // Medium temperature (neutral white)
            10.0,
            0.0,
        );

        let color = light.get_color_with_temperature();
        // All color components should be relatively balanced
        assert!(color.x > 0.0 && color.x <= 1.0);
        assert!(color.y > 0.0 && color.y <= 1.0);
        assert!(color.z > 0.0 && color.z <= 1.0);
    }

    #[test]
    fn test_light_color_temperature_high() {
        let light = Light::new(
            1,
            LightType::Point,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            10000.0, // High temperature (cool/bluish)
            10.0,
            0.0,
        );

        let color = light.get_color_with_temperature();
        // High temperatures should have blue at maximum
        assert_eq!(
            color.z, 1.0,
            "Blue should be at maximum for high temperatures"
        );
    }

    #[test]
    fn test_light_color_temperature_clamping() {
        // Test below minimum temperature
        let light_low = Light::new(
            1,
            LightType::Point,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            500.0, // Below 1000K minimum
            10.0,
            0.0,
        );

        let color_low = light_low.get_color_with_temperature();
        // Should be clamped to 1000K behavior
        assert!(color_low.x <= 1.0 && color_low.y <= 1.0 && color_low.z <= 1.0);

        // Test above maximum temperature
        let light_high = Light::new(
            1,
            LightType::Point,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            50000.0, // Above 40000K maximum
            10.0,
            0.0,
        );

        let color_high = light_high.get_color_with_temperature();
        // Should be clamped to 40000K behavior
        assert!(color_high.x <= 1.0 && color_high.y <= 1.0 && color_high.z <= 1.0);
    }

    #[test]
    fn test_light_color_multiplication() {
        // Test that base color is properly multiplied with temperature color
        let light = Light::new(
            1,
            LightType::Point,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.5, 0.7, 0.3), // Non-white base color
            1.0,
            5000.0,
            10.0,
            0.0,
        );

        let color = light.get_color_with_temperature();
        // Result should be affected by both base color and temperature
        assert!(color.x <= 0.5, "Red should not exceed base color red");
        assert!(color.y <= 0.7, "Green should not exceed base color green");
        assert!(color.z <= 0.3, "Blue should not exceed base color blue");
    }

    #[test]
    fn test_light_manager_new() {
        let manager = LightManager::new();
        assert_eq!(manager.lights.len(), 0);
        assert_eq!(manager.next_light_id, 1);
    }

    #[test]
    fn test_light_manager_add_light() {
        let mut manager = LightManager::new();

        let light = Light::new(
            999, // This ID should be overwritten
            LightType::Point,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            5000.0,
            10.0,
            0.0,
        );

        let id = manager.add_light(light);
        assert_eq!(id, 1);
        assert_eq!(manager.lights.len(), 1);
        assert_eq!(manager.lights[0].id, 1);
        assert_eq!(manager.next_light_id, 2);

        // Add another light
        let light2 = Light::new(
            999,
            LightType::Directional,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            5000.0,
            0.0,
            0.0,
        );

        let id2 = manager.add_light(light2);
        assert_eq!(id2, 2);
        assert_eq!(manager.lights.len(), 2);
        assert_eq!(manager.next_light_id, 3);
    }

    #[test]
    fn test_light_manager_remove_light() {
        let mut manager = LightManager::new();

        // Add multiple lights
        for i in 0..3 {
            let light = Light::new(
                999,
                LightType::Point,
                Vector3::new(i as f32, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
                1.0,
                5000.0,
                10.0,
                0.0,
            );
            manager.add_light(light);
        }

        assert_eq!(manager.lights.len(), 3);

        // Remove middle light
        manager.remove_light(2);
        assert_eq!(manager.lights.len(), 2);

        // Verify correct lights remain
        assert_eq!(manager.lights[0].id, 1);
        assert_eq!(manager.lights[1].id, 3);

        // Remove non-existent light (should not crash)
        manager.remove_light(999);
        assert_eq!(manager.lights.len(), 2);
    }

    #[test]
    fn test_light_manager_get_light() {
        let mut manager = LightManager::new();

        let light = Light::new(
            999,
            LightType::Point,
            Vector3::new(1.0, 2.0, 3.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            5000.0,
            10.0,
            0.0,
        );

        let id = manager.add_light(light);

        // Get existing light
        let retrieved = manager.get_light(id);
        assert!(retrieved.is_some());
        let retrieved_light = retrieved.unwrap();
        assert_eq!(retrieved_light.id, id);
        assert_eq!(retrieved_light.position, Vector3::new(1.0, 2.0, 3.0));

        // Get non-existent light
        let not_found = manager.get_light(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_light_manager_get_light_mut() {
        let mut manager = LightManager::new();

        let light = Light::new(
            999,
            LightType::Point,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            5000.0,
            10.0,
            0.0,
        );

        let id = manager.add_light(light);

        // Modify light through mutable reference
        {
            let light_mut = manager.get_light_mut(id);
            assert!(light_mut.is_some());
            let light_ref = light_mut.unwrap();
            light_ref.position = Vector3::new(5.0, 5.0, 5.0);
            light_ref.intensity = 2.0;
            light_ref.enabled = false;
        }

        // Verify modifications
        let modified = manager.get_light(id).unwrap();
        assert_eq!(modified.position, Vector3::new(5.0, 5.0, 5.0));
        assert_eq!(modified.intensity, 2.0);
        assert!(!modified.enabled);

        // Get non-existent light mut
        let not_found = manager.get_light_mut(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_light_manager_get_all_lights() {
        let mut manager = LightManager::new();

        // Empty manager
        assert_eq!(manager.get_all_lights().len(), 0);

        // Add lights
        for i in 0..5 {
            let light = Light::new(
                999,
                LightType::Point,
                Vector3::new(i as f32, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
                1.0,
                5000.0,
                10.0,
                0.0,
            );
            manager.add_light(light);
        }

        let all_lights = manager.get_all_lights();
        assert_eq!(all_lights.len(), 5);

        // Verify all lights are present with correct IDs
        for (i, light) in all_lights.iter().enumerate() {
            assert_eq!(light.id, (i + 1) as u32);
            assert_eq!(light.position.x, i as f32);
        }
    }

    #[test]
    fn test_light_manager_multiple_operations() {
        let mut manager = LightManager::new();

        // Add several lights
        let id1 = manager.add_light(Light::new(
            0,
            LightType::Point,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            1.0,
            5000.0,
            10.0,
            0.0,
        ));

        let id2 = manager.add_light(Light::new(
            0,
            LightType::Directional,
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            1.0,
            6500.0,
            0.0,
            0.0,
        ));

        let id3 = manager.add_light(Light::new(
            0,
            LightType::Spot,
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            2.0,
            4000.0,
            15.0,
            45.0,
        ));

        assert_eq!(manager.get_all_lights().len(), 3);

        // Remove and re-add
        manager.remove_light(id2);
        assert_eq!(manager.get_all_lights().len(), 2);
        assert!(manager.get_light(id2).is_none());

        let id4 = manager.add_light(Light::new(
            0,
            LightType::Point,
            Vector3::new(2.0, 2.0, 2.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            1.5,
            5500.0,
            12.0,
            0.0,
        ));

        assert_eq!(id4, 4); // Should get new ID, not reuse old one
        assert_eq!(manager.get_all_lights().len(), 3);

        // Verify remaining lights
        assert!(manager.get_light(id1).is_some());
        assert!(manager.get_light(id3).is_some());
        assert!(manager.get_light(id4).is_some());
    }
}
