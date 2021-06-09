use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_shadows::prelude::*;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShadowPlugin::default())
        .add_startup_system(setup.system())
        .add_system(camera_system.system())
        .add_system(light_direction.system())
        .run();
}

#[derive(Default)]
struct CameraState {
    yaw: f32,
    pitch: f32,
}

fn camera_system(
    mut moved: EventReader<MouseMotion>,
    time: Res<Time>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut windows: ResMut<Windows>,
    mut query: Query<(&mut Transform, &mut CameraState)>,
) {
    let window = windows.get_primary_mut().unwrap();

    if mouse.just_pressed(MouseButton::Left) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }

    for event in moved.iter() {
        if !window.cursor_locked() {
            continue;
        }

        for (mut transform, mut state) in query.iter_mut() {
            state.yaw -= event.delta.x * 0.0005;
            state.pitch -= event.delta.y * 0.0005;

            transform.rotation =
                Quat::from_euler(bevy::math::EulerRot::YXZ, state.yaw, state.pitch, 0.0);
        }
    }

    for (mut transform, _) in query.iter_mut() {
        let mut movement = Vec3::ZERO;

        if keyboard.pressed(KeyCode::W) {
            movement -= transform.local_z();
        }

        if keyboard.pressed(KeyCode::S) {
            movement += transform.local_z();
        }

        if keyboard.pressed(KeyCode::D) {
            movement += transform.local_x();
        }

        if keyboard.pressed(KeyCode::A) {
            movement -= transform.local_x();
        }

        transform.translation += movement.normalize_or_zero() * time.delta_seconds() * 2.0;
    }
}

fn light_direction(time: Res<Time>, mut query: Query<&mut DirectionalLight>) {
    for mut light in query.iter_mut() {
        light.set_direction(Vec3::new(
            time.seconds_since_startup().sin() as f32,
            -1.0,
            0.0,
        ));
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

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(2.0, 2.0, 5.0) * 0.5)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(CameraState::default())
        .with_children(|parent| {
            parent
                .spawn()
                .insert(DirectionalLight::new(
                    Color::WHITE,
                    32000.0,
                    Vec3::new(1.0, -1.0, 0.0),
                ))
                .insert(Transform::identity())
                .insert(GlobalTransform::identity())
                .insert(ShadowDirectionalLight {
                    size: 10.0,
                    ..Default::default()
                });
        });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 100.0 }.into()),
        transform: Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        ..Default::default()
    });
}
