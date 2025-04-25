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
