//! FOV system: recomputes visibility for any entity whose `FieldOfView` is
//! `dirty`. Marking dirty is the responsibility of whoever changed the
//! entity's position (the movement system) or the map (level transitions).

use hecs::World;

use crate::ecs::components::{FieldOfView, Position};
use crate::map::Map;

pub fn update(world: &mut World, map: &Map) {
    // Snapshot positions/dirtiness first to avoid holding a query borrow while
    // we mutate FOV data.
    let mut updates: Vec<(hecs::Entity, i32, i32, i32)> = Vec::new();
    for (entity, (pos, fov)) in world.query::<(&Position, &FieldOfView)>().iter() {
        if fov.dirty {
            updates.push((entity, pos.x, pos.y, fov.radius));
        }
    }
    for (entity, x, y, radius) in updates {
        if let Ok(mut fov) = world.get::<&mut FieldOfView>(entity) {
            fov.view.compute(map, x, y, radius);
            fov.dirty = false;
        }
    }
}

#[allow(dead_code)]
pub fn mark_all_dirty(world: &mut World) {
    let entities: Vec<hecs::Entity> =
        world.query::<&FieldOfView>().iter().map(|(e, _)| e).collect();
    for entity in entities {
        if let Ok(mut fov) = world.get::<&mut FieldOfView>(entity) {
            fov.dirty = true;
        }
    }
}
