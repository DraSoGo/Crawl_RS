//! Render helpers for the active run (map + entities + log + HUD,
//! and the death / victory overlays).

use crossterm::style::Color;
use hecs::World;

use crate::ecs::systems::render::{draw_entities, draw_map, player_fov};
use crate::run_state::{
    player_combat, player_hp, player_kills, player_level, player_xp, RunState, UiMode,
    HUD_ROWS, LOG_ROWS, RESERVED_ROWS,
};
use crate::ui::{book, help, menus, status, Buffer, MessageLog, Severity};

pub fn draw_run(buffer: &mut Buffer, state: &RunState) {
    match state.mode {
        UiMode::Playing => draw_world(buffer, state),
        UiMode::Inventory => {
            menus::draw_inventory(&state.world, buffer, state.inventory_cursor)
        }
        UiMode::Book => book::draw_book(state, buffer),
        UiMode::Status => status::draw_status(state, buffer),
        UiMode::Help => help::draw_help(state, buffer),
        UiMode::GameOver => draw_death_screen(buffer, state),
        UiMode::Victory => draw_victory_screen(buffer, state),
    }
}

fn draw_world(buffer: &mut Buffer, state: &RunState) {
    let visibility = player_fov(&state.world);
    draw_map(&state.map, visibility.as_ref(), buffer);
    draw_entities(&state.world, visibility.as_ref(), buffer);
    draw_log(buffer, &state.log);
    draw_hud(buffer, state);
}

fn draw_log(buffer: &mut Buffer, log: &MessageLog) {
    let total_h = buffer.height();
    if total_h <= HUD_ROWS {
        return;
    }
    let log_top = total_h.saturating_sub(RESERVED_ROWS);
    let lines = log.tail(LOG_ROWS as usize);
    for (i, msg) in lines.iter().enumerate() {
        let y = log_top + i as u16;
        if y >= total_h - HUD_ROWS {
            break;
        }
        let truncated = truncate_to_width(&msg.text, buffer.width() as usize);
        buffer.put_str(0, y, &truncated, msg.severity.color(), Color::Reset);
    }
}

fn draw_hud(buffer: &mut Buffer, state: &RunState) {
    if buffer.height() == 0 {
        return;
    }
    let y = buffer.height().saturating_sub(1);
    let (hp, max_hp) = player_hp(&state.world).unwrap_or((0, 0));
    let (atk, def) = player_combat(&state.world).unwrap_or((0, 0));
    let xp = player_xp(&state.world).unwrap_or(0);
    let level = player_level(&state.world).unwrap_or(1);
    let next = crate::ecs::components::Progression::xp_for_next(level);
    let line = format!(
        "lv {level} ({xp}/{next})  depth {}  hp {hp}/{max_hp}  atk+{atk}  def-{def}  seed {:016x}  h-help",
        state.depth, state.seed
    );
    let truncated = truncate_to_width(&line, buffer.width() as usize);
    buffer.put_str(0, y, &truncated, Color::DarkGrey, Color::Reset);
}

fn draw_death_screen(buffer: &mut Buffer, state: &RunState) {
    draw_centered(buffer, &death_summary(state));
}

fn death_summary(state: &RunState) -> Vec<(String, Severity)> {
    let xp = player_xp(&state.world).unwrap_or(0);
    let kills = player_kills(&state.world).unwrap_or(0);
    let level = player_level(&state.world).unwrap_or(1);
    vec![
        ("--- YOU DIED ---".to_string(), Severity::Danger),
        (String::new(), Severity::Info),
        (format!("seed   {:016x}", state.seed), Severity::Info),
        (format!("level  {level}"), Severity::Status),
        (format!("depth  {}", state.depth), Severity::Info),
        (format!("xp     {xp}"), Severity::Status),
        (format!("kills  {kills}"), Severity::Status),
        (String::new(), Severity::Info),
        ("press q / esc / enter to exit".to_string(), Severity::Info),
    ]
}

fn draw_victory_screen(buffer: &mut Buffer, state: &RunState) {
    draw_centered(buffer, &victory_summary(state));
}

fn victory_summary(state: &RunState) -> Vec<(String, Severity)> {
    let xp = player_xp(&state.world).unwrap_or(0);
    let kills = player_kills(&state.world).unwrap_or(0);
    let level = player_level(&state.world).unwrap_or(1);
    vec![
        ("*** YOU WIN! ***".to_string(), Severity::Status),
        (String::new(), Severity::Info),
        (
            "you escape the dungeon clutching the Amulet of Yendor.".to_string(),
            Severity::Status,
        ),
        (String::new(), Severity::Info),
        (format!("seed   {:016x}", state.seed), Severity::Info),
        (format!("level  {level}"), Severity::Status),
        (format!("depth  {}", state.depth), Severity::Info),
        (format!("xp     {xp}"), Severity::Status),
        (format!("kills  {kills}"), Severity::Status),
        (String::new(), Severity::Info),
        ("press q / esc / enter to exit".to_string(), Severity::Info),
    ]
}

fn draw_centered(buffer: &mut Buffer, lines: &[(String, Severity)]) {
    let total_h = buffer.height() as usize;
    let total_w = buffer.width() as usize;
    let start_y = total_h.saturating_sub(lines.len()) / 2;
    for (i, (text, severity)) in lines.iter().enumerate() {
        let y = (start_y + i) as u16;
        let x = total_w.saturating_sub(text.chars().count()) / 2;
        buffer.put_str(x as u16, y, text, severity.color(), Color::Reset);
    }
}

fn truncate_to_width(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect()
}

#[allow(dead_code)]
pub fn _unused_world(world: &World) -> usize {
    world.iter().count()
}
