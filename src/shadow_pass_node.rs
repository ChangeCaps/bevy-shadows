use crate::{prelude::DIRECTIONAL_LIGHT_DEPTH_HANDLE, render_graph::SHADOW_PIPELINE};
use bevy::{
    core::bytes_of,
    ecs::{query::WorldQuery, system::BoxedSystem, world::World},
    pbr::render_graph::MAX_DIRECTIONAL_LIGHTS,
    prelude::*,
    prelude::{QueryState, Res},
    render::{
        draw::{DrawContext, RenderCommand},
        mesh::{Indices, INDEX_BUFFER_ASSET_INDEX, VERTEX_ATTRIBUTE_BUFFER_ID},
        pass::{Operations, PassDescriptor, RenderPassDepthStencilAttachment, TextureAttachment},
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{CommandQueue, Node, ResourceSlotInfo, ResourceSlots, SystemNode},
        renderer::{
            BufferId, BufferInfo, BufferMapMode, BufferUsage, RenderContext, RenderResourceBinding,
            RenderResourceBindings, RenderResourceContext, RenderResourceId, RenderResourceType,
            TextureId,
        },
        texture::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
            SAMPLER_ASSET_INDEX, TEXTURE_ASSET_INDEX,
        },
    },
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::borrow::Cow;
use std::collections::HashMap;

pub(crate) fn shadow_lights_register_system<L: Light>(
    mut shadow_lights: ResMut<ShadowLights>,
    query: Query<Entity, (With<L>, Added<L>)>,
) {
    for entity in query.iter() {
        shadow_lights.add(entity);
    }
}

pub(crate) fn shadow_lights_remove_system<L: Light>(
    mut shadow_lights: ResMut<ShadowLights>,
    removed: RemovedComponents<L>,
) {
    for entity in removed.iter() {
        shadow_lights.remove(entity);
    }
}

#[derive(Default, Clone, Copy)]
pub struct Shadowless;

pub trait Light: Send + Sync + 'static {
    type Config: Send + Sync + 'static;

    fn proj_matrix(&self, config: Option<&Self::Config>) -> Mat4;
    fn view_matrix(&self) -> Mat4;
}

#[derive(Default)]
pub struct ShadowLight {
    staging_buffer: Option<BufferId>,
    draw: Draw,
    texture_index: usize,
    pos: Vec3,
    view_proj: Mat4,
    pub bindings: RenderResourceBindings,
}

#[derive(Default)]
pub struct ShadowLights {
    lights: HashMap<Entity, ShadowLight>,
}

impl ShadowLights {
    fn add(&mut self, entity: Entity) {
        self.lights.insert(entity, Default::default());
    }

