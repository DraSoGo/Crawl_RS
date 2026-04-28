//! Movement system: consumes `MoveIntent`, applies it to `Position`, and
//! removes the intent so it does not bleed into the next turn.
//!
//! Phase 6 introduces entity collisions: any entity carrying `BlocksTile`
//! prevents another such entity from entering its tile. Bumping leaves the
//! intent in place momentarily but consumes it at the end of the system.

use std::collections::HashMap;

use hecs::{Entity, World};

use crate::ecs::components::{
    BlocksTile, FieldOfView, Mob, MoveIntent, Player, Position, Stats, WantsToAttack,
};
use crate::map::Map;

pub fn apply(world: &mut World, map: &Map) {
    let mut blockers = collect_blockers(world);
    let mut moves: Vec<(Entity, i32, i32, bool)> = Vec::new();
    let mut attacks: Vec<(Entity, Entity)> = Vec::new();
    let intent_entities: Vec<Entity> = world
        .query::<&MoveIntent>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for entity in intent_entities {
        let pos = match world.get::<&Position>(entity) {
            Ok(p) => *p,
            Err(_) => continue,
        };
        let intent = match world.get::<&MoveIntent>(entity) {
            Ok(i) => *i,
            Err(_) => continue,
        };
        let nx = pos.x.saturating_add(intent.dx);
        let ny = pos.y.saturating_add(intent.dy);
        let blocks_self = world.get::<&BlocksTile>(entity).is_ok();
        let walls_block = map.is_blocked(nx, ny);
        let blocker_at = blockers.get(&(nx, ny)).copied();

        if let Some(target) = blocker_at {
            if blocks_self && target != entity {
                if hostile_pair(world, entity, target) && has_stats(world, target) {
                    attacks.push((entity, target));
                    moves.push((entity, pos.x, pos.y, false));
                    continue;
                }
            }
        }

        if walls_block
            || (blocks_self && blocker_at.is_some())
            || (nx == pos.x && ny == pos.y)
        {
            moves.push((entity, pos.x, pos.y, false));
        } else {
            moves.push((entity, nx, ny, true));
            if blocks_self {
                blockers.remove(&(pos.x, pos.y));
                blockers.insert((nx, ny), entity);
            }
        }
    }
    for (entity, nx, ny, moved) in moves {
        if let Ok(mut pos) = world.get::<&mut Position>(entity) {
            pos.x = nx;
            pos.y = ny;
        }
        if moved {
            if let Ok(mut fov) = world.get::<&mut FieldOfView>(entity) {
                fov.dirty = true;
            }
        }
        let _ = world.remove_one::<MoveIntent>(entity);
    }
    for (attacker, target) in attacks {
        let _ = world.insert_one(attacker, WantsToAttack { target });
    }
}

fn collect_blockers(world: &World) -> HashMap<(i32, i32), Entity> {
    world
        .query::<(&Position, &BlocksTile)>()
        .iter()
        .map(|(e, (pos, _))| ((pos.x, pos.y), e))
        .collect()
}

fn hostile_pair(world: &World, a: Entity, b: Entity) -> bool {
    let a_player = world.get::<&Player>(a).is_ok();
    let b_player = world.get::<&Player>(b).is_ok();
    let a_mob = world.get::<&Mob>(a).is_ok();
    let b_mob = world.get::<&Mob>(b).is_ok();
    (a_player && b_mob) || (a_mob && b_player)
}

fn has_stats(world: &World, entity: Entity) -> bool {
    world.get::<&Stats>(entity).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::Player;
    use crate::map::Map;

    fn open_map() -> Map {
        Map::test_arena(20, 10)
    }

    #[test]
    fn intent_moves_and_is_consumed() {
        let map = open_map();
        let mut world = World::new();
        let player = world.spawn((
            Position::new(5, 5),
            MoveIntent::new(1, 0),
            BlocksTile,
            Player,
        ));
        apply(&mut world, &map);
        let pos = *world.get::<&Position>(player).expect("position");
        assert!(pos.x == 6 || pos.x == 5);
        assert!(world.get::<&MoveIntent>(player).is_err());
    }

    #[test]
    fn walls_block_movement() {
        let map = open_map();
        let mut world = World::new();
        let player = world.spawn((
            Position::new(1, 1),
            MoveIntent::new(-1, 0),
            BlocksTile,
        ));
        apply(&mut world, &map);
        let pos = *world.get::<&Position>(player).expect("position");
        assert_eq!(pos, Position::new(1, 1));
    }

    #[test]
    fn out_of_bounds_blocked() {
        let map = open_map();
        let mut world = World::new();
        let player = world.spawn((Position::new(0, 0), MoveIntent::new(-1, -1)));
        apply(&mut world, &map);
        let pos = *world.get::<&Position>(player).expect("position");
        assert_eq!(pos, Position::new(0, 0));
    }

    #[test]
    fn blocking_entity_stops_movement() {
        let map = open_map();
        let mut world = World::new();
        let mover = world.spawn((
            Position::new(5, 5),
            MoveIntent::new(1, 0),
            BlocksTile,
        ));
        let _wall = world.spawn((Position::new(6, 5), BlocksTile));
        apply(&mut world, &map);
        let pos = *world.get::<&Position>(mover).expect("position");
        assert_eq!(pos, Position::new(5, 5));
    }
}
