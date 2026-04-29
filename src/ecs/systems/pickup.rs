//! Pickup system: takes any `WantsToPickup` intent on the player, finds an
//! item at the player's tile, and either moves it into the player's
//! `Inventory` or — for the amulet — returns true so the main loop can
//! switch to the victory screen.

use hecs::{Entity, World};

use crate::character::inventory_capacity;
use crate::ecs::components::{
    Amulet, Inventory, Item, Name, Position, Progression, WantsToPickup,
};
use crate::ui::MessageLog;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PickupOutcome {
    pub picked_amulet: bool,
}

pub fn run(world: &mut World, log: &mut MessageLog) -> PickupOutcome {
    let mut outcome = PickupOutcome::default();
    let pickers: Vec<Entity> = world
        .query::<&WantsToPickup>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for picker in pickers {
        let _ = world.remove_one::<WantsToPickup>(picker);
        let pos = match world.get::<&Position>(picker) {
            Ok(p) => *p,
            Err(_) => continue,
        };

        // Amulet first — it short-circuits inventory rules.
        if let Some(amulet) = find_amulet_at(world, pos.x, pos.y) {
            let name = world
                .get::<&Name>(amulet)
                .map(|n| n.0.clone())
                .unwrap_or_else(|_| "the amulet".into());
            log.status(format!("you grasp the {name}!"));
            let _ = world.despawn(amulet);
            outcome.picked_amulet = true;
            continue;
        }

        let item = find_item_at(world, pos.x, pos.y);
        let item = match item {
            Some(e) => e,
            None => {
                log.info("nothing here to pick up.");
                continue;
            }
        };
        let inventory_full = match (
            world.get::<&Inventory>(picker),
            world.get::<&Progression>(picker),
        ) {
            (Ok(inv), Ok(progression)) => {
                inv.items.len() >= inventory_capacity(progression.level)
            }
            _ => true,
        };
        if inventory_full {
            log.info("your pack is full.");
            continue;
        }
        let item_name = world
            .get::<&Name>(item)
            .map(|n| n.0.clone())
            .unwrap_or_else(|_| "item".into());
        if let Ok(mut inv) = world.get::<&mut Inventory>(picker) {
            inv.items.push(item);
        }
        let _ = world.remove_one::<Position>(item);
        log.loot(format!("you pick up the {item_name}."));
    }
    outcome
}

fn find_item_at(world: &World, x: i32, y: i32) -> Option<Entity> {
    for (entity, (pos, _)) in world.query::<(&Position, &Item)>().iter() {
        if pos.x == x && pos.y == y {
            return Some(entity);
        }
    }
    None
}

fn find_amulet_at(world: &World, x: i32, y: i32) -> Option<Entity> {
    for (entity, (pos, _)) in world.query::<(&Position, &Amulet)>().iter() {
        if pos.x == x && pos.y == y {
            return Some(entity);
        }
    }
    None
}
