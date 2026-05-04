//! Render helpers for the active run (top bar + map + entities + log + HUD,
//! and the death / victory overlays).

use crossterm::style::Color;
use hecs::World;

use crate::ecs::components::Progression;
use crate::ecs::systems::render::{draw_entities, draw_map, player_fov};
use crate::run_state::{
    player_combat, player_hp, player_kills, player_level, player_position, player_xp,
    RunState, UiMode, HUD_ROWS, LOG_ROWS, RESERVED_ROWS, TOP_BAR_ROWS,
};
use crate::ui::{book, help, menus, status, threats, Buffer, Cell, MessageLog, Severity};

const TOP_BAR_BG: Color = Color::Reset;

pub fn draw_run(buffer: &mut Buffer, state: &RunState) {
    match state.mode {
        UiMode::Playing => draw_world(buffer, state),
        UiMode::Inventory => {
            menus::draw_inventory(&state.world, buffer, state.inventory_cursor)
        }
        UiMode::Book => book::draw_book(state, buffer),
        UiMode::Status => status::draw_status(state, buffer),
        UiMode::Help => help::draw_help(state, buffer),
        UiMode::Threats => threats::draw_threats(state, buffer),
        UiMode::GameOver => draw_death_screen(buffer, state),
        UiMode::Victory => draw_victory_screen(buffer, state),
    }
}

fn draw_world(buffer: &mut Buffer, state: &RunState) {
    let visibility = player_fov(&state.world);
    let (cam_x, cam_y) = camera_offset(buffer, state);
    draw_top_bar(buffer, state);
    draw_map(&state.map, visibility.as_ref(), buffer, TOP_BAR_ROWS, cam_x, cam_y);
    draw_entities(&state.world, visibility.as_ref(), buffer, TOP_BAR_ROWS, cam_x, cam_y);
    draw_log(buffer, &state.log);
    draw_hud(buffer, state);
}

fn camera_offset(buffer: &Buffer, state: &RunState) -> (i32, i32) {
    let (px, py) = player_position(&state.world)
        .map(|p| (p.x, p.y))
        .unwrap_or((0, 0));
    let vw = buffer.width() as i32;
    let vh = (buffer.height() as i32).saturating_sub(RESERVED_ROWS as i32);
    let map_w = state.map.width();
    let map_h = state.map.height();
    let cam_x = (px - vw / 2).clamp(0, (map_w - vw).max(0));
    let cam_y = (py - vh / 2).clamp(0, (map_h - vh).max(0));
    (cam_x, cam_y)
}

/// Coloured top-bar with depth, level/XP, HP gauge, atk, def, seed.
fn draw_top_bar(buffer: &mut Buffer, state: &RunState) {
    if buffer.height() == 0 || buffer.width() == 0 {
        return;
    }
    let width = buffer.width() as usize;

    // Fill row with the bar background.
    for x in 0..buffer.width() {
        buffer.put(x, 0, Cell::new(' ', Color::White, TOP_BAR_BG));
    }

    let (hp, max_hp) = player_hp(&state.world).unwrap_or((0, 0));
    let (atk, def) = player_combat(&state.world).unwrap_or((0, 0));
    let xp = player_xp(&state.world).unwrap_or(0);
    let level = player_level(&state.world).unwrap_or(1);
    let next = Progression::xp_for_next(level);

    let mut x = 1u16;
    x = put_segment(buffer, x, 0, "DEPTH ", Color::DarkCyan, TOP_BAR_BG);
    x = put_segment(buffer, x, 0, &format!("{:<3}", state.depth), Color::Yellow, TOP_BAR_BG);
    x = put_segment(buffer, x, 0, "  LV ", Color::Cyan, TOP_BAR_BG);
    x = put_segment(
        buffer,
        x,
        0,
        &format!("{:<2} ({}/{})", level, xp, next),
        Color::Yellow,
        TOP_BAR_BG,
    );
    x = put_segment(buffer, x, 0, "  HP ", Color::Red, TOP_BAR_BG);
    x = draw_hp_gauge(buffer, x, 0, hp, max_hp);
    x = put_segment(
        buffer,
        x,
        0,
        &format!(" {}/{}", hp, max_hp),
        hp_color(hp, max_hp),
        TOP_BAR_BG,
    );
    x = put_segment(buffer, x, 0, "  ATK ", Color::Red, TOP_BAR_BG);
    x = put_segment(buffer, x, 0, &format!("{atk}"), Color::Yellow, TOP_BAR_BG);
    x = put_segment(buffer, x, 0, "  DEF ", Color::Cyan, TOP_BAR_BG);
    x = put_segment(buffer, x, 0, &format!("{def}"), Color::Yellow, TOP_BAR_BG);

    // Right-aligned seed tag.
    let seed_tag = format!("SEED {:016x}", state.seed);
    let seed_w = seed_tag.chars().count();
    if width.saturating_sub(seed_w) > x as usize + 2 {
        let sx = (width - seed_w - 1) as u16;
        put_segment(buffer, sx, 0, &seed_tag, Color::DarkGrey, TOP_BAR_BG);
    }
}

