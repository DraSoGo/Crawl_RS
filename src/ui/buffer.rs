//! Double-buffered terminal cell grid with diff-based flushing.
//!
//! The renderer writes into a back buffer each frame. `flush` emits ANSI only
//! for cells that differ from the front buffer, then swaps. This keeps frame
//! cost proportional to actual screen change, not screen size — important for
//! large terminals.

use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{
    cursor,
    queue,
    style::{Color, ContentStyle, Print, ResetColor, SetStyle, StyledContent},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

impl Cell {
    pub const BLANK: Self = Self {
        ch: ' ',
        fg: Color::Reset,
        bg: Color::Reset,
    };

    pub const fn new(ch: char, fg: Color, bg: Color) -> Self {
        Self { ch, fg, bg }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::BLANK
    }
}

pub struct Buffer {
    width: u16,
    height: u16,
    back: Vec<Cell>,
    front: Vec<Cell>,
    /// Front buffer never written to yet — force a full repaint on first flush.
    dirty_all: bool,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        let len = width as usize * height as usize;
        Self {
            width,
            height,
            back: vec![Cell::BLANK; len],
            front: vec![Cell::BLANK; len],
            dirty_all: true,
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if width == self.width && height == self.height {
            return;
        }
        let len = width as usize * height as usize;
        self.width = width;
        self.height = height;
        self.back = vec![Cell::BLANK; len];
        self.front = vec![Cell::BLANK; len];
        self.dirty_all = true;
    }

    pub fn clear(&mut self) {
        for c in &mut self.back {
            *c = Cell::BLANK;
        }
    }

    fn idx(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(y as usize * self.width as usize + x as usize)
    }

    pub fn put(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(i) = self.idx(x, y) {
            self.back[i] = cell;
        }
    }

    pub fn put_str(&mut self, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
        let mut cx = x;
        for ch in s.chars() {
            if cx >= self.width {
                break;
            }
            self.put(cx, y, Cell::new(ch, fg, bg));
            cx = cx.saturating_add(1);
        }
    }

    /// Emit only changed cells. Caller is responsible for flushing stdout if
    /// it wants the writes visible immediately.
    pub fn flush<W: Write>(&mut self, out: &mut W) -> Result<()> {
        let mut last_pos: Option<(u16, u16)> = None;
        let mut last_style: Option<(Color, Color)> = None;
        for y in 0..self.height {
            for x in 0..self.width {
                let i = y as usize * self.width as usize + x as usize;
                let cell = self.back[i];
                if !self.dirty_all && self.front[i] == cell {
                    continue;
                }
                if last_pos != Some((x, y)) {
                    queue!(out, cursor::MoveTo(x, y)).context("move cursor")?;
                }
                if last_style != Some((cell.fg, cell.bg)) {
                    let mut style = ContentStyle::new();
                    style.foreground_color = Some(cell.fg);
                    style.background_color = Some(cell.bg);
                    queue!(out, SetStyle(style)).context("set style")?;
                    last_style = Some((cell.fg, cell.bg));
                }
                queue!(out, Print(StyledContent::new(ContentStyle::new(), cell.ch)))
                    .context("print cell")?;
                self.front[i] = cell;
                last_pos = Some((x.saturating_add(1), y));
            }
        }
        queue!(out, ResetColor).context("reset color")?;
        self.dirty_all = false;
        Ok(())
    }
}
