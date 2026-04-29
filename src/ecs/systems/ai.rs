//! AI: turn an NPC's situation into either a `MoveIntent` or a
//! `WantsToAttack` intent.

use hecs::{Entity, World};
use rand::Rng;

use crate::ecs::components::{
    Ai, AiKind, CasterHeal, Faction, Mob, MoveIntent, Player, Position, Renderable, Stats,
    StatusEffects, WantsToAttack,
};
use crate::map::Map;

pub fn plan<R: Rng>(world: &mut World, map: &Map, rng: &mut R, entity: Entity) {
    if status_skip(world, entity) {
        return;
    }
    handle_caster_heal(world, entity, rng);
    let view = match mob_view(world, entity) {
        Some(v) => v,
        None => return,
    };
    let intent = match view.ai.kind {
        AiKind::Hostile => plan_hostile(world, map, rng, view),
        AiKind::Sleeper { wake_radius } => plan_sleeper(world, entity, view, wake_radius),
        AiKind::Fleeing { flee_below_pct } => {
            plan_fleeing(world, map, rng, view, flee_below_pct)
        }
        AiKind::Ranged { prefer_range } => plan_ranged(world, map, view, prefer_range),
        AiKind::Mimic { revealed, .. } => plan_mimic(world, map, rng, entity, view, revealed),
    };
    apply_intent(world, entity, intent);
}

#[derive(Clone, Copy, Debug)]
enum Intent {
    Move(i32, i32),
    Attack(Entity),
    None,
}

#[derive(Clone, Copy, Debug)]
struct MobView {
    x: i32,
    y: i32,
    ai: Ai,
    faction: Faction,
}

fn mob_view(world: &World, entity: Entity) -> Option<MobView> {
    let pos = world.get::<&Position>(entity).ok()?;
    let ai = world.get::<&Ai>(entity).ok()?;
    let faction = world
        .get::<&Faction>(entity)
        .map(|f| *f)
        .unwrap_or(Faction::Hostile);
    Some(MobView {
        x: pos.x,
        y: pos.y,
        ai: *ai,
        faction,
    })
}

fn status_skip(world: &World, entity: Entity) -> bool {
    world
        .get::<&StatusEffects>(entity)
        .map(|s| s.paralyzed())
        .unwrap_or(false)
}

fn handle_caster_heal<R: Rng>(world: &mut World, entity: Entity, rng: &mut R) {
    let caster = match world.get::<&CasterHeal>(entity) {
        Ok(c) => *c,
        Err(_) => return,
    };
    if rng.gen_range(0..100) >= caster.chance_pct {
        return;
    }
    if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
        if stats.hp < stats.max_hp {
            stats.hp = (stats.hp + caster.heal_amount).min(stats.max_hp);
        }
    }
}

fn apply_intent(world: &mut World, entity: Entity, intent: Intent) {
    match intent {
        Intent::Move(dx, dy) => {
            let _ = world.insert_one(entity, MoveIntent::new(dx, dy));
        }
        Intent::Attack(target) => {
            let _ = world.insert_one(entity, WantsToAttack { target });
        }
        Intent::None => {}
    }
}

fn plan_hostile<R: Rng>(world: &World, map: &Map, rng: &mut R, view: MobView) -> Intent {
    let target_pos = match enemy_position_for(world, view) {
        Some(p) => p,
        None => return random_step(rng),
    };
    let dx = target_pos.0 - view.x;
    let dy = target_pos.1 - view.y;
    let dist_sq = dx * dx + dy * dy;
    let sight_sq = view.ai.sight_radius * view.ai.sight_radius;
    let afraid = world_status_afraid(world, view);
    if dist_sq <= sight_sq && line_of_sight(map, view.x, view.y, target_pos.0, target_pos.1) {
        if afraid {
            return Intent::Move(-dx.signum(), -dy.signum());
        }
        return Intent::Move(dx.signum(), dy.signum());
    }
    random_step(rng)
}

fn plan_sleeper(
    world: &mut World,
    entity: Entity,
    view: MobView,
    wake_radius: i32,
) -> Intent {
    let target = match enemy_position_for(world, view) {
        Some(p) => p,
        None => return Intent::None,
    };
    let dx = target.0 - view.x;
    let dy = target.1 - view.y;
    let dist_sq = dx * dx + dy * dy;
    if dist_sq <= wake_radius * wake_radius {
        if let Ok(mut ai) = world.get::<&mut Ai>(entity) {
            ai.kind = AiKind::Hostile;
        }
        return Intent::Move(dx.signum(), dy.signum());
    }
    Intent::None
}

fn plan_fleeing<R: Rng>(
    world: &World,
    map: &Map,
    rng: &mut R,
    view: MobView,
    flee_below_pct: i32,
) -> Intent {
    let target = match enemy_position_for(world, view) {
        Some(p) => p,
        None => return random_step(rng),
    };
    let dx = target.0 - view.x;
    let dy = target.1 - view.y;
    if mob_hp_pct(world, view) <= flee_below_pct {
        return Intent::Move(-dx.signum(), -dy.signum());
    }
    plan_hostile(world, map, rng, view)
}

