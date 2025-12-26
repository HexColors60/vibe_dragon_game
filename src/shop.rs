use bevy::prelude::*;
use crate::pause::GameState;
use crate::dino::CoinSystem;
use crate::weapon_system::WeaponType;
use crate::vehicle::VehicleHealth;
use crate::input::PlayerInput;

#[derive(Resource, Default)]
pub struct ShopState {
    pub is_open: bool,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct WeaponUpgrades {
    pub machinegun_damage_level: u32,
    pub machinegun_fire_rate_level: u32,
    pub shotgun_damage_level: u32,
    pub shotgun_pellet_level: u32,
    pub rocket_damage_level: u32,
    pub rocket_radius_level: u32,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct VehicleUpgrades {
    pub max_health_level: u32,
    pub speed_level: u32,
    pub acceleration_level: u32,
}

#[derive(Component)]
pub struct ShopMenu;

#[derive(Component)]
pub struct ShopButton;

#[derive(Component)]
pub struct UpgradeButton {
    pub upgrade_type: UpgradeType,
    pub cost: u32,
    pub level: u32,
    pub max_level: u32,
}

#[derive(Clone, Copy)]
pub enum UpgradeType {
    MachineGunDamage,
    MachineGunFireRate,
    ShotgunDamage,
    ShotgunPellets,
    RocketDamage,
    RocketRadius,
    VehicleMaxHealth,
    VehicleSpeed,
    VehicleAcceleration,
}

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopState>()
            .init_resource::<WeaponUpgrades>()
            .init_resource::<VehicleUpgrades>()
            .add_systems(Update, (
                handle_shop_toggle,
                update_shop_ui,
            ).run_if(in_state(GameState::Playing)));
    }
}

fn handle_shop_toggle(
    input: Res<PlayerInput>,
    mut shop_state: ResMut<ShopState>,
    mut commands: Commands,
    weapon_upgrades: Res<WeaponUpgrades>,
    vehicle_upgrades: Res<VehicleUpgrades>,
    coins: Res<CoinSystem>,
) {
    // Toggle shop with TAB key
    if input.pause {
        shop_state.is_open = !shop_state.is_open;

        if shop_state.is_open {
            spawn_shop_menu(&mut commands, &weapon_upgrades, &vehicle_upgrades, &coins);
        }
    }
}

fn spawn_shop_menu(
    commands: &mut Commands,
    weapon_upgrades: &WeaponUpgrades,
    vehicle_upgrades: &VehicleUpgrades,
    coins: &CoinSystem,
) {
    commands.spawn((
        ShopMenu,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("SHOP"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.8, 0.2)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Coins display
        parent.spawn((
            Text::new(format!("Coins: {}", coins.total_coins)),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.84, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        // Weapon Upgrades Section
        parent.spawn((
            Text::new("WEAPON UPGRADES"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        // Machine Gun Damage
        let cost = weapon_upgrades.machinegun_damage_level * 100 + 100;
        parent.spawn((
            ShopButton,
            UpgradeButton {
                upgrade_type: UpgradeType::MachineGunDamage,
                cost,
                level: weapon_upgrades.machinegun_damage_level,
                max_level: 5,
            },
            Node {
                width: Val::Px(400.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new(format!("MG Damage [Level {}] - Cost: {}", weapon_upgrades.machinegun_damage_level, cost)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        // Machine Gun Fire Rate
        let cost = weapon_upgrades.machinegun_fire_rate_level * 120 + 150;
        parent.spawn((
            ShopButton,
            UpgradeButton {
                upgrade_type: UpgradeType::MachineGunFireRate,
                cost,
                level: weapon_upgrades.machinegun_fire_rate_level,
                max_level: 5,
            },
            Node {
                width: Val::Px(400.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new(format!("MG Fire Rate [Level {}] - Cost: {}", weapon_upgrades.machinegun_fire_rate_level, cost)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        // Vehicle Upgrades Section
        parent.spawn((
            Text::new("VEHICLE UPGRADES"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::top(Val::Px(20.0)).with_bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        // Vehicle Max Health
        let cost = vehicle_upgrades.max_health_level * 200 + 200;
        parent.spawn((
            ShopButton,
            UpgradeButton {
                upgrade_type: UpgradeType::VehicleMaxHealth,
                cost,
                level: vehicle_upgrades.max_health_level,
                max_level: 5,
            },
            Node {
                width: Val::Px(400.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new(format!("Vehicle Health [Level {}] - Cost: {}", vehicle_upgrades.max_health_level, cost)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        // Instructions
        parent.spawn((
            Text::new("Press TAB to close shop"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
            Node {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
        ));
    });
}

fn update_shop_ui(
    mut commands: Commands,
    shop_state: Res<ShopState>,
    shop_menu_q: Query<Entity, With<ShopMenu>>,
    mut interaction_q: Query<
        (&Interaction, &UpgradeButton),
        (With<ShopButton>, Changed<Interaction>)
    >,
    mut weapon_upgrades: ResMut<WeaponUpgrades>,
    mut vehicle_upgrades: ResMut<VehicleUpgrades>,
    mut coins: ResMut<CoinSystem>,
    mut vehicle_health: Query<&mut VehicleHealth, With<crate::vehicle::PlayerVehicle>>,
) {
    // Remove shop menu if closed
    if !shop_state.is_open {
        for entity in shop_menu_q.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    // Handle button clicks
    for (interaction, upgrade) in interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            if coins.total_coins >= upgrade.cost && upgrade.level < upgrade.max_level {
                // Deduct coins
                coins.total_coins -= upgrade.cost;

                // Apply upgrade
                match upgrade.upgrade_type {
                    UpgradeType::MachineGunDamage => {
                        weapon_upgrades.machinegun_damage_level += 1;
                    }
                    UpgradeType::MachineGunFireRate => {
                        weapon_upgrades.machinegun_fire_rate_level += 1;
                    }
                    UpgradeType::ShotgunDamage => {
                        weapon_upgrades.shotgun_damage_level += 1;
                    }
                    UpgradeType::ShotgunPellets => {
                        weapon_upgrades.shotgun_pellet_level += 1;
                    }
                    UpgradeType::RocketDamage => {
                        weapon_upgrades.rocket_damage_level += 1;
                    }
                    UpgradeType::RocketRadius => {
                        weapon_upgrades.rocket_radius_level += 1;
                    }
                    UpgradeType::VehicleMaxHealth => {
                        vehicle_upgrades.max_health_level += 1;
                        // Also restore some health when upgrading
                        if let Ok(mut health) = vehicle_health.get_single_mut() {
                            health.max += 20.0;
                            health.current = (health.current + 20.0).min(health.max);
                        }
                    }
                    UpgradeType::VehicleSpeed => {
                        vehicle_upgrades.speed_level += 1;
                    }
                    UpgradeType::VehicleAcceleration => {
                        vehicle_upgrades.acceleration_level += 1;
                    }
                }

                // Respawn shop menu to show updated costs
                for entity in shop_menu_q.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                spawn_shop_menu(&mut commands, &weapon_upgrades, &vehicle_upgrades, &coins);
            }
        }
    }
}
