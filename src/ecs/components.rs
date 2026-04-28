//! Component types stored in the `hecs::World`.
//!
//! Components are kept POD-ish so that future phases can serialise them via
//! bincode without surprises. Behaviour lives in systems, not on components.

use crossterm::style::Color;
use serde::{Deserialize, Serialize};

/// Tile coordinate. Origin is top-left; +x is right, +y is down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// What the entity looks like on screen.
#[derive(Clone, Copy, Debug)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
    pub bg: Color,
    /// Higher = drawn later (on top). Player typically has the highest layer.
    pub layer: u8,
}

impl Renderable {
    pub const fn new(glyph: char, fg: Color, bg: Color, layer: u8) -> Self {
        Self { glyph, fg, bg, layer }
    }
}

/// Marker on the player entity. Kept as a unit struct so queries can use it
/// as a tag without pulling extra data.
#[derive(Clone, Copy, Debug, Default)]
pub struct Player;

/// Field-of-view component. Held by entities that need to see (initially the
/// player; mobs in Phase 6). The visibility/revealed bitmaps live in
/// `map::fov::Visibility`.
#[derive(Clone, Debug)]
pub struct FieldOfView {
    pub radius: i32,
    pub view: crate::map::fov::Visibility,
    /// Set by movement / map regen so the FOV system knows it must recompute.
    pub dirty: bool,
}

impl FieldOfView {
    pub fn new(radius: i32, width: i32, height: i32) -> Self {
        Self {
            radius,
            view: crate::map::fov::Visibility::new(width, height),
            dirty: true,
        }
    }
}

/// Pending movement, queued by the input system and consumed by the movement
/// system within the same turn. Not meant to persist across frames.
#[derive(Clone, Copy, Debug)]
pub struct MoveIntent {
    pub dx: i32,
    pub dy: i32,
}

impl MoveIntent {
    pub const fn new(dx: i32, dy: i32) -> Self {
        Self { dx, dy }
    }
}

/// Combat / scheduler stats for any acting entity.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub max_hp: i32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    /// Energy per scheduler tick. 10 is "normal".
    pub speed: i32,
}

impl Stats {
    pub const fn new(max_hp: i32, attack: i32, defense: i32, speed: i32) -> Self {
        Self { max_hp, hp: max_hp, attack, defense, speed }
    }
}

/// Energy accumulator for the speed-based turn scheduler.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Energy {
    pub value: i32,
}

impl Energy {
    pub const fn new(value: i32) -> Self {
        Self { value }
    }
}

/// Marker on hostile NPC entities. Distinct from `Player`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mob;

/// Marker on entities that occupy a tile and prevent others from sharing it
/// (player, mobs). Items omit this so the player can step onto them.
#[derive(Clone, Copy, Debug, Default)]
pub struct BlocksTile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiKind {
    /// Charge the player when in line of sight, otherwise wander randomly.
    Hostile,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Ai {
    pub kind: AiKind,
    pub sight_radius: i32,
}

impl Ai {
    pub const fn hostile(sight_radius: i32) -> Self {
        Self { kind: AiKind::Hostile, sight_radius }
    }
}

/// Display name used by the message log and HUD.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Name(pub String);

/// Pending attack: attacker (the entity carrying this component) wants to
/// strike `target`. Resolved by the combat system, then removed.
#[derive(Clone, Copy, Debug)]
pub struct WantsToAttack {
    pub target: hecs::Entity,
}

/// Player progression. Phase 7 awards XP on kills; level-ups are deferred to
/// later balance passes.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Progression {
    pub xp: i32,
    pub kills: u32,
}

/// Marker placed on dead entities so they get cleaned up after the combat
/// system finishes. Avoids despawning while we still hold component borrows.
#[derive(Clone, Copy, Debug, Default)]
pub struct Dead;

/// What kind of item this is and how it behaves on use.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ItemKind {
    Potion { heal: i32 },
    Scroll(ScrollKind),
    Weapon { attack_bonus: i32 },
    Armor { defense_bonus: i32 },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ScrollKind {
    Mapping,
    Teleport,
}

/// Marker on item entities. The kind, name, and renderable carry the rest of
/// the data. When an item is on the ground it has a `Position`; when it is in
/// an inventory the position is removed.
#[derive(Clone, Copy, Debug, Default)]
pub struct Item {
    pub kind: ItemKind,
}

impl Default for ItemKind {
    fn default() -> Self {
        ItemKind::Potion { heal: 0 }
    }
}

/// Wants to pick up the item at the entity's current tile.
#[derive(Clone, Copy, Debug, Default)]
pub struct WantsToPickup;

/// Inventory: ordered list of held item entities (max 26 — keys a..z).
#[derive(Clone, Debug, Default)]
pub struct Inventory {
    pub items: Vec<hecs::Entity>,
}

impl Inventory {
    pub const MAX_SLOTS: usize = 26;

    pub fn is_full(&self) -> bool {
        self.items.len() >= Self::MAX_SLOTS
    }
}

/// What's currently equipped. The component values point to entities still
/// stored in `Inventory`; equipping does not remove them from the inventory.
#[derive(Clone, Copy, Debug, Default)]
pub struct Equipment {
    pub weapon: Option<hecs::Entity>,
    pub armor: Option<hecs::Entity>,
}

/// Marker on the win-condition artifact. Distinct from `Item` so the pickup
/// system can branch on it (the amulet doesn't go in the regular inventory —
/// picking it up triggers the victory screen).
#[derive(Clone, Copy, Debug, Default)]
pub struct Amulet;
