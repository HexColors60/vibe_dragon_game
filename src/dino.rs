use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use crate::weapon::BulletHitEvent;
use crate::GameScore;
use crate::pause::GameState;
use crate::combo::ComboSystem;

pub struct DinoPlugin;

#[derive(Component)]
pub struct Dinosaur;

#[derive(Component, Clone, Copy)]
pub enum DinoSpecies {
    Triceratops,
    Velociraptor,
    Brachiosaurus,
}

#[derive(Component)]
pub struct DinoHealth {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Clone, Copy)]
pub enum BodyPart {
    Head,
    Body,
    Legs,
}

#[derive(Component)]
pub struct HitBox {
    pub part: BodyPart,
}

#[derive(Component)]
pub struct DinoAI {
    pub state: AIState,
    pub wander_target: Option<Vec3>,
    pub flee_direction: Vec3,
    pub move_speed: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AIState {
    Idle,
    Roam,
    Flee,
    Dead,
}

#[derive(Component)]
pub struct DinoDeath {
    timer: Timer,
}

#[derive(Resource)]
pub struct DinoSpawnConfig {
    pub count: u32,
    pub spawn_radius: f32,
    pub min_distance_from_player: f32,
}

impl Default for DinoSpawnConfig {
    fn default() -> Self {
        Self {
            count: 15,
            spawn_radius: 150.0,
            min_distance_from_player: 20.0,
        }
    }
}

impl Plugin for DinoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DinoSpawnConfig>()
            .add_event::<RespawnDinosEvent>()
            .add_systems(Startup, spawn_dinosaurs)
            .add_systems(Update, (
                handle_bullet_hits,
                handle_respawn_dinos,
                update_dino_ai,
                update_dino_movement,
                check_dino_death,
                update_dino_death_animation,
            ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Event)]
pub struct RespawnDinosEvent;

fn spawn_dinosaurs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();

    // Spawn dinosaurs
    for _ in 0..15 {
        let species = match rng.gen_range(0..3) {
            0 => DinoSpecies::Triceratops,
            1 => DinoSpecies::Velociraptor,
            _ => DinoSpecies::Brachiosaurus,
        };

        let x: f32 = rng.gen_range(-150.0..150.0);
        let z: f32 = rng.gen_range(-150.0..150.0);

        // Don't spawn too close to origin
        if x.abs() < 20.0 && z.abs() < 20.0 {
            continue;
        }

        spawn_dinosaur(&mut commands, &mut meshes, &mut materials, species, Vec3::new(x, 0.0, z));
    }
}

fn spawn_dinosaur(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    species: DinoSpecies,
    position: Vec3,
) {
    let (body_color, size, health, speed) = match species {
        DinoSpecies::Triceratops => (Color::srgb(0.5, 0.35, 0.2), Vec3::new(1.5, 1.2, 2.5), 150.0, 8.0),
        DinoSpecies::Velociraptor => (Color::srgb(0.4, 0.3, 0.25), Vec3::new(0.6, 0.5, 1.2), 60.0, 15.0),
        DinoSpecies::Brachiosaurus => (Color::srgb(0.45, 0.4, 0.3), Vec3::new(2.5, 4.0, 4.0), 300.0, 4.0),
    };

    let body_material = materials.add(body_color);
    let head_material = materials.add(Color::srgb(0.45, 0.32, 0.18));
    let leg_material = materials.add(Color::srgb(0.42, 0.28, 0.16));

    // Main body
    let dino_entity = commands.spawn((
        Dinosaur,
        species,
        DinoHealth {
            current: health,
            max: health,
        },
        DinoAI {
            state: AIState::Roam,
            wander_target: None,
            flee_direction: Vec3::ZERO,
            move_speed: speed,
        },
        Transform::from_translation(position),
        RigidBody::KinematicPositionBased,
        Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
    )).id();

    // Body mesh
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(size.x * 0.4, size.z * 0.6))),
        MeshMaterial3d(body_material.clone()),
        Transform::from_xyz(0.0, size.y * 0.5, 0.0),
        HitBox { part: BodyPart::Body },
    )).set_parent(dino_entity);

    // Head
    let head_size = size.x * 0.4;
    let head_pos = match species {
        DinoSpecies::Triceratops => Vec3::new(0.0, size.y * 0.7, size.z * 0.4),
        DinoSpecies::Velociraptor => Vec3::new(0.0, size.y * 0.8, size.z * 0.5),
        DinoSpecies::Brachiosaurus => Vec3::new(0.0, size.y * 0.9, size.z * 0.4),
    };

    commands.spawn((
        Mesh3d(meshes.add(Sphere { radius: head_size })),
        MeshMaterial3d(head_material.clone()),
        Transform::from_translation(head_pos),
        HitBox { part: BodyPart::Head },
    )).set_parent(dino_entity);

    // Legs
    let leg_positions = [
        (-size.x * 0.3, 0.0, size.z * 0.2),
        (size.x * 0.3, 0.0, size.z * 0.2),
        (-size.x * 0.3, 0.0, -size.z * 0.2),
        (size.x * 0.3, 0.0, -size.z * 0.2),
    ];

    for leg_pos in leg_positions {
        let leg_height = match species {
            DinoSpecies::Brachiosaurus => size.y * 0.7,
            _ => size.y * 0.5,
        };

        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(size.x * 0.12, leg_height))),
            MeshMaterial3d(leg_material.clone()),
            Transform::from_xyz(leg_pos.0, leg_height * 0.5, leg_pos.2),
            HitBox { part: BodyPart::Legs },
        )).set_parent(dino_entity);
    }
}

