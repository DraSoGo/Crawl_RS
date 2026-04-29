//! Resolve `WantsToAttack` intents, apply damage, kill defenders that drop
//! to zero HP. Combat is deterministic given a seeded RNG: attack/defense
//! values fold into a single die-roll plus a flat damage floor.
//!
//! All queued attacks resolve simultaneously. Damage is rolled from the
//! pre-hit world state, then applied together before kills are processed.

use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};
use rand::Rng;

use crate::data::mobs::TEMPLATES;
use crate::ecs::components::{
    Dead, Mob, Name, OnHit, Player, Progression, Stats, StatusEffects, WantsToAttack,
};
use crate::ui::{MessageLog, Severity};

pub fn resolve<R: Rng>(world: &mut World, log: &mut MessageLog, rng: &mut R) {
    let intents: Vec<(Entity, Entity)> = world
        .query::<&WantsToAttack>()
        .iter()
        .map(|(e, w)| (e, w.target))
        .collect();

    let mut attacks: Vec<ResolvedAttack> = Vec::new();
    for (attacker, target) in &intents {
        if !world.contains(*attacker) || !world.contains(*target) {
            continue;
        }
        if world.get::<&Stats>(*attacker).is_err() || world.get::<&Stats>(*target).is_err() {
            continue;
        }
        attacks.push(ResolvedAttack {
            attacker: *attacker,
            target: *target,
            damage: roll_damage(world, *attacker, *target, rng),
            attacker_name: name_of(world, *attacker),
            target_name: name_of(world, *target),
        });
    }

    for (attacker, _) in intents {
        let _ = world.remove_one::<WantsToAttack>(attacker);
    }

    let mut damage_by_target: HashMap<Entity, i32> = HashMap::new();
    for attack in &attacks {
        *damage_by_target.entry(attack.target).or_default() += attack.damage;
    }
    for (target, total_damage) in damage_by_target {
        if let Ok(mut stats) = world.get::<&mut Stats>(target) {
            stats.hp -= total_damage;
        }
    }

    for attack in &attacks {
        let severity = if world.get::<&Player>(attack.target).is_ok() {
            Severity::Danger
        } else {
            Severity::Combat
        };
        log.push(
            format!(
                "{} hits {} for {}.",
                attack.attacker_name, attack.target_name, attack.damage
            ),
            severity,
        );
    }

    for attack in &attacks {
        apply_on_hit(world, log, attack.attacker, attack.target, &attack.target_name);
    }

    let mut killed: HashSet<Entity> = HashSet::new();
    for attack in attacks {
        let hp_after = world
            .get::<&Stats>(attack.target)
            .ok()
            .map(|stats| stats.hp)
            .unwrap_or(1);
        if hp_after <= 0 && killed.insert(attack.target) {
            on_kill(world, log, attack.attacker, attack.target, &attack.target_name);
        }
    }
}

struct ResolvedAttack {
    attacker: Entity,
    target: Entity,
    damage: i32,
    attacker_name: String,
    target_name: String,
}

fn roll_damage<R: Rng>(world: &World, attacker: Entity, target: Entity, rng: &mut R) -> i32 {
    let attack = world
        .get::<&Stats>(attacker)
        .ok()
        .map(|s| s.attack)
        .unwrap_or(0);
    let defense = world
        .get::<&Stats>(target)
        .ok()
        .map(|s| s.defense)
        .unwrap_or(0);
    let raw = rng.gen_range(1..=6) + attack;
    (raw - defense).max(1)
}

fn apply_on_hit(
    world: &mut World,
    log: &mut MessageLog,
    attacker: Entity,
    target: Entity,
    target_name: &str,
) {
    let on_hit = match world.get::<&OnHit>(attacker) {
        Ok(o) => *o,
        Err(_) => return,
    };
    if on_hit.poison_turns > 0 {
        if let Ok(mut s) = world.get::<&mut StatusEffects>(target) {
            s.poison_turns = s.poison_turns.max(on_hit.poison_turns);
            s.poison_dmg = s.poison_dmg.max(on_hit.poison_dmg.max(1));
        }
        log.danger(format!("{target_name} is poisoned!"));
    }
    if on_hit.paralysis_turns > 0 {
        if let Ok(mut s) = world.get::<&mut StatusEffects>(target) {
            s.paralysis_turns = s.paralysis_turns.max(on_hit.paralysis_turns);
        }
        log.danger(format!("{target_name} is paralyzed!"));
    }
}

