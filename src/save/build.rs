//! Project a live `World` + `Map` + `MessageLog` into a `SaveSnapshot`.

use anyhow::{anyhow, Result};
use hecs::{Entity, World};

use crate::ecs::components::{
    Ai, Amulet, Equipment, FieldOfView, HungerClock, Inventory, Item, ItemKind, Mob, Name,
    Player, Position, Progression, PotionEffect, Renderable, Stats, StatusEffects,
};
use crate::map::Map;
use crate::save::types::{
    encode_color, ItemSnapshot, LogEntry, MobSnapshot, PlayerSnapshot, SaveSnapshot,
    SAVE_VERSION,
};
use crate::ui::messages::{Message, MessageLog};

pub fn build_snapshot(
    seed: u64,
    depth: u32,
    map: &Map,
    world: &World,
    log: &MessageLog,
) -> Result<SaveSnapshot> {
    let player = build_player_snapshot(world)?;
    let mobs = build_mob_snapshots(world);
    let ground_items = build_ground_item_snapshots(world);
    let amulet = build_amulet_snapshot(world);
    let log = log
        .tail(usize::MAX)
        .into_iter()
        .cloned()
        .map(message_to_entry)
        .collect();
    Ok(SaveSnapshot {
        version: SAVE_VERSION,
        seed,
        depth,
        map: map.clone(),
        player,
        mobs,
        ground_items,
        amulet,
        log,
    })
}

fn message_to_entry(msg: Message) -> LogEntry {
    LogEntry {
        text: msg.text,
        severity: msg.severity,
    }
}

fn build_player_snapshot(world: &World) -> Result<PlayerSnapshot> {
    let entity = world
        .query::<&Player>()
        .iter()
        .next()
        .map(|(e, _)| e)
        .ok_or_else(|| anyhow!("no player entity to save"))?;
    let pos = *world.get::<&Position>(entity)?;
    let stats = *world.get::<&Stats>(entity)?;
    let progression = *world.get::<&Progression>(entity)?;
    let fov = world.get::<&FieldOfView>(entity)?;
    let inventory = world.get::<&Inventory>(entity).map(|i| i.clone());
    let equipment = world
        .get::<&Equipment>(entity)
        .ok()
        .map(|e| *e)
        .unwrap_or_default();
    let renderable = world.get::<&Renderable>(entity)?;

    let mut inv_snapshots = Vec::new();
    let mut weapon_idx = None;
    let mut armor_idx = None;
    let mut ring_idx = None;
    let mut amulet_idx = None;
    if let Ok(inv) = inventory {
        for (i, item_entity) in inv.items.iter().enumerate() {
            inv_snapshots.push(item_to_snapshot(world, *item_entity));
            if equipment.weapon == Some(*item_entity) {
                weapon_idx = Some(i);
            }
            if equipment.armor == Some(*item_entity) {
                armor_idx = Some(i);
            }
            if equipment.ring == Some(*item_entity) {
                ring_idx = Some(i);
            }
            if equipment.amulet == Some(*item_entity) {
                amulet_idx = Some(i);
            }
        }
    }
    let status = world
        .get::<&StatusEffects>(entity)
        .map(|s| *s)
        .unwrap_or_default();
    let hunger = world
        .get::<&HungerClock>(entity)
        .map(|h| *h)
        .unwrap_or_else(|_| HungerClock::new(800));

    let (fov_w, fov_h) = (fov.view.width(), fov.view.height());
    let mut visible = Vec::with_capacity((fov_w * fov_h) as usize);
    let mut revealed = Vec::with_capacity((fov_w * fov_h) as usize);
    for y in 0..fov_h {
        for x in 0..fov_w {
            visible.push(fov.view.is_visible(x, y));
            revealed.push(fov.view.is_revealed(x, y));
        }
    }

    Ok(PlayerSnapshot {
        pos,
        stats,
        progression,
        fov_radius: fov.radius,
        fov_revealed: revealed,
        fov_visible: visible,
        fov_w,
        fov_h,
        inventory: inv_snapshots,
        equipped_weapon_idx: weapon_idx,
        equipped_armor_idx: armor_idx,
        equipped_ring_idx: ring_idx,
        equipped_amulet_idx: amulet_idx,
        renderable_glyph: renderable.glyph,
        status,
        hunger,
    })
}

fn build_mob_snapshots(world: &World) -> Vec<MobSnapshot> {
    world
        .query::<(&Mob, &Position, &Stats, &Ai, &Name, &Renderable)>()
        .iter()
        .map(|(_, (_, pos, stats, ai, name, render))| MobSnapshot {
            pos: *pos,
            stats: *stats,
            ai: *ai,
            name: name.0.clone(),
            glyph: render.glyph,
            fg_index: encode_color(render.fg),
        })
        .collect()
}

fn build_ground_item_snapshots(world: &World) -> Vec<ItemSnapshot> {
    world
        .query::<(&Item, &Position, &Name, &Renderable)>()
        .iter()
        .map(|(_, (item, pos, name, render))| ItemSnapshot {
            pos: Some(*pos),
            kind: item.kind,
            name: name.0.clone(),
            glyph: render.glyph,
            fg_index: encode_color(render.fg),
        })
        .collect()
}

fn item_to_snapshot(world: &World, item: Entity) -> ItemSnapshot {
    let kind = world
        .get::<&Item>(item)
        .map(|i| i.kind)
        .unwrap_or(ItemKind::Potion(PotionEffect::Heal(0)));
    let name = world
        .get::<&Name>(item)
        .map(|n| n.0.clone())
        .unwrap_or_else(|_| "?".to_string());
    let (glyph, fg) = world
        .get::<&Renderable>(item)
        .map(|r| (r.glyph, r.fg))
        .unwrap_or(('?', crossterm::style::Color::White));
    ItemSnapshot {
        pos: None,
        kind,
        name,
        glyph,
        fg_index: encode_color(fg),
    }
}

fn build_amulet_snapshot(world: &World) -> Option<Position> {
    world
        .query::<(&Amulet, &Position)>()
        .iter()
        .map(|(_, (_, pos))| *pos)
        .next()
}
