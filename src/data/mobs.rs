//! Static mob definitions. The full v0.1 design calls for ~20 distinct mob
//! kinds; Phase 6 ships the starter set so the AI/combat/scheduler code paths
//! all light up. New mob types just append to `TEMPLATES`.

use crossterm::style::Color;

#[derive(Clone, Copy, Debug)]
pub struct MobTemplate {
    pub name: &'static str,
    pub glyph: char,
    pub fg: Color,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub sight: i32,
    pub xp: i32,
    /// Earliest dungeon depth this mob may spawn at (1-indexed).
    pub min_depth: u32,
}

pub const TEMPLATES: &[MobTemplate] = &[
    MobTemplate {
        name: "rat",
        glyph: 'r',
        fg: Color::DarkYellow,
        max_hp: 4,
        attack: 1,
        defense: 0,
        speed: 12,
        sight: 6,
        xp: 2,
        min_depth: 1,
    },
    MobTemplate {
        name: "goblin",
        glyph: 'g',
        fg: Color::Green,
        max_hp: 8,
        attack: 2,
        defense: 1,
        speed: 10,
        sight: 7,
        xp: 5,
        min_depth: 1,
    },
    MobTemplate {
        name: "kobold",
        glyph: 'k',
        fg: Color::Red,
        max_hp: 6,
        attack: 2,
        defense: 0,
        speed: 11,
        sight: 7,
        xp: 4,
        min_depth: 2,
    },
    MobTemplate {
        name: "orc",
        glyph: 'o',
        fg: Color::Magenta,
        max_hp: 14,
        attack: 4,
        defense: 1,
        speed: 10,
        sight: 7,
        xp: 10,
        min_depth: 3,
    },
];

/// Pick a template appropriate for the supplied dungeon depth.
pub fn pick_for_depth<R: rand::Rng>(depth: u32, rng: &mut R) -> &'static MobTemplate {
    let candidates: Vec<&MobTemplate> = TEMPLATES
        .iter()
        .filter(|t| t.min_depth <= depth)
        .collect();
    let idx = rng.gen_range(0..candidates.len().max(1));
    candidates[idx]
}
