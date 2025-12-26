use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use crate::weapon::BulletHitEvent;
use crate::GameScore;
use crate::pause::GameState;
use crate::combo::ComboSystem;

#[derive(Resource)]
pub struct CoinSystem {
    pub total_coins: u32,
}

impl Default for CoinSystem {
    fn default() -> Self {
        Self { total_coins: 0 }
    }
}

pub struct DinoPlugin;

#[derive(Component)]
pub struct Dinosaur;

#[derive(Component, Clone, Copy, PartialEq)]
pub enum DinoSpecies {
    Triceratops,
    Velociraptor,
    Brachiosaurus,
    Stegosaurus,
    TRex, // Boss
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
    pub attack_cooldown: Timer,
    pub attack_range: f32,
}

impl Default for DinoAI {
    fn default() -> Self {
        Self {
            state: AIState::Roam,
            wander_target: None,
            flee_direction: Vec3::ZERO,
            move_speed: 10.0,
            attack_cooldown: Timer::from_seconds(2.0, TimerMode::Once),
            attack_range: 15.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AIState {
    Idle,
    Roam,
    Flee,
    Attack,
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
            .init_resource::<CoinSystem>()
            .add_event::<RespawnDinosEvent>()
            .add_event::<DinoAttackEvent>()
            .add_systems(Startup, spawn_dinosaurs)
            .add_systems(Update, (
                handle_bullet_hits,
                handle_respawn_dinos,
                update_damage_reaction,
                update_dino_ai,
                update_dino_movement,
                process_dino_attacks,
                check_dino_death,
                update_dino_death_animation,
            ).chain().run_if(in_state(GameState::Playing)));
    }
}

#[derive(Event)]
pub struct RespawnDinosEvent;

#[derive(Event)]
pub struct DinoAttackEvent {
    pub damage: f32,
}

fn spawn_dinosaurs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();

    // Spawn dinosaurs (now 5 species)
    for i in 0..15 {
        // Spawn T-Rex Boss only once (first dinosaur)
        let species = if i == 0 && rng.gen_range(0..10) < 3 {
            // 30% chance for T-Rex to spawn as first dinosaur
            DinoSpecies::TRex
        } else {
            match rng.gen_range(0..5) {
                0 => DinoSpecies::Triceratops,
                1 => DinoSpecies::Velociraptor,
                2 => DinoSpecies::Brachiosaurus,
                3 => DinoSpecies::Stegosaurus,
                _ => DinoSpecies::Triceratops, // Weight toward Triceratops
            }
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
        DinoSpecies::Stegosaurus => (Color::srgb(0.35, 0.4, 0.25), Vec3::new(1.8, 1.0, 3.0), 200.0, 6.0),
        DinoSpecies::TRex => (Color::srgb(0.5, 0.3, 0.2), Vec3::new(2.2, 2.0, 3.5), 500.0, 10.0),
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
            attack_cooldown: Timer::from_seconds(2.0, TimerMode::Once),
            attack_range: if species == DinoSpecies::Velociraptor || species == DinoSpecies::TRex {
                20.0 // Aggressive dinos have longer attack range
            } else {
                0.0 // Other dinos don't attack
            },
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
        DinoSpecies::Stegosaurus => Vec3::new(0.0, size.y * 0.6, size.z * 0.35),
        DinoSpecies::TRex => Vec3::new(0.0, size.y * 0.75, size.z * 0.45),
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

        for i in 0..config.count {
            // First dinosaur might be a T-Rex
            let species = if i == 0 && rng.gen_range(0..10) < 3 {
                DinoSpecies::TRex
            } else {
                match rng.gen_range(0..5) {
                    0 => DinoSpecies::Triceratops,
                    1 => DinoSpecies::Velociraptor,
                    2 => DinoSpecies::Brachiosaurus,
                    3 => DinoSpecies::Stegosaurus,
                    _ => DinoSpecies::Triceratops,
                }
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
    mut dino_q: Query<(&mut DinoHealth, &mut DinoAI, &DinoSpecies)>,
    mut score: ResMut<GameScore>,
    mut combo: ResMut<ComboSystem>,
    mut coins: ResMut<CoinSystem>,
    mut time_attack: ResMut<crate::game_mode::TimeAttackMode>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    mut kill_shake_events: EventWriter<crate::effects::KillShakeEvent>,
) {
    for event in events.read() {
        if let Ok((mut health, mut ai, species)) = dino_q.get_mut(event.target) {
            health.current -= event.damage;

            // Add damage reaction - pause and flee faster
            if commands.get_entity(event.target).is_some() {
                commands.entity(event.target).insert(DamageReaction::new());
            }

            // Visual feedback - flash red
            commands.entity(event.target).insert(FlashDamage {
                timer: Timer::from_seconds(0.1, TimerMode::Once),
            });

            if health.current <= 0.0 {
                ai.state = AIState::Dead;

                // Add combo kill
                combo.add_kill();

                // Increment time attack mode kill counter
                if time_attack.is_active {
                    time_attack.kills += 1;
                }

                // Calculate base score and coins based on species
                let (base_score, coin_reward) = match species {
                    DinoSpecies::Velociraptor => (150, 15),
                    DinoSpecies::Triceratops => (200, 20),
                    DinoSpecies::Stegosaurus => (175, 25),
                    DinoSpecies::Brachiosaurus => (400, 30),
                    DinoSpecies::TRex => (1000, 100), // Boss gives huge rewards
                };

                // Apply hit part multiplier to score
                let base_score = match event.hit_part {
                    BodyPart::Head => base_score * 2,
                    BodyPart::Body => base_score,
                    BodyPart::Legs => base_score / 2,
                };

                // Apply combo multiplier to score
                let final_score = (base_score as f32 * combo.get_score_multiplier()) as u32;
                score.score += final_score;

                // Add coins (not affected by combo or hit part)
                coins.total_coins += coin_reward;

                // Trigger screen shake on kill
                kill_shake_events.send(crate::effects::KillShakeEvent);

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

#[derive(Component)]
pub struct DamageReaction {
    pub pause_timer: Timer,
    pub flee_boost: f32,
}

impl DamageReaction {
    pub fn new() -> Self {
        Self {
            pause_timer: Timer::from_seconds(0.3, TimerMode::Once),
            flee_boost: 1.5, // 50% speed boost when fleeing
        }
    }
}

fn update_dino_ai(
    time: Res<Time>,
    mut queries: ParamSet<(
        Query<(&mut DinoAI, &Transform)>,
        Query<&Transform, (With<super::vehicle::PlayerVehicle>, Without<Dinosaur>)>,
    )>,
) {
    let vehicle_pos = queries.p1().get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);
    let mut rng = rand::thread_rng();

    for (mut ai, transform) in queries.p0().iter_mut() {
        if ai.state == AIState::Dead {
            continue;
        }

        // Update attack cooldown
        ai.attack_cooldown.tick(time.delta());

        let dino_pos = transform.translation;
        let distance_to_vehicle = (vehicle_pos - dino_pos).length();

        // Attack behavior for aggressive dinos (Velociraptor, T-Rex)
        if ai.attack_range > 0.0 && distance_to_vehicle < ai.attack_range && ai.attack_cooldown.finished() {
            if ai.state != AIState::Attack {
                ai.state = AIState::Attack;
            }
        } else if distance_to_vehicle < 30.0 && ai.state != AIState::Flee && ai.state != AIState::Attack {
            // Flee if player is close (and not attacking)
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

fn update_damage_reaction(
    time: Res<Time>,
    mut commands: Commands,
    mut dino_q: Query<(Entity, &mut DinoAI, &mut DamageReaction)>,
) {
    for (entity, mut ai, mut reaction) in dino_q.iter_mut() {
        reaction.pause_timer.tick(time.delta());

        // While paused, don't process AI
        if !reaction.pause_timer.finished() {
            continue;
        }

        // Remove the component after pause expires
        commands.entity(entity).remove::<DamageReaction>();

        // Immediately switch to flee when recovering from damage
        if ai.state != AIState::Dead && ai.state != AIState::Flee {
            ai.state = AIState::Flee;
        }
    }
}

fn update_dino_movement(
    time: Res<Time>,
    mut queries: ParamSet<(
        Query<(&mut Transform, &DinoAI, Option<&DamageReaction>)>,
        Query<&Transform, (With<super::vehicle::PlayerVehicle>, Without<Dinosaur>)>,
    )>,
) {
    let dt = time.delta_secs();
    let vehicle_pos = queries.p1().get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);

    for (mut transform, ai, damage_reaction) in queries.p0().iter_mut() {
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
            AIState::Attack => {
                // Move toward vehicle when attacking
                (vehicle_pos - transform.translation).normalize()
            }
            _ => Vec3::ZERO,
        };

        if direction.length_squared() > 0.01 {
            // Apply speed boost when damaged and fleeing
            let speed_boost = if ai.state == AIState::Flee && damage_reaction.is_some() {
                damage_reaction.unwrap().flee_boost
            } else if ai.state == AIState::Attack {
                1.5 // Move faster when attacking
            } else {
                1.0
            };

            let movement = direction * ai.move_speed * speed_boost * dt;
            transform.translation.x += movement.x;
            transform.translation.z += movement.z;

            // Face movement direction
            let target_rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            transform.rotation = transform.rotation.slerp(target_rotation, 0.1);
        }
    }
}

fn process_dino_attacks(
    time: Res<Time>,
    mut dino_q: Query<(Entity, &mut DinoAI, &Transform, &DinoSpecies)>,
    mut vehicle_queries: ParamSet<(
        Query<&Transform, With<super::vehicle::PlayerVehicle>>,
        Query<&mut super::vehicle::VehicleHealth>,
    )>,
    mut attack_events: EventWriter<DinoAttackEvent>,
    mut hit_feedback: EventWriter<crate::effects::HitFeedbackEvent>,
) {
    let vehicle_pos = vehicle_queries.p0().get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);

    for (entity, mut ai, dino_transform, species) in dino_q.iter_mut() {
        if ai.state != AIState::Attack {
            continue;
        }

        let dino_pos = dino_transform.translation;
        let distance_to_vehicle = (vehicle_pos - dino_pos).length();

        // Check if dino has reached the vehicle to attack
        if distance_to_vehicle < 3.0 && ai.attack_cooldown.finished() {
            // Calculate damage based on species
            let damage = match species {
                DinoSpecies::Velociraptor => 10.0,
                DinoSpecies::TRex => 25.0,
                _ => 5.0,
            };

            // Apply damage to vehicle
            if let Ok(mut vehicle_health) = vehicle_queries.p1().get_single_mut() {
                vehicle_health.current -= damage;
                vehicle_health.current = vehicle_health.current.max(0.0);

                // Trigger hit feedback
                hit_feedback.send(crate::effects::HitFeedbackEvent);
            }

            // Send attack event
            attack_events.send(DinoAttackEvent { damage });

            // Reset attack cooldown
            ai.attack_cooldown.reset();

            // After attacking, switch to flee or roam
            ai.state = AIState::Flee;
        }
        // If too far from vehicle while attacking, switch back to roaming
        else if distance_to_vehicle > ai.attack_range * 1.5 {
            ai.state = AIState::Roam;
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
            DinoSpecies::Stegosaurus => 1.0,
            DinoSpecies::TRex => 2.0,
        };
        transform.translation.y = (height * 0.5) * (1.0 - progress * 0.8);

        // Change color to indicate death
        if death.timer.finished() {
            // Despawn after animation
            commands.entity(entity).despawn_recursive();
        }
    }
}
