//! Single-slot save/load. Bincode encoding of a `SaveSnapshot` that captures
//! everything needed to resume: seed, depth, player state, map, mobs, items,
//! amulet location, FOV memory, and message log.
//!
//! hecs's `World` is not directly serialisable, so we project entities into
//! plain data structs at save time and rebuild them on load.
//!
//! Permadeath: callers must `delete()` the file the moment the player dies.

pub mod build;
pub mod io;
pub mod restore;
pub mod scores;
pub mod types;

pub use build::build_snapshot;
pub use io::{delete, exists, load, save};
pub use restore::restore;
#[cfg(test)]
pub use types::SaveSnapshot;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::{
        Ai, BlocksTile, Energy, Equipment, FieldOfView, Inventory, Item, ItemKind, Mob,
        Name, Player, Position, Progression, Renderable, Stats,
    };
    use crate::map::Map;
    use crate::ui::messages::MessageLog;
    use hecs::World;

    fn build_test_world() -> (Map, World) {
        let map = Map::test_arena(20, 10);
        let mut world = World::new();
        world.spawn((
            Position::new(2, 2),
            Renderable::new(
                '@',
                crossterm::style::Color::Yellow,
                crossterm::style::Color::Reset,
                200,
            ),
            Player,
            BlocksTile,
            Stats::new(20, 4, 1, 10),
            Energy::new(100),
            Progression { xp: 17, kills: 3 },
            Inventory::default(),
            Equipment::default(),
            Name("you".to_string()),
            FieldOfView::new(8, map.width(), map.height()),
        ));
        world.spawn((
            Position::new(5, 5),
            Renderable::new(
                'r',
                crossterm::style::Color::DarkYellow,
                crossterm::style::Color::Reset,
                100,
            ),
            Mob,
            BlocksTile,
            Stats::new(4, 1, 0, 12),
            Energy::new(50),
            Ai::hostile(6),
            Name("rat".to_string()),
        ));
        world.spawn((
            Position::new(7, 3),
            Renderable::new(
                '!',
                crossterm::style::Color::Magenta,
                crossterm::style::Color::Reset,
                50,
            ),
            Item { kind: ItemKind::Potion { heal: 8 } },
            Name("potion of healing".to_string()),
        ));
        (map, world)
    }

    #[test]
    fn snapshot_round_trips_through_bincode() {
        let (map, world) = build_test_world();
        let mut log = MessageLog::new();
        log.combat("you hit the rat for 3.");
        log.status("you gain 2 xp.");
        let snap = build_snapshot(0xdead_beef, 4, &map, &world, &log).expect("snapshot");
        let bytes = bincode::serialize(&snap).expect("serialize");
        let restored: SaveSnapshot =
            bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(restored.seed, 0xdead_beef);
        assert_eq!(restored.depth, 4);
        assert_eq!(restored.player.stats.hp, 20);
        assert_eq!(restored.player.progression.xp, 17);
        assert_eq!(restored.mobs.len(), 1);
        assert_eq!(restored.ground_items.len(), 1);
        assert_eq!(restored.log.len(), 2);

        let r = restore(restored);
        assert_eq!(r.seed, 0xdead_beef);
        assert_eq!(r.depth, 4);
        assert!(r.world.query::<&Player>().iter().next().is_some());
        let mob_count = r.world.query::<&Mob>().iter().count();
        assert_eq!(mob_count, 1);
    }
}
