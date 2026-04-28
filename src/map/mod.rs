//! Tile map. Phase 4 BSP generation lives in `gen::bsp`; the test arena
//! from Phase 3 is retained for unit tests that want a deterministic open
//! room.

pub mod fov;
pub mod gen;
pub mod tile;

pub use tile::Tile;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    width: i32,
    height: i32,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0, "map dimensions must be positive");
        let len = (width as usize) * (height as usize);
        Self {
            width,
            height,
            tiles: vec![Tile::Wall; len],
        }
    }

    #[allow(dead_code)]
    pub fn width(&self) -> i32 {
        self.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if !self.in_bounds(x, y) {
            return None;
        }
        Some((y as usize) * (self.width as usize) + (x as usize))
    }

    pub fn tile(&self, x: i32, y: i32) -> Option<Tile> {
        self.idx(x, y).map(|i| self.tiles[i])
    }

    pub fn set(&mut self, x: i32, y: i32, tile: Tile) {
        if let Some(i) = self.idx(x, y) {
            self.tiles[i] = tile;
        }
    }

    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        match self.tile(x, y) {
            Some(t) => t.blocks_walk(),
            None => true,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, i32, Tile)> + '_ {
        let w = self.width;
        self.tiles
            .iter()
            .copied()
            .enumerate()
            .map(move |(i, t)| ((i as i32) % w, (i as i32) / w, t))
    }

    /// Build a hard-coded test arena: outer wall, central pillars, gap in
    /// the south wall to confirm wall behaviour.
    #[allow(dead_code)]
    pub fn test_arena(width: i32, height: i32) -> Self {
        let mut map = Self::new(width.max(8), height.max(6));
        let w = map.width;
        let h = map.height;
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                map.set(x, y, Tile::Floor);
            }
        }
        // A few interior pillars so the player has something to bump into.
        for &(px, py) in &[(w / 4, h / 2), (w / 2, h / 3), (3 * w / 4, h / 2)] {
            map.set(px, py, Tile::Wall);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_has_walls_around_perimeter() {
        let map = Map::test_arena(20, 10);
        for x in 0..map.width() {
            assert!(map.is_blocked(x, 0));
            assert!(map.is_blocked(x, map.height() - 1));
        }
        for y in 0..map.height() {
            assert!(map.is_blocked(0, y));
            assert!(map.is_blocked(map.width() - 1, y));
        }
    }

    #[test]
    fn test_arena_interior_is_walkable() {
        let map = Map::test_arena(20, 10);
        // Center of arena is unlikely to be a pillar.
        assert!(!map.is_blocked(10, 5) || !map.is_blocked(11, 5));
    }

    #[test]
    fn out_of_bounds_blocks() {
        let map = Map::test_arena(20, 10);
        assert!(map.is_blocked(-1, 5));
        assert!(map.is_blocked(20, 5));
    }
}
