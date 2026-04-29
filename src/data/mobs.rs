//! Static mob definitions. Templates carry both base stats and any optional
//! "extras" — `OnHit` riders, `Regen`, `CasterHeal`, `Summoner`, `Flying`,
//! and the `AiKind` so spawn code can attach the right components.

use crossterm::style::Color;

use crate::ecs::components::{AiKind, OnHit};

#[derive(Clone, Copy, Debug)]
pub struct MobTemplate {
    pub name: &'static str,
    pub glyph: char,
    pub fg: Color,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub move_tiles: i32,
    pub sight: i32,
    pub xp: i32,
    pub difficulty: u32,
    pub min_depth: u32,
    pub ai: AiKind,
    pub on_hit: Option<OnHit>,
    pub regen_per_turn: i32,
    pub invisible: bool,
    pub caster_heal: Option<(i32, i32)>, // (heal_amount, chance_pct)
    pub summoner_chance: Option<i32>,
    pub flying: bool,
}

const fn base(
    name: &'static str,
    glyph: char,
    fg: Color,
    max_hp: i32,
    attack: i32,
    defense: i32,
    move_tiles: i32,
    sight: i32,
    xp: i32,
    difficulty: u32,
    min_depth: u32,
) -> MobTemplate {
    MobTemplate {
        name,
        glyph,
        fg,
        max_hp,
        attack,
        defense,
        move_tiles,
        sight,
        xp,
        difficulty,
        min_depth,
        ai: AiKind::Hostile,
        on_hit: None,
        regen_per_turn: 0,
        invisible: false,
        caster_heal: None,
        summoner_chance: None,
        flying: false,
    }
}

pub const TEMPLATES: &[MobTemplate] = &[
    // ---- Drop-in (Hostile AI, no extras) ------------------------------
    base("rat",         'r', Color::DarkYellow, 4,  1, 0, 1, 6, 2,  1,  1),
    base("bat",         'B', Color::DarkGrey,   3,  1, 0, 2, 6, 2,  1,  1),
    base("giant ant",   'a', Color::Red,        5,  1, 1, 1, 5, 2,  1,  1),
    base("jackal",      'j', Color::DarkYellow, 6,  2, 0, 2, 7, 3,  1,  1),
    base("green slime", 'J', Color::Green,      10, 1, 1, 1, 4, 3,  1,  1),
    base("goblin",      'g', Color::Green,      8,  2, 1, 1, 7, 5,  2,  1),
    base("cave snake",  's', Color::DarkGreen,  6,  2, 0, 1, 6, 4,  2,  2),
    base("kobold",      'k', Color::Red,        6,  2, 0, 1, 7, 4,  2,  2),
    base("gnoll",       'G', Color::DarkYellow, 12, 3, 1, 1, 7, 7,  3,  3),
    base("hobgoblin",   'H', Color::DarkRed,    14, 3, 2, 1, 7, 8,  3,  3),
    base("orc",         'o', Color::Magenta,    14, 4, 1, 1, 7, 10, 3,  3),
    base("zombie",      'Z', Color::DarkGreen,  18, 3, 2, 1, 5, 9,  3,  4),
    base("giant spider",'S', Color::DarkGrey,   16, 4, 1, 2, 7, 12, 4,  5),
    base("ogre",        'O', Color::DarkYellow, 24, 6, 2, 1, 6, 18, 5,  6),
    base("hill giant",  'T', Color::DarkRed,    34, 8, 3, 1, 7, 28, 7,  7),
    base("wraith",      'W', Color::DarkBlue,   22, 7, 2, 2, 8, 24, 7,  8),
    base("minotaur",    'M', Color::Red,        40, 9, 3, 1, 8, 36, 8,  9),

    // ---- Engine-new (composed from extra components) ------------------
    MobTemplate {
        ai: AiKind::Sleeper { wake_radius: 2 },
        ..base("sleeping rat", 'r', Color::Grey, 5, 1, 0, 1, 6, 3, 1, 1)
    },
    MobTemplate {
        ai: AiKind::Fleeing { flee_below_pct: 30 },
        ..base("skittish kobold", 'k', Color::DarkRed, 6, 2, 0, 1, 7, 5, 2, 2)
    },
    MobTemplate {
        ai: AiKind::Ranged { prefer_range: 2 },
        ..base("kobold archer", 'k', Color::Yellow, 6, 2, 0, 1, 8, 6, 3, 2)
    },
    MobTemplate {
        ai: AiKind::Ranged { prefer_range: 2 },
        ..base("skeleton archer", 'q', Color::White, 10, 3, 1, 1, 8, 12, 4, 4)
    },
    MobTemplate {
        ai: AiKind::Mimic { disguise: '!', revealed: false },
        ..base("mimic", 'm', Color::Magenta, 18, 5, 2, 1, 1, 20, 4, 5)
    },
    MobTemplate {
        on_hit: Some(OnHit { paralysis_turns: 2, ..no_on_hit() }),
        ..base("ghoul", 'C', Color::DarkGreen, 16, 4, 1, 1, 7, 16, 5, 6)
    },
    MobTemplate {
        regen_per_turn: 2,
        ..base("troll", 'T', Color::Green, 30, 6, 2, 1, 7, 30, 6, 7)
    },
    MobTemplate {
        invisible: true,
        ..base("shadow imp", 'i', Color::DarkGrey, 12, 4, 1, 2, 7, 16, 5, 6)
    },
    MobTemplate {
        caster_heal: Some((6, 25)),
        ..base("gnoll shaman", 'G', Color::Cyan, 18, 4, 1, 1, 7, 22, 5, 6)
    },
    MobTemplate {
        on_hit: Some(OnHit { poison_turns: 6, poison_dmg: 1, ..no_on_hit() }),
        ..base("wyvern", 'D', Color::DarkGreen, 26, 6, 2, 2, 8, 30, 7, 8)
    },
    MobTemplate {
        // Demon: approximated as high-attack hostile with fire-damage flavour.
        // True AoE breath would need targeting + tile fire — TODO.
        ..base("demon", '&', Color::DarkRed, 32, 8, 3, 1, 8, 40, 8, 9)
    },
    MobTemplate {
        summoner_chance: Some(20),
        ..base("lich", 'L', Color::DarkMagenta, 28, 7, 3, 1, 9, 45, 9, 9)
    },
    MobTemplate {
        flying: true,
        ..base("dragon", 'd', Color::Red, 60, 10, 4, 2, 10, 75, 10, 10)
    },
];

const fn no_on_hit() -> OnHit {
    OnHit { poison_turns: 0, poison_dmg: 0, paralysis_turns: 0 }
}

/// Pick a template appropriate for the supplied dungeon depth and budget.
pub fn pick_for_budget<R: rand::Rng>(
    depth: u32,
    budget: u32,
    rng: &mut R,
) -> Option<&'static MobTemplate> {
    let candidates: Vec<&MobTemplate> = TEMPLATES
        .iter()
        .filter(|t| t.min_depth <= depth && t.difficulty <= budget)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let idx = rng.gen_range(0..candidates.len());
    Some(candidates[idx])
}

/// Look up a template by name (used by the summon scroll and the lich).
pub fn by_name(name: &str) -> Option<&'static MobTemplate> {
    TEMPLATES.iter().find(|t| t.name == name)
}
