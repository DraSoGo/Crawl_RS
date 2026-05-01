//! Plain-data structs that get serialised. Kept separate so the schema is
//! easy to scan when bumping `SAVE_VERSION`.

use serde::{Deserialize, Serialize};

use crate::ecs::components::{
    Ai, HungerClock, ItemKind, Position, Progression, Stats, StatusEffects,
};
use crate::map::Map;
use crate::ui::messages::Severity;

/// Bumped every time the snapshot schema changes incompatibly. v5 adds new
/// item variants for the extended 20-floor content curve.
pub const SAVE_VERSION: u32 = 5;
pub const SAVE_FILENAME: &str = "save.bin";

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveSnapshot {
    pub version: u32,
    pub seed: u64,
    pub depth: u32,
    pub map: Map,
    pub player: PlayerSnapshot,
    pub mobs: Vec<MobSnapshot>,
    pub ground_items: Vec<ItemSnapshot>,
    pub amulet: Option<Position>,
    pub log: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerSnapshot {
    pub pos: Position,
    pub stats: Stats,
    pub progression: Progression,
    pub fov_radius: i32,
    pub fov_revealed: Vec<bool>,
    pub fov_visible: Vec<bool>,
    pub fov_w: i32,
    pub fov_h: i32,
    pub inventory: Vec<ItemSnapshot>,
    pub equipped_weapon_idx: Option<usize>,
    pub equipped_armor_idx: Option<usize>,
    pub equipped_ring_idx: Option<usize>,
    pub equipped_amulet_idx: Option<usize>,
    pub renderable_glyph: char,
    pub status: StatusEffects,
    pub hunger: HungerClock,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MobSnapshot {
    pub pos: Position,
    pub stats: Stats,
    pub ai: Ai,
    pub name: String,
    pub glyph: char,
    pub fg_index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemSnapshot {
    pub pos: Option<Position>,
    pub kind: ItemKind,
    pub name: String,
    pub glyph: char,
    pub fg_index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub text: String,
    pub severity: Severity,
}

pub fn encode_color(c: crossterm::style::Color) -> u8 {
    use crossterm::style::Color;
    match c {
        Color::Reset => 0,
        Color::Black => 1,
        Color::DarkGrey => 2,
        Color::Red => 3,
        Color::DarkRed => 4,
        Color::Green => 5,
        Color::DarkGreen => 6,
        Color::Yellow => 7,
        Color::DarkYellow => 8,
        Color::Blue => 9,
        Color::DarkBlue => 10,
        Color::Magenta => 11,
        Color::DarkMagenta => 12,
        Color::Cyan => 13,
        Color::DarkCyan => 14,
        Color::White => 15,
        Color::Grey => 16,
        _ => 0,
    }
}

pub fn decode_color(idx: u8) -> crossterm::style::Color {
    use crossterm::style::Color;
    match idx {
        1 => Color::Black,
        2 => Color::DarkGrey,
        3 => Color::Red,
        4 => Color::DarkRed,
        5 => Color::Green,
        6 => Color::DarkGreen,
        7 => Color::Yellow,
        8 => Color::DarkYellow,
        9 => Color::Blue,
        10 => Color::DarkBlue,
        11 => Color::Magenta,
        12 => Color::DarkMagenta,
        13 => Color::Cyan,
        14 => Color::DarkCyan,
        15 => Color::White,
        16 => Color::Grey,
        _ => Color::Reset,
    }
}
