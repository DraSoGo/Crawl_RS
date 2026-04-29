use std::collections::BTreeSet;

use hecs::World;
use serde::{Deserialize, Serialize};

use crate::data::{
    items::{self, ItemTemplate},
    mobs::{self, MobTemplate},
};
use crate::ecs::components::{FieldOfView, Item, Mob, Name, Player, Position};

pub use crate::codex_text::{
    describe_item_function, describe_mob_abilities, item_duration, item_range, mob_ai_label,
    mob_attack_range,
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

fn player_visibility(world: &World) -> Option<crate::map::fov::Visibility> {
    world
        .query::<(&Player, &FieldOfView)>()
        .iter()
        .map(|(_, (_, fov))| fov.view.clone())
        .next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color;

    fn visible_world() -> World {
        let mut world = World::new();
        let mut fov = FieldOfView::new(8, 10, 10);
        fov.view.force_visible(2, 2);
        world.spawn((Player, fov));
        world
    }

    #[test]
    fn visible_rat_is_discovered() {
        let mut world = visible_world();
        world.spawn((Mob, Position::new(2, 2), Name("rat".to_string())));

        let discoveries = discover_visible_entries(&world);

        assert!(discoveries.mobs.contains("rat"));
    }

    #[test]
    fn summoned_names_unlock_base_entries() {
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
        world.spawn((Mob, Position::new(2, 2), Name("definitely fake".to_string())));
        world.spawn((
            Position::new(2, 2),
            Item {
                kind: items::TEMPLATES[0].kind,
            },
            Name("mystery loot".to_string()),
        ));

        let discoveries = discover_visible_entries(&world);

        assert!(discoveries.mobs.is_empty());
        assert!(discoveries.items.is_empty());
    }

    #[test]
    fn apply_discoveries_only_reports_new_entries() {
        let mut profile = CodexProfile::default();
        let mut discoveries = VisibleDiscoveries::default();
        discoveries.mobs.insert("rat".to_string());

        assert!(apply_discoveries(&mut profile, discoveries));

        let mut repeated = VisibleDiscoveries::default();
        repeated.mobs.insert("rat".to_string());
        assert!(!apply_discoveries(&mut profile, repeated));
    }

    #[test]
    fn page_lengths_match_templates() {
        assert_eq!(page_len(BookPage::Mob), mobs::TEMPLATES.len());
        assert_eq!(page_len(BookPage::Item), items::TEMPLATES.len());
    }

    #[test]
    fn canonical_item_names_match_templates() {
        let sample = ItemTemplate {
            name: "debug ration",
            glyph: '%',
            fg: Color::White,
            kind: items::TEMPLATES[0].kind,
            min_depth: 1,
        };
        assert_eq!(canonical_item_name(sample.name), None);
        assert_eq!(
            canonical_item_name(items::TEMPLATES[0].name),
            Some(items::TEMPLATES[0].name.to_string())
        );
    }
}
