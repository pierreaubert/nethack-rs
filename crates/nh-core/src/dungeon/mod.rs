//! Dungeon system
//!
//! Contains level structure, cells, dungeon topology, and room types.

mod bones;
mod cell;
mod corridor;
mod dlevel;
mod endgame;
mod generation;
mod level;
mod maze;
mod quest;
mod rect;
mod room;
mod shop;
mod special_level;
mod special_rooms;
mod topology;
pub mod trap;
pub mod economy;

pub use cell::{Cell, CellType, DoorState};
pub use corridor::{generate_corridors, ConnectivityTracker};
pub use dlevel::DLevel;
pub use generation::generate_rooms_and_corridors;
pub use level::{Level, LevelFlags, Stairway, Trap, TrapType, Engraving, EngravingType};
pub use rect::{NhRect, RectManager, MAXRECT, XLIM, YLIM};
pub use room::{Room, RoomType};
pub use shop::{populate_shop, select_shop_type, shop_object_classes, is_shop_room};
pub use special_rooms::{
    court_monster, morgue_monster, squad_monster, beehive_monster, anthole_monster,
    swamp_monster, populate_special_room, populate_vault, needs_population, is_vault,
    SpecialMonsterClass,
};
pub use topology::{Branch, BranchType, Dungeon, DungeonFlags, DungeonId, DungeonSystem};
pub use maze::{generate_maze, is_maze_level};
pub use special_level::{SpecialLevelId, get_special_level, generate_special_level};
pub use quest::{QuestInfo, QuestStatus, generate_quest_home, generate_quest_locate, generate_quest_goal};
pub use endgame::{Plane, generate_plane, WaterBubble, update_water_level, create_water_bubbles};
pub use bones::{BonesFile, BonesHeader, BonesManager};
pub use room::MAX_SUBROOMS;
pub use generation::{generate_rooms_with_rects, generate_irregular_room, create_subroom};
