mod directional_light;
mod render_graph;
mod shadow_pass_node;

use bevy::{prelude::*, transform::TransformSystem};
use bevy_mod_bounding::{sphere, BoundingVolumePlugin};
use shadow_pass_node::ShadowLights;

pub mod prelude {
    pub use crate::directional_light::ShadowDirectionalLight;
    pub use crate::render_graph::{DIRECTIONAL_LIGHT_DEPTH_HANDLE, SHADOW_PBR_PIPELINE};
    pub use crate::shadow_pass_node::Shadowless;
    pub use crate::ShadowPlugin;
}

pub struct ShadowPlugin {
    /// Resolution of directional light shadow maps.
    pub directional_light_resolution: u32,
    /// If true, replaces the default pbr pipeline.
    /// If false use [`prelude::SHADOW_PBR_PIPELINE`].
    pub replace_pbr_pipeline: bool,
    /// If false, the shadow pbr pipeline won't be created.
    /// Disable if you want to implement your own.
    pub create_pbr_pipeline: bool,
    /// If false then the shadow pass won't be connected to main pass.
    pub connect_to_main_pass: bool,
    /// If true, automatically calculate the bounding box of the scene to use
    /// for the directional light's orthographic projection.
    /// If false, use whatever is set in the ShadowDirectionalLight component.
    pub automatic_projection_bounds: bool,
}

impl Default for ShadowPlugin {
    fn default() -> Self {
        Self {
            directional_light_resolution: 4096,
            replace_pbr_pipeline: true,
            create_pbr_pipeline: true,
            connect_to_main_pass: true,
            automatic_projection_bounds: false,
        }
    }
}

impl Plugin for ShadowPlugin {
    fn build(&self, app: &mut AppBuilder) {
        render_graph::add_render_graph(self, app);

        app.insert_resource(ShadowLights::default());
        app.add_system(
            shadow_pass_node::shadow_lights_register_system::<DirectionalLight>.system(),
        );
        app.add_system(shadow_pass_node::shadow_lights_remove_system::<DirectionalLight>.system());
        if self.automatic_projection_bounds {
            app.add_plugin(BoundingVolumePlugin::<sphere::BSphere>::default())
                .add_system_to_stage(
                    CoreStage::PreUpdate,
                    directional_light::add_bounding_spheres.system(),
                )
                .add_system_to_stage(
                    CoreStage::PostUpdate,
                    directional_light::update_scene_bounding_box
                        .system()
                        .after(TransformSystem::TransformPropagate),
                );
        }
    }
}
