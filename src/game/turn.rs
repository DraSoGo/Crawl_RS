//! Energy-accumulator turn scheduler.
//!
//! Each scheduler tick adds `Stats::speed` to every entity's `Energy`. An
//! entity acts when its energy reaches `TURN_THRESHOLD`, then pays that
//! threshold back. `run_npcs_until_player_turn` ticks until the player is
//! ready to act, running each NPC's turn in between.

use hecs::{Entity, World};
use rand::Rng;

use crate::ecs::components::{Ai, Energy, Mob, Player, Stats};
use crate::ecs::systems::{ai, combat, fov as fov_sys, movement, status};
use crate::map::Map;
use crate::ui::MessageLog;

pub const TURN_THRESHOLD: i32 = 100;

const MAX_ITERS: u32 = 1024;

pub fn run_npcs_until_player_turn<R: Rng>(
    world: &mut World,
    map: &Map,
    log: &mut MessageLog,
    rng: &mut R,
) {
    for _ in 0..MAX_ITERS {
        if combat::player_dead(world) {
            return;
        }
        if player_ready(world) {
            return;
        }
        status::tick(world, log, rng);
        if combat::player_dead(world) {
            return;
        }
        tick_energy(world);
        let ready = collect_ready_mobs(world);
        for entity in ready {
            if !world.contains(entity) {
                continue;
            }
            ai::plan(world, map, rng, entity);
            movement::apply(world, map);
            combat::resolve(world, log, rng);
            combat::reap(world);
            spend_energy(world, entity);
            if combat::player_dead(world) {
                return;
            }
        }
        fov_sys::update(world, map);
    }
}

pub fn spend_player_energy(world: &mut World) {
    let mut player_entity = None;
    for (e, _) in world.query::<&Player>().iter() {
        player_entity = Some(e);
        break;
    }
    if let Some(entity) = player_entity {
        spend_energy(world, entity);
    }
}

fn player_ready(world: &World) -> bool {
    for (_, (_, energy)) in world.query::<(&Player, &Energy)>().iter() {
        return energy.value >= TURN_THRESHOLD;
    }
    true
}

fn tick_energy(world: &mut World) {
    let updates: Vec<(Entity, i32)> = world
        .query::<(&Stats, &Energy)>()
        .iter()
        .map(|(e, (stats, _))| (e, stats.speed.max(1)))
        .collect();
    for (entity, gain) in updates {
        if let Ok(mut energy) = world.get::<&mut Energy>(entity) {
            energy.value = energy.value.saturating_add(gain);
        }
    }
}

fn collect_ready_mobs(world: &World) -> Vec<Entity> {
    world
        .query::<(&Mob, &Energy, &Ai)>()
        .iter()
        .filter(|(_, (_, energy, _))| energy.value >= TURN_THRESHOLD)
        .map(|(entity, _)| entity)
        .collect()
}

fn spend_energy(world: &mut World, entity: Entity) {
    if let Ok(mut energy) = world.get::<&mut Energy>(entity) {
        energy.value -= TURN_THRESHOLD;
    }
}