fn on_kill(
    world: &mut World,
    log: &mut MessageLog,
    attacker: Entity,
    target: Entity,
    target_name: &str,
) {
    let target_is_player = world.get::<&Player>(target).is_ok();
    if target_is_player {
        log.danger(format!("{target_name} died!"));
        // Tag the player with `Dead` so the main loop can switch to the
        // game-over screen; do NOT despawn (we still need stats for the
        // summary render).
        let _ = world.insert_one(target, Dead);
        return;
    }

    log.combat(format!("{target_name} dies!"));
    let xp = mob_xp(world, target).unwrap_or(0);
    let attacker_is_player = world.get::<&Player>(attacker).is_ok();
    if let Ok(mut prog) = world.get::<&mut Progression>(attacker) {
        prog.kills = prog.kills.saturating_add(1);
    }
    if attacker_is_player && xp > 0 {
        log.status(format!("you gain {xp} xp."));
        crate::run_state::award_xp(world, log, xp);
    }
    let _ = world.insert_one(target, Dead);
}

fn mob_xp(world: &World, entity: Entity) -> Option<i32> {
    if world.get::<&Mob>(entity).is_err() {
        return None;
    }
    let name = world.get::<&Name>(entity).ok()?;
    TEMPLATES
        .iter()
        .find(|t| t.name == name.0)
        .map(|t| t.xp)
}

fn name_of(world: &World, entity: Entity) -> String {
    world
        .get::<&Name>(entity)
        .map(|n| n.0.clone())
        .unwrap_or_else(|_| "something".to_string())
}

/// Despawn anything tagged `Dead` (except the player — the main loop wants
/// to read stats for the death screen).
pub fn reap(world: &mut World) {
    let dead: Vec<Entity> = world
        .query::<&Dead>()
        .iter()
        .filter(|(e, _)| world.get::<&Player>(*e).is_err())
        .map(|(e, _)| e)
        .collect();
    for entity in dead {
        let _ = world.despawn(entity);
    }
}

pub fn player_dead(world: &World) -> bool {
    world
        .query::<(&Player, &Dead)>()
        .iter()
        .next()
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::{Mob, Name, Player, Stats};
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;

    #[test]
    fn attack_damages_target() {
        let mut world = World::new();
        let mut log = MessageLog::new();
        let attacker = world.spawn((
            Player,
            Stats::new(20, 5, 1, 1),
            Name("you".into()),
        ));
        let target = world.spawn((
            Mob,
            Stats::new(8, 2, 0, 1),
            Name("rat".into()),
        ));
        world.insert_one(attacker, WantsToAttack { target }).unwrap();
        let mut rng = Pcg64Mcg::seed_from_u64(0);
        resolve(&mut world, &mut log, &mut rng);
        let stats = *world.get::<&Stats>(target).unwrap();
        assert!(stats.hp < 8, "target should have taken damage");
        assert!(world.get::<&WantsToAttack>(attacker).is_err());
    }

    #[test]
    fn lethal_attack_marks_dead_and_awards_xp() {
        let mut world = World::new();
        let mut log = MessageLog::new();
        let attacker = world.spawn((
            Player,
            Stats::new(20, 100, 0, 1),
            Progression::default(),
            Name("you".into()),
        ));
        let target = world.spawn((Mob, Stats::new(1, 0, 0, 1), Name("rat".into())));
        world.insert_one(attacker, WantsToAttack { target }).unwrap();
        let mut rng = Pcg64Mcg::seed_from_u64(7);
        resolve(&mut world, &mut log, &mut rng);
        assert!(world.get::<&Dead>(target).is_ok());
        let prog = *world.get::<&Progression>(attacker).unwrap();
        assert!(prog.xp >= 2, "rat awards >=2 xp; got {}", prog.xp);
        assert_eq!(prog.kills, 1);
        reap(&mut world);
        assert!(!world.contains(target));
    }

    #[test]
    fn player_death_marks_game_over() {
        let mut world = World::new();
        let mut log = MessageLog::new();
        let attacker = world.spawn((
            Mob,
            Stats::new(8, 100, 0, 1),
            Name("orc".into()),
        ));
        let player = world.spawn((
            Player,
            Stats::new(1, 1, 0, 1),
            Progression::default(),
            Name("you".into()),
        ));
        world.insert_one(attacker, WantsToAttack { target: player }).unwrap();
        let mut rng = Pcg64Mcg::seed_from_u64(0);
        resolve(&mut world, &mut log, &mut rng);
        assert!(player_dead(&world));
        // Player not despawned even after reap.
        reap(&mut world);
        assert!(world.contains(player));
    }
}
