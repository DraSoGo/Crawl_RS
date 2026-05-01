#[derive(Clone, Copy, Debug)]
pub struct UiConfig {
    pub player_layer: u8,
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
    pub floor_difficulty_base: u32,
    pub floor_difficulty_per_depth: u32,
    pub depth_hp_scale: f32,
    pub depth_attack_scale: f32,
    pub ranged_attack_range: i32,
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
    hud_rows: 1,
    log_rows: 5,
};

pub const PLAYER: PlayerConfig = PlayerConfig {
    fov_radius: 8,
    base_hp: 20,
    base_attack: 4,
    base_defense: 1,
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
    final_depth: 10,
    floor_difficulty_base: 10,
    floor_difficulty_per_depth: 10,
    depth_hp_scale: 0.25,
    depth_attack_scale: 0.251,
    ranged_attack_range: 2,
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
