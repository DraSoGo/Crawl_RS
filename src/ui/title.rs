//! Title screen + main menu.
//!
//! Renders ASCII art and a small selection list (New / Continue / Quit).
//! `Continue` is greyed out when no save file exists.

use crossterm::style::Color;

use crate::ui::Buffer;

const TITLE_LINES: &[&str] = &[
    "                          _",
    "    ___ _ __ __ ___      _| |   _ __ ___",
    "   / __| '__/ _` \\ \\ /\\ / / |  | '__/ __|",
    "  | (__| | | (_| |\\ V  V /| |  | |  \\__ \\",
    "   \\___|_|  \\__,_| \\_/\\_/ |_|  |_|  |___/",
    "",
    "        Terminal Roguelike   v0.1.0",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuChoice {
    NewGame,
    Continue,
    Quit,
}

impl MenuChoice {
    pub fn label(self) -> &'static str {
        match self {
            MenuChoice::NewGame => "[N] new game",
            MenuChoice::Continue => "[C] continue",
            MenuChoice::Quit => "[Q] quit",
        }
    }
}

pub struct MenuState {
    pub selected: usize,
    pub items: Vec<(MenuChoice, bool)>,
}

impl MenuState {
    pub fn new(save_exists: bool) -> Self {
        let items = vec![
            (MenuChoice::NewGame, true),
            (MenuChoice::Continue, save_exists),
            (MenuChoice::Quit, true),
        ];
        Self {
            selected: 0,
            items,
        }
    }

    pub fn move_up(&mut self) {
        loop {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
            if self.items[self.selected].1 {
                break;
            }
        }
    }

    pub fn move_down(&mut self) {
        loop {
            self.selected = (self.selected + 1) % self.items.len();
            if self.items[self.selected].1 {
                break;
            }
        }
    }

    pub fn current(&self) -> MenuChoice {
        self.items[self.selected].0
    }
}

pub fn draw(buffer: &mut Buffer, state: &MenuState) {
    if buffer.height() == 0 || buffer.width() == 0 {
        return;
    }
    for y in 0..buffer.height() {
        for x in 0..buffer.width() {
            buffer.put(x, y, crate::ui::Cell::BLANK);
        }
    }
    let top = (buffer.height() as usize / 2).saturating_sub(TITLE_LINES.len() / 2 + 4);
    for (i, line) in TITLE_LINES.iter().enumerate() {
        let y = (top + i) as u16;
        let text_len = line.chars().count();
        let x = (buffer.width() as usize).saturating_sub(text_len) / 2;
        let color = if i < TITLE_LINES.len() - 1 {
            Color::Yellow
        } else {
            Color::DarkGrey
        };
        buffer.put_str(x as u16, y, line, color, Color::Reset);
    }
    let menu_top = top + TITLE_LINES.len() + 2;
    for (i, (choice, enabled)) in state.items.iter().enumerate() {
        let y = (menu_top + i) as u16;
        let prefix = if i == state.selected { "> " } else { "  " };
        let text = format!("{prefix}{}", choice.label());
        let color = if !enabled {
            Color::DarkGrey
        } else if i == state.selected {
            Color::Cyan
        } else {
            Color::White
        };
        let x = (buffer.width() as usize).saturating_sub(text.len()) / 2;
        buffer.put_str(x as u16, y, &text, color, Color::Reset);
    }
    let footer = "use up/down then enter, or n / c / q";
    let fy = (menu_top + state.items.len() + 2) as u16;
    let fx = (buffer.width() as usize).saturating_sub(footer.len()) / 2;
    buffer.put_str(fx as u16, fy, footer, Color::DarkGrey, Color::Reset);
}
