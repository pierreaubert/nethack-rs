//! Rectangle system for room placement (rect.c)
//!
//! Tracks available space for room placement using a list of free rectangles.
//! When a room is placed, the containing rectangle is split into smaller
//! rectangles representing the remaining free space.

use crate::rng::GameRng;

/// Maximum number of rectangles to track
pub const MAXRECT: usize = 50;

/// Minimum horizontal separation between rooms
pub const XLIM: u8 = 4;

/// Minimum vertical separation between rooms
pub const YLIM: u8 = 3;

/// A rectangle representing free space on the level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NhRect {
    /// Left x coordinate
    pub lx: u8,
    /// Top y coordinate
    pub ly: u8,
    /// Right x coordinate
    pub hx: u8,
    /// Bottom y coordinate
    pub hy: u8,
}

impl NhRect {
    /// Create a new rectangle
    pub fn new(lx: u8, ly: u8, hx: u8, hy: u8) -> Self {
        Self { lx, ly, hx, hy }
    }

    /// Get the width of the rectangle
    pub fn width(&self) -> u8 {
        if self.hx >= self.lx {
            self.hx - self.lx + 1
        } else {
            0
        }
    }

    /// Get the height of the rectangle
    pub fn height(&self) -> u8 {
        if self.hy >= self.ly {
            self.hy - self.ly + 1
        } else {
            0
        }
    }

    /// Check if this rectangle contains another
    pub fn contains(&self, other: &NhRect) -> bool {
        self.lx <= other.lx && self.hx >= other.hx && self.ly <= other.ly && self.hy >= other.hy
    }

    /// Check if this rectangle intersects another
    pub fn intersects(&self, other: &NhRect) -> bool {
        !(self.hx < other.lx || self.lx > other.hx || self.hy < other.ly || self.ly > other.hy)
    }

    /// Calculate the intersection of two rectangles
    pub fn intersection(&self, other: &NhRect) -> Option<NhRect> {
        if !self.intersects(other) {
            return None;
        }

        Some(NhRect {
            lx: self.lx.max(other.lx),
            ly: self.ly.max(other.ly),
            hx: self.hx.min(other.hx),
            hy: self.hy.min(other.hy),
        })
    }

    /// Check if the rectangle is valid (has positive area)
    pub fn is_valid(&self) -> bool {
        self.hx >= self.lx && self.hy >= self.ly
    }

    /// Check if rectangle is large enough for a room
    /// Minimum size is (2*XLIM + 1 + 4) × (2*YLIM + 1 + 4) for margins
    pub fn is_room_size(&self) -> bool {
        let min_w = 2 * XLIM + 5; // Room needs margins
        let min_h = 2 * YLIM + 5;
        self.width() >= min_w && self.height() >= min_h
    }
}

/// Manages free rectangles for room placement
#[derive(Debug, Clone)]
pub struct RectManager {
    /// List of free rectangles
    rects: Vec<NhRect>,
}

impl RectManager {
    /// Create a new rectangle manager for a level
    pub fn new(width: u8, height: u8) -> Self {
        let mut mgr = Self {
            rects: Vec::with_capacity(MAXRECT),
        };
        mgr.init(width, height);
        mgr
    }

    /// Initialize with a single rectangle covering the entire level
    /// with margins for borders
    pub fn init(&mut self, width: u8, height: u8) {
        self.rects.clear();
        // Leave margin for level borders
        let rect = NhRect::new(XLIM, YLIM, width.saturating_sub(XLIM + 1), height.saturating_sub(YLIM + 1));
        if rect.is_valid() {
            self.rects.push(rect);
        }
    }

    /// Get a random free rectangle that's large enough for a room
    pub fn rnd_rect(&self, rng: &mut GameRng) -> Option<NhRect> {
        // Find all rectangles large enough for a room
        let valid: Vec<_> = self.rects.iter().filter(|r| r.is_room_size()).copied().collect();

        if valid.is_empty() {
            return None;
        }

        // Return a random one
        let idx = rng.rn2(valid.len() as u32) as usize;
        Some(valid[idx])
    }

