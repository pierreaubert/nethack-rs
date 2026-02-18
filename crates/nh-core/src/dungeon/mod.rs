//! Dungeon system
//!
//! Contains level structure, cells, dungeon topology, and room types.

mod bones;
mod cell;
mod corridor;
mod dlevel;
pub mod drawbridge;
pub mod economy;
mod endgame;
mod generation;
mod level;
mod mapseen;
mod maze;
mod quest;
mod rect;
mod room;
mod shop;
mod special_level;
mod special_rooms;
mod topology;
pub mod region;
pub mod trap;

pub use bones::{BonesFile, BonesHeader, BonesManager};
#[cfg(feature = "std")]
pub use bones::{
    commit_bonesfile, create_bonesfile, delete_bonesfile, getbones, open_bonesfile, savebones,
    set_bonesfile_name,
};
pub use cell::{Cell, CellType, DoorState, isok};
pub use cell::{
    ceiling, hliquid, is_drawbridge_wall, is_ice, is_lava, is_pool, set_lit, surface,
    waterbody_name,
};
pub use corridor::corr;
pub use rect::{add_rect_to_reg, inside_rect};
pub use corridor::{ConnectivityTracker, generate_corridors};
pub use dlevel::{
    DLevel,
    // Special level checks
    SpecialLevel,
    // Navigation helpers
    assign_level,
    assign_rnd_level,
    builds_up,
    deepest_lev_reached,
    describe_level,
    dname_to_dnum,
    dunlev,
    dunlevs_in_dungeon,
    // Description functions
    endgamelevelname,
    final_level,
    generic_lvl_desc,
    get_level,
    in_endgame,
    in_hell,
    in_knox,
    in_mines,
    in_quest,
    in_sokoban,
    in_v_tower,
    in_w_tower,
    invocation_lev,
    // Level query functions (In_*/Is_* equivalents)
    is_botlevel,
    // Ledger functions
    ledger_no,
    ledger_to_dlev,
    ledger_to_dnum,
    ledgerno_to_dlevel,
    // Core functions
    level_difficulty,
    maxledgerno,
    no_bones_level,
    observable_depth,
    on_w_tower_level,
    parent_dlevel,
    parent_dnum,
    unreachable_level,
};
pub use endgame::{Plane, WaterBubble, create_water_bubbles, generate_plane, update_water_level};
pub use generation::generate_rooms_and_corridors;
pub(crate) use generation::random_monster_name_for_type;
pub use generation::{
    NO_ROOM,
    ROOMOFFSET,
    SHARED,
    // Additional generation functions
    add_door,
    create_door,
    create_secret_door,
    create_subroom,
    ensure_way_out,
    fill_room,
    fill_rooms,
    fix_wall_spines,
    flood_fill_rm,
    generate_irregular_room,
    generate_rooms_with_rects,
    get_roomno,
    in_room,
    init_map,
    is_room_edge,
    mkstairs,
    remove_boundary_syms,
    room_index_from_roomno,
    set_wall,
    solidify_map,
    // New generation functions
    topologize,
    topologize_all,
    wallify_map,
};
pub use level::{
    Engraving, EngravingType, Level, LevelFlags, LightSource, LightSourceFlags, LightSourceType,
    Stairway, Trap, TrapType, enexto, migrate_monster_to_level,
};
pub use mapseen::{
    MAXNROFROOMS,
    MapSeen,
    MapSeenChain,
    MapseenAlignment,
    MapseenFeatures,
    MapseenFlags,
    MapseenRoom,
    // Functions
    format_mapseen,
    interest_mapseen,
    mapseen_temple,
    print_mapseen,
    recalc_mapseen,
};
#[cfg(feature = "std")]
pub use mapseen::{load_mapseen, save_mapseen};
pub use maze::{generate_maze, is_maze_level};
pub use quest::{
    QuestInfo, QuestStatus, generate_quest_goal, generate_quest_home, generate_quest_locate,
    intermed,
};
pub use rect::{MAXRECT, NhRect, RectManager, XLIM, YLIM};
pub use room::{
    MAX_SUBROOMS,
    Room,
    RoomType,
    get_free_room_loc,
    get_room_loc,
    in_rooms,
    inside_room,
    pick_room,
    pos_to_room,
    search_special,
    // New room query functions
    somex,
    somexy,
    somey,
};
pub use shop::{is_shop_room, populate_shop, select_shop_type, shop_object_classes};
pub use special_level::{SpecialLevelId, generate_special_level, get_special_level};
pub use special_rooms::{
    SpecialMonsterClass,
    anthole_monster,
    beehive_monster,
    court_monster,
    create_altar,
    fill_zoo,
    fumaroles,
    is_vault,
    make_grave,
    make_niches,
    mineralize,
    mkaltar,
    // Feature placement functions
    mkfount,
    mkgrave,
    mkshop,
    mksink,
    mkswamp,
    mktemple,
    // Room creation functions
    mkzoo,
    morgue_monster,
    needs_population,
    populate_special_room,
    populate_vault,
    squad_monster,
    swamp_monster,
};
pub use topology::{
    Branch,
    BranchType,
    Dungeon,
    DungeonFlags,
    DungeonId,
    DungeonSystem,
    LRegion,
    LRegionArea,
    LRegionType,
    br_string,
    br_string2,
    dungeon_alignment,
    dungeon_index,
    format_dungeon_info,
    get_branch_by_id,
    has_branch_from_level,
    // Existing functions
    init_dungeons,
    is_hellish,
    is_in_main_dungeon,
    is_main_dungeon,
    is_maze_like,
    is_town,
    level_in_hellish_dungeon,
    levels_connected,
    mk_knox_portal,
    mkportal,
    next_portal,
    num_branches,
    numdungeons,
    parent_level,
    place_lregion,
    print_branch_info,
    print_dungeon,
    put_lregion_here,
};
#[cfg(feature = "std")]
pub use topology::{restore_dungeon, save_dungeon};
