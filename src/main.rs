use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod camera;
mod input;
mod vehicle;
mod dino;
mod weapon;
mod ui;
mod pause;

use camera::CameraPlugin;
use input::InputPlugin;
use vehicle::VehiclePlugin;
use dino::DinoPlugin;
use weapon::WeaponPlugin;
use ui::UIPlugin;
use pause::{PausePlugin, GameState};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.52, 0.77, 0.98)))
        .insert_resource(GameScore { score: 0 })
        .add_plugins((
            CameraPlugin,
            InputPlugin,
            VehiclePlugin,
            DinoPlugin,
            WeaponPlugin,
            UIPlugin,
            PausePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update_score)
        .enable_state_scoped_entities::<GameState>()
        .run();
}

#[derive(Resource)]
struct GameScore {
    score: u32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -0.5)),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.9, 0.85, 0.8),
        brightness: 800.0,
    });

    // Fog (using bevy's built-in fog - add to camera instead)
    // Note: In Bevy 0.15, fog is configured differently

    // Ground
    let ground_size = 500.0;
    commands.spawn((
        Transform::from_xyz(0.0, -0.5, 0.0),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(ground_size)))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.5, 0.15))),
    ));

    // Ground physics
    commands.spawn((
        Transform::from_xyz(0.0, -0.5, 0.0).looking_at(Vec3::Z, Vec3::Y),
        Collider::halfspace(Vec3::Y).unwrap(),
    ));

    // Spawn some trees
    spawn_trees(&mut commands, &mut meshes, &mut materials);

    // Spawn some rocks
    spawn_rocks(&mut commands, &mut meshes, &mut materials);

    // HUD text for instructions
    commands.spawn((
        Text2d::new("WASD: Move | Mouse: Aim | Left Click: Shoot"),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 300.0, 0.0),
        TextLayout::new_with_justify(JustifyText::Center),
    ));
}

fn spawn_trees(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let trunk_material = materials.add(Color::srgb(0.4, 0.25, 0.15));
    let leaves_material = materials.add(Color::srgb(0.1, 0.4, 0.15));

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let x = (rand::Rng::gen_range(&mut rng, -200.0..200.0) as f32).floor();
        let z = (rand::Rng::gen_range(&mut rng, -200.0..200.0) as f32).floor();

        // Skip area near spawn
        if x.abs() < 10.0 && z.abs() < 10.0 {
            continue;
        }

        let tree_transform = Transform::from_xyz(x, 0.0, z);

        // Trunk
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.5, 8.0))),
            MeshMaterial3d(trunk_material.clone()),
            tree_transform,
        ));

        // Leaves (multiple cones for a pine tree look)
        for i in 0..4 {
            let y = 6.0 + i as f32 * 1.5;
            let scale = 3.0 - i as f32 * 0.5;
            commands.spawn((
                Mesh3d(meshes.add(Cone {
                    radius: scale,
                    height: 2.5,
                })),
                MeshMaterial3d(leaves_material.clone()),
                Transform::from_xyz(x, y, z),
            ));
        }
    }
}

fn spawn_rocks(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let rock_material = materials.add(Color::srgb(0.4, 0.4, 0.45));

    let mut rng = rand::thread_rng();

    for _ in 0..50 {
        let x = rand::Rng::gen_range(&mut rng, -150.0..150.0);
        let z = rand::Rng::gen_range(&mut rng, -150.0..150.0);
        let scale = rand::Rng::gen_range(&mut rng, 0.5..2.0);

        commands.spawn((
            Mesh3d(meshes.add(Sphere { radius: scale * 0.5 })),
            MeshMaterial3d(rock_material.clone()),
            Transform::from_xyz(x, scale * 0.3, z).with_scale(Vec3::splat(scale)),
        ));
    }
}

fn update_score(mut score_text: Query<&mut Text, With<ui::ScoreText>>, score: Res<GameScore>) {
    for mut text in score_text.iter_mut() {
        text.0 = format!("Score: {}", score.score);
    }
}
