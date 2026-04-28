//! Rebuild a live `World` + `Map` + `MessageLog` from a `SaveSnapshot`.

use hecs::{Entity, World};

use crate::ecs::components::{
    Amulet, BlocksTile, Equipment, FieldOfView, Inventory, Item, Mob, Name, Player,
    Renderable,
};
use crate::map::{fov::Visibility, Map};
use crate::save::types::{
    decode_color, ItemSnapshot, MobSnapshot, PlayerSnapshot, SaveSnapshot,
};
use crate::ui::messages::MessageLog;

pub struct RestoreResult {
    pub seed: u64,
    pub depth: u32,
    pub map: Map,
    pub world: World,
    pub log: MessageLog,
    pub player_entity: Entity,
}

pub fn restore(snapshot: SaveSnapshot) -> RestoreResult {
    let mut world = World::new();
    let SaveSnapshot {
        seed,
        depth,
        map,
        player,
        mobs,
        ground_items,
        amulet,
        log: log_entries,
        ..
    } = snapshot;

    let mut log = MessageLog::new();
    for entry in log_entries {
        log.push(entry.text, entry.severity);
    }

    let player_entity = restore_player(&mut world, &player);
    for mob in mobs {
        restore_mob(&mut world, mob);
    }
    for item in ground_items {
        restore_ground_item(&mut world, item);
    }
    if let Some(pos) = amulet {
        world.spawn((
            pos,
            Renderable::new(
                '*',
                crossterm::style::Color::Yellow,
                crossterm::style::Color::Reset,
                60,
            ),
            Amulet,
            Name("Amulet of Yendor".to_string()),
        ));
    }

    RestoreResult {
        seed,
        depth,
        map,
        world,
        log,
        player_entity,
    }
}

fn restore_player(world: &mut World, snap: &PlayerSnapshot) -> Entity {
    let mut fov = FieldOfView::new(snap.fov_radius, snap.fov_w, snap.fov_h);
    let mut visibility = Visibility::new(snap.fov_w, snap.fov_h);
    for y in 0..snap.fov_h {
        for x in 0..snap.fov_w {
            let i = (y * snap.fov_w + x) as usize;
            if snap.fov_revealed.get(i).copied().unwrap_or(false) {
                visibility.force_revealed(x, y);
            }
            if snap.fov_visible.get(i).copied().unwrap_or(false) {
                visibility.force_visible(x, y);
            }
        }
    }
    fov.view = visibility;
    fov.dirty = true;

    let mut inventory = Inventory::default();
    let mut entity_for_idx: Vec<Entity> = Vec::with_capacity(snap.inventory.len());
    for item_snap in &snap.inventory {
        let entity = world.spawn((
            Item { kind: item_snap.kind },
            Name(item_snap.name.clone()),
            Renderable::new(
                item_snap.glyph,
                decode_color(item_snap.fg_index),
                crossterm::style::Color::Reset,
                50,
            ),
        ));
        entity_for_idx.push(entity);
        inventory.items.push(entity);
    }
    let equipment = Equipment {
        weapon: snap.equipped_weapon_idx.and_then(|i| entity_for_idx.get(i).copied()),
        armor: snap.equipped_armor_idx.and_then(|i| entity_for_idx.get(i).copied()),
    };

    world.spawn((
        snap.pos,
        Renderable::new(
            snap.renderable_glyph,
            crossterm::style::Color::Yellow,
            crossterm::style::Color::Reset,
            200,
        ),
        Player,
        BlocksTile,
        snap.stats,
        snap.energy,
        snap.progression,
        Name("you".to_string()),
        inventory,
        equipment,
        fov,
    ))
}

fn restore_mob(world: &mut World, snap: MobSnapshot) {
    world.spawn((
        snap.pos,
        Renderable::new(
            snap.glyph,
            decode_color(snap.fg_index),
            crossterm::style::Color::Reset,
            100,
        ),
        Mob,
        BlocksTile,
        snap.stats,
        snap.energy,
        snap.ai,
        Name(snap.name),
    ));
}

fn restore_ground_item(world: &mut World, snap: ItemSnapshot) {
    let pos = match snap.pos {
        Some(p) => p,
        None => return,
    };
    world.spawn((
        pos,
        Renderable::new(
            snap.glyph,
            decode_color(snap.fg_index),
            crossterm::style::Color::Reset,
            50,
        ),
        Item { kind: snap.kind },
        Name(snap.name),
    ));
}
