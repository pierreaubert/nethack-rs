//! Rectangle system for room placement (rect.c)
//!
//! Tracks available space for room placement using a list of free rectangles.
//! When a room is placed, the containing rectangle is split into smaller
//! rectangles representing the remaining free space.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::rng::GameRng;
use super::room::Room;

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
    /// Matches NetHack's is_room_size() exactly.
    pub fn is_room_size(&self) -> bool {
        // C: return (r->hx - r->lx >= XLIM + 2 && r->hy - r->ly >= YLIM + 2);
        (self.hx as i16 - self.lx as i16) >= (XLIM + 2) as i16
            && (self.hy as i16 - self.ly as i16) >= (YLIM + 2) as i16
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
    pub fn init(&mut self, width: u8, height: u8) {
        self.rects.clear();
        let rect = NhRect::new(0, 0, width.saturating_sub(1), height.saturating_sub(1));
        if rect.is_valid() {
            self.rects.push(rect);
        }
    }

    /// Get a random free rectangle that's large enough for a room
    /// Matches NetHack's rnd_rect() exactly.
    pub fn rnd_rect(&self, rng: &mut GameRng) -> Option<NhRect> {
        if self.rects.is_empty() {
            return None;
        }

        let idx = rng.rn2(self.rects.len() as u32) as usize;
        let rect = self.rects[idx];

        if rect.is_room_size() {
            Some(rect)
        } else {
            None
        }
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
            // Check that this NhRect is not included in another one
            if self.get_rect(&r).is_none() {
                self.rects.push(r);
            }
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

    /// Port of NetHack's create_room() total random logic
    pub fn create_room_random(&mut self, rng: &mut GameRng) -> Option<Room> {
        let mut trycnt = 0;
        let xlim = XLIM;
        let ylim = YLIM;

        // Lighting RNG calls to match NetHack (assuming level 1)
        // In C, these are called BEFORE the retry loop starts
        let _ = rng.rn2(2);
        let _ = rng.rn2(77);

        while trycnt < 100 {
            trycnt += 1;
            let r1 = self.rnd_rect(rng)?; // Pick a random rectangle
            
            let hx = r1.hx;
            let hy = r1.hy;
            let lx = r1.lx;
            let ly = r1.ly;

            let dx = 2 + rng.rn2(if hx - lx > 28 { 12 } else { 8 }) as u8;
            let mut dy = 2 + rng.rn2(4) as u8;
            if dx as u16 * dy as u16 > 50 {
                dy = 50 / dx;
            }

            let xborder = if lx > 0 && hx < crate::COLNO as u8 - 1 { 2 * xlim } else { xlim + 1 };
            let yborder = if ly > 0 && hy < crate::ROWNO as u8 - 1 { 2 * ylim } else { ylim + 1 };

            if hx - lx < dx + 3 + xborder || hy - ly < dy + 3 + yborder {
                continue;
            }

            let xabs = lx + (if lx > 0 { xlim } else { 3 })
                + rng.rn2((hx - (if lx > 0 { lx } else { 3 }) - dx - xborder + 1) as u32) as u8;
            let yabs = ly + (if ly > 0 { ylim } else { 2 })
                + rng.rn2((hy - (if ly > 0 { ly } else { 2 }) - dy - yborder + 1) as u32) as u8;

            let wtmp = dx + 1;
            let htmp = dy + 1;

            let r2 = NhRect::new(xabs.saturating_sub(1), yabs.saturating_sub(1), xabs + wtmp, yabs + htmp);
            
            // split_rects in C uses r1 (the original rect) and r2 (the room rect)
            self.split_rects_from_original(r1, &r2);

            return Some(Room::new(xabs as usize, yabs as usize, wtmp as usize, htmp as usize));
        }
        None
    }

    fn split_rects_from_original(&mut self, r1: NhRect, r2: &NhRect) {
        // Find index of r1
        if let Some(idx) = get_rect_ind(&self.rects, &r1) {
            // C logic for split_rects(r1, r2):
            // 1. remove r1
            self.rects.swap_remove(idx);
            
            // 2. recursively split other rects that intersect r2
            let mut i = self.rects.len();
            while i > 0 {
                i -= 1;
                if self.rects[i].intersects(r2) {
                    let intersecting = self.rects.swap_remove(i);
                    if let Some(intersection) = intersecting.intersection(r2) {
                        self.split_rects_from_original(intersecting, &intersection);
                    }
                }
            }

            // 3. add new rects around r2 within r1 (old_r)
            // C logic:
            /*
            if (r2->ly - old_r.ly - 1 > (old_r.hy < ROWNO - 1 ? 2 * YLIM : YLIM + 1) + 4) {
                r = old_r; r.hy = r2->ly - 2; add_rect(&r);
            }
            */
            // I'll match the EXACT C conditions
            if r2.ly as i16 - r1.ly as i16 - 1 > (if r1.hy < crate::ROWNO as u8 - 1 { 2 * YLIM } else { YLIM + 1 }) as i16 + 4 {
                let mut r = r1;
                r.hy = r2.ly.saturating_sub(2);
                self.add_rect(r);
            }
            if r2.lx as i16 - r1.lx as i16 - 1 > (if r1.hx < crate::COLNO as u8 - 1 { 2 * XLIM } else { XLIM + 1 }) as i16 + 4 {
                let mut r = r1;
                r.hx = r2.lx.saturating_sub(2);
                self.add_rect(r);
            }
            if r1.hy as i16 - r2.hy as i16 - 1 > (if r1.ly > 0 { 2 * YLIM } else { YLIM + 1 }) as i16 + 4 {
                let mut r = r1;
                r.ly = r2.hy.saturating_add(2);
                self.add_rect(r);
            }
            if r1.hx as i16 - r2.hx as i16 - 1 > (if r1.lx > 0 { 2 * XLIM } else { XLIM + 1 }) as i16 + 4 {
                let mut r = r1;
                r.lx = r2.hx.saturating_add(2);
                self.add_rect(r);
            }
        }
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

/// Find the index of a rectangle in a list by exact match
/// Matches C's get_rect_ind()
///
/// Searches through a list of rectangles and returns the index of the first
/// rectangle that exactly matches the given rectangle's coordinates.
///
/// # Arguments
/// * `rect_list` - Vector of rectangles to search
/// * `target` - The rectangle to find
///
/// # Returns
/// Some(index) if found, None if not found
pub fn get_rect_ind(rect_list: &[NhRect], target: &NhRect) -> Option<usize> {
    for (i, rect) in rect_list.iter().enumerate() {
        if rect.lx == target.lx
            && rect.ly == target.ly
            && rect.hx == target.hx
            && rect.hy == target.hy
        {
            return Some(i);
        }
    }
    None
}

/// Check if a point is inside a rectangle
/// Matches C's inside_rect()
///
/// Simple inclusion test: a point (x, y) is inside a rectangle if:
/// x >= lx && x <= hx && y >= ly && y <= hy
///
/// # Arguments
/// * `rect` - The rectangle to check
/// * `x` - X coordinate of the point
/// * `y` - Y coordinate of the point
///
/// # Returns
/// true if the point is inside the rectangle (inclusive), false otherwise
pub fn inside_rect(rect: &NhRect, x: u8, y: u8) -> bool {
    x >= rect.lx && x <= rect.hx && y >= rect.ly && y <= rect.hy
}

/// Add a rectangle to a list and update bounding box
/// Matches C's add_rect_to_reg()
///
/// This adds a new rectangle to a collection and updates a bounding box
/// to encompass all rectangles. The bounding box expands to include the
/// new rectangle's bounds.
///
/// # Arguments
/// * `rect_list` - Mutable vector of rectangles to add to
/// * `bounding_box` - Mutable bounding box to update
/// * `new_rect` - The rectangle to add
///
/// # Returns
/// true if added successfully, false if list is full (>= MAXRECT)
pub fn add_rect_to_reg(
    rect_list: &mut Vec<NhRect>,
    bounding_box: &mut NhRect,
    new_rect: &NhRect,
) -> bool {
    // Check if we have space
    if rect_list.len() >= MAXRECT {
        return false;
    }

    // Add the rectangle
    rect_list.push(*new_rect);

    // Update bounding box
    if bounding_box.lx > new_rect.lx {
        bounding_box.lx = new_rect.lx;
    }
    if bounding_box.ly > new_rect.ly {
        bounding_box.ly = new_rect.ly;
    }
    if bounding_box.hx < new_rect.hx {
        bounding_box.hx = new_rect.hx;
    }
    if bounding_box.hy < new_rect.hy {
        bounding_box.hy = new_rect.hy;
    }

    true
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
                println!(
                    "Room {} at ({}, {}), {} rects remaining",
                    rooms_placed,
                    x,
                    y,
                    mgr.count()
                );
            } else {
                println!("No space for room {}", rooms_placed + 1);
                break;
            }
        }

        assert!(rooms_placed >= 3, "Should place at least 3 rooms");
    }

    #[test]
    fn test_get_rect_ind() {
        let mut rects = vec![
            NhRect::new(0, 0, 10, 10),
            NhRect::new(20, 20, 30, 30),
            NhRect::new(5, 5, 15, 15),
        ];

        let target = NhRect::new(20, 20, 30, 30);
        assert_eq!(get_rect_ind(&rects, &target), Some(1));

        let not_found = NhRect::new(50, 50, 60, 60);
        assert_eq!(get_rect_ind(&rects, &not_found), None);

        let empty: Vec<NhRect> = vec![];
        assert_eq!(get_rect_ind(&empty, &target), None);
    }

    #[test]
    fn test_inside_rect() {
        let rect = NhRect::new(10, 10, 20, 20);

        // Points inside (inclusive)
        assert!(inside_rect(&rect, 10, 10)); // Top-left corner
        assert!(inside_rect(&rect, 15, 15)); // Middle
        assert!(inside_rect(&rect, 20, 20)); // Bottom-right corner
        assert!(inside_rect(&rect, 10, 20)); // Bottom-left corner

        // Points outside
        assert!(!inside_rect(&rect, 9, 15)); // Just to the left
        assert!(!inside_rect(&rect, 21, 15)); // Just to the right
        assert!(!inside_rect(&rect, 15, 9)); // Just above
        assert!(!inside_rect(&rect, 15, 21)); // Just below
        assert!(!inside_rect(&rect, 0, 0)); // Far outside
    }

    #[test]
    fn test_inside_rect_edge_cases() {
        // Single-point rectangle
        let point_rect = NhRect::new(5, 5, 5, 5);
        assert!(inside_rect(&point_rect, 5, 5));
        assert!(!inside_rect(&point_rect, 4, 5));
        assert!(!inside_rect(&point_rect, 6, 5));

        // Line rectangle
        let line_rect = NhRect::new(10, 10, 10, 20);
        assert!(inside_rect(&line_rect, 10, 10));
        assert!(inside_rect(&line_rect, 10, 15));
        assert!(inside_rect(&line_rect, 10, 20));
        assert!(!inside_rect(&line_rect, 9, 10));
        assert!(!inside_rect(&line_rect, 11, 10));
    }

    #[test]
    fn test_add_rect_to_reg() {
        let mut rects = vec![NhRect::new(0, 0, 10, 10)];
        let mut bbox = NhRect::new(0, 0, 10, 10);

        // Add a rectangle that expands the bounding box
        let new_rect = NhRect::new(20, 20, 30, 30);
        let result = add_rect_to_reg(&mut rects, &mut bbox, &new_rect);

        assert!(result);
        assert_eq!(rects.len(), 2);
        assert_eq!(bbox.lx, 0); // Left unchanged
        assert_eq!(bbox.ly, 0); // Top unchanged
        assert_eq!(bbox.hx, 30); // Right expanded
        assert_eq!(bbox.hy, 30); // Bottom expanded
    }

    #[test]
    fn test_add_rect_to_reg_contraction() {
        let mut rects = vec![NhRect::new(10, 10, 20, 20)];
        let mut bbox = NhRect::new(10, 10, 20, 20);

        // Add a smaller rectangle inside
        let new_rect = NhRect::new(12, 12, 18, 18);
        let result = add_rect_to_reg(&mut rects, &mut bbox, &new_rect);

        assert!(result);
        assert_eq!(rects.len(), 2);
        // Bounding box should stay the same (doesn't contract)
        assert_eq!(bbox.lx, 10);
        assert_eq!(bbox.hx, 20);
        assert_eq!(bbox.ly, 10);
        assert_eq!(bbox.hy, 20);
    }

    #[test]
    fn test_add_rect_to_reg_at_capacity() {
        let mut rects = vec![];
        for i in 0..MAXRECT {
            rects.push(NhRect::new(i as u8, 0, i as u8 + 1, 1));
        }

        let mut bbox = NhRect::new(0, 0, MAXRECT as u8, 1);
        let new_rect = NhRect::new(100, 100, 101, 101);

        // Should fail when at capacity
        let result = add_rect_to_reg(&mut rects, &mut bbox, &new_rect);
        assert!(!result);
        assert_eq!(rects.len(), MAXRECT);
    }
}
