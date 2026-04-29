//! Consumable item effects: potions, scrolls, wands, throwables, food.

use hecs::{Entity, World};
use rand::Rng;

use crate::ecs::components::{
    Ai, BlocksTile, Equipment, Faction, FieldOfView, Item, ItemKind, Mob, Name, Position,
    PotionEffect, Renderable, ScrollKind, Stats, StatusEffects,
};
use crate::ecs::systems::inventory::consume::zap_nearest;
use crate::map::{Map, Tile};
use crate::ui::MessageLog;

pub fn apply_potion(
    world: &mut World,
    log: &mut MessageLog,
    target: Entity,
    item_name: &str,
    effect: PotionEffect,
) {
    match effect {
        PotionEffect::Heal(n) | PotionEffect::GreaterHeal(n) => {
            heal(world, log, target, item_name, n)
        }
        PotionEffect::FullHeal => {
            if let Ok(mut s) = world.get::<&mut Stats>(target) {
                let before = s.hp;
                s.hp = s.max_hp;
                log.status(format!(
                    "you quaff the {item_name} (+{} hp).",
                    s.hp - before
                ));
            }
        }
        PotionEffect::MaxHpUp(n) => {
            if let Ok(mut s) = world.get::<&mut Stats>(target) {
                s.max_hp += n;
                s.hp += n;
                log.status(format!("you quaff the {item_name} (max hp +{n})."));
            }
        }
        PotionEffect::BuffAttack { amount, turns } => {
            apply_buff_attack(world, target, amount, turns);
            log.status(format!(
                "you quaff the {item_name} (+{amount} atk for {turns})."
            ));
        }
        PotionEffect::BuffVision { amount, turns } => {
            apply_buff_vision(world, target, amount, turns);
            log.status(format!(
                "you quaff the {item_name} (+{amount} sight for {turns})."
            ));
        }
        PotionEffect::CurePoison => {
            if let Ok(mut s) = world.get::<&mut StatusEffects>(target) {
                s.poison_turns = 0;
                s.poison_dmg = 0;
            }
            log.status(format!("you quaff the {item_name}; the poison fades."));
        }
    }
}

fn heal(
    world: &mut World,
    log: &mut MessageLog,
    target: Entity,
    item_name: &str,
    amount: i32,
) {
    if let Ok(mut s) = world.get::<&mut Stats>(target) {
        let before = s.hp;
        s.hp = (s.hp + amount).min(s.max_hp);
        log.status(format!(
            "you quaff the {item_name} (+{} hp).",
            s.hp - before
        ));
    }
}

fn apply_buff_attack(world: &mut World, target: Entity, amount: i32, turns: i32) {
    if let Ok(mut s) = world.get::<&mut StatusEffects>(target) {
        s.attack_buff = s.attack_buff.max(amount);
        s.attack_buff_turns = s.attack_buff_turns.max(turns);
    }
    if let Ok(mut stats) = world.get::<&mut Stats>(target) {
        stats.attack += amount;
    }
}

fn apply_buff_vision(world: &mut World, target: Entity, amount: i32, turns: i32) {
    if let Ok(mut s) = world.get::<&mut StatusEffects>(target) {
        s.vision_buff = s.vision_buff.max(amount);
        s.vision_buff_turns = s.vision_buff_turns.max(turns);
    }
    if let Ok(mut fov) = world.get::<&mut FieldOfView>(target) {
        fov.radius += amount;
        fov.dirty = true;
    }
}

pub fn apply_scroll<R: Rng>(
    world: &mut World,
    map: &mut Map,
    log: &mut MessageLog,
    rng: &mut R,
    player: Entity,
    item_name: &str,
    s: ScrollKind,
) {
    match s {
        ScrollKind::Mapping => {
            if let Ok(mut fov) = world.get::<&mut FieldOfView>(player) {
                fov.view.reveal_all();
                fov.dirty = true;
            }
            log.status(format!("the {item_name} reveals the level."));
        }
        ScrollKind::Teleport => {
            teleport_random(world, map, rng, player);
            log.status(format!("the {item_name} flings you across the level."));
        }
        ScrollKind::Identify => log.info(format!("you read the {item_name} (no effect).")),
        ScrollKind::MagicMissile => {
            zap_nearest(world, log, player, 6, "magic missile");
        }
        ScrollKind::EnchantWeapon => enchant_weapon(world, log, player, item_name),
        ScrollKind::EnchantArmor => enchant_armor(world, log, player, item_name),
        ScrollKind::Fear => apply_fear_aura(world, log, player, 10),
        ScrollKind::Summon => summon_allies(world, log, rng, player),
        ScrollKind::Light => {
            if let Ok(mut s) = world.get::<&mut StatusEffects>(player) {
                s.light_turns = s.light_turns.max(50);
            }
            if let Ok(mut fov) = world.get::<&mut FieldOfView>(player) {
                fov.radius += 4;
                fov.dirty = true;
            }
            log.status(format!("the {item_name} brightens your sight."));
        }
        ScrollKind::Recall => recall_to_up_stair(world, map, log, player),
    }
}

