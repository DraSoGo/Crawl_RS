use crate::config;
use crate::data::{items::ItemTemplate, mobs::MobTemplate};
use crate::ecs::components::{
    AiKind, AmuletEffect, ItemKind, PotionEffect, RingEffect, ScrollKind, ThrowableKind,
    WandKind,
};

pub fn describe_mob_abilities(template: &MobTemplate) -> String {
    let mut parts: Vec<String> = Vec::new();

    match template.ai {
        AiKind::Hostile => {}
        AiKind::Sleeper { wake_radius } => {
            parts.push(format!("sleeps until you come within {wake_radius} tiles"));
        }
        AiKind::Fleeing { flee_below_pct } => {
            parts.push(format!("flees below {flee_below_pct}% hp"));
        }
        AiKind::Ranged { prefer_range } => {
            parts.push(format!("prefers ranged attacks at {prefer_range} tiles"));
        }
        AiKind::Mimic { .. } => {
            parts.push("disguises itself until revealed".to_string());
        }
    }

    if let Some(on_hit) = template.on_hit {
        if on_hit.poison_turns > 0 {
            parts.push(format!(
                "poisons on hit ({} dmg for {} turns)",
                on_hit.poison_dmg, on_hit.poison_turns
            ));
        }
        if on_hit.paralysis_turns > 0 {
            parts.push(format!(
                "paralyzes on hit ({} turns)",
                on_hit.paralysis_turns
            ));
        }
    }

    if template.regen_per_turn > 0 {
        parts.push(format!("regenerates {} hp/turn", template.regen_per_turn));
    }
    if template.invisible {
        parts.push("invisible until adjacent".to_string());
    }
    if let Some((heal_amount, chance_pct)) = template.caster_heal {
        parts.push(format!("self-heals {heal_amount} hp ({chance_pct}% chance/turn)"));
    }
    if let Some(chance_pct) = template.summoner_chance {
        parts.push(format!("summons allies ({chance_pct}% chance/turn)"));
    }
    if template.flying {
        parts.push("flying (currently informational)".to_string());
    }

    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join("; ")
    }
}

pub fn mob_ai_label(template: &MobTemplate) -> &'static str {
    match template.ai {
        AiKind::Hostile => "hostile",
        AiKind::Sleeper { .. } => "sleeper",
        AiKind::Fleeing { .. } => "fleeing",
        AiKind::Ranged { .. } => "ranged",
        AiKind::Mimic { .. } => "mimic",
    }
}

pub fn mob_attack_range(template: &MobTemplate) -> String {
    // Always report the tile count so the codex line reads consistently
    // alongside item ranges. Melee mobs hit at distance 1 (adjacent);
    // ranged mobs hit at the configured ranged-attack range.
    match template.ai {
        AiKind::Ranged { .. } => format!("{} tiles", config::WORLD.ranged_attack_range),
        _ => "1 tile (adjacent)".to_string(),
    }
}

pub fn describe_item_function(template: &ItemTemplate) -> String {
    match template.kind {
        ItemKind::Potion(effect) => describe_potion(effect),
        ItemKind::Scroll(scroll) => describe_scroll(scroll).to_string(),
        ItemKind::Weapon { attack_bonus } => {
            format!("Equip to gain +{attack_bonus} attack.")
        }
        ItemKind::Armor { defense_bonus } => {
            format!("Equip to gain +{defense_bonus} defense.")
        }
        ItemKind::Ring(effect) => describe_ring(effect).to_string(),
        ItemKind::AmuletItem(effect) => describe_amulet(effect).to_string(),
        ItemKind::Wand { kind, charges } => describe_wand(kind, charges),
        ItemKind::Throwable(kind) => describe_throwable(kind).to_string(),
        ItemKind::Food { nutrition, poisonous } => {
            if poisonous {
                format!("Eat to restore {nutrition} satiation, but it poisons you.")
            } else {
                format!("Eat to restore {nutrition} satiation.")
            }
        }
        ItemKind::Corpse => "No current use.".to_string(),
    }
}

