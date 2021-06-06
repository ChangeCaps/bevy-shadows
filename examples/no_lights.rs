//! Example exists to make sure we don't panic when no directional lights are present.

use bevy::prelude::*;
use bevy_shadows::prelude::*;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShadowPlugin::default())
        .add_startup_system(setup.system())
        .run();
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
        transform: Transform::from_translation(Vec3::new(2.0, 2.0, 5.0) * 0.5)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 100.0 }.into()),
        transform: Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        ..Default::default()
    });
}
