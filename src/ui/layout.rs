//! Shared layout helpers for centered modal panels (Status, Help, Inventory
//! summaries). Each panel is a header/body/footer block; the block itself is
//! centered on the screen, but lines inside are left-aligned together so a
//! ragged row (long stat breakdown, long key chord) does not push neighbours
//! out of column.

use crossterm::style::Color;

use super::Buffer;

pub struct PanelBlock<'a> {
    pub title: &'a str,
    pub body: &'a [String],
    pub footer: &'a str,
    pub title_color: Color,
    pub body_color: Color,
    pub footer_color: Color,
}

impl<'a> PanelBlock<'a> {
    pub fn new(title: &'a str, body: &'a [String], footer: &'a str) -> Self {
        Self {
            title,
            body,
            footer,
            title_color: Color::Yellow,
            body_color: Color::White,
            footer_color: Color::DarkGrey,
        }
    }
}

pub fn draw_panel(buffer: &mut Buffer, panel: PanelBlock<'_>) {
    let title_w = panel.title.chars().count();
    let footer_w = panel.footer.chars().count();
    let body_w = panel
        .body
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let block_w = title_w.max(footer_w).max(body_w);

    // Vertical layout: title, blank, body lines..., blank, footer.
    let height = panel.body.len() + 4;
    let total_h = buffer.height() as usize;
    let total_w = buffer.width() as usize;
    let block_x = total_w.saturating_sub(block_w) / 2;
    let mut y = total_h.saturating_sub(height) / 2;

    // Title centered within block.
    let title_offset = block_w.saturating_sub(title_w) / 2;
    buffer.put_str(
        (block_x + title_offset) as u16,
        y as u16,
        panel.title,
        panel.title_color,
        Color::Reset,
    );
    y += 2;

    for line in panel.body {
        buffer.put_str(block_x as u16, y as u16, line, panel.body_color, Color::Reset);
        y += 1;
    }
    y += 1;

    let footer_offset = block_w.saturating_sub(footer_w) / 2;
    buffer.put_str(
        (block_x + footer_offset) as u16,
        y as u16,
        panel.footer,
        panel.footer_color,
        Color::Reset,
    );
}