    fn remove(&mut self, entity: Entity) {
        self.lights.remove(&entity);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DirectionalLightUniform {
    pub texture_index: [u32; 4],
    pub pos: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}

#[derive(Default)]
pub struct ShadowLightsBindNode {
    command_queue: CommandQueue,
}

impl Node for ShadowLightsBindNode {
    fn prepare(&mut self, world: &mut World) {
        let mut lights = world.get_resource_mut::<ShadowLights>().unwrap();

        for (i, light) in lights.lights.values_mut().enumerate() {
            light.texture_index = i;
        }
    }

    fn update(
        &mut self,
        _world: &World,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        self.command_queue.execute(render_context);
    }
}

impl SystemNode for ShadowLightsBindNode {
    fn get_system(&self) -> BoxedSystem {
        let system = shadow_lights_bind_system.system().config(|config| {
            config.0 = Some(ShadowLightsSystemState {
                command_queue: self.command_queue.clone(),
                ..Default::default()
            });
        });

        Box::new(system)
    }
}

#[derive(Default)]
pub struct ShadowLightsSystemState {
    light_buffer: Option<BufferId>,
    staging_buffer: Option<BufferId>,
    command_queue: CommandQueue,
}

fn shadow_lights_bind_system(
    mut state: Local<ShadowLightsSystemState>,
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    lights: Res<ShadowLights>,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
) {
    let directional_size = std::mem::size_of::<DirectionalLightUniform>() * MAX_DIRECTIONAL_LIGHTS;

    let buffer_size = directional_size;

    let mut directional_lights = Vec::new();

    for light in lights.lights.values() {
        let directional_light = DirectionalLightUniform {
            texture_index: [light.texture_index as u32; 4],
            pos: light.pos.extend(0.0).into(),
            view_proj: light.view_proj.to_cols_array_2d(),
        };

        directional_lights.push(directional_light);
    }

    let directional_size =
        std::mem::size_of::<DirectionalLightUniform>() * directional_lights.len();

    let staging_buffer = if let Some(staging_buffer) = state.staging_buffer {
        render_resource_context.map_buffer(staging_buffer, BufferMapMode::Write);
        staging_buffer
    } else {
        let staging_buffer = render_resource_context.create_buffer(BufferInfo {
            size: buffer_size,
            mapped_at_creation: true,
            buffer_usage: BufferUsage::MAP_WRITE | BufferUsage::COPY_SRC,
        });

        state.staging_buffer = Some(staging_buffer);
        staging_buffer
    };

    let light_buffer = if let Some(light_buffer) = state.light_buffer {
        light_buffer
    } else {
        let light_buffer = render_resource_context.create_buffer(BufferInfo {
            size: buffer_size,
            mapped_at_creation: false,
            buffer_usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });

        state.light_buffer = Some(light_buffer);

        light_buffer
    };

    if directional_lights.len() > 0 {
        render_resource_context.write_mapped_buffer(
            staging_buffer,
            0..directional_size as u64,
            &mut |data, _| {
                data.copy_from_slice(cast_slice(&directional_lights));
            },
        );
    }

    render_resource_context.unmap_buffer(staging_buffer);

    if directional_lights.len() > 0 {
        state.command_queue.copy_buffer_to_buffer(
            staging_buffer,
            0,
            light_buffer,
            0,
            directional_size as u64,
        );
    }

    render_resource_bindings.set(
        "ShadowLights",
        RenderResourceBinding::Buffer {
            buffer: light_buffer,
            range: 0..buffer_size as u64,
            dynamic_index: None,
        },
    );
}

pub struct LightsNode<L: Light> {
    query_state: Option<
        QueryState<(
            &'static L,
            &'static GlobalTransform,
            Option<&'static L::Config>,
        )>,
    >,
    command_queue: CommandQueue,
}

impl<L: Light> Default for LightsNode<L> {
    fn default() -> Self {
        Self {
            query_state: Default::default(),
            command_queue: Default::default(),
        }
    }
}

impl<L: Light> Node for LightsNode<L> {
    fn prepare(&mut self, world: &mut World) {
        const MATRIX_SIZE: usize = std::mem::size_of::<Mat4>();

        let query_state = self.query_state.get_or_insert_with(|| world.query());

        let command_queue = &mut self.command_queue;

        world.resource_scope(
            |world, render_resource_context: Mut<Box<dyn RenderResourceContext>>| {
                world.resource_scope(|world, mut lights: Mut<ShadowLights>| {
                    for (entity, shadow_light) in &mut lights.lights {
                        if let Ok((light, global_transform, config)) =
                            query_state.get(world, *entity)
                        {
                            let proj = light.proj_matrix(config);
                            let view = light.view_matrix();
                            let view_proj = proj * view;

                            shadow_light.pos = global_transform.translation;
                            shadow_light.view_proj = view_proj;

                            let staging_buffer =
                                if let Some(staging_buffer) = shadow_light.staging_buffer {
                                    render_resource_context
                                        .map_buffer(staging_buffer, BufferMapMode::Write);
                                    staging_buffer
                                } else {
                                    let staging_buffer =
                                        render_resource_context.create_buffer(BufferInfo {
                                            size: MATRIX_SIZE,
                                            buffer_usage: BufferUsage::COPY_SRC
                                                | BufferUsage::MAP_WRITE,
                                            mapped_at_creation: true,
                                        });

                                    shadow_light.staging_buffer = Some(staging_buffer);
                                    staging_buffer
                                };

                            let buffer =
                                if let Some(RenderResourceBinding::Buffer { buffer, .. }) =
                                    shadow_light.bindings.get("ViewProj")
                                {
                                    *buffer
                                } else {
                                    let buffer =
                                        render_resource_context.create_buffer(BufferInfo {
                                            size: MATRIX_SIZE,
                                            buffer_usage: BufferUsage::COPY_DST
                                                | BufferUsage::UNIFORM,
                                            mapped_at_creation: false,
                                        });

                                    shadow_light.bindings.set(
                                        "ViewProj",
                                        RenderResourceBinding::Buffer {
                                            buffer,
                                            range: 0..MATRIX_SIZE as u64,
                                            dynamic_index: Some(0),
                                        },
                                    );

                                    buffer
                                };

                            render_resource_context.write_mapped_buffer(
                                staging_buffer,
                                0..MATRIX_SIZE as u64,
                                &mut |bytes, _| {
                                    bytes.copy_from_slice(bytes_of(&view_proj));
                                },
                            );

                            render_resource_context.unmap_buffer(staging_buffer);

                            command_queue.copy_buffer_to_buffer(
                                staging_buffer,
                                0,
                                buffer,
                                0,
                                MATRIX_SIZE as u64,
                            );
                        }
                    }
                });
            },
        );
    }

    fn update(
        &mut self,
        _world: &World,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        self.command_queue.execute(render_context);
    }
}

pub struct ShadowPassNode {
    num_textures: u32,
    textures: Vec<TextureId>,
    extent: Extent3d,
}

impl ShadowPassNode {
    pub const TEXTURE: &'static str = "texture";

    pub fn new(textures: u32, size: u32) -> Self {
        Self {
            num_textures: textures,
            textures: Vec::with_capacity(textures as usize),
            extent: Extent3d::new(size, size, 1),
        }
    }
}

impl Node for ShadowPassNode {
    fn input(&self) -> &[ResourceSlotInfo] {
        &[ResourceSlotInfo {
            name: Cow::Borrowed(Self::TEXTURE),
            resource_type: RenderResourceType::Texture,
        }]
    }

    fn prepare(&mut self, world: &mut World) {
        let render_resource_context = world
            .get_resource::<Box<dyn RenderResourceContext>>()
            .unwrap();

        let texture = render_resource_context
            .get_asset_resource_untyped(DIRECTIONAL_LIGHT_DEPTH_HANDLE, TEXTURE_ASSET_INDEX);
        let sampler = render_resource_context
            .get_asset_resource_untyped(DIRECTIONAL_LIGHT_DEPTH_HANDLE, SAMPLER_ASSET_INDEX);

        let mut bindings = world.get_resource_mut::<RenderResourceBindings>().unwrap();

        if let Some(texture) = texture {
            bindings.set(
                "DirectionalLightTexture",
                RenderResourceBinding::Texture(texture.get_texture().unwrap()),
            );
            bindings.set(
                "DirectionalLightSampler",
                RenderResourceBinding::Sampler(sampler.unwrap().get_sampler().unwrap()),
            );
        }
    }

    fn update(
        &mut self,
        world: &World,
        render_context: &mut dyn RenderContext,
        input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        if self.textures.len() < self.num_textures as usize {
            let render_resources_context = render_context.resources_mut();
            let n = self.num_textures as usize - self.textures.len();

            for _ in 0..n {
                let texture = render_resources_context.create_texture(TextureDescriptor {
                    size: self.extent.clone(),
                    sample_count: 1,
                    mip_level_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Depth32Float,
                    usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::COPY_SRC,
                });

                self.textures.push(texture);
            }
        }

        let lights = world.get_resource::<ShadowLights>().unwrap();
        let render_resource_bindings = world.get_resource::<RenderResourceBindings>().unwrap();
        let pipelines = world.get_resource::<Assets<PipelineDescriptor>>().unwrap();

        for shadow_light in lights.lights.values() {
            let texture = self.textures[shadow_light.texture_index];

            let desc = PassDescriptor {
                color_attachments: vec![],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    attachment: TextureAttachment::Id(texture),
                    depth_ops: Some(Operations {
                        load: bevy::render::pass::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
                sample_count: 1,
            };

            let mut bindings = RenderResourceBindings::default();

            bindings.extend(render_resource_bindings);
            bindings.extend(&shadow_light.bindings);

            render_context.begin_pass(&desc, &bindings, &mut |render_pass| {
                let mut current_pipeline = None;

                for render_command in &shadow_light.draw.render_commands {
                    match render_command {
                        RenderCommand::SetPipeline { pipeline } => {
                            render_pass.set_pipeline(pipeline);
                            current_pipeline = Some(pipeline);
                        }
                        RenderCommand::SetBindGroup {
                            index,
                            bind_group,
                            dynamic_uniform_indices,
                        } => {
                            let pipeline = pipelines.get(current_pipeline.unwrap()).unwrap();
                            let layout = pipeline.get_layout().unwrap();
                            let bind_group_descriptor = layout.get_bind_group(*index).unwrap();

                            render_pass.set_bind_group(
                                *index,
                                bind_group_descriptor.id,
                                *bind_group,
                                dynamic_uniform_indices.as_deref(),
                            );
                        }
                        RenderCommand::SetVertexBuffer {
                            slot,
                            buffer,
                            offset,
                        } => {
                            render_pass.set_vertex_buffer(*slot, *buffer, *offset);
                        }
                        RenderCommand::SetIndexBuffer {
                            buffer,
                            offset,
                            index_format,
                        } => {
                            render_pass.set_index_buffer(*buffer, *offset, *index_format);
                        }
                        RenderCommand::DrawIndexed {
                            base_vertex,
                            indices,
                            instances,
                        } => {
                            render_pass.draw_indexed(
                                indices.clone(),
                                *base_vertex,
                                instances.clone(),
                            );
                        }
                        RenderCommand::Draw {
                            vertices,
                            instances,
                        } => {
                            render_pass.draw(vertices.clone(), instances.clone());
                        }
                    }
                }
            });
        }

        if let Some(RenderResourceId::Texture(array_texture)) = input.get(Self::TEXTURE) {
            for (i, texture) in self.textures.iter().enumerate() {
                render_context.copy_texture_to_texture(
                    *texture,
                    [0, 0, 0],
                    0,
                    array_texture,
                    [0, 0, i as u32],
                    0,
                    self.extent,
                );
            }
        }
    }
}

impl SystemNode for ShadowPassNode {
    fn get_system(&self) -> BoxedSystem {
        Box::new(shadow_pass_system.system().config(|config| {
            config.0 = Some(RenderPipeline::new(SHADOW_PIPELINE.typed()));
        }))
    }
}

fn shadow_pass_system(
    pipeline: Local<RenderPipeline>,
    mut draw_context: DrawContext,
    mut lights: ResMut<ShadowLights>,
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    meshes: Res<Assets<Mesh>>,
    mut query: Query<(&Handle<Mesh>, &mut RenderPipelines), Without<Shadowless>>,
) {
    for light in lights.lights.values_mut() {
        light.draw.render_commands.clear();

        for (mesh_handle, mut render_pipelines) in query.iter_mut() {
            let mesh = if let Some(mesh) = meshes.get(mesh_handle) {
                mesh
            } else {
                continue;
            };

            if light.bindings.get("ViewProj").is_some() {
                let mut pipeline_specialization =
                    render_pipelines.pipelines[0].specialization.clone();
                pipeline_specialization
                    .dynamic_bindings
                    .insert("ViewProj".to_string());
                pipeline_specialization
                    .dynamic_bindings
                    .insert("Transform".to_string());
                pipeline_specialization.primitive_topology = mesh.primitive_topology();
                pipeline_specialization.vertex_buffer_layout = mesh.get_vertex_buffer_layout();
                pipeline_specialization.sample_count = 1;

                let bindings = &mut [
                    &mut light.bindings,
                    &mut render_pipelines.bindings,
                    &mut render_resource_bindings,
                ];

                draw_context
                    .set_pipeline(
                        &mut light.draw,
                        &pipeline.pipeline,
                        &pipeline_specialization,
                    )
                    .unwrap();

                draw_context
                    .set_bind_groups_from_bindings(&mut light.draw, bindings)
                    .unwrap();

                if let Some(RenderResourceId::Buffer(index_buffer_resource)) = draw_context
                    .render_resource_context
                    .get_asset_resource(mesh_handle, INDEX_BUFFER_ASSET_INDEX)
                {
                    let index_format = mesh.indices().unwrap().into();

                    light
                        .draw
                        .set_index_buffer(index_buffer_resource, 0, index_format);
                }

                if let Some(RenderResourceId::Buffer(vertex_attribute_buffer_resource)) =
                    draw_context
                        .render_resource_context
                        .get_asset_resource(mesh_handle, VERTEX_ATTRIBUTE_BUFFER_ID)
                {
                    light
                        .draw
                        .set_vertex_buffer(0, vertex_attribute_buffer_resource, 0);
                }

                let index_range = match mesh.indices() {
                    Some(Indices::U16(indices)) => Some(0..indices.len() as u32),
                    Some(Indices::U32(indices)) => Some(0..indices.len() as u32),
                    None => None,
                };

                if let Some(indices) = index_range.clone() {
                    light.draw.draw_indexed(indices, 0, 0..1);
                } else {
                    light.draw.draw(0..mesh.count_vertices() as u32, 0..1);
                }
            }
        }
    }
}
