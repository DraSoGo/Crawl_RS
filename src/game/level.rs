//! Level lifecycle: descending to a new floor while preserving the player.
//!
//! On descent we despawn every non-player entity, regenerate a fresh BSP
//! dungeon with a depth-mixed seed, place the player on the new up-stair,
//! and repopulate mobs/items scaled by the new depth.

use hecs::{Entity, World};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

use crate::config;
use crate::data::items::{self, ItemTemplate};
use crate::data::mobs::{self, MobTemplate};
use crate::ecs::components::{
    Ai, Amulet, BlocksTile, CasterHeal, Faction, FieldOfView, Flying, Item, Mob, Name,
    Player, Position, Regen, Renderable, Stats, StatusEffects, Summoner,
};
use crate::map::gen::{bsp_generate, BspConfig, Dungeon};
use crate::map::Map;

pub const FINAL_DEPTH: u32 = config::WORLD.final_depth;
const MOB_LAYER: u8 = 100;
const ITEM_LAYER: u8 = 50;
const AMULET_LAYER: u8 = 60;

/// Stir the run seed and depth together so each floor is deterministic but
/// distinct. SplitMix-style mix keeps adjacent depths visually different.
pub fn level_seed(run_seed: u64, depth: u32) -> u64 {
    let mut s = run_seed.wrapping_add((depth as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    s ^= s >> 30;
    s = s.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    s ^= s >> 27;
    s = s.wrapping_mul(0x94d0_49bb_1331_11eb);
    s ^= s >> 31;
    s
}

/// Generate a fresh dungeon and (re)populate mobs/items for the given depth.
/// Returns the new map. The caller is expected to have already despawned any
/// previous level's entities and moved the player to `dungeon.start`.
pub fn build_level(
    world: &mut World,
    run_seed: u64,
    depth: u32,
    width: i32,
    height: i32,
) -> Map {
    let cfg = BspConfig::default();
    let mut rng = Pcg64Mcg::seed_from_u64(level_seed(run_seed, depth));
    let dungeon: Dungeon = bsp_generate(width, height, &cfg, &mut rng);

    place_player(world, &dungeon);
    spawn_mobs(world, &dungeon, depth, &mut rng);
    spawn_items(world, &dungeon, depth, &mut rng);
    if depth >= FINAL_DEPTH {
        spawn_amulet(world, &dungeon, &mut rng);
    }
    dungeon.map
}

/// Despawn every on-map non-player entity (mobs, ground items, the amulet).
///
/// Items carried in the player's `Inventory` lack a `Position` after pickup,
/// so filtering on `Position` keeps them alive across level transitions —
/// otherwise the inventory would hold dangling `Entity` references that
/// render as "?" in the inventory menu.
pub fn purge_non_player(world: &mut World) {
    let to_remove: Vec<Entity> = world
        .query::<&Position>()
        .iter()
        .map(|(entity, _)| entity)
        .filter(|entity| world.get::<&Player>(*entity).is_err())
        .collect();
    for entity in to_remove {
        let _ = world.despawn(entity);
    }
}

fn place_player(world: &mut World, dungeon: &Dungeon) {
    let mut player_entity = None;
    for (e, _) in world.query::<&Player>().iter() {
        player_entity = Some(e);
        break;
    }
    let entity = match player_entity {
        Some(e) => e,
        None => return,
    };
    if let Ok(mut pos) = world.get::<&mut Position>(entity) {
        pos.x = dungeon.start.0;
        pos.y = dungeon.start.1;
    }
    // Resize FOV to match new map dims.
    let map_w = dungeon.map.width();
    let map_h = dungeon.map.height();
    let radius = world
        .get::<&FieldOfView>(entity)
        .map(|f| f.radius)
        .unwrap_or(8);
    let _ = world.insert_one(entity, FieldOfView::new(radius, map_w, map_h));
}

fn spawn_mobs(world: &mut World, dungeon: &Dungeon, depth: u32, rng: &mut Pcg64Mcg) {
    let rooms: Vec<_> = dungeon.rooms.iter().skip(1).copied().collect();
    let rooms = if rooms.is_empty() {
        dungeon.rooms.clone()
    } else {
        rooms
    };
    let mut budget = floor_difficulty_budget(depth);
    let mut attempts = 0u32;
    while budget > 0 && attempts < 256 {
        attempts += 1;
        let template = match mobs::pick_for_budget(depth, budget, rng) {
            Some(template) => template,
            None => break,
        };
        let room = rooms[rng.gen_range(0..rooms.len())];
        let x = rng.gen_range(room.x..room.x + room.w);
        let y = rng.gen_range(room.y..room.y + room.h);
        if tile_has_blocker(world, x, y) {
            continue;
        }
        spawn_mob(world, template, x, y, depth);
        budget = budget.saturating_sub(template.difficulty);
    }
}

fn spawn_mob(world: &mut World, t: &MobTemplate, x: i32, y: i32, depth: u32) {
    // At +12% HP/atk per depth level, floor 20 mobs are ~3.28× their base
    // template stats. The content tables are tuned around that multiplier.
    let hp_scale = 1.0 + config::WORLD.depth_hp_scale * ((depth as f32) - 1.0).max(0.0);
    let attack_scale =
        1.0 + config::WORLD.depth_attack_scale * ((depth as f32) - 1.0).max(0.0);
    let max_hp = ((t.max_hp as f32) * hp_scale).ceil() as i32;
    let attack = ((t.attack as f32) * attack_scale).round() as i32;
    let glyph = match t.ai {
        // Mimics start hidden as their disguise glyph.
        crate::ecs::components::AiKind::Mimic { disguise, .. } => disguise,
        _ => t.glyph,
    };
    let entity = world.spawn((
        Position::new(x, y),
        Renderable::new(glyph, t.fg, crossterm::style::Color::Reset, MOB_LAYER),
        Mob,
        BlocksTile,
        Stats::new(max_hp, attack.max(1), t.defense, t.move_tiles),
        Ai { kind: t.ai, sight_radius: t.sight },
        Faction::Hostile,
        StatusEffects::default(),
        Name(t.name.to_string()),
    ));
    if let Some(on_hit) = t.on_hit {
        let _ = world.insert_one(entity, on_hit);
    }
    if t.regen_per_turn > 0 {
        let _ = world.insert_one(entity, Regen { per_turn: t.regen_per_turn });
    }
    if t.invisible {
        if let Ok(mut s) = world.get::<&mut StatusEffects>(entity) {
            s.invisible = true;
        }
    }
    if let Some((heal_amount, chance_pct)) = t.caster_heal {
        let _ = world.insert_one(entity, CasterHeal { heal_amount, chance_pct });
    }
    if let Some(chance_pct) = t.summoner_chance {
        let _ = world.insert_one(
            entity,
            Summoner { chance_pct, summon_template: 0 },
        );
    }
    if t.flying {
        let _ = world.insert_one(entity, Flying);
    }
}

fn spawn_items(world: &mut World, dungeon: &Dungeon, depth: u32, rng: &mut Pcg64Mcg) {
    for room in dungeon.rooms.iter() {
        let chance = 0.7 + 0.06 * (depth as f64);
        if !rng.gen_bool(chance.clamp(0.0, 0.98)) {
            continue;
        }
        let template = match items::pick_for_depth(depth, rng) {
            Some(t) => t,
            None => continue,
        };
        let x = rng.gen_range(room.x..room.x + room.w);
        let y = rng.gen_range(room.y..room.y + room.h);
        if tile_has_item(world, x, y) {
            continue;
        }
        spawn_item(world, template, x, y);
    }
}

fn spawn_item(world: &mut World, t: &ItemTemplate, x: i32, y: i32) {
    world.spawn((
        Position::new(x, y),
        Renderable::new(t.glyph, t.fg, crossterm::style::Color::Reset, ITEM_LAYER),
        Item { kind: t.kind },
        Name(t.name.to_string()),
    ));
}

fn spawn_amulet(world: &mut World, dungeon: &Dungeon, rng: &mut Pcg64Mcg) {
    // Drop the amulet in the same room as the down-stair so the descent
    // narrative is "you reached the bottom and found it".
    let room = dungeon
        .rooms
        .last()
        .copied()
        .unwrap_or_else(|| dungeon.rooms[0]);
    let x = rng.gen_range(room.x..room.x + room.w);
    let y = rng.gen_range(room.y..room.y + room.h);
    world.spawn((
        Position::new(x, y),
        Renderable::new(
            '*',
            crossterm::style::Color::Yellow,
            crossterm::style::Color::Reset,
            AMULET_LAYER,
        ),
        Amulet,
        Name("Amulet of Yendor".to_string()),
    ));
}

fn tile_has_blocker(world: &World, x: i32, y: i32) -> bool {
    world
        .query::<(&Position, &BlocksTile)>()
        .iter()
        .any(|(_, (pos, _))| pos.x == x && pos.y == y)
}

fn tile_has_item(world: &World, x: i32, y: i32) -> bool {
    world
        .query::<(&Position, &Item)>()
        .iter()
        .any(|(_, (pos, _))| pos.x == x && pos.y == y)
}

fn floor_difficulty_budget(depth: u32) -> u32 {
    config::WORLD.floor_difficulty_base
        + depth.saturating_mul(config::WORLD.floor_difficulty_per_depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_seed_is_deterministic_and_distinct() {
        let s1 = level_seed(42, 1);
        let s2 = level_seed(42, 2);
        assert_ne!(s1, s2);
        assert_eq!(s1, level_seed(42, 1));
    }
}
