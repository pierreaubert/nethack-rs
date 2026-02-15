//! Worm segment management (worm.c)
//!
//! Long worms are special monsters that occupy multiple tiles.
//! Each segment has its own position and can be attacked independently.

use crate::monster::MonsterId;

/// Maximum worm length (matches C: WORMNO 32 segments)
pub const MAX_WORM_LENGTH: usize = 32;

/// A single worm segment
#[derive(Debug, Clone, Copy)]
pub struct WormSegment {
    pub x: i8,
    pub y: i8,
}

/// Worm tail data — tracks all segments of a long worm.
#[derive(Debug, Clone)]
pub struct WormTail {
    /// Monster this tail belongs to
    pub monster_id: MonsterId,
    /// Segments from head-adjacent to tip (head itself is the Monster position)
    pub segments: Vec<WormSegment>,
}

impl WormTail {
    pub fn new(monster_id: MonsterId) -> Self {
        Self {
            monster_id,
            segments: Vec::new(),
        }
    }

    /// Get the tail tip position
    pub fn tail_tip(&self) -> Option<(i8, i8)> {
        self.segments.last().map(|s| (s.x, s.y))
    }

    /// Get worm length (number of segments including head)
    pub fn length(&self) -> usize {
        self.segments.len() + 1 // +1 for head
    }
}

/// Initialize a worm's tail when first created (initworm from worm.c:44).
///
/// New worms start with a short tail behind them.
pub fn initworm(monster_id: MonsterId, head_x: i8, head_y: i8) -> WormTail {
    let mut tail = WormTail::new(monster_id);
    // Start with one segment behind the head
    tail.segments.push(WormSegment {
        x: head_x - 1,
        y: head_y,
    });
    tail
}

/// Grow the worm by one segment (worm_move from worm.c:100).
///
/// When the worm moves, the head moves to (new_x, new_y), old head becomes
/// first segment, and tail tip is removed unless growing.
///
/// Returns the old tail tip position (for clearing display).
pub fn worm_move(
    tail: &mut WormTail,
    old_head_x: i8,
    old_head_y: i8,
    growing: bool,
) -> Option<(i8, i8)> {
    // Add old head position as new first segment
    tail.segments.insert(0, WormSegment {
        x: old_head_x,
        y: old_head_y,
    });

    if growing && tail.segments.len() < MAX_WORM_LENGTH {
        // Growing — keep all segments
        None
    } else {
        // Not growing or at max length — remove tail tip
        tail.segments.pop().map(|s| (s.x, s.y))
    }
}

/// Check if any worm segment is at (x, y) (worm_at from worm.c).
pub fn worm_at(tail: &WormTail, x: i8, y: i8) -> bool {
    tail.segments.iter().any(|s| s.x == x && s.y == y)
}

/// Cut the worm in half at a specific segment (cutworm from worm.c:285).
///
/// When a segment is hit, the worm splits. The front half stays with the
/// original monster; the back half either dies or becomes a new worm.
///
/// Returns the segments that were cut off (back half).
pub fn cutworm(tail: &mut WormTail, cut_x: i8, cut_y: i8) -> Vec<WormSegment> {
    if let Some(idx) = tail.segments.iter().position(|s| s.x == cut_x && s.y == cut_y) {
        let cut_off = tail.segments.split_off(idx);
        cut_off
    } else {
        Vec::new()
    }
}

/// Get all positions occupied by a worm (for collision detection).
pub fn worm_positions(tail: &WormTail, head_x: i8, head_y: i8) -> Vec<(i8, i8)> {
    let mut positions = vec![(head_x, head_y)];
    for seg in &tail.segments {
        positions.push((seg.x, seg.y));
    }
    positions
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initworm() {
        let tail = initworm(MonsterId(1), 10, 10);
        assert_eq!(tail.length(), 2); // head + 1 segment
        assert_eq!(tail.tail_tip(), Some((9, 10)));
    }

    #[test]
    fn test_worm_move_no_grow() {
        let mut tail = initworm(MonsterId(1), 10, 10);
        let old_tip = worm_move(&mut tail, 10, 10, false);
        assert!(old_tip.is_some());
        assert_eq!(tail.length(), 2); // Same length
    }

    #[test]
    fn test_worm_move_grow() {
        let mut tail = initworm(MonsterId(1), 10, 10);
        let old_tip = worm_move(&mut tail, 10, 10, true);
        assert!(old_tip.is_none()); // No segment removed
        assert_eq!(tail.length(), 3); // Grew by 1
    }

    #[test]
    fn test_worm_at() {
        let tail = initworm(MonsterId(1), 10, 10);
        assert!(worm_at(&tail, 9, 10)); // First segment
        assert!(!worm_at(&tail, 10, 10)); // Head is not in segments
        assert!(!worm_at(&tail, 11, 10)); // Not occupied
    }

    #[test]
    fn test_cutworm() {
        let mut tail = WormTail::new(MonsterId(1));
        tail.segments.push(WormSegment { x: 9, y: 10 });
        tail.segments.push(WormSegment { x: 8, y: 10 });
        tail.segments.push(WormSegment { x: 7, y: 10 });
        tail.segments.push(WormSegment { x: 6, y: 10 });

        let cut_off = cutworm(&mut tail, 8, 10);
        assert_eq!(tail.segments.len(), 1); // Only first segment remains
        assert_eq!(cut_off.len(), 3); // 3 segments cut off
    }

    #[test]
    fn test_worm_positions() {
        let tail = initworm(MonsterId(1), 10, 10);
        let positions = worm_positions(&tail, 10, 10);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], (10, 10)); // head
        assert_eq!(positions[1], (9, 10)); // segment
    }

    #[test]
    fn test_worm_max_length() {
        let mut tail = WormTail::new(MonsterId(1));
        for i in 0..MAX_WORM_LENGTH {
            tail.segments.push(WormSegment { x: i as i8, y: 0 });
        }
        // Growing at max length should still remove tail
        let old_tip = worm_move(&mut tail, 50, 0, true);
        assert!(old_tip.is_some());
        assert_eq!(tail.segments.len(), MAX_WORM_LENGTH);
    }
}