fn handle_respawn_dinos(
    mut commands: Commands,
    mut events: EventReader<RespawnDinosEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<DinoSpawnConfig>,
) {
    for _event in events.read() {
        let mut rng = rand::thread_rng();

        for _ in 0..config.count {
            let species = match rng.gen_range(0..3) {
                0 => DinoSpecies::Triceratops,
                1 => DinoSpecies::Velociraptor,
                _ => DinoSpecies::Brachiosaurus,
            };

            let x: f32 = rng.gen_range(-config.spawn_radius..config.spawn_radius);
            let z: f32 = rng.gen_range(-config.spawn_radius..config.spawn_radius);

            // Don't spawn too close to origin
            if x.abs() < config.min_distance_from_player && z.abs() < config.min_distance_from_player {
                continue;
            }

            spawn_dinosaur(&mut commands, &mut meshes, &mut materials, species, Vec3::new(x, 0.0, z));
        }
    }
}

fn handle_bullet_hits(
    mut commands: Commands,
    mut events: EventReader<BulletHitEvent>,
    mut dino_q: Query<(&mut DinoHealth, &mut DinoAI)>,
    mut score: ResMut<GameScore>,
    mut combo: ResMut<ComboSystem>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        if let Ok((mut health, mut ai)) = dino_q.get_mut(event.target) {
            health.current -= event.damage;

            // Visual feedback - flash red
            commands.entity(event.target).insert(FlashDamage {
                timer: Timer::from_seconds(0.1, TimerMode::Once),
            });

            if health.current <= 0.0 {
                ai.state = AIState::Dead;

                // Add combo kill
                combo.add_kill();

                // Calculate base score
                let base_score = match event.hit_part {
                    BodyPart::Head => 200,
                    BodyPart::Body => 100,
                    BodyPart::Legs => 50,
                };

                // Apply combo multiplier
                let final_score = (base_score as f32 * combo.get_score_multiplier()) as u32;
                score.score += final_score;

                // Add death animation component
                commands.entity(event.target).insert(DinoDeath {
                    timer: Timer::from_seconds(3.0, TimerMode::Once),
                });
            }
        }
    }
}

#[derive(Component)]
struct FlashDamage {
    timer: Timer,
}

fn update_dino_ai(
    _time: Res<Time>,
    mut dino_q: Query<(&mut DinoAI, &Transform)>,
    vehicle_q: Query<&Transform, (With<super::vehicle::PlayerVehicle>, Without<Dinosaur>)>,
) {
    let Ok(vehicle_transform) = vehicle_q.get_single() else {
        return;
    };

    let vehicle_pos = vehicle_transform.translation;
    let mut rng = rand::thread_rng();

    for (mut ai, transform) in dino_q.iter_mut() {
        if ai.state == AIState::Dead {
            continue;
        }

        let dino_pos = transform.translation;
        let distance_to_vehicle = (vehicle_pos - dino_pos).length();

        // Flee if player is close
        if distance_to_vehicle < 30.0 && ai.state != AIState::Flee {
            ai.state = AIState::Flee;
            let flee_dir = (dino_pos - vehicle_pos).normalize();
            ai.flee_direction = Vec3::new(flee_dir.x, 0.0, flee_dir.z).normalize();
        }

        // Return to roaming after fleeing far enough
        if ai.state == AIState::Flee && distance_to_vehicle > 60.0 {
            ai.state = AIState::Roam;
        }

        // Roam behavior
        if ai.state == AIState::Roam {
            if ai.wander_target.is_none() || (dino_pos - ai.wander_target.unwrap()).length() < 5.0 {
                let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                let dist = rng.gen_range(20.0..50.0);
                ai.wander_target = Some(dino_pos + Vec3::new(angle.cos() * dist, 0.0, angle.sin() * dist));
            }
        }
    }
}

fn update_dino_movement(
    time: Res<Time>,
    mut dino_q: Query<(&mut Transform, &DinoAI)>,
) {
    let dt = time.delta_secs();

    for (mut transform, ai) in dino_q.iter_mut() {
        if ai.state == AIState::Dead || ai.state == AIState::Idle {
            continue;
        }

        let direction = match ai.state {
            AIState::Roam => {
                if let Some(target) = ai.wander_target {
                    (target - transform.translation).normalize()
                } else {
                    Vec3::ZERO
                }
            }
            AIState::Flee => ai.flee_direction,
            _ => Vec3::ZERO,
        };

        if direction.length_squared() > 0.01 {
            let movement = direction * ai.move_speed * dt;
            transform.translation.x += movement.x;
            transform.translation.z += movement.z;

            // Face movement direction
            let target_rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            transform.rotation = transform.rotation.slerp(target_rotation, 0.1);
        }
    }
}

fn check_dino_death(
    _dino_q: Query<(Entity, &DinoAI)>,
) {
    // Death is now handled in handle_bullet_hits
    // This function can be used for additional death checks
}

fn update_dino_death_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut dino_q: Query<(Entity, &mut DinoDeath, &mut Transform, &DinoSpecies)>,
) {
    for (entity, mut death, mut transform, species) in dino_q.iter_mut() {
        death.timer.tick(time.delta());

        // Fall over animation
        let progress = 1.0 - (death.timer.elapsed_secs() / death.timer.duration().as_secs_f32());
        let fall_angle = progress * std::f32::consts::FRAC_PI_2;

        // Rotate to fall on side
        transform.rotation = Quat::from_rotation_z(fall_angle);

        // Lower to ground
        let height = match species {
            DinoSpecies::Triceratops => 1.2,
            DinoSpecies::Velociraptor => 0.5,
            DinoSpecies::Brachiosaurus => 4.0,
        };
        transform.translation.y = (height * 0.5) * (1.0 - progress * 0.8);

        // Change color to indicate death
        if death.timer.finished() {
            // Despawn after animation
            commands.entity(entity).despawn_recursive();
        }
    }
}
