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
    match template.ai {
        AiKind::Ranged { .. } => format!("{} tiles", config::WORLD.ranged_attack_range),
        _ => "adjacent".to_string(),
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

pub fn item_range(template: &ItemTemplate) -> &'static str {
    match template.kind {
        ItemKind::Potion(_) => "self",
        ItemKind::Scroll(ScrollKind::Mapping) => "whole floor",
        ItemKind::Scroll(ScrollKind::Teleport) => "self",
        ItemKind::Scroll(ScrollKind::Identify) => "none",
        ItemKind::Scroll(ScrollKind::MagicMissile) => "nearest mob",
        ItemKind::Scroll(ScrollKind::EnchantWeapon) => "equipped weapon",
        ItemKind::Scroll(ScrollKind::EnchantArmor) => "equipped armor",
        ItemKind::Scroll(ScrollKind::Fear) => "nearby mobs",
        ItemKind::Scroll(ScrollKind::Summon) => "nearby floor",
        ItemKind::Scroll(ScrollKind::Light) => "self",
        ItemKind::Scroll(ScrollKind::Recall) => "self",
        ItemKind::Weapon { .. } => "melee",
        ItemKind::Armor { .. } => "self",
        ItemKind::Ring(_) => "self",
        ItemKind::AmuletItem(_) => "self",
        ItemKind::Wand { .. } => "nearest mob",
        ItemKind::Throwable(ThrowableKind::OilFlask) => "adjacent burst",
        ItemKind::Throwable(ThrowableKind::SmokeBomb) => "2-tile burst",
        ItemKind::Food { .. } => "self",
        ItemKind::Corpse => "self",
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
        assert_eq!(item_range(wand), "nearest mob");
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
