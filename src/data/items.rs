//! Static item templates: name, glyph, kind, depth gating.

use crossterm::style::Color;

use crate::ecs::components::{ItemKind, ScrollKind};

#[derive(Clone, Copy, Debug)]
pub struct ItemTemplate {
    pub name: &'static str,
    pub glyph: char,
    pub fg: Color,
    pub kind: ItemKind,
    pub min_depth: u32,
    /// Relative spawn weight at eligible depths.
    pub weight: u32,
}

pub const TEMPLATES: &[ItemTemplate] = &[
    ItemTemplate {
        name: "potion of healing",
        glyph: '!',
        fg: Color::Magenta,
        kind: ItemKind::Potion { heal: 8 },
        min_depth: 1,
        weight: 30,
    },
    ItemTemplate {
        name: "scroll of mapping",
        glyph: '?',
        fg: Color::Cyan,
        kind: ItemKind::Scroll(ScrollKind::Mapping),
        min_depth: 1,
        weight: 12,
    },
    ItemTemplate {
        name: "scroll of teleport",
        glyph: '?',
        fg: Color::Blue,
        kind: ItemKind::Scroll(ScrollKind::Teleport),
        min_depth: 2,
        weight: 8,
    },
    ItemTemplate {
        name: "iron dagger",
        glyph: '/',
        fg: Color::White,
        kind: ItemKind::Weapon { attack_bonus: 1 },
        min_depth: 1,
        weight: 10,
    },
    ItemTemplate {
        name: "iron sword",
        glyph: '/',
        fg: Color::White,
        kind: ItemKind::Weapon { attack_bonus: 3 },
        min_depth: 3,
        weight: 6,
    },
    ItemTemplate {
        name: "leather armor",
        glyph: '[',
        fg: Color::DarkYellow,
        kind: ItemKind::Armor { defense_bonus: 1 },
        min_depth: 1,
        weight: 10,
    },
    ItemTemplate {
        name: "chain mail",
        glyph: '[',
        fg: Color::Grey,
        kind: ItemKind::Armor { defense_bonus: 2 },
        min_depth: 4,
        weight: 5,
    },
];

/// Pick a template appropriate for the supplied dungeon depth via weighted
/// sampling.
pub fn pick_for_depth<R: rand::Rng>(depth: u32, rng: &mut R) -> Option<&'static ItemTemplate> {
    let candidates: Vec<&ItemTemplate> =
        TEMPLATES.iter().filter(|t| t.min_depth <= depth).collect();
    if candidates.is_empty() {
        return None;
    }
    let total: u32 = candidates.iter().map(|t| t.weight).sum();
    let mut roll = rng.gen_range(0..total);
    for t in candidates {
        if roll < t.weight {
            return Some(t);
        }
        roll -= t.weight;
    }
    None
}
