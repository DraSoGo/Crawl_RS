//! Inventory actions: use a held item by index. Returns whether a turn was
//! consumed (so the scheduler advances) along with whether the player is
//! still inside the inventory screen.

use hecs::{Entity, World};
use rand::Rng;

use crate::ecs::components::{
    Equipment, FieldOfView, Inventory, Item, ItemKind, Name, Player, Position, ScrollKind,
    Stats,
};
use crate::map::{Map, Tile};
use crate::ui::MessageLog;

pub fn use_index<R: Rng>(
    world: &mut World,
    map: &mut Map,
    log: &mut MessageLog,
    rng: &mut R,
    index: usize,
) -> bool {
    let player = match find_player(world) {
        Some(p) => p,
        None => return false,
    };
    let item = {
        let inv = match world.get::<&Inventory>(player) {
            Ok(i) => i,
            Err(_) => return false,
        };
        match inv.items.get(index) {
            Some(e) => *e,
            None => {
                log.info("no such item.");
                return false;
            }
        }
    };
    let kind = match world.get::<&Item>(item) {
        Ok(i) => i.kind,
        Err(_) => return false,
    };
    let item_name = world
        .get::<&Name>(item)
        .map(|n| n.0.clone())
        .unwrap_or_else(|_| "item".into());

    match kind {
        ItemKind::Potion { heal } => {
            apply_heal(world, log, player, &item_name, heal);
            consume_item(world, player, item);
            true
        }
        ItemKind::Scroll(ScrollKind::Mapping) => {
            apply_mapping(world, log, player);
            log.status(format!("the {item_name} reveals the level."));
            consume_item(world, player, item);
            true
        }
        ItemKind::Scroll(ScrollKind::Teleport) => {
            apply_teleport(world, log, map, rng, player);
            log.status(format!("the {item_name} flings you across the level."));
            consume_item(world, player, item);
            true
        }
        ItemKind::Weapon { attack_bonus } => {
            equip_weapon(world, log, player, item, &item_name, attack_bonus);
            true
        }
        ItemKind::Armor { defense_bonus } => {
            equip_armor(world, log, player, item, &item_name, defense_bonus);
            true
        }
    }
}

fn find_player(world: &World) -> Option<Entity> {
    world.query::<&Player>().iter().next().map(|(e, _)| e)
}

fn apply_heal(
    world: &mut World,
    log: &mut MessageLog,
    target: Entity,
    item_name: &str,
    heal: i32,
) {
    if let Ok(mut stats) = world.get::<&mut Stats>(target) {
        let before = stats.hp;
        stats.hp = (stats.hp + heal).min(stats.max_hp);
        let actual = stats.hp - before;
        log.status(format!(
            "you quaff the {item_name} (+{actual} hp)."
        ));
    }
}

fn apply_mapping(world: &mut World, log: &mut MessageLog, target: Entity) {
    if let Ok(mut fov) = world.get::<&mut FieldOfView>(target) {
        fov.view.reveal_all();
        fov.dirty = true;
    }
    let _ = log;
}

fn apply_teleport<R: Rng>(
    world: &mut World,
    log: &mut MessageLog,
    map: &Map,
    rng: &mut R,
    target: Entity,
) {
    let mut floors: Vec<(i32, i32)> = Vec::new();
    for (x, y, tile) in map.iter() {
        if matches!(tile, Tile::Floor | Tile::DownStairs | Tile::UpStairs) {
            floors.push((x, y));
        }
    }
    if floors.is_empty() {
        return;
    }
    let (x, y) = floors[rng.gen_range(0..floors.len())];
    if let Ok(mut pos) = world.get::<&mut Position>(target) {
        pos.x = x;
        pos.y = y;
    }
    if let Ok(mut fov) = world.get::<&mut FieldOfView>(target) {
        fov.dirty = true;
    }
    let _ = log;
}

fn equip_weapon(
    world: &mut World,
    log: &mut MessageLog,
    player: Entity,
    item: Entity,
    item_name: &str,
    bonus: i32,
) {
    let prev = swap_equipment_weapon(world, player, Some(item));
    if let Some(prev_entity) = prev {
        if let Some(prev_bonus) = weapon_bonus(world, prev_entity) {
            adjust_attack(world, player, -prev_bonus);
        }
    }
    adjust_attack(world, player, bonus);
    log.status(format!("you wield the {item_name}."));
}

fn equip_armor(
    world: &mut World,
    log: &mut MessageLog,
    player: Entity,
    item: Entity,
    item_name: &str,
    bonus: i32,
) {
    let prev = swap_equipment_armor(world, player, Some(item));
    if let Some(prev_entity) = prev {
        if let Some(prev_bonus) = armor_bonus(world, prev_entity) {
            adjust_defense(world, player, -prev_bonus);
        }
    }
    adjust_defense(world, player, bonus);
    log.status(format!("you don the {item_name}."));
}

fn swap_equipment_weapon(world: &mut World, player: Entity, new: Option<Entity>) -> Option<Entity> {
    let prev = world
        .get::<&Equipment>(player)
        .ok()
        .and_then(|eq| eq.weapon);
    if let Ok(mut eq) = world.get::<&mut Equipment>(player) {
        eq.weapon = new;
    }
    prev
}

fn swap_equipment_armor(world: &mut World, player: Entity, new: Option<Entity>) -> Option<Entity> {
    let prev = world
        .get::<&Equipment>(player)
        .ok()
        .and_then(|eq| eq.armor);
    if let Ok(mut eq) = world.get::<&mut Equipment>(player) {
        eq.armor = new;
    }
    prev
}

fn weapon_bonus(world: &World, item: Entity) -> Option<i32> {
    match world.get::<&Item>(item).ok()?.kind {
        ItemKind::Weapon { attack_bonus } => Some(attack_bonus),
        _ => None,
    }
}

fn armor_bonus(world: &World, item: Entity) -> Option<i32> {
    match world.get::<&Item>(item).ok()?.kind {
        ItemKind::Armor { defense_bonus } => Some(defense_bonus),
        _ => None,
    }
}

fn adjust_attack(world: &mut World, entity: Entity, delta: i32) {
    if let Ok(mut s) = world.get::<&mut Stats>(entity) {
        s.attack += delta;
    }
}

fn adjust_defense(world: &mut World, entity: Entity, delta: i32) {
    if let Ok(mut s) = world.get::<&mut Stats>(entity) {
        s.defense += delta;
    }
}

fn consume_item(world: &mut World, player: Entity, item: Entity) {
    if let Ok(mut inv) = world.get::<&mut Inventory>(player) {
        inv.items.retain(|e| *e != item);
    }
    let _ = world.despawn(item);
}