/// Display label for an item's "Range" line in the codex.
///
/// Two buckets:
/// * Self-targeted, equip-only, or passive items → `-`. They never strike
///   a mob, so a tile count would be misleading.
/// * Mob-attacking items → tile count, with the **minimum 2-tile rule**: any
///   item that damages mobs must reach at least 2 tiles. A 1-tile attack
///   item would put the player on the wrong side of the bump-attack rule
///   (you swing, the mob retaliates next turn). The throwables below are
///   tuned to a 2-tile burst in `inventory::consume::throw_item` to honour
///   this; the wand/missile auto-targets the nearest visible mob, which is
///   always at least 2 tiles away because step 1 = melee (use weapon).
pub fn item_range(template: &ItemTemplate) -> String {
    match template.kind {
        // Self-target / equip / passive — no mob targeting from this slot.
        ItemKind::Potion(_)
        | ItemKind::Armor { .. }
        | ItemKind::Ring(_)
        | ItemKind::AmuletItem(_)
        | ItemKind::Weapon { .. }
        | ItemKind::Food { .. }
        | ItemKind::Corpse => "-".to_string(),
        ItemKind::Scroll(ScrollKind::Mapping)
        | ItemKind::Scroll(ScrollKind::Teleport)
        | ItemKind::Scroll(ScrollKind::Identify)
        | ItemKind::Scroll(ScrollKind::EnchantWeapon)
        | ItemKind::Scroll(ScrollKind::EnchantArmor)
        | ItemKind::Scroll(ScrollKind::Light)
        | ItemKind::Scroll(ScrollKind::Recall)
        | ItemKind::Scroll(ScrollKind::Summon) => "-".to_string(),
        // Mob-attacking — clamp to >= 2 tiles per the min-range rule.
        ItemKind::Scroll(ScrollKind::MagicMissile) => "any visible mob".to_string(),
        ItemKind::Scroll(ScrollKind::Fear) => "8 tiles around you".to_string(),
        ItemKind::Wand { .. } => "any visible mob".to_string(),
        ItemKind::Throwable(ThrowableKind::OilFlask) => "2 tiles around you".to_string(),
        ItemKind::Throwable(ThrowableKind::SmokeBomb) => "2 tiles around you".to_string(),
    }
}

pub fn item_duration(template: &ItemTemplate) -> &'static str {
    match template.kind {
        ItemKind::Potion(effect) => match effect {
            PotionEffect::Heal(_) => "instant",
            PotionEffect::GreaterHeal(_) => "instant",
            PotionEffect::FullHeal => "instant",
            PotionEffect::MaxHpUp(_) => "permanent",
            PotionEffect::BuffAttack { turns, .. } => turns_to_label(turns),
            PotionEffect::BuffVision { turns, .. } => turns_to_label(turns),
            PotionEffect::CurePoison => "instant",
        },
        ItemKind::Scroll(ScrollKind::Mapping) => "this floor",
        ItemKind::Scroll(ScrollKind::Teleport) => "instant",
        ItemKind::Scroll(ScrollKind::Identify) => "none",
        ItemKind::Scroll(ScrollKind::MagicMissile) => "instant",
        ItemKind::Scroll(ScrollKind::EnchantWeapon) => "permanent",
        ItemKind::Scroll(ScrollKind::EnchantArmor) => "permanent",
        ItemKind::Scroll(ScrollKind::Fear) => "10 turns",
        ItemKind::Scroll(ScrollKind::Summon) => "until allies die",
        ItemKind::Scroll(ScrollKind::Light) => "100 turns",
        ItemKind::Scroll(ScrollKind::Recall) => "instant",
        ItemKind::Weapon { .. } => "while equipped",
        ItemKind::Armor { .. } => "while equipped",
        ItemKind::Ring(_) => "while equipped",
        ItemKind::AmuletItem(_) => "while equipped",
        ItemKind::Wand { .. } => "instant",
        ItemKind::Throwable(ThrowableKind::OilFlask) => "instant",
        ItemKind::Throwable(ThrowableKind::SmokeBomb) => "5 turns",
        ItemKind::Food { .. } => "instant",
        ItemKind::Corpse => "instant",
    }
}

fn turns_to_label(turns: i32) -> &'static str {
    match turns {
        50 => "50 turns",
        _ => "timed",
    }
}

