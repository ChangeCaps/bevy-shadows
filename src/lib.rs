mod directional_light;
mod render_graph;
mod shadow_pass_node;

use bevy::prelude::*;
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
}

impl Default for ShadowPlugin {
    fn default() -> Self {
        Self {
            directional_light_resolution: 4096,
            replace_pbr_pipeline: true,
            create_pbr_pipeline: true,
            connect_to_main_pass: true,
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
    }
}
