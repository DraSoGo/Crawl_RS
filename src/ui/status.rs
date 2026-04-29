use crate::character::inventory_capacity;
use crate::config;
use crate::ecs::components::{
    Equipment, FieldOfView, Inventory, Item, ItemKind, Name, Player, Progression, RingEffect,
    Stats, StatusEffects,
};
use crate::run_state::RunState;

use super::Buffer;

pub fn draw_status(state: &RunState, buffer: &mut Buffer) {
    let body = status_body(state);
    super::layout::draw_panel(
        buffer,
        super::layout::PanelBlock::new(
            "Status",
            &body,
            "press k / esc / enter to close",
        ),
    );
}

fn status_body(state: &RunState) -> Vec<String> {
    let Some((stats, progression, fov, inventory, equipment, effects)) =
        player_snapshot(state)
    else {
        return vec!["player not found".to_string()];
    };

    let level = progression.level;
    let cap = inventory_capacity(level);
    let active = active_effects(&effects);
    let attack = attack_breakdown(state, level, equipment, effects);
    let defense = defense_breakdown(state, level, equipment);
    let attack_line = format_stat_line(stats.attack, attack);
    let defense_line = format_stat_line(stats.defense, defense);

    // Two-column layout: label column padded to the longest label so values
    // align in a single column regardless of which row holds them. Labels
    // sit flush left within the block.
    let rows: Vec<(&str, String)> = vec![
        ("hp", format!("{}/{}", stats.hp, stats.max_hp)),
        ("atk", attack_line),
        ("def", defense_line),
        ("move", format!("{}", stats.move_tiles)),
        ("sight", format!("{}", fov.radius)),
        ("level", format!("{level}")),
        ("xp", format!("{}/{}", progression.xp, Progression::xp_for_next(level))),
        ("kills", format!("{}", progression.kills)),
        ("depth", format!("{}", state.depth)),
        ("pack", format!("{}/{}", inventory.items.len(), cap)),
        ("weapon", equipment_name(state, equipment.weapon)),
        ("armor", equipment_name(state, equipment.armor)),
        ("ring", equipment_name(state, equipment.ring)),
        ("amulet", equipment_name(state, equipment.amulet)),
        ("effects", active),
    ];
    let label_width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    rows.into_iter()
        .map(|(label, value)| format!("{label:<label_width$}   {value}"))
        .collect()
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

#[derive(Clone, Copy)]
struct StatBreakdown {
    #[allow(dead_code)]
    total: i32,
    base: i32,
    level: i32,
    gear: i32,
    buff: i32,
}

fn attack_breakdown(
    state: &RunState,
    level: u32,
    equipment: Equipment,
    effects: StatusEffects,
) -> StatBreakdown {
    let base = config::PLAYER.base_attack;
    let level_bonus = config::PLAYER.level_up_attack * level.saturating_sub(1) as i32;
    let gear_bonus = equipment
        .weapon
        .and_then(|entity| item_attack_bonus(state, entity))
        .unwrap_or(0);
    let buff_bonus = if effects.attack_buff_turns > 0 {
        effects.attack_buff
    } else {
        0
    };
    StatBreakdown {
        total: base + level_bonus + gear_bonus + buff_bonus,
        base,
        level: level_bonus,
        gear: gear_bonus,
        buff: buff_bonus,
    }
}

fn defense_breakdown(state: &RunState, level: u32, equipment: Equipment) -> StatBreakdown {
    let base = config::PLAYER.base_defense;
    let level_bonus = config::PLAYER.level_up_defense * level.saturating_sub(1) as i32;
    let armor_bonus = equipment
        .armor
        .and_then(|entity| item_defense_bonus(state, entity))
        .unwrap_or(0);
    let ring_bonus = equipment
        .ring
        .and_then(|entity| ring_defense_bonus(state, entity))
        .unwrap_or(0);
    let gear_bonus = armor_bonus + ring_bonus;
    StatBreakdown {
        total: base + level_bonus + gear_bonus,
        base,
        level: level_bonus,
        gear: gear_bonus,
        buff: 0,
    }
}

/// Render a stat as the live `Stats` value followed by a parenthesised
/// breakdown. The headline is the value combat uses; the parts are
/// computed from gear/level/buff/base. If `Stats` carries any residual
/// modifier the four parts can't account for (legacy buff stacking,
/// pre-patch saves), surface it as `+ N extra` so the breakdown always
/// sums to the headline.
fn format_stat_line(actual: i32, breakdown: StatBreakdown) -> String {
    let mut parts = vec![
        format!("{} base", breakdown.base),
        format!("{} lvl", breakdown.level),
        format!("{} gear", breakdown.gear),
    ];
    if breakdown.buff != 0 {
        parts.push(format!("{} buff", breakdown.buff));
    }
    let known = breakdown.base + breakdown.level + breakdown.gear + breakdown.buff;
    let extra = actual - known;
    if extra != 0 {
        parts.push(format!("{extra} extra"));
    }
    format!("{} ({})", actual, parts.join(" + "))
}

fn item_attack_bonus(state: &RunState, entity: hecs::Entity) -> Option<i32> {
    match state.world.get::<&Item>(entity).ok()?.kind {
        ItemKind::Weapon { attack_bonus } => Some(attack_bonus),
        _ => None,
    }
}

fn item_defense_bonus(state: &RunState, entity: hecs::Entity) -> Option<i32> {
    match state.world.get::<&Item>(entity).ok()?.kind {
        ItemKind::Armor { defense_bonus } => Some(defense_bonus),
        _ => None,
    }
}

fn ring_defense_bonus(state: &RunState, entity: hecs::Entity) -> Option<i32> {
    match state.world.get::<&Item>(entity).ok()?.kind {
        ItemKind::Ring(RingEffect::Protection) => Some(1),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_line_omits_zero_extra_and_zero_buff() {
        let text = format_stat_line(
            7,
            StatBreakdown { total: 7, base: 0, level: 1, gear: 6, buff: 0 },
        );
        assert_eq!(text, "7 (0 base + 1 lvl + 6 gear)");
    }

    #[test]
    fn stat_line_includes_buff_segment_when_active() {
        let text = format_stat_line(
            9,
            StatBreakdown { total: 9, base: 0, level: 1, gear: 6, buff: 2 },
        );
        assert_eq!(text, "9 (0 base + 1 lvl + 6 gear + 2 buff)");
    }

    #[test]
    fn stat_line_surfaces_residual_when_breakdown_undercounts() {
        // Legacy buff-stack residual: live stat is 7 but base+lvl+gear+buff
        // only sums to 4. The extra +3 must be visible so the parens add up.
        let text = format_stat_line(
            7,
            StatBreakdown { total: 4, base: 1, level: 1, gear: 2, buff: 0 },
        );
        assert_eq!(text, "7 (1 base + 1 lvl + 2 gear + 3 extra)");
    }
}
