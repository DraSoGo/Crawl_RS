//! Inventory dispatch: take an inventory slot index, decide what kind of
//! item it is, and route to the appropriate effect/equip handler.

mod consume;
mod effects;
mod equip;
mod sell;

pub use sell::sell_index;

use hecs::{Entity, World};
use rand::Rng;

use crate::ecs::components::{
    Inventory, Item, ItemKind, Name, Player,
};
use crate::map::Map;
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
        ItemKind::Potion(effect) => {
            effects::apply_potion(world, log, player, &item_name, effect);
            consume_item(world, player, item);
            true
        }
        ItemKind::Scroll(s) => {
            effects::apply_scroll(world, map, log, rng, player, &item_name, s);
            consume_item(world, player, item);
            true
        }
        ItemKind::Weapon { attack_bonus } => {
            equip::equip_weapon(world, log, player, item, &item_name, attack_bonus);
            true
        }
        ItemKind::Armor { defense_bonus } => {
            equip::equip_armor(world, log, player, item, &item_name, defense_bonus);
            true
        }
        ItemKind::Ring(r) => {
            equip::equip_ring(world, log, player, item, &item_name, r);
            true
        }
        ItemKind::AmuletItem(a) => {
            equip::equip_amulet(world, log, player, item, &item_name, a);
            true
        }
        ItemKind::Wand { kind, charges } => {
            consume::zap_wand(world, log, rng, player, item, &item_name, kind, charges);
            true
        }
        ItemKind::Throwable(t) => {
            consume::throw_item(world, log, player, &item_name, t);
            consume_item(world, player, item);
            true
        }
        ItemKind::Food { nutrition, poisonous } => {
            consume::eat_food(world, log, player, &item_name, nutrition, poisonous);
            consume_item(world, player, item);
            true
        }
        ItemKind::Corpse => {
            consume::eat_food(world, log, player, &item_name, 200, false);
            consume_item(world, player, item);
            true
        }
    }
}

fn find_player(world: &World) -> Option<Entity> {
    world.query::<&Player>().iter().next().map(|(e, _)| e)
}

fn consume_item(world: &mut World, player: Entity, item: Entity) {
    if let Ok(mut inv) = world.get::<&mut Inventory>(player) {
        inv.items.retain(|e| *e != item);
    }
    let _ = world.despawn(item);
}
