use bevy::{
    asset::Asset,
    prelude::*,
    render::{pipeline::RenderPipeline, render_graph::base::MainPass},
};
use bevy_shadows::prelude::*;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShadowPlugin::default())
        .add_startup_system(setup.system())
        .add_system(rotate_cubes.system())
        .add_system(add_shadow_caster.system())
        .run();
}

struct Rotate;

fn rotate_cubes(mut query: Query<&mut Transform, With<Rotate>>) {
    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_rotation_y(0.01));
    }
}

fn add_shadow_caster(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut RenderPipelines, &mut Transform),
        (Without<ShadowCaster>, With<Handle<Mesh>>, With<MainPass>),
    >,
) {
    for (entity, mut render_pipelines, mut transform) in query.iter_mut() {
        commands.entity(entity).insert(ShadowCaster).insert(Rotate);

        transform.scale = Vec3::splat(6.0);

        *render_pipelines =
            RenderPipelines::from_pipelines(vec![RenderPipeline::new(SHADOW_PBR_PIPELINE.typed())]);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let gltf = asset_server.load("FlightHelmet/FlightHelmet.gltf#Scene0");

    commands.spawn_scene(gltf);

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(2.0, 2.0, 5.0) * 2.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let cube = shape::Box::new(1.0, 1.0, 1.0);

    commands
        .spawn()
        .insert(DirectionalLight::new(
            Color::WHITE,
            32000.0,
            Vec3::new(1.0, -1.0, 0.0),
        ))
        .insert(Transform::identity())
        .insert(GlobalTransform::identity())
        .insert(ShadowLight::default());

    /*
    for x in -5..5 {
        for z in -5..5 {
            commands
                .spawn_bundle(PbrBundle {
                    transform: Transform::from_translation(
                        Vec3::new(x as f32, x as f32 + z as f32, z as f32) * 2.0,
                    ),
                    mesh: meshes.add(cube.into()),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        SHADOW_PBR_PIPELINE.typed(),
                    )]),
                    ..Default::default()
                })
                .insert(ShadowCaster)
                .insert(Rotate);
        }
    }
    */

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(shape::Plane { size: 100.0 }.into()),
            transform: Transform::from_translation(Vec3::new(0.0, -2.0, 0.0)),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                SHADOW_PBR_PIPELINE.typed(),
            )]),
            ..Default::default()
        })
        .insert(ShadowCaster);
}