fn describe_potion(effect: PotionEffect) -> String {
    match effect {
        PotionEffect::Heal(amount) => format!("Drink to heal {amount} hp."),
        PotionEffect::GreaterHeal(amount) => format!("Drink to heal {amount} hp."),
        PotionEffect::FullHeal => "Drink to fully restore hp.".to_string(),
        PotionEffect::MaxHpUp(amount) => {
            format!("Drink to gain +{amount} max hp and heal by the same amount.")
        }
        PotionEffect::BuffAttack { amount, turns } => {
            format!("Drink to gain +{amount} attack for {turns} turns.")
        }
        PotionEffect::BuffVision { amount, turns } => {
            format!("Drink to gain +{amount} sight for {turns} turns.")
        }
        PotionEffect::CurePoison => "Drink to cure poison.".to_string(),
    }
}

fn describe_scroll(scroll: ScrollKind) -> &'static str {
    match scroll {
        ScrollKind::Mapping => "Read to reveal the whole level.",
        ScrollKind::Teleport => "Read to teleport to a random tile.",
        ScrollKind::Identify => "Read for no effect right now.",
        ScrollKind::MagicMissile => "Read to zap the nearest mob for 6-10 damage.",
        ScrollKind::EnchantWeapon => "Read to enchant your equipped weapon by +1 attack.",
        ScrollKind::EnchantArmor => "Read to enchant your equipped armor by +1 defense.",
        ScrollKind::Fear => "Read to frighten nearby mobs for 10 turns.",
        ScrollKind::Summon => "Read to summon nearby allied creatures.",
        ScrollKind::Light => "Read to extend sight for 100 turns.",
        ScrollKind::Recall => "Read to teleport back to the up-stair.",
    }
}

fn describe_ring(effect: RingEffect) -> &'static str {
    match effect {
        RingEffect::Regen => "Equip to regenerate 1 hp per turn.",
        RingEffect::Protection => "Equip to gain +1 defense.",
        RingEffect::Vision => "Equip to gain +2 sight.",
    }
}

fn describe_amulet(effect: AmuletEffect) -> &'static str {
    match effect {
        AmuletEffect::TeleportControl => "Equip for no effect right now.",
    }
}

fn describe_wand(kind: WandKind, charges: i32) -> String {
    match kind {
        WandKind::Fire => format!("Zap the nearest mob for 6-10 damage ({charges} charges)."),
        WandKind::Cold => format!("Zap the nearest mob for 5-8 damage ({charges} charges)."),
        WandKind::Lightning => {
            format!("Zap the nearest mob for 8-12 damage ({charges} charges).")
        }
    }
}

fn describe_throwable(kind: ThrowableKind) -> &'static str {
    match kind {
        ThrowableKind::OilFlask => "Smash to burn adjacent mobs for 6 damage.",
        ThrowableKind::SmokeBomb => "Smash to choke nearby mobs for 5 turns.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color;
    use crate::data::{items, mobs};

    #[test]
    fn mob_ability_descriptions_cover_plain_and_special_mobs() {
        let plain = mobs::by_name("rat").expect("rat template");
        let special = mobs::by_name("wyvern").expect("wyvern template");

        assert_eq!(describe_mob_abilities(plain), "none");
        assert!(describe_mob_abilities(special).contains("poisons on hit"));
    }

    #[test]
    fn item_function_descriptions_cover_multiple_item_types() {
        let wand = items::TEMPLATES
            .iter()
            .find(|template| matches!(template.kind, ItemKind::Wand { kind: WandKind::Fire, .. }))
            .expect("wand template");
        let mapping = items::TEMPLATES
            .iter()
            .find(|template| matches!(template.kind, ItemKind::Scroll(ScrollKind::Mapping)))
            .expect("mapping template");

        assert!(describe_item_function(wand).contains("6-10 damage"));
        assert!(describe_item_function(mapping).contains("reveal the whole level"));
        assert_eq!(item_range(wand), "any visible mob");
        assert_eq!(item_duration(wand), "instant");
    }

    #[test]
    fn ranged_mobs_use_configured_range() {
        let ranged = mobs::by_name("kobold archer").expect("ranged mob");
        assert_eq!(mob_attack_range(ranged), "2 tiles");
    }

    #[test]
    fn identify_scroll_stub_is_explicit() {
        let identify = ItemTemplate {
            name: "scroll of identify",
            glyph: '?',
            fg: Color::White,
            kind: ItemKind::Scroll(ScrollKind::Identify),
            min_depth: 1,
        };

        assert_eq!(
            describe_item_function(&identify),
            "Read for no effect right now."
        );
    }
}
