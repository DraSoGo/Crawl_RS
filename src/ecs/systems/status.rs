//! Per-tick processing of `StatusEffects`, `Regen`, `HungerClock`, and
//! `Summoner`. Called once per scheduler iteration before NPC actions.

use hecs::{Entity, World};
use rand::Rng;

use crate::ecs::components::{
    Ai, BlocksTile, Energy, Faction, FieldOfView, HungerClock, HungerState, Mob, Name,
    Player, Position, Regen, Renderable, Stats, StatusEffects, Summoner,
};
use crate::ui::{MessageLog, Severity};

pub fn tick<R: Rng>(world: &mut World, log: &mut MessageLog, rng: &mut R) {
    tick_status_effects(world, log);
    tick_regen_components(world);
    tick_hunger(world, log);
    tick_summoners(world, log, rng);
}

fn tick_status_effects(world: &mut World, log: &mut MessageLog) {
    let entities: Vec<Entity> = world
        .query::<&StatusEffects>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for entity in entities {
        // Pull a snapshot, then commit changes back.
        let snap = match world.get::<&StatusEffects>(entity) {
            Ok(s) => *s,
            Err(_) => continue,
        };
        let is_player = world.get::<&Player>(entity).is_ok();

        let mut next = snap;

        // Poison DoT.
        if next.poison_turns > 0 {
            if let Ok(mut s) = world.get::<&mut Stats>(entity) {
                s.hp -= next.poison_dmg.max(1);
            }
            next.poison_turns -= 1;
            if is_player {
                log.push("poison gnaws at you.", Severity::Danger);
            }
            if next.poison_turns == 0 {
                next.poison_dmg = 0;
                if is_player {
                    log.status("the poison fades.");
                }
            }
        }

        // Regen via StatusEffects (rings, etc.).
        if next.regen_per_turn > 0 {
            if let Ok(mut s) = world.get::<&mut Stats>(entity) {
                s.hp = (s.hp + next.regen_per_turn).min(s.max_hp);
            }
        }

        // Decrement timed flags.
        if next.paralysis_turns > 0 {
            next.paralysis_turns -= 1;
            if next.paralysis_turns == 0 && is_player {
                log.status("you can move again.");
            }
        }
        if next.fear_turns > 0 {
            next.fear_turns -= 1;
        }

        // Buffs: when timer hits zero, undo amount on Stats.
        if next.speed_buff_turns > 0 {
            next.speed_buff_turns -= 1;
            if next.speed_buff_turns == 0 && next.speed_buff > 0 {
                if let Ok(mut s) = world.get::<&mut Stats>(entity) {
                    s.speed -= next.speed_buff;
                }
                next.speed_buff = 0;
                if is_player {
                    log.status("the speed buff fades.");
                }
            }
        }
        if next.attack_buff_turns > 0 {
            next.attack_buff_turns -= 1;
            if next.attack_buff_turns == 0 && next.attack_buff > 0 {
                if let Ok(mut s) = world.get::<&mut Stats>(entity) {
                    s.attack -= next.attack_buff;
                }
                next.attack_buff = 0;
                if is_player {
                    log.status("the strength buff fades.");
                }
            }
        }
        if next.vision_buff_turns > 0 {
            next.vision_buff_turns -= 1;
            if next.vision_buff_turns == 0 && next.vision_buff > 0 {
                if let Ok(mut fov) = world.get::<&mut FieldOfView>(entity) {
                    fov.radius = (fov.radius - next.vision_buff).max(1);
                    fov.dirty = true;
                }
                next.vision_buff = 0;
                if is_player {
                    log.status("your vision contracts.");
                }
            }
        }
        if next.light_turns > 0 {
            next.light_turns -= 1;
            if next.light_turns == 0 {
                if let Ok(mut fov) = world.get::<&mut FieldOfView>(entity) {
                    fov.radius = (fov.radius - 4).max(1);
                    fov.dirty = true;
                }
            }
        }

        if let Ok(mut s) = world.get::<&mut StatusEffects>(entity) {
            *s = next;
        }
    }
}

fn tick_regen_components(world: &mut World) {
    let updates: Vec<(Entity, i32)> = world
        .query::<(&Regen, &Stats)>()
        .iter()
        .map(|(e, (r, _))| (e, r.per_turn))
        .collect();
    for (entity, amount) in updates {
        if let Ok(mut s) = world.get::<&mut Stats>(entity) {
            s.hp = (s.hp + amount).min(s.max_hp);
        }
    }
}

fn tick_hunger(world: &mut World, log: &mut MessageLog) {
    let player_entity = match world.query::<&Player>().iter().next().map(|(e, _)| e) {
        Some(e) => e,
        None => return,
    };
    let (state, drain) = {
        let mut h = match world.get::<&mut HungerClock>(player_entity) {
            Ok(h) => h,
            Err(_) => return,
        };
        h.satiation -= 1;
        let state = h.state();
        (state, state == HungerState::Starving)
    };
    if state == HungerState::Hungry {
        // Single-shot warning when entering hungry; status field tracked by
        // a 0/1 latch in `vision_buff_turns`? Keep simple — log only when
        // satiation hits exactly threshold.
        let h = world.get::<&HungerClock>(player_entity).ok();
        if let Some(h) = h {
            if h.satiation == HungerClock::HUNGRY_THRESHOLD {
                log.danger("you are getting hungry.");
            }
        }
    }
    if drain {
        if let Ok(mut s) = world.get::<&mut Stats>(player_entity) {
            s.hp -= 1;
        }
    }
}

fn tick_summoners<R: Rng>(world: &mut World, log: &mut MessageLog, rng: &mut R) {
    let summoners: Vec<(Entity, Position, Summoner)> = world
        .query::<(&Position, &Summoner)>()
        .iter()
        .map(|(e, (p, s))| (e, *p, *s))
        .collect();
    for (_caster, pos, summoner) in summoners {
        if rng.gen_range(0..100) >= summoner.chance_pct {
            continue;
        }
        let dirs = [
            (-1, -1), (0, -1), (1, -1),
            (-1, 0),           (1, 0),
            (-1, 1),  (0, 1),  (1, 1),
        ];
        let (dx, dy) = dirs[rng.gen_range(0..dirs.len())];
        let nx = pos.x + dx;
        let ny = pos.y + dy;
        if tile_has_blocker(world, nx, ny) {
            continue;
        }
        spawn_summoned_skeleton(world, nx, ny);
        log.danger("a skeleton claws out of the floor!");
    }
}

fn tile_has_blocker(world: &World, x: i32, y: i32) -> bool {
    world
        .query::<(&Position, &BlocksTile)>()
        .iter()
        .any(|(_, (pos, _))| pos.x == x && pos.y == y)
}

fn spawn_summoned_skeleton(world: &mut World, x: i32, y: i32) {
    let template = match crate::data::mobs::by_name("skeleton archer") {
        Some(t) => t,
        None => &crate::data::mobs::TEMPLATES[0],
    };
    world.spawn((
        Position { x, y },
        Renderable::new(
            template.glyph,
            template.fg,
            crossterm::style::Color::Reset,
            100,
        ),
        Mob,
        BlocksTile,
        Stats::new(template.max_hp, template.attack, template.defense, template.speed),
        Energy::new(0),
        Ai::hostile(template.sight),
        Faction::Hostile,
        StatusEffects::default(),
        Name(format!("summoned {}", template.name)),
    ));
}
