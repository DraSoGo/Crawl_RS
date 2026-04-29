//! Sell an inventory slot for XP. Equipped items auto-unequip on sale so
//! their stat bonuses are correctly removed.

use hecs::{Entity, World};

use crate::ecs::components::{
    Equipment, FieldOfView, Inventory, Item, ItemKind, Name, Player, PotionEffect,
    RingEffect, ScrollKind, Stats, StatusEffects,
};
use crate::ui::MessageLog;

/// Convert the inventory slot at `index` into XP. Returns `true` if a sale
/// happened (and a turn elapsed).
pub fn sell_index(world: &mut World, log: &mut MessageLog, index: usize) -> bool {
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

    unequip_if_equipped(world, player, item, kind);
    let value = sell_value(kind);
    if value <= 0 {
        log.info(format!("the {item_name} is worthless."));
    } else {
        log.loot(format!("you sell the {item_name} for {value} xp."));
    }
    if let Ok(mut inv) = world.get::<&mut Inventory>(player) {
        inv.items.retain(|e| *e != item);
    }
    let _ = world.despawn(item);
    if value > 0 {
        crate::run_state::award_xp(world, log, value);
    }
    true
}

fn find_player(world: &World) -> Option<Entity> {
    world.query::<&Player>().iter().next().map(|(e, _)| e)
}

fn unequip_if_equipped(world: &mut World, player: Entity, item: Entity, kind: ItemKind) {
    let eq = match world.get::<&Equipment>(player) {
        Ok(e) => *e,
        Err(_) => return,
    };
    if eq.weapon == Some(item) {
        if let ItemKind::Weapon { attack_bonus } = kind {
            if let Ok(mut s) = world.get::<&mut Stats>(player) {
                s.attack -= attack_bonus;
            }
        }
        if let Ok(mut e) = world.get::<&mut Equipment>(player) {
            e.weapon = None;
        }
    }
    if eq.armor == Some(item) {
        if let ItemKind::Armor { defense_bonus } = kind {
            if let Ok(mut s) = world.get::<&mut Stats>(player) {
                s.defense -= defense_bonus;
            }
        }
        if let Ok(mut e) = world.get::<&mut Equipment>(player) {
            e.armor = None;
        }
    }
    if eq.ring == Some(item) {
        if let ItemKind::Ring(r) = kind {
            unapply_ring(world, player, r);
        }
        if let Ok(mut e) = world.get::<&mut Equipment>(player) {
            e.ring = None;
        }
    }
    if eq.amulet == Some(item) {
        if let Ok(mut e) = world.get::<&mut Equipment>(player) {
            e.amulet = None;
        }
    }
}

fn unapply_ring(world: &mut World, player: Entity, effect: RingEffect) {
    match effect {
        RingEffect::Regen => {
            if let Ok(mut s) = world.get::<&mut StatusEffects>(player) {
                s.regen_per_turn = (s.regen_per_turn - 1).max(0);
            }
        }
        RingEffect::Protection => {
            if let Ok(mut s) = world.get::<&mut Stats>(player) {
                s.defense -= 1;
            }
        }
        RingEffect::Vision => {
            if let Ok(mut fov) = world.get::<&mut FieldOfView>(player) {
                fov.radius -= 2;
                fov.dirty = true;
            }
        }
    }
}

/// Map an item kind to its XP value when sold. Tuned so basic consumables
/// sell for ~3 xp (a fraction of a kill) and high-end gear sells for 15+.
pub fn sell_value(kind: ItemKind) -> i32 {
    match kind {
        ItemKind::Potion(p) => match p {
            PotionEffect::Heal(_) => 3,
            PotionEffect::GreaterHeal(_) | PotionEffect::FullHeal => 8,
            PotionEffect::MaxHpUp(_) => 10,
            PotionEffect::BuffSpeed { .. }
            | PotionEffect::BuffAttack { .. }
            | PotionEffect::BuffVision { .. } => 5,
            PotionEffect::CurePoison => 4,
        },
        ItemKind::Scroll(g) => match g {
            ScrollKind::Mapping | ScrollKind::Identify | ScrollKind::Light => 2,
            ScrollKind::Teleport | ScrollKind::Recall => 4,
            ScrollKind::MagicMissile => 3,
            ScrollKind::EnchantWeapon | ScrollKind::EnchantArmor => 6,
            ScrollKind::Fear | ScrollKind::Summon => 5,
        },
        ItemKind::Weapon { attack_bonus } => 2 + attack_bonus.max(0) * 3,
        ItemKind::Armor { defense_bonus } => 2 + defense_bonus.max(0) * 3,
        ItemKind::Ring(_) => 6,
        ItemKind::AmuletItem(_) => 8,
        ItemKind::Wand { charges, .. } => 2 + charges.max(0) * 2,
        ItemKind::Throwable(_) => 2,
        ItemKind::Food { .. } => 1,
        ItemKind::Corpse => 0,
    }
}
