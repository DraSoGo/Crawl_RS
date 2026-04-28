//! Recursive binary-space-partition dungeon generation.
//!
//! Algorithm: split the map into a tree of axis-aligned rectangles until
//! each leaf is small enough to hold a single room, carve a room inside
//! each leaf, then connect sibling rooms with L-shaped corridors. The tree
//! shape gives us guaranteed connectivity: every leaf is reachable from
//! every other via corridors that bubble up through the tree.

use rand::Rng;

use crate::map::{Map, Tile};

/// Inclusive rectangle in tile coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    #[allow(dead_code)]
    pub const fn right(&self) -> i32 {
        self.x + self.w - 1
    }

    #[allow(dead_code)]
    pub const fn bottom(&self) -> i32 {
        self.y + self.h - 1
    }

    pub fn center(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BspConfig {
    pub min_leaf: i32,
    pub max_leaf: i32,
    pub min_room: i32,
    pub max_depth: u8,
}

impl Default for BspConfig {
    fn default() -> Self {
        Self {
            min_leaf: 8,
            max_leaf: 14,
            min_room: 4,
            max_depth: 6,
        }
    }
}

#[derive(Debug)]
struct Node {
    bounds: Rect,
    room: Option<Rect>,
    children: Option<(Box<Node>, Box<Node>)>,
}

impl Node {
    fn leaf(bounds: Rect) -> Self {
        Self {
            bounds,
            room: None,
            children: None,
        }
    }

    fn split<R: Rng>(&mut self, cfg: &BspConfig, depth: u8, rng: &mut R) {
        if depth >= cfg.max_depth {
            return;
        }
        let split_h = pick_split_axis(self.bounds, rng);
        let Some((a, b)) = split_rect(self.bounds, split_h, cfg.min_leaf, rng) else {
            return;
        };
        let mut left = Box::new(Node::leaf(a));
        let mut right = Box::new(Node::leaf(b));
        left.split(cfg, depth + 1, rng);
        right.split(cfg, depth + 1, rng);
        self.children = Some((left, right));
    }

    fn carve_rooms<R: Rng>(&mut self, cfg: &BspConfig, rng: &mut R) {
        if let Some((l, r)) = &mut self.children {
            l.carve_rooms(cfg, rng);
            r.carve_rooms(cfg, rng);
            return;
        }
        // Leaf: fit a room with at least `min_room` width/height and a 1-tile
        // border so corridors can attach to its edges.
        let max_w = (self.bounds.w - 2).max(cfg.min_room);
        let max_h = (self.bounds.h - 2).max(cfg.min_room);
        if max_w < cfg.min_room || max_h < cfg.min_room {
            return;
        }
        let w = rng.gen_range(cfg.min_room..=max_w);
        let h = rng.gen_range(cfg.min_room..=max_h);
        let x = self.bounds.x + 1 + rng.gen_range(0..=(self.bounds.w - w - 2).max(0));
        let y = self.bounds.y + 1 + rng.gen_range(0..=(self.bounds.h - h - 2).max(0));
        self.room = Some(Rect::new(x, y, w, h));
    }

    /// Returns the centre of any room in this subtree (left-leaning).
    fn any_room_centre(&self) -> Option<(i32, i32)> {
        if let Some(r) = self.room {
            return Some(r.center());
        }
        if let Some((l, r)) = &self.children {
            return l.any_room_centre().or_else(|| r.any_room_centre());
        }
        None
    }

    fn collect_rooms(&self, out: &mut Vec<Rect>) {
        if let Some(r) = self.room {
            out.push(r);
        }
        if let Some((l, r)) = &self.children {
            l.collect_rooms(out);
            r.collect_rooms(out);
        }
    }

    fn carve_corridors<R: Rng>(&self, map: &mut Map, rng: &mut R) {
        if let Some((l, r)) = &self.children {
            l.carve_corridors(map, rng);
            r.carve_corridors(map, rng);
            if let (Some(a), Some(b)) =
                (l.any_room_centre(), r.any_room_centre())
            {
                connect(map, a, b, rng);
            }
        }
    }
}

fn pick_split_axis<R: Rng>(rect: Rect, rng: &mut R) -> bool {
    // Return true = horizontal split (i.e. cut along the y axis -> two
    // vertically stacked rects). Bias toward splitting the longer axis so
    // rooms stay closer to square.
    let ratio = rect.w as f32 / rect.h as f32;
    if ratio > 1.25 {
        false
    } else if ratio < 0.8 {
        true
    } else {
        rng.gen_bool(0.5)
    }
}

fn split_rect<R: Rng>(rect: Rect, horizontal: bool, min_leaf: i32, rng: &mut R) -> Option<(Rect, Rect)> {
    if horizontal {
        if rect.h < min_leaf * 2 {
            return None;
        }
        let split = rng.gen_range(min_leaf..=(rect.h - min_leaf));
        Some((
            Rect::new(rect.x, rect.y, rect.w, split),
            Rect::new(rect.x, rect.y + split, rect.w, rect.h - split),
        ))
    } else {
        if rect.w < min_leaf * 2 {
            return None;
        }
        let split = rng.gen_range(min_leaf..=(rect.w - min_leaf));
        Some((
            Rect::new(rect.x, rect.y, split, rect.h),
            Rect::new(rect.x + split, rect.y, rect.w - split, rect.h),
        ))
    }
}

fn carve_room(map: &mut Map, rect: Rect) {
    for y in rect.y..rect.y + rect.h {
        for x in rect.x..rect.x + rect.w {
            map.set(x, y, Tile::Floor);
        }
    }
}

fn connect<R: Rng>(map: &mut Map, a: (i32, i32), b: (i32, i32), rng: &mut R) {
    // L-corridor: pick which axis goes first uniformly. The other axis fills
    // in second so the path is always reachable.
    if rng.gen_bool(0.5) {
        carve_h(map, a.0, b.0, a.1);
        carve_v(map, a.1, b.1, b.0);
    } else {
        carve_v(map, a.1, b.1, a.0);
        carve_h(map, a.0, b.0, b.1);
    }
}

fn carve_h(map: &mut Map, x0: i32, x1: i32, y: i32) {
    let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
    for x in lo..=hi {
        map.set(x, y, Tile::Floor);
    }
}

fn carve_v(map: &mut Map, y0: i32, y1: i32, x: i32) {
    let (lo, hi) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
    for y in lo..=hi {
        map.set(x, y, Tile::Floor);
    }
}

/// Carved dungeon plus a chosen player start and stairs-down location.
#[derive(Debug)]
pub struct Dungeon {
    pub map: Map,
    pub start: (i32, i32),
    pub stairs_down: (i32, i32),
    pub rooms: Vec<Rect>,
}

pub fn generate<R: Rng>(width: i32, height: i32, cfg: &BspConfig, rng: &mut R) -> Dungeon {
    let width = width.max(cfg.min_leaf * 2);
    let height = height.max(cfg.min_leaf * 2);
    let mut map = Map::new(width, height);

    let mut root = Node::leaf(Rect::new(0, 0, width, height));
    root.split(cfg, 0, rng);
    root.carve_rooms(cfg, rng);

    let mut rooms = Vec::new();
    root.collect_rooms(&mut rooms);
    if rooms.is_empty() {
        // Degenerate fallback: one big room. Should not happen for sensible
        // configs but guarantees we never return an unwalkable level.
        let r = Rect::new(1, 1, width - 2, height - 2);
        rooms.push(r);
        carve_room(&mut map, r);
    } else {
        for r in &rooms {
            carve_room(&mut map, *r);
        }
    }
    root.carve_corridors(&mut map, rng);

    // Player starts in the first room; stairs go in the room farthest from it
    // (Manhattan), to push the player toward exploring.
    let start = rooms[0].center();
    let stairs_down = farthest_room(&rooms, start).center();
    map.set(start.0, start.1, Tile::UpStairs);
    map.set(stairs_down.0, stairs_down.1, Tile::DownStairs);

    Dungeon {
        map,
        start,
        stairs_down,
        rooms,
    }
}

fn farthest_room(rooms: &[Rect], from: (i32, i32)) -> Rect {
    rooms
        .iter()
        .copied()
        .max_by_key(|r| {
            let c = r.center();
            (c.0 - from.0).abs() + (c.1 - from.1).abs()
        })
        .unwrap_or(rooms[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;

    fn rng(seed: u64) -> Pcg64Mcg {
        Pcg64Mcg::seed_from_u64(seed)
    }

    fn flood_fill_floor(map: &Map, start: (i32, i32)) -> usize {
        use std::collections::VecDeque;
        let mut seen = vec![false; (map.width() * map.height()) as usize];
        let mut queue = VecDeque::new();
        queue.push_back(start);
        let idx = |x: i32, y: i32| (y * map.width() + x) as usize;
        seen[idx(start.0, start.1)] = true;
        let mut count = 0;
        while let Some((x, y)) = queue.pop_front() {
            count += 1;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = x + dx;
                let ny = y + dy;
                if !map.in_bounds(nx, ny) {
                    continue;
                }
                if map.is_blocked(nx, ny) {
                    continue;
                }
                let i = idx(nx, ny);
                if !seen[i] {
                    seen[i] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
        count
    }

    fn floor_count(map: &Map) -> usize {
        map.iter().filter(|(_, _, t)| !t.blocks_walk()).count()
    }

    #[test]
    fn dungeon_is_fully_connected() {
        let cfg = BspConfig::default();
        for seed in 0u64..20 {
            let mut r = rng(seed);
            let d = generate(60, 30, &cfg, &mut r);
            let reachable = flood_fill_floor(&d.map, d.start);
            assert_eq!(
                reachable,
                floor_count(&d.map),
                "seed {seed}: {reachable} reachable vs {} walkable",
                floor_count(&d.map)
            );
            assert!(!d.map.is_blocked(d.start.0, d.start.1));
            assert!(!d.map.is_blocked(d.stairs_down.0, d.stairs_down.1));
        }
    }

    #[test]
    fn dungeon_is_deterministic() {
        let cfg = BspConfig::default();
        let mut a = rng(42);
        let mut b = rng(42);
        let da = generate(60, 30, &cfg, &mut a);
        let db = generate(60, 30, &cfg, &mut b);
        assert_eq!(da.start, db.start);
        assert_eq!(da.stairs_down, db.stairs_down);
        let ta: Vec<Tile> = da.map.iter().map(|(_, _, t)| t).collect();
        let tb: Vec<Tile> = db.map.iter().map(|(_, _, t)| t).collect();
        assert_eq!(ta, tb);
    }

    #[test]
    fn rooms_are_within_bounds() {
        let cfg = BspConfig::default();
        let mut r = rng(7);
        let d = generate(60, 30, &cfg, &mut r);
        for room in &d.rooms {
            assert!(room.x >= 0 && room.y >= 0);
            assert!(room.right() < d.map.width());
            assert!(room.bottom() < d.map.height());
        }
    }
}
