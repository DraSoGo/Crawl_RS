//! Component types stored in the `hecs::World`. POD-ish; behaviour in systems.

use crossterm::style::Color;
use serde::{Deserialize, Serialize};

use crate::config;

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

/// FOV component. Bitmaps live in `map::fov::Visibility`.
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

/// Pending movement; consumed by movement system within the same turn.
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
    /// Tiles this actor may move during one round.
    pub move_tiles: i32,
}

impl Stats {
    pub const fn new(max_hp: i32, attack: i32, defense: i32, move_tiles: i32) -> Self {
        Self { max_hp, hp: max_hp, attack, defense, move_tiles }
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
    /// Idle until the player gets within `wake_radius` tiles, then becomes
    /// Hostile permanently.
    Sleeper { wake_radius: i32 },
    /// Hostile while above `flee_below_pct` HP percent; flees the player when
    /// HP drops below.
    Fleeing { flee_below_pct: i32 },
    /// Stays at distance ≥ `prefer_range`; otherwise behaves like Hostile.
    /// Combat handles the actual ranged attack as a "bump-at-distance".
    Ranged { prefer_range: i32 },
    /// Disguised as `disguise` glyph; switches to Hostile when player is
    /// adjacent.
    Mimic { disguise: char, revealed: bool },
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

/// Faction tag distinguishes friendly from hostile actors when summoned
/// allies appear. Default is `Hostile` for any mob.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Faction {
    PlayerAlly,
    Hostile,
}

impl Default for Faction {
    fn default() -> Self {
        Faction::Hostile
    }
}

/// Status effects with timers. Zero-valued field = no effect.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct StatusEffects {
    pub poison_turns: i32,
    pub poison_dmg: i32,
    pub paralysis_turns: i32,
    pub fear_turns: i32,
    pub attack_buff: i32,
    pub attack_buff_turns: i32,
    pub vision_buff: i32,
    pub vision_buff_turns: i32,
    pub light_turns: i32,
    pub regen_per_turn: i32,
    pub invisible: bool,
}

impl StatusEffects {
    pub fn paralyzed(&self) -> bool {
        self.paralysis_turns > 0
    }
    pub fn afraid(&self) -> bool {
        self.fear_turns > 0
    }
}

/// Inflicted on hit by certain mobs (ghoul, wyvern). Combat reads this off
/// the *attacker* and applies the listed effects to the defender.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct OnHit {
    pub poison_turns: i32,
    pub poison_dmg: i32,
    pub paralysis_turns: i32,
}

/// Per-turn passive heal (troll, ring of regen). Applied by status-tick.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Regen {
    pub per_turn: i32,
}

/// Hunger clock. Player ticks down each turn; below threshold → HP drain.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HungerClock {
    pub satiation: i32,
    pub max_satiation: i32,
}

impl HungerClock {
    pub const STARVE_THRESHOLD: i32 = config::HUNGER.starve_threshold;
    pub const HUNGRY_THRESHOLD: i32 = config::HUNGER.hungry_threshold;
    pub fn new(max: i32) -> Self {
        Self { satiation: max, max_satiation: max }
    }
    pub fn state(&self) -> HungerState {
        if self.satiation <= Self::STARVE_THRESHOLD {
            HungerState::Starving
        } else if self.satiation <= Self::HUNGRY_THRESHOLD {
            HungerState::Hungry
        } else {
            HungerState::Sated
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HungerState {
    Sated,
    Hungry,
    Starving,
}

/// Casts a self-heal occasionally (gnoll shaman). Probability is rolled per
/// turn during the AI step.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CasterHeal {
    pub heal_amount: i32,
    pub chance_pct: i32,
}

/// Each turn, has `chance_pct` chance to spawn a `summon_glyph` mob nearby
/// (lich → skeleton). Summoned mobs use the template name.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Summoner {
    pub chance_pct: i32,
    pub summon_template: u32,
}

/// Marker that this mob is "flying" — informational; current movement code
/// treats it as hostile but with full pathing freedom rules unchanged.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Flying;

/// Display name used by the message log and HUD.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Name(pub String);

/// Pending attack: attacker (the entity carrying this component) wants to
/// strike `target`. Resolved by the combat system, then removed.
#[derive(Clone, Copy, Debug)]
pub struct WantsToAttack {
    pub target: hecs::Entity,
}

/// Player progression. XP accumulates from kills and item sales; level-ups
/// are checked in `award_xp` (see `run_state`). Level starts at 1.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Progression {
    pub xp: i32,
    pub level: u32,
    pub kills: u32,
}

impl Default for Progression {
    fn default() -> Self {
        Self { xp: 0, level: 1, kills: 0 }
    }
}

impl Progression {
    /// XP required to advance from `level` to `level + 1`.
    pub fn xp_for_next(level: u32) -> i32 {
        config::PROGRESSION.xp_per_level * (level as i32).max(1)
    }
}

/// Marker placed on dead entities so they get cleaned up after the combat
/// system finishes. Avoids despawning while we still hold component borrows.
#[derive(Clone, Copy, Debug, Default)]
pub struct Dead;

/// What kind of item this is and how it behaves on use.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ItemKind {
    Potion(PotionEffect),
    Scroll(ScrollKind),
    Weapon { attack_bonus: i32 },
    Armor { defense_bonus: i32 },
    Ring(RingEffect),
    AmuletItem(AmuletEffect),
    Wand { kind: WandKind, charges: i32 },
    Throwable(ThrowableKind),
    Food { nutrition: i32, poisonous: bool },
    Corpse,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PotionEffect {
    /// Direct heal. Old `Potion { heal }` maps to this.
    Heal(i32),
    GreaterHeal(i32),
    FullHeal,
    MaxHpUp(i32),
    BuffAttack { amount: i32, turns: i32 },
    BuffVision { amount: i32, turns: i32 },
    CurePoison,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ScrollKind {
    Mapping,
    Teleport,
    Identify,
    MagicMissile,
    ChainLightning,
    EnchantWeapon,
    EnchantArmor,
    Fear,
    GreaterFear,
    Summon,
    Legion,
    Light,
    Recall,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WandKind {
    Fire,
    Cold,
    Lightning,
    Storms,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RingEffect {
    Regen,
    Protection,
    Vision,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AmuletEffect {
    TeleportControl,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ThrowableKind {
    OilFlask,
    SmokeBomb,
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
        ItemKind::Potion(PotionEffect::Heal(0))
    }
}

/// Wants to pick up the item at the entity's current tile.
#[derive(Clone, Copy, Debug, Default)]
pub struct WantsToPickup;

/// Inventory: ordered list of held item entities. Capacity is derived from the
/// player's current level.
#[derive(Clone, Debug, Default)]
pub struct Inventory {
    pub items: Vec<hecs::Entity>,
}

/// What's currently equipped. Slot values point to entities held in
/// `Inventory`; equipping does not remove them from the inventory.
#[derive(Clone, Copy, Debug, Default)]
pub struct Equipment {
    pub weapon: Option<hecs::Entity>,
    pub armor: Option<hecs::Entity>,
    pub ring: Option<hecs::Entity>,
    pub amulet: Option<hecs::Entity>,
}

/// Marker on the win-condition artifact. Distinct from `Item` so the pickup
/// system can branch on it (the amulet doesn't go in the regular inventory —
/// picking it up triggers the victory screen).
#[derive(Clone, Copy, Debug, Default)]
pub struct Amulet;