    /// Find a free rectangle that can contain the given rectangle
    pub fn get_rect(&self, target: &NhRect) -> Option<usize> {
        for (i, rect) in self.rects.iter().enumerate() {
            if rect.contains(target) {
                return Some(i);
            }
        }
        None
    }

    /// Add a rectangle to the free list
    pub fn add_rect(&mut self, r: NhRect) {
        if self.rects.len() < MAXRECT && r.is_valid() {
            self.rects.push(r);
        }
    }

    /// Remove a rectangle at the given index
    pub fn remove_rect(&mut self, idx: usize) {
        if idx < self.rects.len() {
            self.rects.swap_remove(idx);
        }
    }

    /// Split rectangles when a room is placed
    ///
    /// When a room is placed inside a rectangle, that rectangle is removed
    /// and up to 4 new rectangles are created for the remaining space
    /// (above, below, left, right of the room).
    pub fn split_rects(&mut self, room: &NhRect) {
        // Find all rectangles that intersect with the room
        let mut to_remove = Vec::new();
        let mut to_add = Vec::new();

        for (i, rect) in self.rects.iter().enumerate() {
            if rect.intersects(room) {
                to_remove.push(i);

                // Calculate remaining rectangles
                // Left strip
                if rect.lx < room.lx {
                    let left = NhRect::new(rect.lx, rect.ly, room.lx.saturating_sub(1), rect.hy);
                    if left.is_valid() && left.is_room_size() {
                        to_add.push(left);
                    }
                }

                // Right strip
                if rect.hx > room.hx {
                    let right = NhRect::new(room.hx + 1, rect.ly, rect.hx, rect.hy);
                    if right.is_valid() && right.is_room_size() {
                        to_add.push(right);
                    }
                }

                // Top strip (only the part not covered by left/right)
                if rect.ly < room.ly {
                    let top_lx = rect.lx.max(room.lx);
                    let top_hx = rect.hx.min(room.hx);
                    let top = NhRect::new(top_lx, rect.ly, top_hx, room.ly.saturating_sub(1));
                    if top.is_valid() && top.is_room_size() {
                        to_add.push(top);
                    }
                }

                // Bottom strip (only the part not covered by left/right)
                if rect.hy > room.hy {
                    let bot_lx = rect.lx.max(room.lx);
                    let bot_hx = rect.hx.min(room.hx);
                    let bottom = NhRect::new(bot_lx, room.hy + 1, bot_hx, rect.hy);
                    if bottom.is_valid() && bottom.is_room_size() {
                        to_add.push(bottom);
                    }
                }
            }
        }

        // Remove intersecting rectangles (in reverse order to preserve indices)
        to_remove.sort_unstable();
        for i in to_remove.into_iter().rev() {
            self.remove_rect(i);
        }

        // Add new rectangles
        for r in to_add {
            self.add_rect(r);
        }
    }

    /// Get the number of free rectangles
    pub fn count(&self) -> usize {
        self.rects.len()
    }

    /// Get the number of rectangles large enough for rooms
    pub fn room_rect_count(&self) -> usize {
        self.rects.iter().filter(|r| r.is_room_size()).count()
    }

    /// Check if there's space for more rooms
    pub fn has_space(&self) -> bool {
        self.room_rect_count() > 0
    }

    /// Get all free rectangles (for debugging/testing)
    pub fn rects(&self) -> &[NhRect] {
        &self.rects
    }

