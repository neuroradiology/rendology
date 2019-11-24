use nalgebra as na;

use glium::uniform;

use crate::render::pipeline::InstanceParams;

#[derive(Debug, Clone)]
pub struct Light {
    pub position: na::Point3<f32>,
    pub attenuation: na::Vector3<f32>,
    pub color: na::Vector3<f32>,
    pub is_main: bool,
}

impl InstanceParams for Light {
    type U = impl glium::uniforms::Uniforms;

    fn uniforms(&self) -> Self::U {
        let position: [f32; 3] = self.position.coords.into();
        let attenuation: [f32; 3] = self.attenuation.into();
        let color: [f32; 3] = self.color.into();

        uniform! {
            light_position: position,
            light_attenuation: attenuation,
            light_color: color,
            light_is_main: self.is_main,
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: na::Point3::origin(),
            attenuation: na::Vector3::zeros(),
            color: na::Vector3::zeros(),
            is_main: false,
        }
    }
}
