use bevy::{
    prelude::*,
    render::{
        draw::DrawContext,
        pipeline::{PipelineDescriptor, RenderPipeline},
        shader::ShaderStages,
    },
};

const VERTEX: &str = r#"
#version 450

void main() {

}
"#;

const FRAGMENT: &str = r#"
#version 450

void main() {

}
"#;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut draw_context: DrawContext) {
    let vertex = Shader::from_glsl(bevy::render::shader::ShaderStage::Vertex, VERTEX);
    let fragment = Shader::from_glsl(bevy::render::shader::ShaderStage::Vertex, FRAGMENT);

    let pipeline = PipelineDescriptor::default_config(ShaderStages {
        vertex: draw_context.shaders.add(vertex),
        fragment: Some(draw_context.shaders.add(fragment)),
    });

    let pipeline_handle = draw_context.pipelines.add(pipeline);

    let render_pipeline = RenderPipeline::new(pipeline_handle);

    let mut draw = Draw::default();

    let _ = draw_context.set_pipeline(
        &mut draw,
        &render_pipeline.pipeline,
        &render_pipeline.specialization,
    );
}