fn put_segment(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    text: &str,
    fg: Color,
    bg: Color,
) -> u16 {
    let mut cx = x;
    for ch in text.chars() {
        if cx >= buffer.width() {
            break;
        }
        buffer.put(cx, y, Cell::new(ch, fg, bg));
        cx += 1;
    }
    cx
}

/// 10-cell HP gauge: filled portion in HP-state colour, remainder in dim grey.
/// Returns the next x cursor position.
fn draw_hp_gauge(buffer: &mut Buffer, x: u16, y: u16, hp: i32, max_hp: i32) -> u16 {
    const WIDTH: u16 = 10;
    let max_hp = max_hp.max(1);
    let hp = hp.clamp(0, max_hp);
    let filled = ((hp as u32 * WIDTH as u32) / max_hp as u32) as u16;
    let colour = hp_color(hp, max_hp);
    let mut cx = x;
    if cx >= buffer.width() {
        return cx;
    }
    buffer.put(cx, y, Cell::new('[', Color::White, TOP_BAR_BG));
    cx += 1;
    for i in 0..WIDTH {
        if cx >= buffer.width() {
            return cx;
        }
        let (ch, fg) = if i < filled {
            ('█', colour)
        } else {
            ('░', Color::DarkGrey)
        };
        buffer.put(cx, y, Cell::new(ch, fg, TOP_BAR_BG));
        cx += 1;
    }
    if cx < buffer.width() {
        buffer.put(cx, y, Cell::new(']', Color::White, TOP_BAR_BG));
        cx += 1;
    }
    cx
}

fn hp_color(hp: i32, max_hp: i32) -> Color {
    let max = max_hp.max(1);
    let pct = (hp.max(0) as f32) / (max as f32);
    if pct >= 0.6 {
        Color::Green
    } else if pct >= 0.3 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn draw_log(buffer: &mut Buffer, log: &MessageLog) {
    let total_h = buffer.height();
    if total_h <= HUD_ROWS {
        return;
    }
    let log_top = total_h.saturating_sub(RESERVED_ROWS - TOP_BAR_ROWS);
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
    // Hint line: short colour-coded shortcut keys.
    for x in 0..buffer.width() {
        buffer.put(x, y, Cell::new(' ', Color::DarkGrey, Color::Reset));
    }
    let mut x = 0u16;
    x = put_segment(buffer, x, y, " ", Color::Reset, Color::Reset);
    x = put_kbd(buffer, x, y, "wasd", "move");
    x = put_kbd(buffer, x, y, "qezx", "diag");
    x = put_kbd(buffer, x, y, "f", "pick");
    x = put_kbd(buffer, x, y, "i", "inv");
    x = put_kbd(buffer, x, y, "k", "stat");
    x = put_kbd(buffer, x, y, "b", "book");
    x = put_kbd(buffer, x, y, "t", "threat");
    x = put_kbd(buffer, x, y, "h", "help");
    x = put_kbd(buffer, x, y, ">", "stairs");
    let _ = put_kbd(buffer, x, y, "esc", "quit");

    // If a depth/seed footer is desired right-aligned, add later.
    let _ = state;
}

fn put_kbd(buffer: &mut Buffer, x: u16, y: u16, key: &str, label: &str) -> u16 {
    let mut cx = put_segment(buffer, x, y, " [", Color::DarkGrey, Color::Reset);
    cx = put_segment(buffer, cx, y, key, Color::Yellow, Color::Reset);
    cx = put_segment(buffer, cx, y, "] ", Color::DarkGrey, Color::Reset);
    cx = put_segment(buffer, cx, y, label, Color::White, Color::Reset);
    cx
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
