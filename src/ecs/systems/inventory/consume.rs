//! Wand zaps, throwables, and food.

use hecs::{Entity, World};
use rand::Rng;

use crate::ecs::components::{
    HungerClock, Item, ItemKind, Mob, Name, Position, Stats, StatusEffects,
    ThrowableKind, WandKind,
};
use crate::ui::MessageLog;

pub fn zap_wand<R: Rng>(
    world: &mut World,
    log: &mut MessageLog,
    rng: &mut R,
    player: Entity,
    item: Entity,
    item_name: &str,
    kind: WandKind,
    charges: i32,
) {
    if charges <= 0 {
        log.info(format!("the {item_name} is spent."));
        return;
    }
    let damage = match kind {
        WandKind::Fire => rng.gen_range(6..=10),
        WandKind::Cold => rng.gen_range(5..=8),
        WandKind::Lightning => rng.gen_range(8..=12),
    };
    zap_nearest(world, log, player, damage, &format!("the {item_name}"));
    if let Ok(mut item_comp) = world.get::<&mut Item>(item) {
        if let ItemKind::Wand { ref mut charges, .. } = item_comp.kind {
            *charges -= 1;
        }
    }
}

pub(super) fn zap_nearest(
    world: &mut World,
    log: &mut MessageLog,
    player: Entity,
    damage: i32,
    label: &str,
) {
    let pos = match world.get::<&Position>(player) {
        Ok(p) => *p,
        Err(_) => return,
    };
    let mut nearest: Option<(Entity, i32)> = None;
    for (entity, (_, mob_pos)) in world.query::<(&Mob, &Position)>().iter() {
        let dx = mob_pos.x - pos.x;
        let dy = mob_pos.y - pos.y;
        let dist = dx * dx + dy * dy;
        if nearest.map_or(true, |(_, d)| dist < d) {
            nearest = Some((entity, dist));
        }
    }
    let target = match nearest {
        Some((e, _)) => e,
        None => {
            log.info("you find no target.");
            return;
        }
    };
    let target_name = world
        .get::<&Name>(target)
        .map(|n| n.0.clone())
        .unwrap_or_else(|_| "something".into());
    if let Ok(mut s) = world.get::<&mut Stats>(target) {
        s.hp -= damage;
    }
    log.combat(format!("{label} hits {target_name} for {damage}."));
}

pub fn throw_item(
    world: &mut World,
    log: &mut MessageLog,
    player: Entity,
    item_name: &str,
    t: ThrowableKind,
) {
    let pos = match world.get::<&Position>(player) {
        Ok(p) => *p,
        Err(_) => return,
    };
    match t {
        ThrowableKind::OilFlask => {
            let mob_entities: Vec<Entity> = world
                .query::<(&Mob, &Position)>()
                .iter()
                .filter(|(_, (_, p))| {
                    (p.x - pos.x).abs() <= 1 && (p.y - pos.y).abs() <= 1
                })
                .map(|(e, _)| e)
                .collect();
            for entity in &mob_entities {
                if let Ok(mut s) = world.get::<&mut Stats>(*entity) {
                    s.hp -= 6;
                }
            }
            log.status(format!(
                "you smash the {item_name}; flames engulf {} mobs.",
                mob_entities.len()
            ));
        }
        ThrowableKind::SmokeBomb => {
            let mob_entities: Vec<Entity> = world
                .query::<(&Mob, &Position)>()
                .iter()
                .filter(|(_, (_, p))| {
                    (p.x - pos.x).abs() <= 2 && (p.y - pos.y).abs() <= 2
                })
                .map(|(e, _)| e)
                .collect();
            for entity in &mob_entities {
                if let Ok(mut s) = world.get::<&mut StatusEffects>(*entity) {
                    s.paralysis_turns = s.paralysis_turns.max(5);
                }
            }
            log.status(format!(
                "you smash the {item_name}; nearby mobs choke."
            ));
        }
    }
}

pub fn eat_food(
    world: &mut World,
    log: &mut MessageLog,
    target: Entity,
    item_name: &str,
    nutrition: i32,
    poisonous: bool,
) {
    if let Ok(mut h) = world.get::<&mut HungerClock>(target) {
        h.satiation = (h.satiation + nutrition).min(h.max_satiation);
    }
    if poisonous {
        if let Ok(mut s) = world.get::<&mut StatusEffects>(target) {
            s.poison_turns = s.poison_turns.max(8);
            s.poison_dmg = s.poison_dmg.max(1);
        }
        log.danger(format!("you eat the {item_name}; you feel sick."));
    } else {
        log.status(format!("you eat the {item_name}."));
    }
}
