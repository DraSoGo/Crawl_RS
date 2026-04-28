//! Recursive shadowcasting field-of-view.
//!
//! Standard 8-octant algorithm: for each octant we sweep rows out from the
//! origin, narrowing the visible slope range whenever an opaque tile is hit.
//! See <http://www.roguebasin.com/index.php?title=FOV_using_recursive_shadowcasting>
//! for the textbook treatment.
//!
//! `Visibility` owns two flat bitsets sized to the map. `compute` fills the
//! `visible` set fresh each call and unions into the persistent `revealed`
//! set so the renderer can dim explored-but-out-of-FOV tiles.

use crate::map::Map;

/// Octant transforms. Each row is a column of the multiplier matrix used to
/// rotate (row, col) offsets into world (dx, dy) for that octant.
const OCTANT_MULT: [[i32; 8]; 4] = [
    [1, 0, 0, -1, -1, 0, 0, 1],
    [0, 1, -1, 0, 0, -1, 1, 0],
    [0, 1, 1, 0, 0, -1, -1, 0],
    [1, 0, 0, 1, -1, 0, 0, -1],
];

#[derive(Clone, Debug)]
pub struct Visibility {
    width: i32,
    height: i32,
    visible: Vec<bool>,
    revealed: Vec<bool>,
}

impl Visibility {
    pub fn new(width: i32, height: i32) -> Self {
        let len = (width as usize) * (height as usize);
        Self {
            width,
            height,
            visible: vec![false; len],
            revealed: vec![false; len],
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

    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some((y as usize) * (self.width as usize) + (x as usize))
    }

    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        self.idx(x, y).map_or(false, |i| self.visible[i])
    }

    pub fn is_revealed(&self, x: i32, y: i32) -> bool {
        self.idx(x, y).map_or(false, |i| self.revealed[i])
    }

    fn clear_visible(&mut self) {
        for v in &mut self.visible {
            *v = false;
        }
    }

    fn mark(&mut self, x: i32, y: i32) {
        if let Some(i) = self.idx(x, y) {
            self.visible[i] = true;
            self.revealed[i] = true;
        }
    }

    /// Force every tile to "revealed". Used by the mapping scroll. Visibility
    /// (the bright cone) is not touched — players still need to move to see
    /// what is currently in view.
    pub fn reveal_all(&mut self) {
        for r in &mut self.revealed {
            *r = true;
        }
    }

    /// Direct setters used by save/load to restore exact state.
    pub fn force_revealed(&mut self, x: i32, y: i32) {
        if let Some(i) = self.idx(x, y) {
            self.revealed[i] = true;
        }
    }

    pub fn force_visible(&mut self, x: i32, y: i32) {
        if let Some(i) = self.idx(x, y) {
            self.visible[i] = true;
        }
    }

    /// Recompute visibility from `(ox, oy)` outward to `radius`.
    pub fn compute(&mut self, map: &Map, ox: i32, oy: i32, radius: i32) {
        self.clear_visible();
        if radius <= 0 {
            self.mark(ox, oy);
            return;
        }
        // Origin always visible.
        self.mark(ox, oy);
        for octant in 0..8 {
            cast_light(self, map, ox, oy, 1, 1.0, 0.0, radius, octant);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn cast_light(
    fov: &mut Visibility,
    map: &Map,
    ox: i32,
    oy: i32,
    row_start: i32,
    mut start_slope: f64,
    end_slope: f64,
    radius: i32,
    octant: usize,
) {
    if start_slope < end_slope {
        return;
    }
    let xx = OCTANT_MULT[0][octant];
    let xy = OCTANT_MULT[1][octant];
    let yx = OCTANT_MULT[2][octant];
    let yy = OCTANT_MULT[3][octant];
    let radius_sq = radius * radius;
    let mut next_start = start_slope;
    for row in row_start..=radius {
        let dy = -row;
        let mut blocked = false;
        // Iterate steep slopes first (large |dx|) to shallow (dx=0). This is
        // the canonical RogueBasin order: it allows the algorithm to enter a
        // blocked state, then exit it again when it scans past a wall back
        // into open space within the same row.
        for dx in -row..=0 {
            let l_slope = (dx as f64 - 0.5) / (dy as f64 + 0.5);
            let r_slope = (dx as f64 + 0.5) / (dy as f64 - 0.5);
            if start_slope < r_slope {
                continue;
            }
            if end_slope > l_slope {
                break;
            }
            let mx = ox + dx * xx + dy * xy;
            let my = oy + dx * yx + dy * yy;
            if dx * dx + dy * dy <= radius_sq {
                fov.mark(mx, my);
            }
            let opaque = match map.tile(mx, my) {
                Some(t) => t.blocks_sight(),
                None => true,
            };
            if blocked {
                if opaque {
                    next_start = r_slope;
                } else {
                    blocked = false;
                    start_slope = next_start;
                }
            } else if opaque && row < radius {
                blocked = true;
                cast_light(
                    fov,
                    map,
                    ox,
                    oy,
                    row + 1,
                    start_slope,
                    l_slope,
                    radius,
                    octant,
                );
                next_start = r_slope;
            }
        }
        if blocked {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Tile;

    fn open_room(w: i32, h: i32) -> Map {
        let mut m = Map::new(w, h);
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                m.set(x, y, Tile::Floor);
            }
        }
        m
    }

    #[test]
    fn origin_is_visible() {
        let map = open_room(10, 10);
        let mut fov = Visibility::new(10, 10);
        fov.compute(&map, 5, 5, 6);
        assert!(fov.is_visible(5, 5));
    }

    #[test]
    fn walls_block_line_of_sight() {
        // 9-wide room with a single wall at x=4, player at x=2.
        // Tile at x=6 should be hidden.
        let mut map = open_room(9, 5);
        map.set(4, 2, Tile::Wall);
        let mut fov = Visibility::new(9, 5);
        fov.compute(&map, 2, 2, 8);
        assert!(fov.is_visible(3, 2));
        assert!(fov.is_visible(4, 2)); // wall itself is seen
        assert!(!fov.is_visible(6, 2)); // behind wall, hidden
    }

    #[test]
    fn radius_limits_visibility() {
        let map = open_room(20, 5);
        let mut fov = Visibility::new(20, 5);
        fov.compute(&map, 2, 2, 3);
        assert!(fov.is_visible(5, 2));
        assert!(!fov.is_visible(10, 2));
    }

    #[test]
    fn revealed_persists_across_recompute() {
        let map = open_room(15, 5);
        let mut fov = Visibility::new(15, 5);
        fov.compute(&map, 2, 2, 4);
        assert!(fov.is_visible(5, 2));
        // Move far away; (5,2) leaves the visible window but stays revealed.
        fov.compute(&map, 12, 2, 2);
        assert!(!fov.is_visible(5, 2));
        assert!(fov.is_revealed(5, 2));
    }

    #[test]
    fn out_of_bounds_is_safe() {
        let map = open_room(5, 5);
        let mut fov = Visibility::new(5, 5);
        // Origin near edge — algorithm must not panic.
        fov.compute(&map, 0, 0, 10);
        assert!(fov.is_visible(0, 0));
    }
}
