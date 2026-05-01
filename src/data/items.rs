//! Static item templates: name, glyph, kind, depth gating.

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
}

const fn potion(
    name: &'static str,
    fg: Color,
    effect: PotionEffect,
    min_depth: u32,
) -> ItemTemplate {
    ItemTemplate {
        name,
        glyph: '!',
        fg,
        kind: ItemKind::Potion(effect),
        min_depth,
    }
}

const fn scroll(
    name: &'static str,
    fg: Color,
    kind: ScrollKind,
    min_depth: u32,
) -> ItemTemplate {
    ItemTemplate {
        name,
        glyph: '?',
        fg,
        kind: ItemKind::Scroll(kind),
        min_depth,
    }
}

const fn weapon(
    name: &'static str,
    fg: Color,
    attack_bonus: i32,
    min_depth: u32,
) -> ItemTemplate {
    ItemTemplate {
        name,
        glyph: '/',
        fg,
        kind: ItemKind::Weapon { attack_bonus },
        min_depth,
    }
}

const fn armor(
    name: &'static str,
    fg: Color,
    defense_bonus: i32,
    min_depth: u32,
) -> ItemTemplate {
    ItemTemplate {
        name,
        glyph: '[',
        fg,
        kind: ItemKind::Armor { defense_bonus },
        min_depth,
    }
}

const fn ring(
    name: &'static str,
    fg: Color,
    effect: RingEffect,
    min_depth: u32,
) -> ItemTemplate {
    ItemTemplate {
        name,
        glyph: '=',
        fg,
        kind: ItemKind::Ring(effect),
        min_depth,
    }
}

const fn amulet(
    name: &'static str,
    fg: Color,
    effect: AmuletEffect,
    min_depth: u32,
) -> ItemTemplate {
    ItemTemplate {
        name,
        glyph: '"',
        fg,
        kind: ItemKind::AmuletItem(effect),
        min_depth,
    }
}

const fn wand(
    name: &'static str,
    fg: Color,
    kind: WandKind,
    charges: i32,
    min_depth: u32,
) -> ItemTemplate {
    ItemTemplate {
        name,
        glyph: '/',
        fg,
        kind: ItemKind::Wand { kind, charges },
        min_depth,
    }
}

const fn throwable(
    name: &'static str,
    fg: Color,
    kind: ThrowableKind,
    min_depth: u32,
) -> ItemTemplate {
    ItemTemplate {
        name,
        glyph: '!',
        fg,
        kind: ItemKind::Throwable(kind),
        min_depth,
    }
}

