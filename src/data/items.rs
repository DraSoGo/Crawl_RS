//! Static item templates: name, glyph, kind, depth gating, spawn weight.

use crossterm::style::Color;

use crate::ecs::components::{
    AmuletEffect, ItemKind, PotionEffect, RingEffect, ScrollKind, ThrowableKind, WandKind,
};

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
    // ---- Potions -------------------------------------------------------
    ItemTemplate {
        name: "potion of healing",
        glyph: '!',
        fg: Color::Magenta,
        kind: ItemKind::Potion(PotionEffect::Heal(8)),
        min_depth: 1,
        weight: 30,
    },
    ItemTemplate {
        name: "potion of greater healing",
        glyph: '!',
        fg: Color::Red,
        kind: ItemKind::Potion(PotionEffect::GreaterHeal(18)),
        min_depth: 3,
        weight: 14,
    },
    ItemTemplate {
        name: "potion of full healing",
        glyph: '!',
        fg: Color::White,
        kind: ItemKind::Potion(PotionEffect::FullHeal),
        min_depth: 5,
        weight: 6,
    },
    ItemTemplate {
        name: "potion of vitality",
        glyph: '!',
        fg: Color::Yellow,
        kind: ItemKind::Potion(PotionEffect::MaxHpUp(2)),
        min_depth: 4,
        weight: 5,
    },
    ItemTemplate {
        name: "potion of speed",
        glyph: '!',
        fg: Color::Cyan,
        kind: ItemKind::Potion(PotionEffect::BuffSpeed { amount: 5, turns: 30 }),
        min_depth: 2,
        weight: 8,
    },
    ItemTemplate {
        name: "potion of strength",
        glyph: '!',
        fg: Color::DarkRed,
        kind: ItemKind::Potion(PotionEffect::BuffAttack { amount: 2, turns: 50 }),
        min_depth: 2,
        weight: 8,
    },
    ItemTemplate {
        name: "potion of vision",
        glyph: '!',
        fg: Color::Blue,
        kind: ItemKind::Potion(PotionEffect::BuffVision { amount: 4, turns: 50 }),
        min_depth: 2,
        weight: 6,
    },
    ItemTemplate {
        name: "potion of cure poison",
        glyph: '!',
        fg: Color::Green,
        kind: ItemKind::Potion(PotionEffect::CurePoison),
        min_depth: 2,
        weight: 6,
    },
    // ---- Scrolls -------------------------------------------------------
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
        name: "scroll of identify",
        glyph: '?',
        fg: Color::White,
        kind: ItemKind::Scroll(ScrollKind::Identify),
        min_depth: 1,
        weight: 6,
    },
    ItemTemplate {
        name: "scroll of magic missile",
        glyph: '?',
        fg: Color::Red,
        kind: ItemKind::Scroll(ScrollKind::MagicMissile),
        min_depth: 2,
        weight: 8,
    },
    ItemTemplate {
        name: "scroll of enchant weapon",
        glyph: '?',
        fg: Color::Yellow,
        kind: ItemKind::Scroll(ScrollKind::EnchantWeapon),
        min_depth: 3,
        weight: 5,
    },
    ItemTemplate {
        name: "scroll of enchant armor",
        glyph: '?',
        fg: Color::Yellow,
        kind: ItemKind::Scroll(ScrollKind::EnchantArmor),
        min_depth: 3,
        weight: 5,
    },
    ItemTemplate {
        name: "scroll of fear",
        glyph: '?',
        fg: Color::Magenta,
        kind: ItemKind::Scroll(ScrollKind::Fear),
        min_depth: 4,
        weight: 4,
    },
    ItemTemplate {
        name: "scroll of summon",
        glyph: '?',
        fg: Color::Green,
        kind: ItemKind::Scroll(ScrollKind::Summon),
        min_depth: 4,
        weight: 4,
    },
    ItemTemplate {
        name: "scroll of light",
        glyph: '?',
        fg: Color::DarkYellow,
        kind: ItemKind::Scroll(ScrollKind::Light),
        min_depth: 1,
        weight: 6,
    },
    ItemTemplate {
        name: "scroll of recall",
        glyph: '?',
        fg: Color::DarkCyan,
        kind: ItemKind::Scroll(ScrollKind::Recall),
        min_depth: 3,
        weight: 4,
    },
    // ---- Weapons -------------------------------------------------------
    ItemTemplate {
        name: "club",
        glyph: '/',
        fg: Color::DarkYellow,
        kind: ItemKind::Weapon { attack_bonus: 0 },
        min_depth: 1,
        weight: 6,
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
        name: "short sword",
        glyph: '/',
        fg: Color::White,
        kind: ItemKind::Weapon { attack_bonus: 2 },
        min_depth: 2,
        weight: 8,
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
        name: "war hammer",
        glyph: '/',
        fg: Color::Grey,
        kind: ItemKind::Weapon { attack_bonus: 4 },
        min_depth: 4,
        weight: 5,
    },
    ItemTemplate {
        name: "battle axe",
        glyph: '/',
        fg: Color::DarkGrey,
        kind: ItemKind::Weapon { attack_bonus: 4 },
        min_depth: 4,
        weight: 4,
    },
    ItemTemplate {
        name: "elven blade",
        glyph: '/',
        fg: Color::Green,
        kind: ItemKind::Weapon { attack_bonus: 3 },
        min_depth: 3,
        weight: 4,
    },
    ItemTemplate {
        name: "greatsword",
        glyph: '/',
        fg: Color::White,
        kind: ItemKind::Weapon { attack_bonus: 5 },
        min_depth: 6,
        weight: 4,
    },
    ItemTemplate {
        name: "enchanted longsword",
        glyph: '/',
        fg: Color::Cyan,
        kind: ItemKind::Weapon { attack_bonus: 6 },
        min_depth: 8,
        weight: 2,
    },
    // ---- Armor ---------------------------------------------------------
    ItemTemplate {
        name: "leather armor",
        glyph: '[',
        fg: Color::DarkYellow,
        kind: ItemKind::Armor { defense_bonus: 1 },
        min_depth: 1,
        weight: 10,
    },
    ItemTemplate {
        name: "studded leather",
        glyph: '[',
        fg: Color::DarkYellow,
        kind: ItemKind::Armor { defense_bonus: 1 },
        min_depth: 1,
        weight: 8,
    },
    ItemTemplate {
        name: "ring mail",
        glyph: '[',
        fg: Color::Grey,
        kind: ItemKind::Armor { defense_bonus: 2 },
        min_depth: 3,
        weight: 6,
    },
    ItemTemplate {
        name: "chain mail",
        glyph: '[',
        fg: Color::Grey,
        kind: ItemKind::Armor { defense_bonus: 2 },
        min_depth: 4,
        weight: 5,
    },
    ItemTemplate {
        name: "mythril chain",
        glyph: '[',
        fg: Color::Cyan,
        kind: ItemKind::Armor { defense_bonus: 3 },
        min_depth: 6,
        weight: 3,
    },
    ItemTemplate {
        name: "plate mail",
        glyph: '[',
        fg: Color::White,
        kind: ItemKind::Armor { defense_bonus: 4 },
        min_depth: 7,
        weight: 3,
    },
    // ---- Rings ---------------------------------------------------------
    ItemTemplate {
        name: "ring of regen",
        glyph: '=',
        fg: Color::Green,
        kind: ItemKind::Ring(RingEffect::Regen),
        min_depth: 4,
        weight: 3,
    },
    ItemTemplate {
        name: "ring of protection",
        glyph: '=',
        fg: Color::Cyan,
        kind: ItemKind::Ring(RingEffect::Protection),
        min_depth: 3,
        weight: 4,
    },
    ItemTemplate {
        name: "ring of vision",
        glyph: '=',
        fg: Color::Yellow,
        kind: ItemKind::Ring(RingEffect::Vision),
        min_depth: 4,
        weight: 3,
    },
    // ---- Amulets -------------------------------------------------------
    ItemTemplate {
        name: "amulet of teleport control",
        glyph: '"',
        fg: Color::Magenta,
        kind: ItemKind::AmuletItem(AmuletEffect::TeleportControl),
        min_depth: 6,
        weight: 2,
    },
    // ---- Wands ---------------------------------------------------------
    ItemTemplate {
        name: "wand of fire",
        glyph: '/',
        fg: Color::Red,
        kind: ItemKind::Wand { kind: WandKind::Fire, charges: 5 },
        min_depth: 3,
        weight: 4,
    },
    ItemTemplate {
        name: "wand of cold",
        glyph: '/',
        fg: Color::Cyan,
        kind: ItemKind::Wand { kind: WandKind::Cold, charges: 5 },
        min_depth: 3,
        weight: 4,
    },
    ItemTemplate {
        name: "wand of lightning",
        glyph: '/',
        fg: Color::Yellow,
        kind: ItemKind::Wand { kind: WandKind::Lightning, charges: 4 },
        min_depth: 4,
        weight: 3,
    },
    // ---- Throwables ----------------------------------------------------
    ItemTemplate {
        name: "oil flask",
        glyph: '!',
        fg: Color::DarkRed,
        kind: ItemKind::Throwable(ThrowableKind::OilFlask),
        min_depth: 2,
        weight: 5,
    },
    ItemTemplate {
        name: "smoke bomb",
        glyph: '!',
        fg: Color::DarkGrey,
        kind: ItemKind::Throwable(ThrowableKind::SmokeBomb),
        min_depth: 3,
        weight: 4,
    },
    // ---- Food ----------------------------------------------------------
    ItemTemplate {
        name: "food ration",
        glyph: '%',
        fg: Color::DarkYellow,
        kind: ItemKind::Food { nutrition: 400, poisonous: false },
        min_depth: 1,
        weight: 12,
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
