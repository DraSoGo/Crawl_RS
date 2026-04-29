//! Equipment slot management. Equipping an item modifies the wearer's
//! `Stats` (or other components for rings/amulets) and tracks the previous
//! item so it can be unequipped cleanly.

use hecs::{Entity, World};

use crate::ecs::components::{
    AmuletEffect, Equipment, FieldOfView, Item, ItemKind, RingEffect, Stats, StatusEffects,
};
use crate::ui::MessageLog;

#[derive(Clone, Copy, Debug)]
enum EquipSlot {
    Weapon,
    Armor,
    Ring,
    Amulet,
}

pub fn equip_weapon(
    world: &mut World,
    log: &mut MessageLog,
    player: Entity,
    item: Entity,
    item_name: &str,
    bonus: i32,
) {
    let prev = swap_equipment(world, player, EquipSlot::Weapon, Some(item));
    if let Some(prev_entity) = prev {
        if let Some(prev_bonus) = weapon_bonus(world, prev_entity) {
            adjust_attack(world, player, -prev_bonus);
        }
    }
    adjust_attack(world, player, bonus);
    log.status(format!("you wield the {item_name}."));
}

pub fn equip_armor(
    world: &mut World,
    log: &mut MessageLog,
    player: Entity,
    item: Entity,
    item_name: &str,
    bonus: i32,
) {
    let prev = swap_equipment(world, player, EquipSlot::Armor, Some(item));
    if let Some(prev_entity) = prev {
        if let Some(prev_bonus) = armor_bonus(world, prev_entity) {
            adjust_defense(world, player, -prev_bonus);
        }
    }
    adjust_defense(world, player, bonus);
    log.status(format!("you don the {item_name}."));
}

pub fn equip_ring(
    world: &mut World,
    log: &mut MessageLog,
    player: Entity,
    item: Entity,
    item_name: &str,
    effect: RingEffect,
) {
    let prev = swap_equipment(world, player, EquipSlot::Ring, Some(item));
    if let Some(prev_entity) = prev {
        if let Some(prev_effect) = ring_effect(world, prev_entity) {
            unapply_ring(world, player, prev_effect);
        }
    }
    apply_ring(world, player, effect);
    log.status(format!("you slip on the {item_name}."));
}

pub fn equip_amulet(
    world: &mut World,
    log: &mut MessageLog,
    player: Entity,
    item: Entity,
    item_name: &str,
    _effect: AmuletEffect,
) {
    swap_equipment(world, player, EquipSlot::Amulet, Some(item));
    log.status(format!("you don the {item_name}."));
}

fn apply_ring(world: &mut World, player: Entity, effect: RingEffect) {
    match effect {
        RingEffect::Regen => {
            if let Ok(mut s) = world.get::<&mut StatusEffects>(player) {
                s.regen_per_turn += 1;
            }
        }
        RingEffect::Protection => adjust_defense(world, player, 1),
        RingEffect::Vision => {
            if let Ok(mut fov) = world.get::<&mut FieldOfView>(player) {
                fov.radius += 2;
                fov.dirty = true;
            }
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
        RingEffect::Protection => adjust_defense(world, player, -1),
        RingEffect::Vision => {
            if let Ok(mut fov) = world.get::<&mut FieldOfView>(player) {
                fov.radius -= 2;
                fov.dirty = true;
            }
        }
    }
}

fn swap_equipment(
    world: &mut World,
    player: Entity,
    slot: EquipSlot,
    new: Option<Entity>,
) -> Option<Entity> {
    let prev = world
        .get::<&Equipment>(player)
        .ok()
        .and_then(|eq| match slot {
            EquipSlot::Weapon => eq.weapon,
            EquipSlot::Armor => eq.armor,
            EquipSlot::Ring => eq.ring,
            EquipSlot::Amulet => eq.amulet,
        });
    if let Ok(mut eq) = world.get::<&mut Equipment>(player) {
        match slot {
            EquipSlot::Weapon => eq.weapon = new,
            EquipSlot::Armor => eq.armor = new,
            EquipSlot::Ring => eq.ring = new,
            EquipSlot::Amulet => eq.amulet = new,
        }
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

fn ring_effect(world: &World, item: Entity) -> Option<RingEffect> {
    match world.get::<&Item>(item).ok()?.kind {
        ItemKind::Ring(r) => Some(r),
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