pub const TEMPLATES: &[ItemTemplate] = &[
    // Potions
    potion("potion of healing", Color::Magenta, PotionEffect::Heal(8), 1),
    potion(
        "potion of greater healing",
        Color::Red,
        PotionEffect::GreaterHeal(18),
        3,
    ),
    potion("potion of full healing", Color::White, PotionEffect::FullHeal, 6),
    potion(
        "potion of vitality",
        Color::Yellow,
        PotionEffect::MaxHpUp(2),
        5,
    ),
    potion(
        "potion of strength",
        Color::DarkRed,
        PotionEffect::BuffAttack { amount: 2, turns: 50 },
        3,
    ),
    potion(
        "potion of vision",
        Color::Blue,
        PotionEffect::BuffVision { amount: 4, turns: 50 },
        3,
    ),
    potion(
        "potion of cure poison",
        Color::Green,
        PotionEffect::CurePoison,
        3,
    ),
    potion(
        "potion of giant strength",
        Color::DarkMagenta,
        PotionEffect::BuffAttack { amount: 4, turns: 50 },
        10,
    ),
    potion(
        "potion of far sight",
        Color::Cyan,
        PotionEffect::BuffVision { amount: 6, turns: 50 },
        11,
    ),
    potion(
        "potion of fortitude",
        Color::DarkYellow,
        PotionEffect::MaxHpUp(4),
        12,
    ),
    // Scrolls
    scroll("scroll of mapping", Color::Cyan, ScrollKind::Mapping, 1),
    scroll("scroll of teleport", Color::Blue, ScrollKind::Teleport, 3),
    scroll(
        "scroll of magic missile",
        Color::Red,
        ScrollKind::MagicMissile,
        3,
    ),
    scroll(
        "scroll of enchant weapon",
        Color::Yellow,
        ScrollKind::EnchantWeapon,
        4,
    ),
    scroll(
        "scroll of enchant armor",
        Color::Yellow,
        ScrollKind::EnchantArmor,
        4,
    ),
    scroll("scroll of fear", Color::Magenta, ScrollKind::Fear, 5),
    scroll("scroll of summon", Color::Green, ScrollKind::Summon, 5),
    scroll("scroll of light", Color::DarkYellow, ScrollKind::Light, 2),
    scroll("scroll of recall", Color::DarkCyan, ScrollKind::Recall, 5),
    scroll(
        "scroll of chain lightning",
        Color::White,
        ScrollKind::ChainLightning,
        12,
    ),
    scroll(
        "scroll of greater fear",
        Color::DarkMagenta,
        ScrollKind::GreaterFear,
        13,
    ),
    scroll("scroll of legion", Color::Green, ScrollKind::Legion, 14),
    // Weapons
    weapon("iron dagger", Color::White, 1, 1),
    weapon("short sword", Color::White, 2, 2),
    weapon("iron sword", Color::White, 3, 4),
    weapon("war hammer", Color::Grey, 4, 5),
    weapon("battle axe", Color::DarkGrey, 4, 5),
    weapon("elven blade", Color::Green, 3, 4),
    weapon("greatsword", Color::White, 5, 7),
    weapon("enchanted longsword", Color::Cyan, 6, 10),
    weapon("runed greatsword", Color::Yellow, 7, 12),
    weapon("obsidian blade", Color::DarkGrey, 8, 16),
    // Armor
    armor("leather armor", Color::DarkYellow, 1, 1),
    armor("studded leather", Color::DarkYellow, 1, 2),
    armor("ring mail", Color::Grey, 2, 4),
    armor("chain mail", Color::Grey, 2, 5),
    armor("mythril chain", Color::Cyan, 3, 8),
    armor("plate mail", Color::White, 4, 9),
    armor("gothic plate", Color::DarkGrey, 5, 12),
    armor("dragon scale armor", Color::Red, 6, 17),
    // Rings
    ring("ring of protection", Color::Cyan, RingEffect::Protection, 4),
    ring("ring of vision", Color::Yellow, RingEffect::Vision, 6),
    ring("ring of regen", Color::Green, RingEffect::Regen, 13),
    // Amulets
    amulet(
        "amulet of teleport control",
        Color::Magenta,
        AmuletEffect::TeleportControl,
        12,
    ),
    // Wands
    wand("wand of fire", Color::Red, WandKind::Fire, 5, 4),
    wand("wand of cold", Color::Cyan, WandKind::Cold, 5, 4),
    wand("wand of lightning", Color::Yellow, WandKind::Lightning, 4, 6),
    wand("wand of storms", Color::White, WandKind::Storms, 4, 15),
    // Throwables
    throwable("oil flask", Color::DarkRed, ThrowableKind::OilFlask, 3),
    throwable("smoke bomb", Color::DarkGrey, ThrowableKind::SmokeBomb, 4),
];

/// Pick a template appropriate for the supplied dungeon depth.
pub fn pick_for_depth<R: rand::Rng>(depth: u32, rng: &mut R) -> Option<&'static ItemTemplate> {
    let candidates: Vec<(&ItemTemplate, u32)> = TEMPLATES
        .iter()
        .filter(|t| t.min_depth <= depth)
        .map(|t| (t, pick_weight(depth, t.min_depth)))
        .collect();
    if candidates.is_empty() {
        return None;
    }
    pick_weighted(&candidates, rng)
}

fn pick_weight(depth: u32, min_depth: u32) -> u32 {
    let age = depth.saturating_sub(min_depth).min(8);
    1 + (8 - age)
}

fn pick_weighted<R: rand::Rng>(
    candidates: &[(&'static ItemTemplate, u32)],
    rng: &mut R,
) -> Option<&'static ItemTemplate> {
    let total: u32 = candidates.iter().map(|(_, weight)| *weight).sum();
    if total == 0 {
        return None;
    }
    let mut roll = rng.gen_range(0..total);
    for (template, weight) in candidates {
        if roll < *weight {
            return Some(*template);
        }
        roll -= *weight;
    }
    candidates.last().map(|(template, _)| *template)
}
