//! Render systems: tiles first, entities on top. Both write into the
//! shared draw buffer; the buffer's diff flush decides what hits stdout.

use crossterm::style::Color;
use hecs::World;

use crate::ecs::components::{FieldOfView, Player, Position, Renderable, StatusEffects};
use crate::map::fov::Visibility;
use crate::map::Map;
use crate::ui::{Buffer, Cell};

pub fn draw_map(map: &Map, fov: Option<&Visibility>, buffer: &mut Buffer, y_offset: u16) {
    let bw = buffer.width() as i32;
    let bh = buffer.height() as i32;
    let oy = y_offset as i32;
    for (x, y, tile) in map.iter() {
        let py = y + oy;
        if x < 0 || py < 0 || x >= bw || py >= bh {
            continue;
        }
        let (visible, revealed) = match fov {
            Some(v) => (v.is_visible(x, y), v.is_revealed(x, y)),
            None => (true, true),
        };
        if !visible && !revealed {
            buffer.put(x as u16, py as u16, Cell::BLANK);
            continue;
        }
        let fg = if visible {
            tile.fg()
        } else {
            Color::DarkGrey
        };
        buffer.put(x as u16, py as u16, Cell::new(tile.glyph(), fg, tile.bg()));
    }
}

/// Locates the player's FOV component so `draw_map` and `draw_entities`
/// stay in sync with the same visibility data.
pub fn player_fov(world: &World) -> Option<Visibility> {
    let mut found: Option<Visibility> = None;
    for (_, (_, fov)) in world.query::<(&Player, &FieldOfView)>().iter() {
        found = Some(fov.view.clone());
        break;
    }
    found
}

pub fn draw_entities(world: &World, fov: Option<&Visibility>, buffer: &mut Buffer, y_offset: u16) {
    let player_pos = world
        .query::<(&Player, &Position)>()
        .iter()
        .map(|(_, (_, p))| (p.x, p.y))
        .next();
    let mut entries: Vec<(i32, i32, Renderable)> = world
        .query::<(&Position, &Renderable)>()
        .iter()
        .filter(|(entity, (pos, _))| {
            let in_fov = match fov {
                Some(v) => v.is_visible(pos.x, pos.y),
                None => true,
            };
            if !in_fov {
                return false;
            }
            // Invisible mobs only render when adjacent to the player.
            if let Ok(status) = world.get::<&StatusEffects>(*entity) {
                if status.invisible {
                    if let Some((px, py)) = player_pos {
                        return (pos.x - px).abs() <= 1 && (pos.y - py).abs() <= 1;
                    }
                    return false;
                }
            }
            true
        })
        .map(|(_, (pos, render))| (pos.x, pos.y, *render))
        .collect();
    entries.sort_by_key(|(_, _, r)| r.layer);

    let w = buffer.width() as i32;
    let h = buffer.height() as i32;
    let oy = y_offset as i32;
    for (x, y, render) in entries {
        let py = y + oy;
        if x < 0 || py < 0 || x >= w || py >= h {
            continue;
        }
        buffer.put(
            x as u16,
            py as u16,
            Cell::new(render.glyph, render.fg, render.bg),
        );
    }
}