    /// Pick a random position within a free rectangle for a room of given size
    pub fn pick_room_position(
        &self,
        width: u8,
        height: u8,
        rng: &mut GameRng,
    ) -> Option<(NhRect, u8, u8)> {
        // Find rectangles that can fit this room size
        let margin = 2; // Leave space for walls
        let needed_w = width + margin * 2;
        let needed_h = height + margin * 2;

        let valid: Vec<_> = self
            .rects
            .iter()
            .filter(|r| r.width() >= needed_w && r.height() >= needed_h)
            .copied()
            .collect();

        if valid.is_empty() {
            return None;
        }

        // Pick a random valid rectangle
        let rect = valid[rng.rn2(valid.len() as u32) as usize];

        // Pick a random position within the rectangle
        let max_x = rect.hx.saturating_sub(width + margin);
        let max_y = rect.hy.saturating_sub(height + margin);

        if max_x < rect.lx + margin || max_y < rect.ly + margin {
            return None;
        }

        let x = rect.lx + margin + rng.rn2((max_x - rect.lx - margin + 1) as u32) as u8;
        let y = rect.ly + margin + rng.rn2((max_y - rect.ly - margin + 1) as u32) as u8;

        Some((rect, x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_dimensions() {
        let r = NhRect::new(10, 20, 15, 25);
        assert_eq!(r.width(), 6);
        assert_eq!(r.height(), 6);
    }

    #[test]
    fn test_rect_contains() {
        let outer = NhRect::new(0, 0, 20, 20);
        let inner = NhRect::new(5, 5, 10, 10);
        let outside = NhRect::new(25, 25, 30, 30);

        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
        assert!(!outer.contains(&outside));
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = NhRect::new(0, 0, 10, 10);
        let r2 = NhRect::new(5, 5, 15, 15);
        let r3 = NhRect::new(20, 20, 30, 30);

        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn test_rect_intersection() {
        let r1 = NhRect::new(0, 0, 10, 10);
        let r2 = NhRect::new(5, 5, 15, 15);

        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection, NhRect::new(5, 5, 10, 10));
    }

    #[test]
    fn test_rect_manager_init() {
        let mgr = RectManager::new(80, 21);
        assert_eq!(mgr.count(), 1);

        let rect = mgr.rects()[0];
        assert_eq!(rect.lx, XLIM);
        assert_eq!(rect.ly, YLIM);
    }

    #[test]
    fn test_rect_manager_split() {
        let mut mgr = RectManager::new(80, 21);
        let _initial_count = mgr.count();

        // Place a room in the middle
        let room = NhRect::new(30, 8, 40, 12);
        mgr.split_rects(&room);

        // Should have split into multiple rectangles
        // (or fewer if strips are too small)
        println!("After split: {} rectangles", mgr.count());
        for r in mgr.rects() {
            println!("  {:?} ({}x{})", r, r.width(), r.height());
        }

        // The room area should not be in any free rectangle
        for r in mgr.rects() {
            assert!(!r.contains(&room), "Room should not be in free space");
        }
    }

    #[test]
    fn test_rect_manager_rnd_rect() {
        let mgr = RectManager::new(80, 21);
        let mut rng = GameRng::new(42);

        let rect = mgr.rnd_rect(&mut rng);
        assert!(rect.is_some());
    }

    #[test]
    fn test_pick_room_position() {
        let mgr = RectManager::new(80, 21);
        let mut rng = GameRng::new(42);

        let result = mgr.pick_room_position(5, 4, &mut rng);
        assert!(result.is_some());

        let (rect, x, y) = result.unwrap();
        println!("Picked position ({}, {}) in rect {:?}", x, y, rect);

        // Position should be within bounds
        assert!(x >= XLIM);
        assert!(y >= YLIM);
    }

    #[test]
    fn test_multiple_rooms() {
        let mut mgr = RectManager::new(80, 21);
        let mut rng = GameRng::new(42);
        let mut rooms_placed = 0;

        for _ in 0..10 {
            if let Some((_, x, y)) = mgr.pick_room_position(5, 4, &mut rng) {
                // Create room rectangle with walls
                let room = NhRect::new(x.saturating_sub(1), y.saturating_sub(1), x + 5, y + 4);
                mgr.split_rects(&room);
                rooms_placed += 1;
                println!("Room {} at ({}, {}), {} rects remaining", rooms_placed, x, y, mgr.count());
            } else {
                println!("No space for room {}", rooms_placed + 1);
                break;
            }
        }

        assert!(rooms_placed >= 3, "Should place at least 3 rooms");
    }
}
