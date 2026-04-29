//! Modal screens overlaid on top of the map: inventory, death, etc.

use crossterm::style::Color;
use hecs::{Entity, World};

use crate::ecs::components::{
    Equipment, Inventory, Item, ItemKind, Name, Player, ScrollKind,
};
use crate::ui::Buffer;

pub fn draw_inventory(world: &World, buffer: &mut Buffer, cursor: usize) {
    if buffer.height() == 0 || buffer.width() == 0 {
        return;
    }
    // Translucent darken: clear to background.
    for y in 0..buffer.height() {
        for x in 0..buffer.width() {
            buffer.put(x, y, crate::ui::Cell::BLANK);
        }
    }

    let player = match world.query::<&Player>().iter().next().map(|(e, _)| e) {
        Some(p) => p,
        None => return,
    };
    let lines = build_lines(world, player, cursor);
    let start_y = (buffer.height() as usize)
        .saturating_sub(lines.len())
        / 2;
    for (i, (text, color)) in lines.iter().enumerate() {
        let y = (start_y + i) as u16;
        let truncated = take_chars(text, buffer.width() as usize);
        buffer.put_str(2, y, &truncated, *color, Color::Reset);
    }
}

fn build_lines(world: &World, player: Entity, cursor: usize) -> Vec<(String, Color)> {
    let mut out: Vec<(String, Color)> = Vec::new();
    out.push(("Inventory".to_string(), Color::Yellow));
    out.push((String::new(), Color::Reset));
    let inv = match world.get::<&Inventory>(player) {
        Ok(i) => i.clone(),
        Err(_) => return out,
    };
    let equipment = world
        .get::<&Equipment>(player)
        .map(|e| *e)
        .unwrap_or_default();
    if inv.items.is_empty() {
        out.push(("  (empty)".to_string(), Color::DarkGrey));
    }
    for (idx, entity) in inv.items.iter().enumerate() {
        let name = world
            .get::<&Name>(*entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|_| "?".into());
        let descriptor = describe(world, *entity);
        let equipped = matches!(equipment.weapon, Some(w) if w == *entity)
            || matches!(equipment.armor, Some(a) if a == *entity);
        let mark = if equipped { " [E]" } else { "" };
        let prefix = if idx == cursor { ">" } else { " " };
        let color = if idx == cursor { Color::Cyan } else { Color::White };
        out.push((format!("  {prefix} {name}{mark} {descriptor}"), color));
    }
    out.push((String::new(), Color::Reset));
    out.push((
        "  up/down select   f use/equip   esc/i close".to_string(),
        Color::DarkGrey,
    ));
    out
}

fn describe(world: &World, item: Entity) -> String {
    let kind = match world.get::<&Item>(item) {
        Ok(i) => i.kind,
        Err(_) => return String::new(),
    };
    match kind {
        ItemKind::Potion { heal } => format!("(+{heal} hp)"),
        ItemKind::Scroll(ScrollKind::Mapping) => "(reveals level)".to_string(),
        ItemKind::Scroll(ScrollKind::Teleport) => "(random teleport)".to_string(),
        ItemKind::Weapon { attack_bonus } => format!("(+{attack_bonus} atk)"),
        ItemKind::Armor { defense_bonus } => format!("(+{defense_bonus} def)"),
    }
}

fn take_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
