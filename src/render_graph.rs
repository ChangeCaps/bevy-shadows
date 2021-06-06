use crate::shadow_pass_node::{Light, LightsNode};
use crate::shadow_pass_node::{ShadowLightsBindNode, ShadowPassNode};
use bevy::pbr::render_graph::{MAX_DIRECTIONAL_LIGHTS, PBR_PIPELINE_HANDLE};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::camera::CameraProjection;
use bevy::render::camera::OrthographicProjection;
use bevy::render::pipeline::{Face, PipelineDescriptor, PrimitiveState};
use bevy::render::shader::{ShaderStage, ShaderStages};
use bevy::render::texture::TextureDescriptor;
use bevy::render::{
    render_graph::{base, RenderGraph, TextureNode},
    texture::{Extent3d, SamplerDescriptor, TextureDimension, TextureFormat, TextureUsage},
};

impl Light for DirectionalLight {
    fn proj_matrix(&self) -> Mat4 {
        let dir = self.get_direction().normalize();
        let rot = Quat::from_rotation_arc(Vec3::Z, dir);

        OrthographicProjection {
            left: -25.0,
            right: 25.0,
            bottom: -25.0,
            top: 25.0,
            far: 500.0,
            near: -500.0,
            ..Default::default()
        }
        .get_projection_matrix()
            * Mat4::from_quat(rot)
    }
}

pub const DIRECTIONAL_LIGHT_DEPTH_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Texture::TYPE_UUID, 4328462394);

pub const SHADOW_PIPELINE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 219384239876);

pub const SHADOW_PBR_PIPELINE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 983456781236);

pub const DIRECTIONAL_LIGHT_DEPTH: &str = "directional_light_texture";
pub const DIRECTIONAL_LIGHTS_NODE: &str = "direction_lights_node";
pub const SHADOW_LIGHTS_BIND_NODE: &str = "shadow_lights_bind_node";
pub const SHADOW_PASS_NODE: &str = "shadow_pass_node";

pub(crate) fn add_render_graph(shadow_plugin: &crate::ShadowPlugin, app: &mut AppBuilder) {
    let mut shaders = app
        .world_mut()
        .get_resource_mut::<Assets<Shader>>()
        .unwrap();

    let vertex = shaders.add(Shader::from_glsl(
        ShaderStage::Vertex,
        include_str!("shaders/shadow.vert"),
    ));

    let shadow_pipeline = PipelineDescriptor {
        color_target_states: vec![],
        primitive: PrimitiveState {
            cull_mode: Some(Face::Front),
            ..Default::default()
        },
        ..PipelineDescriptor::default_config(ShaderStages {
            vertex,
            fragment: None,
        })
    };

    // only create pbr pipeline if desired
    if shadow_plugin.create_pbr_pipeline {
        let vertex = shaders.add(Shader::from_glsl(
            ShaderStage::Vertex,
            include_str!("shaders/shadow_pbr.vert"),
        ));
        let fragment = shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            include_str!("shaders/shadow_pbr.frag"),
        ));

        let shadow_pbr_pipeline = PipelineDescriptor::default_config(ShaderStages {
            vertex,
            fragment: Some(fragment),
        });

        let mut pipelines = app
            .world_mut()
            .get_resource_mut::<Assets<PipelineDescriptor>>()
            .unwrap();

        if shadow_plugin.replace_pbr_pipeline {
            pipelines.set_untracked(PBR_PIPELINE_HANDLE, shadow_pbr_pipeline);
        } else {
            pipelines.set_untracked(SHADOW_PBR_PIPELINE, shadow_pbr_pipeline);
        }
    }

    let mut pipelines = app
        .world_mut()
        .get_resource_mut::<Assets<PipelineDescriptor>>()
        .unwrap();

    pipelines.set_untracked(SHADOW_PIPELINE, shadow_pipeline);

    let mut render_graph = app.world_mut().get_resource_mut::<RenderGraph>().unwrap();

    let extent = Extent3d::new(
        shadow_plugin.directional_light_resolution,
        shadow_plugin.directional_light_resolution,
        // this is wasteful if MAX_DIRECTIONAL_LIGHTS is 1
        // but it will insure that the texture will always be
        // interpreted as a D2Array texture rather than a D2
        MAX_DIRECTIONAL_LIGHTS as u32,
    );

    render_graph.add_node(
        DIRECTIONAL_LIGHTS_NODE,
        LightsNode::<DirectionalLight>::default(),
    );

    render_graph.add_system_node(
        SHADOW_PASS_NODE,
        ShadowPassNode::new(
            MAX_DIRECTIONAL_LIGHTS as u32,
            shadow_plugin.directional_light_resolution,
        ),
    );

    render_graph.add_node(
        DIRECTIONAL_LIGHT_DEPTH,
        TextureNode::new(
            TextureDescriptor {
                size: extent,
                sample_count: 1,
                mip_level_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT
                    | TextureUsage::SAMPLED
                    | TextureUsage::COPY_DST,
            },
            Some(SamplerDescriptor {
                ..Default::default()
            }),
            Some(DIRECTIONAL_LIGHT_DEPTH_HANDLE),
        ),
    );

    render_graph.add_system_node(SHADOW_LIGHTS_BIND_NODE, ShadowLightsBindNode::default());

    render_graph
        .add_slot_edge(
            DIRECTIONAL_LIGHT_DEPTH,
            TextureNode::TEXTURE,
            SHADOW_PASS_NODE,
            ShadowPassNode::TEXTURE,
        )
        .unwrap();

    render_graph
        .add_node_edge(DIRECTIONAL_LIGHTS_NODE, SHADOW_PASS_NODE)
        .unwrap();

    if shadow_plugin.connect_to_main_pass {
        render_graph
            .add_node_edge(SHADOW_LIGHTS_BIND_NODE, base::node::MAIN_PASS)
            .unwrap();
        render_graph
            .add_node_edge(SHADOW_PASS_NODE, base::node::MAIN_PASS)
            .unwrap();
    }
}
