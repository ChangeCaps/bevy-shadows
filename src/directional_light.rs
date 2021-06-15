use crate::shadow_pass_node::*;
use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, OrthographicProjection};
use bevy::render::pipeline::PrimitiveTopology;
use bevy_mod_bounding::{sphere, Bounded};

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

pub fn add_bounding_spheres(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    query: Query<
        (Entity, &Handle<Mesh>),
        (
            Without<Shadowless>,
            Without<sphere::BSphere>,
            Without<Bounded<sphere::BSphere>>,
        ),
    >,
) {
    for (entity, mesh_handle) in query.iter() {
        let mesh = meshes.get(mesh_handle).unwrap();
        if mesh.primitive_topology() == PrimitiveTopology::TriangleList {
            commands
                .entity(entity)
                .insert(Bounded::<sphere::BSphere>::default());
        }
    }
}

struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn min_max() -> Self {
        Self {
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(f32::MIN),
        }
    }
}

pub fn update_scene_bounding_box(
    mut lights: Query<(&DirectionalLight, &mut ShadowDirectionalLight)>,
    bounds: Query<(&GlobalTransform, &sphere::BSphere), (With<Handle<Mesh>>, Without<Shadowless>)>,
) {
    for (dir_light, mut shadow_light) in lights.iter_mut() {
        let view = dir_light.view_matrix();
        let mut bb = BoundingBox::min_max();
        for (transform, bsphere) in bounds.iter() {
            let origin = bsphere.origin(*transform);
            let origin_l = view * origin.extend(1.0);
            let radius = bsphere.radius(transform);
            if origin_l.x - radius < bb.min.x {
                bb.min.x = origin_l.x - radius;
            }
            if origin_l.y - radius < bb.min.y {
                bb.min.y = origin_l.y - radius;
            }
            if origin_l.z - radius < bb.min.z {
                bb.min.z = origin_l.z - radius;
            }
            if origin_l.x + radius > bb.max.x {
                bb.max.x = origin_l.x + radius;
            }
            if origin_l.y + radius > bb.max.y {
                bb.max.y = origin_l.y + radius;
            }
            if origin_l.z + radius > bb.max.z {
                bb.max.z = origin_l.z + radius;
            }
        }
        shadow_light.left = bb.min.x;
        shadow_light.right = bb.max.x;
        shadow_light.bottom = bb.min.y;
        shadow_light.top = bb.max.y;
        // NOTE: Positive near/far are in front of the camera but light space has -Z in front of the camera
        // so we have to flip along Z
        shadow_light.near = -bb.max.z;
        shadow_light.far = -bb.min.z;
    }
}
