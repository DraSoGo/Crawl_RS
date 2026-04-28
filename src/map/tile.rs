//! Tile types and their rendering / pathing properties.

use crossterm::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tile {
    Floor,
    Wall,
    DownStairs,
    UpStairs,
}

impl Tile {
    pub const fn blocks_walk(self) -> bool {
        matches!(self, Tile::Wall)
    }

    /// Whether the tile blocks line of sight. Phase 5 reads this for FOV.
    #[allow(dead_code)]
    pub const fn blocks_sight(self) -> bool {
        matches!(self, Tile::Wall)
    }

    pub const fn glyph(self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall => '#',
            Tile::DownStairs => '>',
            Tile::UpStairs => '<',
        }
    }

    pub const fn fg(self) -> Color {
        match self {
            Tile::Floor => Color::DarkGrey,
            Tile::Wall => Color::Grey,
            Tile::DownStairs | Tile::UpStairs => Color::Cyan,
        }
    }

    pub const fn bg(self) -> Color {
        Color::Reset
    }
}
