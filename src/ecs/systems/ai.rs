//! AI: turn an NPC's situation into a `MoveIntent`.
//!
//! Phase 6 covers the `Hostile` archetype: chase the player when there's an
//! unobstructed line of sight within the mob's `sight_radius`, otherwise step
//! in a random cardinal direction (or stand still, with low probability).

use hecs::{Entity, World};
use rand::Rng;

use crate::ecs::components::{Ai, AiKind, MoveIntent, Player, Position};
use crate::map::Map;

pub fn plan<R: Rng>(world: &mut World, map: &Map, rng: &mut R, entity: Entity) {
    let Some((mx, my, ai)) = mob_view(world, entity) else { return };
    let intent = match ai.kind {
        AiKind::Hostile => plan_hostile(world, map, rng, mx, my, ai),
    };
    if let Some(intent) = intent {
        let _ = world.insert_one(entity, intent);
    }
}

fn mob_view(world: &World, entity: Entity) -> Option<(i32, i32, Ai)> {
    let pos = world.get::<&Position>(entity).ok()?;
    let ai = world.get::<&Ai>(entity).ok()?;
    Some((pos.x, pos.y, *ai))
}

fn plan_hostile<R: Rng>(
    world: &World,
    map: &Map,
    rng: &mut R,
    mx: i32,
    my: i32,
    ai: Ai,
) -> Option<MoveIntent> {
    if let Some((px, py)) = player_position(world) {
        let dx = px - mx;
        let dy = py - my;
        let dist_sq = dx * dx + dy * dy;
        let sight_sq = ai.sight_radius * ai.sight_radius;
        if dist_sq <= sight_sq && line_of_sight(map, mx, my, px, py) {
            return Some(step_toward(dx, dy));
        }
    }
    random_step(rng)
}

fn player_position(world: &World) -> Option<(i32, i32)> {
    for (_, (_, pos)) in world.query::<(&Player, &Position)>().iter() {
        return Some((pos.x, pos.y));
    }
    None
}

fn step_toward(dx: i32, dy: i32) -> MoveIntent {
    MoveIntent::new(dx.signum(), dy.signum())
}

fn random_step<R: Rng>(rng: &mut R) -> Option<MoveIntent> {
    // 1-in-5 chance of standing still — keeps idle mobs from twitching.
    if rng.gen_bool(0.2) {
        return None;
    }
    let dirs = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let (dx, dy) = dirs[rng.gen_range(0..dirs.len())];
    Some(MoveIntent::new(dx, dy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::{BlocksTile, Mob, Player};
    use crate::ecs::systems::movement;
    use crate::map::{Map, Tile};
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;

    /// Solid floor inside a wall border; no pillars.
    fn open_room(w: i32, h: i32) -> Map {
        let mut m = Map::new(w, h);
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                m.set(x, y, Tile::Floor);
            }
        }
        m
    }

    #[test]
    fn hostile_mob_steps_toward_visible_player() {
        let map = open_room(30, 10);
        let mut world = World::new();
        world.spawn((Position::new(5, 5), Player, BlocksTile));
        let mob = world.spawn((
            Position::new(15, 5),
            Mob,
            BlocksTile,
            Ai::hostile(20),
        ));
        let mut rng = Pcg64Mcg::seed_from_u64(0);
        plan(&mut world, &map, &mut rng, mob);
        movement::apply(&mut world, &map);
        let pos = *world.get::<&Position>(mob).expect("mob pos");
        assert_eq!(pos, Position::new(14, 5));
    }

    #[test]
    fn los_blocked_by_wall() {
        let mut map = open_room(20, 7);
        map.set(6, 3, Tile::Wall);
        assert!(!line_of_sight(&map, 3, 3, 10, 3));
    }

    #[test]
    fn los_clear_in_open_room() {
        let map = open_room(20, 7);
        assert!(line_of_sight(&map, 3, 3, 10, 3));
    }
}

/// Bresenham line-of-sight check: walks the line from (x0, y0) to (x1, y1)
/// and returns false the moment it hits an opaque tile (excluding the
/// endpoints themselves).
fn line_of_sight(map: &Map, mut x0: i32, mut y0: i32, x1: i32, y1: i32) -> bool {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x0 == x1 && y0 == y1 {
            return true;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
        if x0 == x1 && y0 == y1 {
            return true;
        }
        if let Some(t) = map.tile(x0, y0) {
            if t.blocks_sight() {
                return false;
            }
        } else {
            return false;
        }
    }
}
