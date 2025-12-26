use bevy::prelude::*;
use crate::pause::GameState;
use crate::vehicle::PlayerVehicle;
use crate::dino::Dinosaur;
use crate::input::TargetLock;

#[derive(Component)]
pub struct MinimapContainer;

#[derive(Component)]
pub struct MinimapBackground;

#[derive(Component)]
pub struct PlayerDot;

#[derive(Component)]
pub struct EnemyDot;

#[derive(Component)]
pub struct LockedTargetIndicator;

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_minimap)
            .add_systems(Update, update_minimap.run_if(in_state(GameState::Playing)));
    }
}

const MINIMAP_SIZE: f32 = 150.0;
const MINIMAP_SCALE: f32 = 0.5; // 1 unit on minimap = 2 units in world

fn setup_minimap(mut commands: Commands) {
    // Minimap container - positioned in bottom right corner
    commands.spawn((
        MinimapContainer,
        Node {
            width: Val::Px(MINIMAP_SIZE),
            height: Val::Px(MINIMAP_SIZE),
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..default()
        },
    )).with_children(|parent| {
        // Background
        parent.spawn((
            MinimapBackground,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.15, 0.2, 0.8)),
            BorderColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
        ));

        // Player dot (center of minimap)
        parent.spawn((
            PlayerDot,
            Node {
                width: Val::Px(8.0),
                height: Val::Px(8.0),
                position_type: PositionType::Absolute,
                left: Val::Px(MINIMAP_SIZE / 2.0 - 4.0),
                top: Val::Px(MINIMAP_SIZE / 2.0 - 4.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
            BorderRadius::MAX,
        ));
    });
}

fn update_minimap(
    mut commands: Commands,
    minimap_q: Query<Entity, With<MinimapContainer>>,
    vehicle_q: Query<&Transform, With<PlayerVehicle>>,
    dino_q: Query<&Transform, (With<Dinosaur>, Without<PlayerVehicle>)>,
    target_lock: Res<TargetLock>,
    existing_enemy_dots: Query<Entity, With<EnemyDot>>,
    existing_locked_indicator: Query<Entity, With<LockedTargetIndicator>>,
) {
    let Ok(minimap_entity) = minimap_q.get_single() else {
        return;
    };

    let Ok(vehicle_transform) = vehicle_q.get_single() else {
        return;
    };

    let vehicle_pos = vehicle_transform.translation;

    // Remove old enemy dots
    for entity in existing_enemy_dots.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Remove old locked indicator
    for entity in existing_locked_indicator.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Spawn new enemy dots
    for dino_transform in dino_q.iter() {
        let dino_pos = dino_transform.translation;

        // Calculate relative position
        let rel_x = (dino_pos.x - vehicle_pos.x) * MINIMAP_SCALE;
        let rel_z = (dino_pos.z - vehicle_pos.z) * MINIMAP_SCALE;

        // Only show if within minimap bounds
        if rel_x.abs() < MINIMAP_SIZE / 2.0 && rel_z.abs() < MINIMAP_SIZE / 2.0 {
            // Convert to screen coordinates (center is MINIMAP_SIZE/2)
            let screen_x = MINIMAP_SIZE / 2.0 + rel_x;
            let screen_y = MINIMAP_SIZE / 2.0 + rel_z;

            commands.entity(minimap_entity).with_children(|parent| {
                parent.spawn((
                    EnemyDot,
                    Node {
                        width: Val::Px(6.0),
                        height: Val::Px(6.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(screen_x - 3.0),
                        top: Val::Px(screen_y - 3.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                    BorderRadius::MAX,
                ));
            });
        }
    }

    // Show locked target indicator
    if let Some(locked_entity) = target_lock.locked_entity {
        if let Ok(dino_transform) = dino_q.get(locked_entity) {
            let dino_pos = dino_transform.translation;

            let rel_x = (dino_pos.x - vehicle_pos.x) * MINIMAP_SCALE;
            let rel_z = (dino_pos.z - vehicle_pos.z) * MINIMAP_SCALE;

            if rel_x.abs() < MINIMAP_SIZE / 2.0 && rel_z.abs() < MINIMAP_SIZE / 2.0 {
                let screen_x = MINIMAP_SIZE / 2.0 + rel_x;
                let screen_y = MINIMAP_SIZE / 2.0 + rel_z;

                commands.entity(minimap_entity).with_children(|parent| {
                    // Yellow circle around locked target
                    parent.spawn((
                        LockedTargetIndicator,
                        Node {
                            width: Val::Px(12.0),
                            height: Val::Px(12.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(screen_x - 6.0),
                            top: Val::Px(screen_y - 6.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.9, 0.7, 0.2, 0.0)),
                        BorderColor(Color::srgb(0.9, 0.7, 0.2)),
                    ));
                });
            }
        }
    }
}
