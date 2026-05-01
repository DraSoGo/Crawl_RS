//! Floor threats panel: lists every mob currently alive on the floor, in a
//! compact table with the numbers a player needs for combat math —
//! mob HP, attack, hits-to-kill given player atk, damage-per-hit taken,
//! and hits-to-die given player HP.

use crate::config;
use crate::ecs::components::{Mob, Name, Player, Stats};
use crate::run_state::RunState;

use super::Buffer;

pub fn draw_threats(state: &RunState, buffer: &mut Buffer) {
    let body = threats_body(state);
    super::layout::draw_panel(
        buffer,
        super::layout::PanelBlock::new(
            "Threats on this floor",
            &body,
            "press t / esc / enter to close",
        ),
    );
}

fn threats_body(state: &RunState) -> Vec<String> {
    let Some(player) = player_stats(&state.world) else {
        return vec!["player not found".to_string()];
    };
    let groups = collect_threat_groups(&state.world);
    if groups.is_empty() {
        return vec!["no living mobs on this floor.".to_string()];
    }

    let header = ("name", "n", "hp", "atk", "def", "you→hits", "they→dmg×hits");
    let mut rows: Vec<Row> = vec![Row::header(header)];
    for group in groups {
        rows.push(Row::from_group(&group, player));
    }

    let widths = column_widths(&rows);
    let mut out: Vec<String> = Vec::with_capacity(rows.len() + 2);
    for (idx, row) in rows.iter().enumerate() {
        out.push(row.format(&widths));
        if idx == 0 {
            out.push(separator(&widths));
        }
    }
    out
}

#[derive(Clone, Copy, Debug)]
struct PlayerCombat {
    hp: i32,
    attack: i32,
    defense: i32,
}

fn player_stats(world: &hecs::World) -> Option<PlayerCombat> {
    world
        .query::<(&Player, &Stats)>()
        .iter()
        .map(|(_, (_, s))| PlayerCombat {
            hp: s.hp.max(0),
            attack: s.attack,
            defense: s.defense,
        })
        .next()
}

#[derive(Clone, Debug)]
struct ThreatGroup {
    name: String,
    count: u32,
    hp: i32,
    attack: i32,
    defense: i32,
}

fn collect_threat_groups(world: &hecs::World) -> Vec<ThreatGroup> {
    use std::collections::BTreeMap;
    let mut by_name: BTreeMap<String, ThreatGroup> = BTreeMap::new();
    for (_, (_, name, stats)) in world.query::<(&Mob, &Name, &Stats)>().iter() {
        if stats.hp <= 0 {
            continue;
        }
        let entry = by_name.entry(name.0.clone()).or_insert(ThreatGroup {
            name: name.0.clone(),
            count: 0,
            hp: stats.hp,
            attack: stats.attack,
            defense: stats.defense,
        });
        entry.count += 1;
        // First instance defines the displayed stats; later instances of
        // the same template share them within scaling tolerance.
    }
    by_name.into_values().collect()
}

#[derive(Clone, Debug)]
struct Row {
    cells: [String; 7],
}

impl Row {
    fn header(h: (&str, &str, &str, &str, &str, &str, &str)) -> Self {
        Self {
            cells: [
                h.0.to_string(),
                h.1.to_string(),
                h.2.to_string(),
                h.3.to_string(),
                h.4.to_string(),
                h.5.to_string(),
                h.6.to_string(),
            ],
        }
    }

    fn from_group(group: &ThreatGroup, player: PlayerCombat) -> Self {
        let dmg_dealt = combat_damage(player.attack, group.defense);
        let dmg_taken = combat_damage(group.attack, player.defense);
        let hits_to_kill = ceil_div(group.hp, dmg_dealt);
        let hits_to_die = ceil_div(player.hp.max(1), dmg_taken);
        Self {
            cells: [
                group.name.clone(),
                format!("{}", group.count),
                format!("{}", group.hp),
                format!("{}", group.attack),
                format!("{}", group.defense),
                format!("{hits_to_kill}"),
                format!("{dmg_taken}×{hits_to_die}"),
            ],
        }
    }

    fn format(&self, widths: &[usize; 7]) -> String {
        // Left-align the name column, right-align numeric columns so the
        // digits line up under the header.
        format!(
            "{:<w0$}  {:>w1$}  {:>w2$}  {:>w3$}  {:>w4$}  {:>w5$}  {:>w6$}",
            self.cells[0],
            self.cells[1],
            self.cells[2],
            self.cells[3],
            self.cells[4],
            self.cells[5],
            self.cells[6],
            w0 = widths[0],
            w1 = widths[1],
            w2 = widths[2],
            w3 = widths[3],
            w4 = widths[4],
            w5 = widths[5],
            w6 = widths[6],
        )
    }
}

fn column_widths(rows: &[Row]) -> [usize; 7] {
    let mut widths = [0usize; 7];
    for row in rows {
        for (i, cell) in row.cells.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }
    widths
}

fn separator(widths: &[usize; 7]) -> String {
    format!(
        "{0:-<w0$}  {0:-<w1$}  {0:-<w2$}  {0:-<w3$}  {0:-<w4$}  {0:-<w5$}  {0:-<w6$}",
        "",
        w0 = widths[0],
        w1 = widths[1],
        w2 = widths[2],
        w3 = widths[3],
        w4 = widths[4],
        w5 = widths[5],
        w6 = widths[6],
    )
}

fn combat_damage(attack: i32, defense: i32) -> i32 {
    let raw = config::COMBAT.flat_damage_bonus + attack;
    (raw - defense).max(config::COMBAT.minimum_damage)
}

fn ceil_div(a: i32, b: i32) -> i32 {
    let b = b.max(1);
    (a + b - 1) / b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceil_div_rounds_up() {
        assert_eq!(ceil_div(10, 3), 4);
        assert_eq!(ceil_div(9, 3), 3);
        assert_eq!(ceil_div(1, 5), 1);
    }

    #[test]
    fn combat_damage_floors_at_minimum() {
        // High defense vs weak attacker still deals at least minimum_damage.
        assert_eq!(combat_damage(0, 10), config::COMBAT.minimum_damage);
    }

    #[test]
    fn combat_damage_subtracts_defense() {
        assert_eq!(combat_damage(7, 2), 5);
    }
}
