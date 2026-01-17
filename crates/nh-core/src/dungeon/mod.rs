//! Dungeon system
//!
//! Contains level structure, cells, dungeon topology, and room types.

mod cell;
mod corridor;
mod dlevel;
mod generation;
mod level;
mod rect;
mod room;
mod shop;
mod special_rooms;
mod topology;

pub use cell::{Cell, CellType, DoorState};
pub use corridor::{generate_corridors, ConnectivityTracker};
pub use dlevel::DLevel;
pub use generation::generate_rooms_and_corridors;
pub use level::{Level, LevelFlags, Stairway, Trap, TrapType};
pub use rect::{NhRect, RectManager, MAXRECT, XLIM, YLIM};
pub use room::{Room, RoomType};
pub use shop::{populate_shop, select_shop_type, shop_object_classes, is_shop_room};
pub use special_rooms::{
    court_monster, morgue_monster, squad_monster, beehive_monster, anthole_monster,
    swamp_monster, populate_special_room, populate_vault, needs_population, is_vault,
    SpecialMonsterClass,
};
pub use topology::{Branch, BranchType, Dungeon, DungeonFlags};
