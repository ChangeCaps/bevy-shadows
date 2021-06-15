use crate::shadow_pass_node::*;
use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, OrthographicProjection};

const HALF_SIZE: f32 = 25.0;
pub struct ShadowDirectionalLight {
    /// Left plane of projection.
    pub left: f32,
    /// Right plane of projection.
    pub right: f32,
    /// Bottom plane of projection.
    pub bottom: f32,
    /// Top plane of projection.
    pub top: f32,
    /// Near plane of projection.
    pub near: f32,
    /// Far plane of projection.
    pub far: f32,
}

impl Default for ShadowDirectionalLight {
    fn default() -> Self {
        Self {
            left: -HALF_SIZE,
            right: HALF_SIZE,
            bottom: -HALF_SIZE,
            top: HALF_SIZE,
            near: -20.0 * HALF_SIZE,
            far: 20.0 * HALF_SIZE,
        }
    }
}

impl Light for DirectionalLight {
    type Config = ShadowDirectionalLight;

    fn proj_matrix(&self, config: Option<&Self::Config>) -> Mat4 {
        let left = config.map_or(-HALF_SIZE, |config| config.left);
        let right = config.map_or(HALF_SIZE, |config| config.right);
        let bottom = config.map_or(-HALF_SIZE, |config| config.bottom);
        let top = config.map_or(HALF_SIZE, |config| config.top);
        let near = config.map_or(-20.0 * HALF_SIZE, |config| config.near);
        let far = config.map_or(20.0 * HALF_SIZE, |config| config.far);

        OrthographicProjection {
            left,
            right,
            bottom,
            top,
            far,
            near,
            ..Default::default()
        }
        .get_projection_matrix()
    }

    fn view_matrix(&self) -> Mat4 {
        let eye_position = -40.0 * self.get_direction();
        Mat4::look_at_rh(eye_position, Vec3::ZERO, Vec3::Y)
    }
}
