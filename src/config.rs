#[derive(Clone, Copy, Debug)]
pub struct UiConfig {
    pub player_layer: u8,
    pub top_bar_rows: u16,
    pub hud_rows: u16,
    pub log_rows: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerConfig {
    pub fov_radius: i32,
    pub base_hp: i32,
    pub base_attack: i32,
    pub base_defense: i32,
    pub base_move: i32,
    pub descent_heal: i32,
    pub start_satiation: i32,
    pub level_up_hp: i32,
    pub level_up_attack: i32,
    pub level_up_defense: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct ProgressionConfig {
    pub xp_per_level: i32,
    pub inventory_base_slots: usize,
    pub inventory_slots_per_level: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct WorldConfig {
    pub final_depth: u32,
    pub depth_hp_scale: f32,
    pub depth_attack_scale: f32,
    pub ranged_attack_range: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct MapConfig {
    pub width: i32,
    pub height: i32,
}

/// Per-room spawning. Each room always gets 1 mob and 1 item.
/// Extra mobs roll against `extra_mob_chance_per_depth * (depth - 1)`,
/// up to `max_extra_mobs_per_room` additional mobs.
/// depth 5 → 4 * 0.0625 = 25% chance of a 2nd mob per room.
#[derive(Clone, Copy, Debug)]
pub struct MobSpawnConfig {
    pub extra_mob_chance_per_depth: f32,
    pub max_extra_mobs_per_room: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct CombatConfig {
    pub flat_damage_bonus: i32,
    pub damage_roll_sides: i32,
    pub minimum_damage: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct HungerConfig {
    pub starve_threshold: i32,
    pub hungry_threshold: i32,
}

pub const UI: UiConfig = UiConfig {
    player_layer: 200,
    top_bar_rows: 1,
    hud_rows: 1,
    log_rows: 5,
};

pub const PLAYER: PlayerConfig = PlayerConfig {
    fov_radius: 8,
    base_hp: 20,
    base_attack: 3,
    base_defense: 0,
    base_move: 1,
    descent_heal: 5,
    start_satiation: 800,
    level_up_hp: 5,
    level_up_attack: 1,
    level_up_defense: 1,
};

pub const PROGRESSION: ProgressionConfig = ProgressionConfig {
    xp_per_level: 50,
    inventory_base_slots: 26,
    inventory_slots_per_level: 5,
};

pub const WORLD: WorldConfig = WorldConfig {
    final_depth: 20,
    depth_hp_scale: 0.12,
    depth_attack_scale: 0.12,
    ranged_attack_range: 2,
};

pub const MAP: MapConfig = MapConfig {
    width: 80,
    height: 40,
};

pub const MOB_SPAWN: MobSpawnConfig = MobSpawnConfig {
    extra_mob_chance_per_depth: 0.0625,
    max_extra_mobs_per_room: 3,
};

pub const COMBAT: CombatConfig = CombatConfig {
    flat_damage_bonus: 0,
    damage_roll_sides: 0,
    minimum_damage: 1,
};

pub const HUNGER: HungerConfig = HungerConfig {
    starve_threshold: 0,
    hungry_threshold: 200,
};