fn teleport_random<R: Rng>(world: &mut World, map: &Map, rng: &mut R, target: Entity) {
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
}

fn enchant_weapon(world: &mut World, log: &mut MessageLog, player: Entity, item_name: &str) {
    let weapon = world
        .get::<&Equipment>(player)
        .ok()
        .and_then(|e| e.weapon);
    let weapon = match weapon {
        Some(w) => w,
        None => {
            log.info("you have no weapon to enchant.");
            return;
        }
    };
    if let Ok(mut item) = world.get::<&mut Item>(weapon) {
        if let ItemKind::Weapon { ref mut attack_bonus } = item.kind {
            *attack_bonus += 1;
        }
    }
    if let Ok(mut s) = world.get::<&mut Stats>(player) {
        s.attack += 1;
    }
    log.status(format!("you read the {item_name}; your weapon glows."));
}

fn enchant_armor(world: &mut World, log: &mut MessageLog, player: Entity, item_name: &str) {
    let armor = world
        .get::<&Equipment>(player)
        .ok()
        .and_then(|e| e.armor);
    let armor = match armor {
        Some(a) => a,
        None => {
            log.info("you have no armor to enchant.");
            return;
        }
    };
    if let Ok(mut item) = world.get::<&mut Item>(armor) {
        if let ItemKind::Armor { ref mut defense_bonus } = item.kind {
            *defense_bonus += 1;
        }
    }
    if let Ok(mut s) = world.get::<&mut Stats>(player) {
        s.defense += 1;
    }
    log.status(format!("you read the {item_name}; your armor hardens."));
}

fn apply_fear_aura(world: &mut World, log: &mut MessageLog, player: Entity, turns: i32) {
    let pos = match world.get::<&Position>(player) {
        Ok(p) => *p,
        Err(_) => return,
    };
    let radius_sq = 8 * 8;
    let mob_entities: Vec<Entity> = world
        .query::<(&Mob, &Position)>()
        .iter()
        .filter(|(_, (_, p))| {
            let dx = p.x - pos.x;
            let dy = p.y - pos.y;
            dx * dx + dy * dy <= radius_sq
        })
        .map(|(e, _)| e)
        .collect();
    let count = mob_entities.len();
    for entity in mob_entities {
        if let Ok(mut s) = world.get::<&mut StatusEffects>(entity) {
            s.fear_turns = s.fear_turns.max(turns);
        }
    }
    if count > 0 {
        log.status(format!("nearby {count} mobs flee in terror."));
    } else {
        log.info("the air shudders.");
    }
}

fn summon_allies<R: Rng>(world: &mut World, log: &mut MessageLog, rng: &mut R, player: Entity) {
    let pos = match world.get::<&Position>(player) {
        Ok(p) => *p,
        Err(_) => return,
    };
    let dirs = [
        (-1, -1), (0, -1), (1, -1),
        (-1, 0),           (1, 0),
        (-1, 1),  (0, 1),  (1, 1),
    ];
    let template = crate::data::mobs::by_name("rat").unwrap_or(&crate::data::mobs::TEMPLATES[0]);
    let mut spawned = 0;
    for _ in 0..2 {
        let (dx, dy) = dirs[rng.gen_range(0..dirs.len())];
        let nx = pos.x + dx;
        let ny = pos.y + dy;
        world.spawn((
            Position { x: nx, y: ny },
            Renderable::new(template.glyph, template.fg, crossterm::style::Color::Reset, 100),
            Mob,
            BlocksTile,
            Stats::new(
                template.max_hp,
                template.attack,
                template.defense,
                template.move_tiles,
            ),
            Ai::hostile(template.sight),
            Faction::PlayerAlly,
            StatusEffects::default(),
            Name(format!("summoned {}", template.name)),
        ));
        spawned += 1;
    }
    log.status(format!("you summon {spawned} allies."));
}

fn recall_to_up_stair(
    world: &mut World,
    map: &Map,
    log: &mut MessageLog,
    target: Entity,
) {
    for (x, y, tile) in map.iter() {
        if matches!(tile, Tile::UpStairs) {
            if let Ok(mut pos) = world.get::<&mut Position>(target) {
                pos.x = x;
                pos.y = y;
            }
            if let Ok(mut fov) = world.get::<&mut FieldOfView>(target) {
                fov.dirty = true;
            }
            log.status("you blink to the up-stair.");
            return;
        }
    }
    log.info("nothing happens.");
}
