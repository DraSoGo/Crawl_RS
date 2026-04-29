use crossterm::style::Color;

use crate::codex::{self, BookPage};
use crate::run_state::RunState;
use crate::ui::{Buffer, Cell};

const PADDING_X: u16 = 2;
const HEADER_Y: u16 = 1;
const TAB_Y: u16 = 2;
const BODY_TOP: u16 = 4;

pub fn draw_book(state: &RunState, buffer: &mut Buffer) {
    if buffer.width() == 0 || buffer.height() == 0 {
        return;
    }

    clear_overlay(buffer);
    draw_header(state, buffer);
    draw_body(state, buffer);
    draw_footer(buffer);
}

fn clear_overlay(buffer: &mut Buffer) {
    for y in 0..buffer.height() {
        for x in 0..buffer.width() {
            buffer.put(x, y, Cell::BLANK);
        }
    }
}

fn draw_header(state: &RunState, buffer: &mut Buffer) {
    buffer.put_str(PADDING_X, HEADER_Y, "Book", Color::Yellow, Color::Reset);

    let mob_tab = format_tab(BookPage::Mob.label(), state.book_page == BookPage::Mob);
    let item_tab = format_tab(BookPage::Item.label(), state.book_page == BookPage::Item);
    buffer.put_str(PADDING_X, TAB_Y, &mob_tab, Color::Reset, Color::Reset);
    buffer.put_str(
        PADDING_X + mob_tab.chars().count() as u16 + 1,
        TAB_Y,
        &item_tab,
        Color::Reset,
        Color::Reset,
    );
}

fn draw_body(state: &RunState, buffer: &mut Buffer) {
    if buffer.height() <= BODY_TOP + 2 {
        return;
    }

    let left_width = ((buffer.width() as usize) / 3).clamp(22, 34) as u16;
    let divider_x = left_width.saturating_add(PADDING_X + 1);
    let list_height = buffer.height().saturating_sub(BODY_TOP + 2) as usize;
    let detail_x = divider_x.saturating_add(2);
    let detail_width = buffer.width().saturating_sub(detail_x + PADDING_X) as usize;

    for y in BODY_TOP..buffer.height().saturating_sub(1) {
        buffer.put(divider_x, y, Cell::new('|', Color::DarkGrey, Color::Reset));
    }

    match state.book_page {
        BookPage::Mob => {
            draw_mob_entries(buffer, BODY_TOP, left_width, list_height, state);
            draw_detail_lines(
                buffer,
                detail_x,
                BODY_TOP,
                detail_width,
                mob_detail_lines(state),
            );
        }
        BookPage::Item => {
            draw_item_entries(buffer, BODY_TOP, left_width, list_height, state);
            draw_detail_lines(
                buffer,
                detail_x,
                BODY_TOP,
                detail_width,
                item_detail_lines(state),
            );
        }
    }
}

fn draw_mob_entries(
    buffer: &mut Buffer,
    top: u16,
    width: u16,
    height: usize,
    state: &RunState,
) {
    draw_entries(
        buffer,
        top,
        width,
        height,
        state.book_mob_cursor,
        codex::mob_templates().iter().map(|template| {
            (
                template.glyph,
                template.fg,
                template.name,
                state.codex.discovered_mobs.contains(template.name),
            )
        }),
    );
}

fn draw_item_entries(
    buffer: &mut Buffer,
    top: u16,
    width: u16,
    height: usize,
    state: &RunState,
) {
    draw_entries(
        buffer,
        top,
        width,
        height,
        state.book_item_cursor,
        codex::item_templates().iter().map(|template| {
            (
                template.glyph,
                template.fg,
                template.name,
                state.codex.discovered_items.contains(template.name),
            )
        }),
    );
}

fn draw_entries<I>(
    buffer: &mut Buffer,
    top: u16,
    width: u16,
    height: usize,
    cursor: usize,
    entries: I,
) where
    I: Iterator<Item = (char, Color, &'static str, bool)>,
{
    let entries: Vec<(char, Color, &'static str, bool)> = entries.collect();
    if height == 0 || entries.is_empty() {
        return;
    }

    let start = cursor.saturating_add(1).saturating_sub(height);
    for (row, (glyph, glyph_color, name, discovered)) in
        entries.into_iter().skip(start).take(height).enumerate()
    {
        let idx = start + row;
        let label = if discovered { name } else { "???" };
        let prefix = if idx == cursor { ">" } else { " " };
        let text_x = PADDING_X + 4;
        let text_width = width.saturating_sub(4) as usize;
        let text = trim_to_width(&format!("{prefix} {label}"), text_width);
        let text_color = if idx == cursor {
            Color::Cyan
        } else if discovered {
            Color::White
        } else {
            Color::DarkGrey
        };
        let glyph_to_draw = if discovered { glyph } else { '?' };
        let glyph_fg = if discovered { glyph_color } else { Color::DarkGrey };
        buffer.put_str(PADDING_X, top + row as u16, &format!(" {glyph_to_draw} "), glyph_fg, Color::Reset);
        buffer.put_str(text_x, top + row as u16, &text, text_color, Color::Reset);
    }
}

fn draw_detail_lines(
    buffer: &mut Buffer,
    x: u16,
    top: u16,
    width: usize,
    lines: Vec<String>,
) {
    if width == 0 {
        return;
    }

    let mut y = top;
    for line in lines {
        for wrapped in wrap_text(&line, width) {
            if y >= buffer.height().saturating_sub(1) {
                return;
            }
            buffer.put_str(x, y, &trim_to_width(&wrapped, width), Color::White, Color::Reset);
            y = y.saturating_add(1);
        }
    }
}

fn mob_detail_lines(state: &RunState) -> Vec<String> {
    let template = &codex::mob_templates()[state.book_mob_cursor];
    if !state.codex.discovered_mobs.contains(template.name) {
        return vec!["???".to_string()];
    }

    vec![
        format!("{} {}", template.glyph, template.name),
        String::new(),
        format!("HP: {}", template.max_hp),
        format!("ATK: {}", template.attack),
        format!("DEF: {}", template.defense),
        format!("Move: {}", template.move_tiles),
        format!("Abilities: {}", codex::describe_mob_abilities(template)),
        format!("Appears from floor {}", template.min_depth),
    ]
}

fn item_detail_lines(state: &RunState) -> Vec<String> {
    let template = &codex::item_templates()[state.book_item_cursor];
    if !state.codex.discovered_items.contains(template.name) {
        return vec!["???".to_string()];
    }

    vec![
        format!("{} {}", template.glyph, template.name),
        String::new(),
        format!("Function: {}", codex::describe_item_function(template)),
    ]
}

fn draw_footer(buffer: &mut Buffer) {
    let y = buffer.height().saturating_sub(1);
    let help = "left/right tab   up/down select   b/esc close";
    buffer.put_str(
        PADDING_X,
        y,
        &trim_to_width(help, buffer.width() as usize),
        Color::DarkGrey,
        Color::Reset,
    );
}

fn format_tab(label: &str, selected: bool) -> String {
    if selected {
        format!("[{label}]")
    } else {
        format!(" {label} ")
    }
}

fn trim_to_width(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    if width == 0 {
        return Vec::new();
    }

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let next_len = current.chars().count() + usize::from(!current.is_empty()) + word.chars().count();
        if next_len > width && !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else if current.is_empty() {
            current.push_str(word);
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(text.to_string());
    }

    lines
}
