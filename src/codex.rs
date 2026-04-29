use std::collections::BTreeSet;

use hecs::World;
use serde::{Deserialize, Serialize};

use crate::data::{
    items::{self, ItemTemplate},
    mobs::{self, MobTemplate},
};
use crate::ecs::components::{
    AiKind, AmuletEffect, FieldOfView, Item, ItemKind, Mob, Name, Player, Position,
    PotionEffect, RingEffect, ScrollKind, ThrowableKind, WandKind,
};

const SUMMONED_PREFIX: &str = "summoned ";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookPage {
    Mob,
    Item,
}

impl BookPage {
    pub const fn label(self) -> &'static str {
        match self {
            BookPage::Mob => "Mob",
            BookPage::Item => "Item",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            BookPage::Mob => BookPage::Item,
            BookPage::Item => BookPage::Mob,
        }
    }

    pub const fn previous(self) -> Self {
        self.next()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CodexProfile {
    pub discovered_mobs: BTreeSet<String>,
    pub discovered_items: BTreeSet<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct VisibleDiscoveries {
    pub mobs: BTreeSet<String>,
    pub items: BTreeSet<String>,
}

pub fn mob_templates() -> &'static [MobTemplate] {
    mobs::TEMPLATES
}

pub fn item_templates() -> &'static [ItemTemplate] {
    items::TEMPLATES
}

pub fn page_len(page: BookPage) -> usize {
    match page {
        BookPage::Mob => mob_templates().len(),
        BookPage::Item => item_templates().len(),
    }
}

pub fn canonical_mob_name(name: &str) -> Option<String> {
    let base_name = name.strip_prefix(SUMMONED_PREFIX).unwrap_or(name);
    mobs::by_name(base_name).map(|template| template.name.to_string())
}

pub fn canonical_item_name(name: &str) -> Option<String> {
    items::TEMPLATES
        .iter()
        .find(|template| template.name == name)
        .map(|template| template.name.to_string())
}

pub fn discover_visible_entries(world: &World) -> VisibleDiscoveries {
    let mut discoveries = VisibleDiscoveries::default();
    let visibility = match player_visibility(world) {
        Some(visibility) => visibility,
        None => return discoveries,
    };

    for (_, (_, pos, name)) in world.query::<(&Mob, &Position, &Name)>().iter() {
        if visibility.is_visible(pos.x, pos.y) {
            if let Some(canonical_name) = canonical_mob_name(&name.0) {
                discoveries.mobs.insert(canonical_name);
            }
        }
    }

    for (_, (_, pos, name)) in world.query::<(&Item, &Position, &Name)>().iter() {
        if visibility.is_visible(pos.x, pos.y) {
            if let Some(canonical_name) = canonical_item_name(&name.0) {
                discoveries.items.insert(canonical_name);
            }
        }
    }

    discoveries
}

pub fn apply_discoveries(profile: &mut CodexProfile, discoveries: VisibleDiscoveries) -> bool {
    let mut changed = false;

    for name in discoveries.mobs {
        changed |= profile.discovered_mobs.insert(name);
    }
    for name in discoveries.items {
        changed |= profile.discovered_items.insert(name);
    }

    changed
}

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

fn player_visibility(world: &World) -> Option<crate::map::fov::Visibility> {
    world
        .query::<(&Player, &FieldOfView)>()
        .iter()
        .next()
        .map(|(_, (_, fov))| fov.view.clone())
}

fn describe_potion(effect: PotionEffect) -> String {
    match effect {
        PotionEffect::Heal(amount) => format!("Drink to restore up to {amount} HP."),
        PotionEffect::GreaterHeal(amount) => {
            format!("Drink to restore up to {amount} HP.")
        }
        PotionEffect::FullHeal => "Drink to fully restore HP.".to_string(),
        PotionEffect::MaxHpUp(amount) => {
            format!("Drink to raise max HP by {amount} and heal the same amount.")
        }
        PotionEffect::BuffAttack { amount, turns } => {
            format!("Drink to gain +{amount} attack for {turns} turns.")
        }
        PotionEffect::BuffVision { amount, turns } => {
            format!("Drink to gain +{amount} sight for {turns} turns.")
        }
        PotionEffect::CurePoison => "Drink to clear poison.".to_string(),
    }
}

fn describe_scroll(scroll: ScrollKind) -> &'static str {
    match scroll {
        ScrollKind::Mapping => "Read to reveal the whole level map.",
        ScrollKind::Teleport => "Read to teleport to a random floor tile.",
        ScrollKind::Identify => "Read for no effect right now.",
        ScrollKind::MagicMissile => "Read to hit the nearest mob for 6 damage.",
        ScrollKind::EnchantWeapon => {
            "Read to enchant your equipped weapon by +1 attack."
        }
        ScrollKind::EnchantArmor => {
            "Read to enchant your equipped armor by +1 defense."
        }
        ScrollKind::Fear => "Read to frighten nearby mobs for 10 turns.",
        ScrollKind::Summon => "Read to summon allied rats nearby.",
        ScrollKind::Light => "Read to extend your sight radius by 4 for 50 turns.",
        ScrollKind::Recall => "Read to return to the up-stair tile.",
    }
}

