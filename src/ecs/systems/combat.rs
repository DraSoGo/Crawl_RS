//! Resolve `WantsToAttack` intents, apply damage, kill defenders that drop
//! to zero HP. Combat is deterministic given a seeded RNG: attack/defense
//! values fold into a single die-roll plus a flat damage floor.
//!
//! On kill: the defender is tagged `Dead` (cleaned up by `reap`), and the
//! attacker — if it is the player — gains XP based on the slain mob's worth.

use hecs::{Entity, World};
use rand::Rng;

use crate::data::mobs::TEMPLATES;
use crate::ecs::components::{
    Dead, Mob, Name, Player, Progression, Stats, WantsToAttack,
};
use crate::ui::{MessageLog, Severity};

pub fn resolve<R: Rng>(world: &mut World, log: &mut MessageLog, rng: &mut R) {
    let intents: Vec<(Entity, Entity)> = world
        .query::<&WantsToAttack>()
        .iter()
        .map(|(e, w)| (e, w.target))
        .collect();
    for (attacker, target) in intents {
        // Either party may have died earlier in the same scheduler step.
        if !world.contains(attacker) || !world.contains(target) {
            let _ = world.remove_one::<WantsToAttack>(attacker);
            continue;
        }
        let damage = roll_damage(world, attacker, target, rng);
        apply_damage(world, log, attacker, target, damage);
        let _ = world.remove_one::<WantsToAttack>(attacker);
    }
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
    // d6 roll: bigger swing than d4 so combat feels punchy and stat changes
    // (equipping weapons, scaling) read clearly in the message log.
    let raw = rng.gen_range(1..=6) + attack;
    (raw - defense).max(1)
}

fn apply_damage(
    world: &mut World,
    log: &mut MessageLog,
    attacker: Entity,
    target: Entity,
    damage: i32,
) {
    let attacker_name = name_of(world, attacker);
    let target_name = name_of(world, target);
    let (severity, hp_after) = {
        let mut hp_after = 0;
        let mut sev = Severity::Combat;
        if let Ok(mut stats) = world.get::<&mut Stats>(target) {
            stats.hp -= damage;
            hp_after = stats.hp;
            if world.get::<&Player>(target).is_ok() {
                sev = Severity::Danger;
            }
        }
        (sev, hp_after)
    };
    log.push(
        format!("{attacker_name} hits {target_name} for {damage}."),
        severity,
    );
    if hp_after <= 0 {
        on_kill(world, log, attacker, target, &target_name);
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
    if let Ok(mut prog) = world.get::<&mut Progression>(attacker) {
        prog.xp = prog.xp.saturating_add(xp);
        prog.kills = prog.kills.saturating_add(1);
        if xp > 0 {
            log.status(format!("you gain {xp} xp."));
        }
    }
    // Tag for reaping rather than despawning here so we don't invalidate
    // entity references held by other intents in the same frame.
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
            Stats::new(20, 5, 1, 10),
            Name("you".into()),
        ));
        let target = world.spawn((
            Mob,
            Stats::new(8, 2, 0, 10),
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
            Stats::new(20, 100, 0, 10),
            Progression::default(),
            Name("you".into()),
        ));
        let target = world.spawn((Mob, Stats::new(1, 0, 0, 10), Name("rat".into())));
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
            Stats::new(8, 100, 0, 10),
            Name("orc".into()),
        ));
        let player = world.spawn((
            Player,
            Stats::new(1, 1, 0, 10),
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
