//! Rectangle system for room placement (rect.c)
//!
//! Tracks available space for room placement using a list of free rectangles.
//! When a room is placed, the containing rectangle is split into smaller
//! rectangles representing the remaining free space.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::rng::GameRng;
use super::room::Room;
use super::level::Level;

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
        (self.hx as i16 - self.lx as i16) >= (XLIM + 4) as i16
            && (self.hy as i16 - self.ly as i16) >= (YLIM + 4) as i16
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

    /// Get a random free rectangle from the list
    /// Matches NetHack's rnd_rect() exactly.
    pub fn rnd_rect(&self, rng: &mut GameRng) -> Option<NhRect> {
        if self.rects.is_empty() {
            return None;
        }

        let idx = rng.rn2(self.rects.len() as u32) as usize;
        Some(self.rects[idx])
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

    /// Port of NetHack's create_room() total random logic
    pub fn create_room_random(&mut self, level: &Level, rng: &mut GameRng, num_rooms: usize) -> Option<Room> {
        let mut trycnt = 0;
        let xlim = XLIM;
        let ylim = YLIM;

        // Lighting RNG calls to match NetHack
        // C: rlit = (rnd(1 + abs(depth(&u.uz))) < 11 && rn2(77)) ? TRUE : FALSE;
        let depth = level.dlevel.depth();
        if rng.rnd(1 + depth.abs() as u32) < 11 {
            let _ = rng.rn2(77);
        }

        while trycnt < 100 {
            trycnt += 1;
            let r1 = self.rnd_rect(rng)?; // Pick a random rectangle
            
            let hx = r1.hx;
            let hy = r1.hy;
            let lx = r1.lx;
            let ly = r1.ly;

            let mut dx = 2 + rng.rn2(if hx - lx > 28 { 12 } else { 8 }) as u8;
            let mut dy = 2 + rng.rn2(4) as u8;
            if dx as u16 * dy as u16 > 50 {
                dy = 50 / dx;
            }

            let xborder = if lx > 0 && hx < crate::COLNO as u8 - 1 { 2 * xlim } else { xlim + 1 };
            let yborder = if ly > 0 && hy < crate::ROWNO as u8 - 1 { 2 * ylim } else { ylim + 1 };

            if hx - lx < dx + 3 + xborder || hy - ly < dy + 3 + yborder {
                continue;
            }

            let mut xabs = lx + (if lx > 0 { xlim } else { 3 })
                + rng.rn2((hx - (if lx > 0 { lx } else { 3 }) - dx - xborder + 1) as u32) as u8;
            let mut yabs = ly + (if ly > 0 { ylim } else { 2 })
                + rng.rn2((hy - (if ly > 0 { ly } else { 2 }) - dy - yborder + 1) as u32) as u8;

            // Big room logic (sp_lev.c:1203-1208 in 3.6.7)
            if ly == 0 && hy >= crate::ROWNO as u8 - 1 && (num_rooms == 0 || rng.rn2(num_rooms as u32) == 0)
                && (yabs + dy > crate::ROWNO as u8 / 2) {
                yabs = 2 + rng.rn2(3) as u8;
                if num_rooms < 4 && dy > 1 {
                    dy -= 1;
                }
            }

            if !self.check_room(level, &mut xabs, &mut dx, &mut yabs, &mut dy, false, rng) {
                continue;
            }

            let wtmp = dx + 1;
            let htmp = dy + 1;

            let r2 = NhRect::new(xabs.saturating_sub(1), yabs.saturating_sub(1), xabs + wtmp, yabs + htmp);
            
            // split_rects in C uses r1 (the original rect) and r2 (the room rect)
            self.split_rects(r1, &r2);

            return Some(Room::new(xabs as usize, yabs as usize, wtmp as usize, htmp as usize));
        }
        None
    }

    /// Check if a room can be placed and potentially shrink it
    pub fn check_room(
        &self,
        level: &Level,
        lowx: &mut u8,
        ddx: &mut u8,
        lowy: &mut u8,
        ddy: &mut u8,
        vault: bool,
        rng: &mut GameRng,
    ) -> bool {
        let mut hix = *lowx + *ddx;
        let mut hiy = *lowy + *ddy;
        let xlim = XLIM + (if vault { 1 } else { 0 });
        let ylim = YLIM + (if vault { 1 } else { 0 });

        if *lowx < 3 { *lowx = 3; }
        if *lowy < 2 { *lowy = 2; }
        if hix > crate::COLNO as u8 - 3 { hix = crate::COLNO as u8 - 3; }
        if hiy > crate::ROWNO as u8 - 3 { hiy = crate::ROWNO as u8 - 3; }

        'chk: loop {
            if hix <= *lowx || hiy <= *lowy {
                return false;
            }

            for x in (*lowx as i16 - xlim as i16)..=(hix as i16 + xlim as i16) {
                if x <= 0 || x >= crate::COLNO as i16 {
                    continue;
                }
                
                let ymin = (*lowy as i16 - ylim as i16).max(0) as usize;
                let ymax = (hiy as i16 + ylim as i16).min(crate::ROWNO as i16 - 1) as usize;
                
                for y in ymin..=ymax {
                    if level.cells[x as usize][y].typ != crate::dungeon::CellType::Stone {
                        if rng.rn2(3) == 0 {
                            return false;
                        }
                        
                        if (x as u8) < *lowx {
                            *lowx = (x as u8) + xlim + 1;
                        } else {
                            hix = (x as u8).saturating_sub(xlim).saturating_sub(1);
                        }
                        
                        if (y as u8) < *lowy {
                            *lowy = (y as u8) + ylim + 1;
                        } else {
                            hiy = (y as u8).saturating_sub(ylim).saturating_sub(1);
                        }
                        
                        continue 'chk;
                    }
                }
            }
            break;
        }

        *ddx = hix - *lowx;
        *ddy = hiy - *lowy;
        true
    }

    /// Split rectangles when a room is placed
    /// Matches NetHack's split_rects() in rect.c exactly.
    pub fn split_rects(&mut self, r1: NhRect, r2: &NhRect) {
        if let Some(idx) = get_rect_ind(&self.rects, &r1) {
            self.rects.swap_remove(idx);
            
            let mut i = self.rects.len();
            while i > 0 {
                i -= 1;
                // Recursive split_rects may shrink self.rects via swap_remove
                if i >= self.rects.len() {
                    continue;
                }
                if self.rects[i].intersects(r2) {
                    let intersecting = self.rects[i];
                    if let Some(intersection) = intersecting.intersection(r2) {
                        self.split_rects(intersecting, &intersection);
                    }
                }
            }

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

    /// Legacy split_rects for non-port usage
    pub fn split_rects_legacy(&mut self, room: &NhRect) {
        let mut to_remove = Vec::new();
        let mut to_add = Vec::new();

        for (i, rect) in self.rects.iter().enumerate() {
            if rect.intersects(room) {
                to_remove.push(i);
                if rect.lx < room.lx {
                    let left = NhRect::new(rect.lx, rect.ly, room.lx.saturating_sub(1), rect.hy);
                    if left.is_valid() && left.is_room_size() { to_add.push(left); }
                }
                if rect.hx > room.hx {
                    let right = NhRect::new(room.hx + 1, rect.ly, rect.hx, rect.hy);
                    if right.is_valid() && right.is_room_size() { to_add.push(right); }
                }
                if rect.ly < room.ly {
                    let top = NhRect::new(rect.lx.max(room.lx), rect.ly, rect.hx.min(room.hx), room.ly.saturating_sub(1));
                    if top.is_valid() && top.is_room_size() { to_add.push(top); }
                }
                if rect.hy > room.hy {
                    let bot = NhRect::new(rect.lx.max(room.lx), room.hy + 1, rect.hx.min(room.hx), rect.hy);
                    if bot.is_valid() && bot.is_room_size() { to_add.push(bot); }
                }
            }
        }
        to_remove.sort_unstable();
        for i in to_remove.into_iter().rev() { self.remove_rect(i); }
        for r in to_add { self.add_rect(r); }
    }

    /// Create a vault room (2x2 fixed size) - port of C's create_vault()
    /// which calls create_room(-1,-1,2,2,-1,-1,VAULT,TRUE)
    pub fn create_room_vault(&mut self, level: &Level, rng: &mut GameRng) -> Option<Room> {
        let mut trycnt = 0;

        while trycnt < 100 {
            trycnt += 1;
            println!("Rust: rnd_rect (create_room_vault)");
            let r1 = self.rnd_rect(rng)?;

            let hx = r1.hx;
            let hy = r1.hy;
            let lx = r1.lx;
            let ly = r1.ly;

            // Vault is always 2x2
            let dx: u8 = 1;
            let dy: u8 = 1;

            let xlim = XLIM + 1;
            let ylim = YLIM + 1;

            let xborder = if lx > 0 && hx < crate::COLNO as u8 - 1 { 2 * xlim } else { xlim + 1 };
            let yborder = if ly > 0 && hy < crate::ROWNO as u8 - 1 { 2 * ylim } else { ylim + 1 };

            if hx - lx < dx + 3 + xborder || hy - ly < dy + 3 + yborder {
                continue;
            }

            let mut xabs = lx + (if lx > 0 { xlim } else { 3 })
                + rng.rn2((hx - (if lx > 0 { lx } else { 3 }) - dx - xborder + 1) as u32) as u8;
            let mut yabs = ly + (if ly > 0 { ylim } else { 2 })
                + rng.rn2((hy - (if ly > 0 { ly } else { 2 }) - dy - yborder + 1) as u32) as u8;

            let mut ddx = dx;
            let mut ddy = dy;
            if !self.check_room(level, &mut xabs, &mut ddx, &mut yabs, &mut ddy, true, rng) {
                continue;
            }

            let wtmp = ddx + 1;
            let htmp = ddy + 1;

            let r2 = NhRect::new(xabs.saturating_sub(1), yabs.saturating_sub(1), xabs + wtmp, yabs + htmp);
            self.split_rects(r1, &r2);

            let mut room = Room::new(xabs as usize, yabs as usize, wtmp as usize, htmp as usize);
            room.room_type = super::room::RoomType::Vault;
            return Some(room);
        }
        None
    }

    pub fn pick_room_position(&self, width: u8, height: u8, rng: &mut GameRng) -> Option<(NhRect, u8, u8)> {
        let margin = 2;
        let needed_w = width + margin * 2;
        let needed_h = height + margin * 2;

        let valid: Vec<_> = self.rects.iter()
            .filter(|r| r.width() >= needed_w && r.height() >= needed_h)
            .copied()
            .collect();

        if valid.is_empty() { return None; }
        let rect = valid[rng.rn2(valid.len() as u32) as usize];
        let max_x = rect.hx.saturating_sub(width + margin);
        let max_y = rect.hy.saturating_sub(height + margin);
        if max_x < rect.lx + margin || max_y < rect.ly + margin { return None; }
        let x = rect.lx + margin + rng.rn2((max_x - rect.lx - margin + 1) as u32) as u8;
        let y = rect.ly + margin + rng.rn2((max_y - rect.ly - margin + 1) as u32) as u8;
        Some((rect, x, y))
    }

    pub fn count(&self) -> usize { self.rects.len() }
    pub fn room_rect_count(&self) -> usize { self.rects.iter().filter(|r| r.is_room_size()).count() }
    pub fn has_space(&self) -> bool { self.room_rect_count() > 0 }
    pub fn rects(&self) -> &[NhRect] { &self.rects }
}

/// Find the index of a rectangle in a list by exact match
pub fn get_rect_ind(rect_list: &[NhRect], target: &NhRect) -> Option<usize> {
    rect_list.iter().position(|r| r.lx == target.lx && r.ly == target.ly && r.hx == target.hx && r.hy == target.hy)
}

/// Check if a point is inside a rectangle
pub fn inside_rect(rect: &NhRect, x: u8, y: u8) -> bool {
    x >= rect.lx && x <= rect.hx && y >= rect.ly && y <= rect.hy
}

/// Add a rectangle to a list and update bounding box
pub fn add_rect_to_reg(rect_list: &mut Vec<NhRect>, bounding_box: &mut NhRect, new_rect: &NhRect) -> bool {
    if rect_list.len() >= MAXRECT { return false; }
    rect_list.push(*new_rect);
    if bounding_box.lx > new_rect.lx { bounding_box.lx = new_rect.lx; }
    if bounding_box.ly > new_rect.ly { bounding_box.ly = new_rect.ly; }
    if bounding_box.hx < new_rect.hx { bounding_box.hx = new_rect.hx; }
    if bounding_box.hy < new_rect.hy { bounding_box.hy = new_rect.hy; }
    true
}
