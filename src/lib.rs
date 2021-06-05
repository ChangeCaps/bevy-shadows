mod render_graph;
mod shadow_pass_node;

use bevy::prelude::*;
use shadow_pass_node::ShadowLights;

pub mod prelude {
    pub use crate::render_graph::{DIRECTIONAL_LIGHT_DEPTH_HANDLE, SHADOW_PBR_PIPELINE};
    pub use crate::shadow_pass_node::{Light, ShadowCaster, ShadowLight};
    pub use crate::ShadowPlugin;
}

pub struct ShadowPlugin {
    pub direction_light_size: u32,
}

impl Default for ShadowPlugin {
    fn default() -> Self {
        Self {
            direction_light_size: 4096,
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
