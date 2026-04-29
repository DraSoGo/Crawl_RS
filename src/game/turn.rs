//! One-round enemy turn runner.
//!
//! Each player action advances the game by one round. During that round, mobs
//! tick status effects, plan up to `Stats::move_tiles` movement steps, then all
//! queued attacks resolve simultaneously in a single combat phase.

use std::collections::HashSet;

use hecs::{Entity, World};
use rand::Rng;

use crate::ecs::components::{Mob, Stats, WantsToAttack};
use crate::ecs::systems::{ai, combat, movement, status};
use crate::map::Map;
use crate::ui::MessageLog;

pub fn run_enemy_turn<R: Rng>(
    world: &mut World,
    map: &Map,
    log: &mut MessageLog,
    rng: &mut R,
) {
    if combat::player_dead(world) {
        return;
    }
    status::tick(world, log, rng);
    if combat::player_dead(world) {
        return;
    }

    let mut moved_this_round: HashSet<Entity> = HashSet::new();
    let max_move = world
        .query::<(&Mob, &Stats)>()
        .iter()
        .map(|(_, (_, stats))| stats.move_tiles.max(0))
        .max()
        .unwrap_or(0);

    for step in 0..max_move {
        let actors = collect_mobs_for_step(world, step);
        for entity in actors {
            if !world.contains(entity) || world.get::<&WantsToAttack>(entity).is_ok() {
                continue;
            }
            ai::plan(world, map, rng, entity, &moved_this_round);
        }
        moved_this_round.extend(movement::apply(world, map));
    }

    combat::resolve(world, log, rng);
    combat::reap(world);
}

fn collect_mobs_for_step(world: &World, step: i32) -> Vec<Entity> {
    world
        .query::<(&Mob, &Stats)>()
        .iter()
        .filter(|(_, (_, stats))| stats.move_tiles > step)
        .map(|(entity, _)| entity)
        .collect()
}
