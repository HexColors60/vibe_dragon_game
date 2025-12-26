use bevy::prelude::*;

/// Different weapon types available in the game
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeaponType {
    #[default]
    MachineGun,
    Shotgun,
    RocketLauncher,
}

impl WeaponType {
    pub fn name(&self) -> &str {
        match self {
            WeaponType::MachineGun => "Machine Gun",
            WeaponType::Shotgun => "Shotgun",
            WeaponType::RocketLauncher => "Rocket Launcher",
        }
    }

    pub fn fire_rate(&self) -> f32 {
        match self {
            WeaponType::MachineGun => 0.1,
            WeaponType::Shotgun => 0.8,
            WeaponType::RocketLauncher => 2.0,
        }
    }

    pub fn damage(&self) -> f32 {
        match self {
            WeaponType::MachineGun => 10.0,
            WeaponType::Shotgun => 15.0, // Per pellet
            WeaponType::RocketLauncher => 100.0,
        }
    }

    pub fn pellet_count(&self) -> u32 {
        match self {
            WeaponType::MachineGun => 1,
            WeaponType::Shotgun => 8,
            WeaponType::RocketLauncher => 1,
        }
    }

    pub fn spread(&self) -> f32 {
        match self {
            WeaponType::MachineGun => 0.0,
            WeaponType::Shotgun => 0.15, // Spread angle for shotgun
            WeaponType::RocketLauncher => 0.0,
        }
    }

    pub fn bullet_speed(&self) -> f32 {
        match self {
            WeaponType::MachineGun => 100.0,
            WeaponType::Shotgun => 80.0,
            WeaponType::RocketLauncher => 60.0,
        }
    }

    pub fn bullet_radius(&self) -> f32 {
        match self {
            WeaponType::MachineGun => 0.2,
            WeaponType::Shotgun => 0.15,
            WeaponType::RocketLauncher => 0.3,
        }
    }

    pub fn explosive(&self) -> bool {
        matches!(self, WeaponType::RocketLauncher)
    }

    pub fn explosion_radius(&self) -> f32 {
        match self {
            WeaponType::RocketLauncher => 8.0,
            _ => 0.0,
        }
    }

    pub fn rocket_delay(&self) -> f32 {
        match self {
            WeaponType::RocketLauncher => 1.0, // seconds before explosion
            _ => 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct WeaponInventory {
    pub current_weapon: WeaponType,
    pub unlocked_weapons: Vec<WeaponType>,
}

impl WeaponInventory {
    pub fn new() -> Self {
        Self {
            current_weapon: WeaponType::MachineGun,
            unlocked_weapons: vec![
                WeaponType::MachineGun,
                WeaponType::Shotgun,
                WeaponType::RocketLauncher,
            ],
        }
    }

    pub fn switch_to(&mut self, weapon: WeaponType) {
        if self.unlocked_weapons.contains(&weapon) {
            self.current_weapon = weapon;
        }
    }

    pub fn next_weapon(&mut self) {
        let current_idx = self.unlocked_weapons.iter()
            .position(|w| *w == self.current_weapon)
            .unwrap_or(0);

        let next_idx = (current_idx + 1) % self.unlocked_weapons.len();
        self.current_weapon = self.unlocked_weapons[next_idx];
    }

    pub fn previous_weapon(&mut self) {
        let current_idx = self.unlocked_weapons.iter()
            .position(|w| *w == self.current_weapon)
            .unwrap_or(0);

        let prev_idx = if current_idx == 0 {
            self.unlocked_weapons.len() - 1
        } else {
            current_idx - 1
        };
        self.current_weapon = self.unlocked_weapons[prev_idx];
    }

    pub fn get_current_stats(&self) -> WeaponStats {
        WeaponStats {
            weapon_type: self.current_weapon,
            name: self.current_weapon.name().to_string(),
            fire_rate: self.current_weapon.fire_rate(),
            damage: self.current_weapon.damage(),
            pellet_count: self.current_weapon.pellet_count(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WeaponStats {
    pub weapon_type: WeaponType,
    pub name: String,
    pub fire_rate: f32,
    pub damage: f32,
    pub pellet_count: u32,
}

/// Event fired when weapon is switched
#[derive(Event)]
pub struct WeaponSwitchedEvent {
    pub new_weapon: WeaponType,
}
