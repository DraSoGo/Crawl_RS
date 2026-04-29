use crossterm::style::Color;

use crate::character::inventory_capacity;
use crate::ecs::components::{
    Equipment, FieldOfView, Inventory, Name, Player, Progression, Stats, StatusEffects,
};
use crate::run_state::RunState;

use super::Buffer;

pub fn draw_status(state: &RunState, buffer: &mut Buffer) {
    let lines = status_lines(state);
    let total_h = buffer.height() as usize;
    let total_w = buffer.width() as usize;
    let start_y = total_h.saturating_sub(lines.len()) / 2;
    for (offset, line) in lines.iter().enumerate() {
        let y = (start_y + offset) as u16;
        let x = total_w.saturating_sub(line.chars().count()) / 2;
        buffer.put_str(x as u16, y, line, Color::White, Color::Reset);
    }
}

fn status_lines(state: &RunState) -> Vec<String> {
    let Some((stats, progression, fov, inventory, equipment, effects)) =
        player_snapshot(state)
    else {
        return vec![
            "Status".to_string(),
            String::new(),
            "player not found".to_string(),
            String::new(),
            "press k / esc / enter to close".to_string(),
        ];
    };

    let level = progression.level;
    let cap = inventory_capacity(level);
    let active = active_effects(&effects);
    vec![
        "Status".to_string(),
        String::new(),
        format!("hp       {}/{}", stats.hp, stats.max_hp),
        format!("atk+     {}", stats.attack),
        format!("def-     {}", stats.defense),
        format!("move     {}", stats.move_tiles),
        format!("sight    {}", fov.radius),
        format!("level    {}", level),
        format!("xp       {}/{}", progression.xp, Progression::xp_for_next(level)),
        format!("kills    {}", progression.kills),
        format!("depth    {}", state.depth),
        format!("pack     {}/{}", inventory.items.len(), cap),
        format!("weapon   {}", equipment_name(state, equipment.weapon)),
        format!("armor    {}", equipment_name(state, equipment.armor)),
        format!("ring     {}", equipment_name(state, equipment.ring)),
        format!("amulet   {}", equipment_name(state, equipment.amulet)),
        format!("effects  {active}"),
        String::new(),
        "press k / esc / enter to close".to_string(),
    ]
}

fn player_snapshot(
    state: &RunState,
) -> Option<(Stats, Progression, FieldOfView, Inventory, Equipment, StatusEffects)> {
    state
        .world
        .query::<(
            &Player,
            &Stats,
            &Progression,
            &FieldOfView,
            &Inventory,
            &Equipment,
            &StatusEffects,
        )>()
        .iter()
        .map(|(_, (_, stats, progression, fov, inventory, equipment, effects))| {
            (*stats, *progression, fov.clone(), inventory.clone(), *equipment, *effects)
        })
        .next()
}

fn equipment_name(state: &RunState, entity: Option<hecs::Entity>) -> String {
    let Some(entity) = entity else {
        return "-".to_string();
    };
    state
        .world
        .get::<&Name>(entity)
        .map(|name| name.0.clone())
        .unwrap_or_else(|_| "?".to_string())
}

fn active_effects(effects: &StatusEffects) -> String {
    let mut parts = Vec::new();
    if effects.poison_turns > 0 {
        parts.push(format!(
            "poison {}x{}",
            effects.poison_turns, effects.poison_dmg
        ));
    }
    if effects.paralysis_turns > 0 {
        parts.push(format!("paralysis {}", effects.paralysis_turns));
    }
    if effects.fear_turns > 0 {
        parts.push(format!("fear {}", effects.fear_turns));
    }
    if effects.attack_buff_turns > 0 {
        parts.push(format!(
            "attack+{} {}t",
            effects.attack_buff, effects.attack_buff_turns
        ));
    }
    if effects.vision_buff_turns > 0 {
        parts.push(format!(
            "sight+{} {}t",
            effects.vision_buff, effects.vision_buff_turns
        ));
    }
    if effects.regen_per_turn > 0 {
        parts.push(format!("regen {}", effects.regen_per_turn));
    }
    if effects.light_turns > 0 {
        parts.push(format!("light {}", effects.light_turns));
    }
    if effects.invisible {
        parts.push("invisible".to_string());
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(", ")
    }
}
