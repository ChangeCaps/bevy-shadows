mod render_graph;
mod shadow_pass_node;

use bevy::prelude::*;
use shadow_pass_node::ShadowLights;

pub mod prelude {
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
}

impl Default for ShadowPlugin {
    fn default() -> Self {
        Self {
            directional_light_resolution: 4096,
            replace_pbr_pipeline: true,
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