fn describe_ring(effect: RingEffect) -> &'static str {
    match effect {
        RingEffect::Regen => "Equip to regenerate 1 HP each turn.",
        RingEffect::Protection => "Equip to gain +1 defense.",
        RingEffect::Vision => "Equip to gain +2 sight radius.",
    }
}

fn describe_amulet(effect: AmuletEffect) -> &'static str {
    match effect {
        AmuletEffect::TeleportControl => "Equip as an amulet. Currently no effect.",
    }
}

fn describe_wand(kind: WandKind, charges: i32) -> String {
    let effect = match kind {
        WandKind::Fire => "hit the nearest mob for 6-10 damage",
        WandKind::Cold => "hit the nearest mob for 5-8 damage",
        WandKind::Lightning => "hit the nearest mob for 8-12 damage",
    };
    format!("Zap to {effect}. Starts with {charges} charges.")
}

fn describe_throwable(kind: ThrowableKind) -> &'static str {
    match kind {
        ThrowableKind::OilFlask => {
            "Throw to deal 6 damage to adjacent mobs."
        }
        ThrowableKind::SmokeBomb => {
            "Throw to paralyze nearby mobs for 5 turns."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color;
    use hecs::World;

    use crate::data::{items, mobs};
    use crate::ecs::components::{Item, Player, PotionEffect, Renderable};

    fn visible_world() -> World {
        let mut world = World::new();
        let mut fov = FieldOfView::new(8, 10, 10);
        fov.view.force_visible(2, 2);
        fov.view.force_visible(4, 4);
        world.spawn((Player, fov));
        world
    }

    #[test]
    fn visible_rat_unlocks_rat() {
        let mut world = visible_world();
        world.spawn((Mob, Position::new(2, 2), Name("rat".to_string())));

        let discoveries = discover_visible_entries(&world);

        assert!(discoveries.mobs.contains("rat"));
    }

    #[test]
    fn visible_summoned_mob_unlocks_base_entry() {
        let mut world = visible_world();
        world.spawn((
            Mob,
            Position::new(2, 2),
            Name("summoned skeleton archer".to_string()),
        ));

        let discoveries = discover_visible_entries(&world);

        assert!(discoveries.mobs.contains("skeleton archer"));
    }

    #[test]
    fn unknown_names_are_ignored() {
        let mut world = visible_world();
        world.spawn((Mob, Position::new(2, 2), Name("weird blob".to_string())));
        world.spawn((
            Position::new(4, 4),
            Renderable::new('!', Color::White, Color::Reset, 50),
            Item { kind: ItemKind::Potion(PotionEffect::Heal(8)) },
            Name("mystery tonic".to_string()),
        ));

        let discoveries = discover_visible_entries(&world);

        assert!(discoveries.mobs.is_empty());
        assert!(discoveries.items.is_empty());
    }

    #[test]
    fn mob_ability_descriptions_cover_plain_and_special_mobs() {
        let rat = mobs::by_name("rat").expect("rat template");
        let shaman = mobs::by_name("gnoll shaman").expect("shaman template");

        assert_eq!(describe_mob_abilities(rat), "none");
        assert!(describe_mob_abilities(shaman).contains("self-heals 6 hp"));
    }

    #[test]
    fn item_function_descriptions_cover_multiple_item_types() {
        let potion = items::TEMPLATES
            .iter()
            .find(|template| template.name == "potion of healing")
            .expect("potion template");
        let weapon = items::TEMPLATES
            .iter()
            .find(|template| template.name == "short sword")
            .expect("weapon template");
        let wand = items::TEMPLATES
            .iter()
            .find(|template| template.name == "wand of fire")
            .expect("wand template");
        let identify = items::TEMPLATES
            .iter()
            .find(|template| template.name == "scroll of mapping")
            .expect("mapping template");

        assert!(describe_item_function(potion).contains("restore up to 8 HP"));
        assert!(describe_item_function(weapon).contains("+2 attack"));
        assert!(describe_item_function(wand).contains("6-10 damage"));
        assert!(describe_item_function(identify).contains("reveal the whole level"));
        assert_eq!(
            describe_item_function(&ItemTemplate {
                name: "scroll of identify",
                glyph: '?',
                fg: Color::White,
                kind: ItemKind::Scroll(ScrollKind::Identify),
                min_depth: 1,
            }),
            "Read for no effect right now."
        );
    }
}