fn mob_hp_pct(world: &World, view: MobView) -> i32 {
    for (_, (pos, stats)) in world.query::<(&Position, &Stats)>().iter() {
        if pos.x == view.x && pos.y == view.y {
            return (stats.hp * 100) / stats.max_hp.max(1);
        }
    }
    100
}

fn plan_ranged(world: &World, map: &Map, view: MobView, prefer_range: i32) -> Intent {
    let (target_pos, target_entity) = match enemy_with_entity(world, view) {
        Some(p) => p,
        None => return Intent::None,
    };
    let dx = target_pos.0 - view.x;
    let dy = target_pos.1 - view.y;
    let dist_sq = dx * dx + dy * dy;
    let sight_sq = view.ai.sight_radius * view.ai.sight_radius;
    if dist_sq > sight_sq
        || !line_of_sight(map, view.x, view.y, target_pos.0, target_pos.1)
    {
        return Intent::None;
    }
    let prefer_sq = prefer_range * prefer_range;
    if dist_sq < (prefer_range / 2).max(1).pow(2) {
        // Too close: back up.
        return Intent::Move(-dx.signum(), -dy.signum());
    }
    if dist_sq <= prefer_sq {
        return Intent::Attack(target_entity);
    }
    Intent::Move(dx.signum(), dy.signum())
}

fn plan_mimic<R: Rng>(
    world: &mut World,
    map: &Map,
    rng: &mut R,
    entity: Entity,
    view: MobView,
    revealed: bool,
) -> Intent {
    let target = match enemy_position_for(world, view) {
        Some(p) => p,
        None => return Intent::None,
    };
    let dx = target.0 - view.x;
    let dy = target.1 - view.y;
    let adjacent = dx.abs() <= 1 && dy.abs() <= 1;
    if !revealed && !adjacent {
        return Intent::None;
    }
    if !revealed {
        // Reveal: switch glyph and AI.
        if let Ok(mut ai) = world.get::<&mut Ai>(entity) {
            ai.kind = AiKind::Hostile;
        }
        if let Ok(mut r) = world.get::<&mut Renderable>(entity) {
            r.glyph = 'm';
        }
    }
    let view = MobView {
        ai: Ai { kind: AiKind::Hostile, sight_radius: view.ai.sight_radius },
        ..view
    };
    plan_hostile(world, map, rng, view)
}

/// Look up the position of the closest enemy entity to this mob, taking
/// faction into account: a `Hostile` mob targets `Player` and `PlayerAlly`
/// mobs; a `PlayerAlly` mob targets `Hostile` mobs.
fn enemy_position_for(world: &World, view: MobView) -> Option<(i32, i32)> {
    enemy_with_entity(world, view).map(|(p, _)| p)
}

fn enemy_with_entity(world: &World, view: MobView) -> Option<((i32, i32), Entity)> {
    match view.faction {
        Faction::Hostile => {
            // Prefer the player; if player not in range, target nearest ally.
            for (e, (_, pos)) in world.query::<(&Player, &Position)>().iter() {
                return Some(((pos.x, pos.y), e));
            }
            nearest_ally(world, view)
        }
        Faction::PlayerAlly => nearest_hostile(world, view),
    }
}

fn nearest_ally(world: &World, view: MobView) -> Option<((i32, i32), Entity)> {
    let mut best: Option<((i32, i32), Entity, i32)> = None;
    for (e, (_, pos, fac)) in world.query::<(&Mob, &Position, &Faction)>().iter() {
        if *fac != Faction::PlayerAlly {
            continue;
        }
        let d = (pos.x - view.x).pow(2) + (pos.y - view.y).pow(2);
        if best.map_or(true, |(_, _, bd)| d < bd) {
            best = Some(((pos.x, pos.y), e, d));
        }
    }
    best.map(|(p, e, _)| (p, e))
}

fn nearest_hostile(world: &World, view: MobView) -> Option<((i32, i32), Entity)> {
    let mut best: Option<((i32, i32), Entity, i32)> = None;
    for (e, (_, pos, fac)) in world.query::<(&Mob, &Position, &Faction)>().iter() {
        if *fac != Faction::Hostile {
            continue;
        }
        let d = (pos.x - view.x).pow(2) + (pos.y - view.y).pow(2);
        if best.map_or(true, |(_, _, bd)| d < bd) {
            best = Some(((pos.x, pos.y), e, d));
        }
    }
    best.map(|(p, e, _)| (p, e))
}

fn world_status_afraid(world: &World, view: MobView) -> bool {
    for (_, (_, pos, status)) in world.query::<(&Mob, &Position, &StatusEffects)>().iter() {
        if pos.x == view.x && pos.y == view.y {
            return status.afraid();
        }
    }
    false
}

fn random_step<R: Rng>(rng: &mut R) -> Intent {
    if rng.gen_bool(0.2) {
        return Intent::None;
    }
    let dirs = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let (dx, dy) = dirs[rng.gen_range(0..dirs.len())];
    Intent::Move(dx, dy)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::{BlocksTile, Mob, Player};
    use crate::ecs::systems::movement;
    use crate::map::{Map, Tile};
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;

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
            Faction::Hostile,
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
