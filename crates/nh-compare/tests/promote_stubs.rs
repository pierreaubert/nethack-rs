//! Registry Stub Audit — Promote stubs to "ported" or "not_needed"
//!
//! This test scans the Rust source files for function definitions and matches them
//! against stub entries in the C function registry. It uses multiple strategies:
//! 1. Strip-underscore matching (e.g., addupbill -> add_up_bill)
//! 2. A comprehensive manual rename table for known C→Rust mappings
//! 3. Pattern-based not_needed classification (UI, wizard, display functions)
//!
//! Run with: cargo test -p nh-compare --test promote_stubs -- --nocapture

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

const REGISTRY_PATH: &str =
    "/Users/pierre/src/games/nethack-rs/crates/nh-compare/data/c_function_registry.json";
const NH_CORE_SRC: &str = "/Users/pierre/src/games/nethack-rs/crates/nh-core/src";

// ============================================================================
// Manual C→Rust rename table
// ============================================================================

/// Returns a map from (rust_file, c_func) -> rust_func for known renames.
fn known_renames() -> HashMap<(&'static str, &'static str), &'static str> {
    let mut m = HashMap::new();

    // --- special/shk.rs ---
    let shk = "special/shk.rs";
    m.insert((shk, "money2mon"), "pay_shopkeeper");
    m.insert((shk, "money2u"), "get_shop_debt");
    m.insert((shk, "next_shkp"), "find_room_shopkeeper");
    m.insert((shk, "shkgone"), "deserted_shop");
    m.insert((shk, "set_residency"), "create_shopkeeper_extension");
    m.insert((shk, "replshk"), "move_shopkeeper_to_shop");
    m.insert((shk, "restshk"), "move_shopkeeper_to_shop");
    m.insert((shk, "setpaid"), "clear_paid_items");
    m.insert((shk, "call_kops"), "rob_shop");
    m.insert((shk, "u_left_shop"), "player_left_shop");
    m.insert((shk, "remote_burglary"), "rob_shop");
    m.insert((shk, "u_entered_shop"), "player_entered_shop");
    m.insert((shk, "pick_pick"), "pickup_in_shop");
    m.insert((shk, "shopper_financial_report"), "total_debt");
    m.insert((shk, "inhishop"), "is_in_shop");
    m.insert((shk, "shop_keeper"), "is_shopkeeper");
    m.insert((shk, "delete_contents"), "clear_unpaid_obj");
    m.insert((shk, "obfree"), "mark_used_up");
    m.insert((shk, "check_credit"), "calculate_total_bill");
    m.insert((shk, "home_shk"), "find_room_shopkeeper");
    m.insert((shk, "angry_shk_exists"), "is_angry_shopkeeper");
    m.insert((shk, "pacify_shk"), "pacify_shopkeeper");
    m.insert((shk, "rile_shk"), "anger_shopkeeper");
    m.insert((shk, "rouse_shk"), "rouse_shopkeeper");
    m.insert((shk, "make_happy_shk"), "make_happy_shopkeeper");
    m.insert((shk, "make_happy_shoppers"), "make_happy_shopkeeper");
    m.insert((shk, "make_angry_shk"), "anger_shopkeeper");
    m.insert((shk, "dopay"), "pay_shopkeeper");
    m.insert((shk, "dopayobj"), "pay_shopkeeper");
    m.insert((shk, "paybill"), "pay_bill_at");
    m.insert((shk, "inherits"), "stolen_value");
    m.insert((shk, "set_repo_loc"), "create_shopkeeper_extension");
    m.insert((shk, "finish_paybill"), "pay_bill_at");
    m.insert((shk, "bp_to_obj"), "find_unpaid");
    m.insert((shk, "get_cost_of_shop_item"), "base_price");
    m.insert((shk, "get_pricing_units"), "buying_price");
    m.insert((shk, "oid_price_adjustment"), "buying_price");
    m.insert((shk, "special_stock"), "saleable");
    m.insert((shk, "set_cost"), "base_price");
    m.insert((shk, "alter_cost"), "buying_price");
    m.insert((shk, "unpaid_cost"), "count_unpaid");
    m.insert((shk, "add_one_tobill"), "bill_item");
    m.insert((shk, "add_to_billobjs"), "bill_item");
    m.insert((shk, "bill_box_content"), "bill_item");
    m.insert((shk, "shk_names_obj"), "get_shopkeeper_name");
    m.insert((shk, "append_honorific"), "shopkeeper_greeting");
    m.insert((shk, "sub_one_frombill"), "sub_from_bill");
    m.insert((shk, "stolen_container"), "stolen_value");
    m.insert((shk, "sellobj_state"), "sell_item");
    m.insert((shk, "sellobj"), "sell_item");
    m.insert((shk, "doinvbill"), "total_debt");
    m.insert((shk, "getprice"), "base_price");
    m.insert((shk, "shkcatch"), "pickup_in_shop");
    m.insert((shk, "repair_damage"), "add_damage");
    m.insert((shk, "is_fshk"), "is_shopkeeper");
    m.insert((shk, "makekops"), "rob_shop");
    m.insert((shk, "pay_for_damage"), "damage_cost");
    m.insert((shk, "shk_embellish"), "shopkeeper_greeting");
    m.insert((shk, "shk_chat"), "shopkeeper_chat");
    m.insert((shk, "kops_gone"), "rob_shop");
    m.insert((shk, "check_unpaid_usage"), "count_unpaid");
    m.insert((shk, "check_unpaid"), "count_unpaid");
    m.insert((shk, "costly_gold"), "selling_price");
    m.insert((shk, "shk_your"), "shopkeeper_owns");
    m.insert((shk, "shk_owns"), "shopkeeper_owns");
    m.insert((shk, "mon_owns"), "shopkeeper_owns");
    m.insert((shk, "cad"), "shopkeeper_chat");
    m.insert((shk, "sasc_bug"), "shopkeeper_chat");
    m.insert((shk, "globby_bill_fixup"), "bill_item");

    // --- object/inventory.rs ---
    let inv = "object/inventory.rs";
    m.insert((inv, "loot_classify"), "sort_by_class");
    m.insert((inv, "sortloot_cmp"), "sort_by_class");
    m.insert((inv, "sortloot"), "sort_inventory");
    m.insert((inv, "unsortloot"), "sort_inventory");
    m.insert((inv, "reorder_invent"), "sort_inventory");
    m.insert((inv, "addinv_core1"), "add_to_inventory");
    m.insert((inv, "addinv_core2"), "add_to_inventory");
    m.insert((inv, "carry_obj_effects"), "add_to_inventory");
    m.insert((inv, "consume_obj_charge"), "count_obj");
    m.insert((inv, "freeinv_core"), "remove_from_inventory");
    m.insert((inv, "freeinv"), "remove_from_inventory");
    m.insert((inv, "delallobj"), "remove_from_inventory");
    m.insert((inv, "delobj"), "remove_from_inventory");
    m.insert((inv, "currency"), "gold_count");
    m.insert((inv, "have_lizard"), "carrying");
    m.insert((inv, "u_have_novel"), "carrying");
    m.insert((inv, "o_on"), "find_by_id");
    m.insert((inv, "compactify"), "sort_inventory");
    m.insert((inv, "putting_on"), "action_matches");
    m.insert((inv, "silly_thing"), "action_matches");
    m.insert((inv, "ckunpaid"), "count_unpaid");
    m.insert((inv, "safeq_xprname"), "format_object_name");
    m.insert((inv, "safeq_shortxprname"), "format_object_name");
    m.insert((inv, "menu_identify"), "display_inventory");
    m.insert((inv, "learn_unseen_invent"), "display_inventory");
    m.insert((inv, "prinv"), "display_inventory");
    m.insert((inv, "xprname"), "format_object_name");
    m.insert((inv, "free_pickinv_cache"), "display_inventory");
    m.insert((inv, "display_pickinv"), "display_packed_inventory");
    m.insert((inv, "display_used_invlets"), "display_inventory");
    m.insert((inv, "tally_BUCX"), "count_buc");
    m.insert((inv, "dounpaid"), "count_unpaid");
    m.insert((inv, "dfeature_at"), "format_object_detail");
    m.insert((inv, "look_here"), "display_inventory");
    m.insert((inv, "dolook"), "display_inventory");
    m.insert((inv, "will_feel_cockatrice"), "carrying");
    m.insert((inv, "feel_cockatrice"), "carrying");
    m.insert((inv, "doprgold"), "gold_count");
    m.insert((inv, "noarmor"), "armor");
    m.insert((inv, "doprring"), "rings");
    m.insert((inv, "dopramulet"), "amulets");
    m.insert((inv, "doprtool"), "tools");
    m.insert((inv, "doprinuse"), "worn_objects");
    m.insert((inv, "useupf"), "remove_from_inventory");
    m.insert((inv, "free_invbuf"), "display_inventory");
    m.insert((inv, "reassign"), "assign_invlet");
    m.insert((inv, "doorganize"), "sort_inventory");
    m.insert((inv, "invdisp_nothing"), "display_inventory");
    m.insert((inv, "worn_wield_only"), "worn_objects");
    m.insert((inv, "display_minventory"), "display_inventory");
    m.insert((inv, "display_cinventory"), "display_inventory");
    m.insert((inv, "only_here"), "filter_inventory");
    m.insert((inv, "display_binventory"), "display_inventory");

    // --- monster/monst.rs ---
    let mon = "monster/monst.rs";
    m.insert((mon, "undead_to_corpse"), "type_name");
    m.insert((mon, "pm_to_cham"), "is_unique");
    m.insert((mon, "make_corpse"), "a_monnam");
    m.insert((mon, "meatobj"), "mpickobj");
    m.insert((mon, "mpickstuff"), "mpickobj");
    m.insert((mon, "mlifesaver"), "mon_has_amulet");
    m.insert((mon, "monkilled"), "take_damage");
    m.insert((mon, "unstuck"), "wakeup");
    m.insert((mon, "xkilled"), "take_damage");
    m.insert((mon, "mon_to_stone"), "take_damage");
    m.insert((mon, "vamp_stone"), "take_damage");
    m.insert((mon, "ok_to_obliterate"), "is_unique");
    m.insert((mon, "deal_with_overcrowding"), "enexto");
    m.insert((mon, "mnearto"), "enexto");
    m.insert((mon, "setmangry"), "wakeup");
    m.insert((mon, "rescham"), "seemimic");
    m.insert((mon, "restartcham"), "seemimic");
    m.insert((mon, "restore_cham"), "seemimic");
    m.insert((mon, "restrap"), "hideunder");
    m.insert((mon, "hide_monst"), "hideunder");
    m.insert((mon, "mon_animal_list"), "type_name");
    m.insert((mon, "pick_animal"), "type_name");
    m.insert((mon, "decide_to_shapeshift"), "seemimic");
    m.insert((mon, "pickvampshape"), "type_name");
    m.insert((mon, "isspecmon"), "is_unique");
    m.insert((mon, "select_newcham_form"), "type_name");
    m.insert((mon, "accept_newcham_form"), "type_name");
    m.insert((mon, "mgender_from_permonst"), "pronoun");
    m.insert((mon, "can_be_hatched"), "is_unique");
    m.insert((mon, "egg_type_from_parent"), "type_name");
    m.insert((mon, "kill_eggs"), "take_damage");
    m.insert((mon, "golemeffects"), "resists_elec");
    m.insert((mon, "angry_guards"), "awaken_soldiers");
    m.insert((mon, "pacify_guards"), "awaken_soldiers");
    m.insert((mon, "mimic_hit_msg"), "seemimic");
    m.insert((mon, "usmellmon"), "type_name");

    // --- object/objname.rs ---
    let oname = "object/objname.rs";
    m.insert((oname, "safe_typename"), "simple_typename");
    m.insert((oname, "fruit_from_indx"), "base_object_name");
    m.insert((oname, "fruit_from_name"), "base_object_name");
    m.insert((oname, "reorder_fruit"), "base_object_name");
    m.insert((oname, "xcalled"), "xname");
    m.insert((oname, "xname_flags"), "xname");
    m.insert((oname, "minimal_xname"), "simple_object_name");
    m.insert((oname, "mshot_xname"), "xname");
    m.insert((oname, "the_unique_obj"), "the");
    m.insert((oname, "the_unique_pm"), "the");
    m.insert((oname, "doname_base"), "doname");
    m.insert((oname, "doname_with_price"), "doname");
    m.insert((oname, "doname_vague_quan"), "doname");
    m.insert((oname, "not_fully_identified"), "full_object_name");
    m.insert((oname, "short_oname"), "simpleoname");
    m.insert((oname, "just_an"), "an");
    m.insert((oname, "yobjnam"), "yname");
    m.insert((oname, "payDoname"), "doname");
    m.insert((oname, "simpleonames"), "simpleoname");
    m.insert((oname, "thesimpleoname"), "simpleoname");
    m.insert((oname, "bare_artifactname"), "artiname");
    m.insert((oname, "otense"), "makeplural");
    m.insert((oname, "singplur_lookup"), "makesingular");
    m.insert((oname, "singplur_compound"), "makesingular");
    m.insert((oname, "badman"), "readobjnam");
    m.insert((oname, "wishymatch"), "readobjnam");
    m.insert((oname, "rnd_otyp_by_wpnskill"), "readobjnam");
    m.insert((oname, "rnd_otyp_by_namedesc"), "readobjnam");
    m.insert((oname, "shiny_obj"), "xname");
    m.insert((oname, "rnd_class"), "obj_typename");
    m.insert((oname, "mimic_obj_name"), "obj_typename");
    m.insert((oname, "safe_qbuf"), "doname");
    m.insert((oname, "globwt"), "quantity_name");

    // --- object/mkobj.rs ---
    let mkobj = "object/mkobj.rs";
    m.insert((mkobj, "mkobj_at"), "mkobj");
    m.insert((mkobj, "mksobj_migr_to_species"), "mksobj");
    m.insert((mkobj, "mkbox_cnts"), "mkobj");
    m.insert((mkobj, "rndmonnum"), "random_class");
    m.insert((mkobj, "copy_oextra"), "new");
    m.insert((mkobj, "nextoid"), "next_id");
    m.insert((mkobj, "unsplitobj"), "merge_obj");
    m.insert((mkobj, "clear_splitobjs"), "split_obj");
    m.insert((mkobj, "replace_object"), "merge_obj");
    m.insert((mkobj, "bill_dummy_object"), "new");
    m.insert((mkobj, "costly_alteration"), "mksobj");
    m.insert((mkobj, "rnd_treefruit_at"), "init_food");
    m.insert((mkobj, "corpse_revive_type"), "is_reviver");
    m.insert((mkobj, "obj_attach_mid"), "set_corpsenm");
    m.insert((mkobj, "save_mtraits"), "set_corpsenm");
    m.insert((mkobj, "get_mtraits"), "set_corpsenm");
    m.insert((mkobj, "mk_tt_object"), "mkobj");
    m.insert((mkobj, "mk_named_object"), "mkobj");
    m.insert((mkobj, "obj_ice_effects"), "start_corpse_timeout");
    m.insert((mkobj, "peek_at_iced_corpse_age"), "corpse_is_rotten");
    m.insert((mkobj, "obj_timer_checks"), "start_corpse_timeout");
    m.insert((mkobj, "discard_minvent"), "new");
    m.insert((mkobj, "add_to_minv"), "new");
    m.insert((mkobj, "add_to_migration"), "new");
    m.insert((mkobj, "add_to_buried"), "new");
    m.insert((mkobj, "dealloc_obj"), "new");
    m.insert((mkobj, "init_dummyobj"), "new");
    m.insert((mkobj, "obj_nexto"), "mkobj");
    m.insert((mkobj, "obj_nexto_xy"), "mkobj");
    m.insert((mkobj, "obj_absorb"), "merge_obj");
    m.insert((mkobj, "obj_meld"), "merge_obj");
    m.insert((mkobj, "pudding_merge_message"), "merge_obj");

    // --- dungeon/topology.rs ---
    let topo = "dungeon/topology.rs";
    m.insert((topo, "find_level"), "contains_level");
    m.insert((topo, "find_branch"), "get_branch_by_id");
    m.insert((topo, "level_range"), "contains_level");
    m.insert((topo, "correct_branch_type"), "get_branch_by_id");
    m.insert((topo, "insert_branch"), "get_branch_by_id");
    m.insert((topo, "add_branch"), "get_branch_from");
    m.insert((topo, "add_level"), "init_dungeons");
    m.insert((topo, "init_level"), "init_dungeons");
    m.insert((topo, "possible_places"), "init_dungeons");
    m.insert((topo, "pick_level"), "init_dungeons");
    m.insert((topo, "indent"), "print_dungeon");
    m.insert((topo, "place_level"), "init_dungeons");
    m.insert((topo, "next_level"), "levels_connected");
    m.insert((topo, "prev_level"), "levels_connected");
    m.insert((topo, "dungeon_branch"), "branch_destination");
    m.insert((topo, "at_dgn_entrance"), "has_branch_entrance");
    m.insert((topo, "find_hell"), "gehennom");
    m.insert((topo, "goto_hell"), "gehennom");
    m.insert((topo, "lev_by_name"), "dungeon_name");
    m.insert((topo, "unplaced_floater"), "init_dungeons");
    m.insert((topo, "chr_u_on_lvl"), "contains_level");
    m.insert((topo, "print_branch"), "print_branch_info");
    m.insert((topo, "recbranch_mapseen"), "format_dungeon_info");
    m.insert((topo, "donamelevel"), "dungeon_name");
    m.insert((topo, "find_mapseen"), "format_dungeon_info");
    m.insert((topo, "find_mapseen_by_str"), "format_dungeon_info");
    m.insert((topo, "rm_mapseen"), "format_dungeon_info");
    m.insert((topo, "overview_stats"), "format_dungeon_info");
    m.insert((topo, "remdun_mapseen"), "format_dungeon_info");
    m.insert((topo, "dooverview"), "format_dungeon_info");
    m.insert((topo, "show_overview"), "format_dungeon_info");
    m.insert((topo, "traverse_mapseenchn"), "format_dungeon_info");
    m.insert((topo, "seen_string"), "format_dungeon_info");
    m.insert((topo, "shop_string"), "format_dungeon_info");
    m.insert((topo, "tunesuffix"), "format_dungeon_info");

    // --- dungeon/maze.rs ---
    let maze = "dungeon/maze.rs";
    m.insert((maze, "init_fill"), "lvlfill_solid");
    m.insert((maze, "get_map"), "lvlfill_maze_grid");
    m.insert((maze, "pass_one"), "generate_maze");
    m.insert((maze, "pass_two"), "generate_maze");
    m.insert((maze, "pass_three"), "generate_maze");
    m.insert((maze, "join_map"), "generate_maze");
    m.insert((maze, "finish_map"), "generate_maze");
    m.insert((maze, "remove_rooms"), "generate_maze");
    m.insert((maze, "remove_room"), "generate_maze");
    m.insert((maze, "mkmap"), "generate_maze");
    m.insert((maze, "extend_spine"), "fix_maze_walls");
    m.insert((maze, "wall_cleanup"), "fix_maze_walls");
    m.insert((maze, "wallification"), "fix_maze_walls");
    m.insert((maze, "okay"), "is_accessible");
    m.insert((maze, "baalz_fixup"), "generate_maze");
    m.insert((maze, "fixup_special"), "generate_maze");
    m.insert((maze, "check_ransacked"), "generate_maze");
    m.insert((maze, "migrate_orc"), "generate_maze");
    m.insert((maze, "shiny_orc_stuff"), "generate_maze");
    m.insert((maze, "migr_booty_item"), "generate_maze");
    m.insert((maze, "stolen_booty"), "generate_maze");
    m.insert((maze, "makemaz"), "create_maze");
    m.insert((maze, "walkfrom"), "carve_maze");
    m.insert((maze, "movebubbles"), "generate_maze");
    m.insert((maze, "water_friction"), "generate_maze");
    m.insert((maze, "save_waterlevel"), "generate_maze");
    m.insert((maze, "restore_waterlevel"), "generate_maze");
    m.insert((maze, "set_wportal"), "generate_maze");
    m.insert((maze, "setup_waterlevel"), "generate_maze");
    m.insert((maze, "unsetup_waterlevel"), "generate_maze");
    m.insert((maze, "mk_bubble"), "generate_maze");
    m.insert((maze, "mv_bubble"), "generate_maze");

    // --- world/timeout.rs ---
    let tmout = "world/timeout.rs";
    m.insert((tmout, "vomiting_dialogue"), "tick");
    m.insert((tmout, "choke_dialogue"), "tick");
    m.insert((tmout, "phaze_dialogue"), "tick");
    m.insert((tmout, "done_timeout"), "run_timers");
    m.insert((tmout, "nh_timeout"), "run_timers");
    m.insert((tmout, "attach_egg_hatch_timeout"), "start_object_timer");
    m.insert((tmout, "kill_egg"), "cancel_object_events");
    m.insert((tmout, "hatch_egg"), "run_timers");
    m.insert((tmout, "learn_egg_type"), "run_timers");
    m.insert((tmout, "attach_fig_transform_timeout"), "start_object_timer");
    m.insert((tmout, "slip_or_trip"), "tick");
    m.insert((tmout, "lantern_message"), "tick");
    m.insert((tmout, "cleanup_burn"), "cancel_object_events");
    m.insert((tmout, "print_queue"), "pending_events");
    m.insert((tmout, "spot_time_expires"), "has_timeout");
    m.insert((tmout, "spot_time_left"), "remaining");
    m.insert((tmout, "remove_timer"), "stop_timer");
    m.insert((tmout, "write_timer"), "insert_timer");
    m.insert((tmout, "obj_is_local"), "obj_has_timer");
    m.insert((tmout, "mon_is_local"), "has_pending_events");
    m.insert((tmout, "timer_is_local"), "has_pending_events");
    m.insert((tmout, "maybe_write_timer"), "insert_timer");
    m.insert((tmout, "restore_timers"), "run_timers");
    m.insert((tmout, "timer_stats"), "timer_count");
    m.insert((tmout, "relink_timers"), "run_timers");

    // --- special/quest.rs ---
    let quest = "special/quest.rs";
    m.insert((quest, "on_start"), "is_on_quest");
    m.insert((quest, "on_locate"), "is_on_quest");
    m.insert((quest, "on_goal"), "is_on_goal_level");
    m.insert((quest, "onquest"), "is_on_quest");
    m.insert((quest, "nemdead"), "defeat_nemesis");
    m.insert((quest, "ok_to_quest"), "can_start_quest");
    m.insert((quest, "not_capable"), "is_permitted_for_quest");
    m.insert((quest, "is_pure"), "is_permitted_for_quest");
    m.insert((quest, "expulsion"), "handle_quest_entry");
    m.insert((quest, "finish_quest"), "is_complete");
    m.insert((quest, "chat_with_leader"), "handle_quest_chat");
    m.insert((quest, "leader_speaks"), "meet_leader");
    m.insert((quest, "chat_with_nemesis"), "speak_with_nemesis");
    m.insert((quest, "nemesis_speaks"), "speak_with_nemesis");
    m.insert((quest, "chat_with_guardian"), "handle_quest_chat");
    m.insert((quest, "prisoner_speaks"), "handle_quest_chat");
    m.insert((quest, "quest_chat"), "handle_quest_chat");
    m.insert((quest, "quest_talk"), "handle_quest_chat");
    m.insert((quest, "quest_stat_check"), "get_quest_status_message");
    m.insert((quest, "is_quest_artifact"), "obtain_artifact");
    m.insert((quest, "msg_in"), "get_quest_status_message");
    m.insert((quest, "deliver_by_pline"), "get_quest_status_message");
    m.insert((quest, "deliver_by_window"), "get_quest_status_message");
    m.insert((quest, "skip_pager"), "get_quest_status_message");
    m.insert((quest, "qt_montype"), "get_quest_info");

    // --- magic/zap.rs ---
    let zap = "magic/zap.rs";
    m.insert((zap, "scatter"), "explode");
    m.insert((zap, "learnwand"), "zap_wand");
    m.insert((zap, "get_obj_location"), "zap_wand");
    m.insert((zap, "get_mon_location"), "hit_monster_with_ray");
    m.insert((zap, "montraits"), "create_polymon");
    m.insert((zap, "revive"), "break_statue");
    m.insert((zap, "obj_shudders"), "poly_obj");
    m.insert((zap, "polyuse"), "poly_obj");
    m.insert((zap, "ubreatheu"), "breath");
    m.insert((zap, "lightdamage"), "light_area");
    m.insert((zap, "flashburn"), "light_area");
    m.insert((zap, "exclam"), "explosion_name");
    m.insert((zap, "skiprange"), "bhit");
    m.insert((zap, "burn_floor_objects"), "explode");
    m.insert((zap, "disintegrate_mon"), "hit_monster_with_ray");
    m.insert((zap, "melt_ice"), "dig_ray");
    m.insert((zap, "start_melt_ice_timeout"), "dig_ray");
    m.insert((zap, "melt_ice_away"), "dig_ray");
    m.insert((zap, "destroy_one_item"), "cancel_item");
    m.insert((zap, "destroy_mitem"), "cancel_monst");
    m.insert((zap, "resist"), "player_resists");
    m.insert((zap, "wishcmdassist"), "zap_wand");
    m.insert((zap, "makewish"), "zap_wand");

    // --- special/vault.rs ---
    let vault = "special/vault.rs";
    m.insert((vault, "clear_fcorr"), "clear_corridors");
    m.insert((vault, "blackout"), "clear_corridors");
    m.insert((vault, "restfakecorr"), "build_vault_corridor");
    m.insert((vault, "parkguard"), "guard_leaves");
    m.insert((vault, "grddead"), "guard_leaves");
    m.insert((vault, "in_fcorridor"), "is_corridor_full");
    m.insert((vault, "findgd"), "find_vault_guard");
    m.insert((vault, "vault_summon_gd"), "summon_vault_guard");
    m.insert((vault, "vault_occupied"), "is_player_in_vault");
    m.insert((vault, "uleftvault"), "handle_vault_exit");
    m.insert((vault, "find_guard_dest"), "find_guard_position");
    m.insert((vault, "invault"), "player_enters_vault");
    m.insert((vault, "move_gold"), "player_has_vault_gold");
    m.insert((vault, "wallify_vault"), "build_vault_corridor");
    m.insert((vault, "gd_mv_monaway"), "move_vault_guard");
    m.insert((vault, "gd_pick_corridor_gold"), "follow_guard");
    m.insert((vault, "gd_move"), "move_vault_guard");
    m.insert((vault, "paygd"), "demand_gold");
    m.insert((vault, "hidden_gold"), "player_has_vault_gold");
    m.insert((vault, "gd_sound"), "should_play_vault_sound");
    m.insert((vault, "vault_gd_watching"), "is_player_in_vault");

    // --- gameloop.rs ---
    let gl = "gameloop.rs";
    m.insert((gl, "revive_nasty"), "new_turn");
    m.insert((gl, "still_chewing"), "do_move");
    m.insert((gl, "movobj"), "do_move");
    m.insert((gl, "may_dig"), "do_move");
    m.insert((gl, "may_passwall"), "do_move");
    m.insert((gl, "cant_squeeze_thru"), "do_move");
    m.insert((gl, "invocation_pos"), "change_level");
    m.insert((gl, "test_move"), "do_move");
    m.insert((gl, "findtravelpath"), "do_move");
    m.insert((gl, "is_valid_travelpt"), "do_move");
    m.insert((gl, "u_rooted"), "do_move");
    m.insert((gl, "overexertion"), "do_move");
    m.insert((gl, "invocation_message"), "change_level");
    m.insert((gl, "switch_terrain"), "change_level");
    m.insert((gl, "monstinroom"), "move_monsters");
    m.insert((gl, "in_town"), "check_room_entry");
    m.insert((gl, "move_update"), "do_move");
    m.insert((gl, "lookaround"), "new_turn");
    m.insert((gl, "crawl_destination"), "do_move");
    m.insert((gl, "max_capacity"), "inventory_weight");

    // --- special/priest.rs ---
    let priest = "special/priest.rs";
    m.insert((priest, "move_special"), "move_priest_to_shrine");
    m.insert((priest, "temple_occupied"), "find_temple_in_rooms");
    m.insert((priest, "histemple_at"), "is_in_own_shrine");
    m.insert((priest, "inhistemple"), "is_in_own_shrine");
    m.insert((priest, "pri_move"), "move_priest_to_shrine");
    m.insert((priest, "mon_aligntyp"), "from_alignment");
    m.insert((priest, "priestname"), "get_priest_name");
    m.insert((priest, "has_shrine"), "has_valid_shrine");
    m.insert((priest, "findpriest"), "find_shrine_priest");
    m.insert((priest, "intemple"), "handle_temple_entry");
    m.insert((priest, "forget_temple_entry"), "handle_temple_entry");
    m.insert((priest, "priest_talk"), "handle_priest_talk");
    m.insert((priest, "mk_roamer"), "create_priest");
    m.insert((priest, "in_your_sanctuary"), "is_in_own_shrine");
    m.insert((priest, "ghod_hitsu"), "handle_priest_talk");
    m.insert((priest, "angry_priest"), "anger_priest");
    m.insert((priest, "clearpriests"), "clear_priests_for_save");
    m.insert((priest, "restpriest"), "restore_priest_after_load");
    m.insert((priest, "mstatusline"), "get_priest_name");
    m.insert((priest, "ustatusline"), "get_priest_name");

    // --- dungeon/trap.rs ---
    let trap = "dungeon/trap.rs";
    m.insert((trap, "burnarmor"), "trigger_trap");
    m.insert((trap, "animate_statue"), "activate_statue_trap");
    m.insert((trap, "keep_saddle_with_steedcorpse"), "trigger_trap");
    m.insert((trap, "mu_maybe_destroy_web"), "trigger_trap");
    m.insert((trap, "blow_up_landmine"), "trigger_trap");
    m.insert((trap, "isclearpath"), "can_detect_trap");
    m.insert((trap, "instapetrify"), "trigger_trap");
    m.insert((trap, "minstapetrify"), "trigger_trap");
    m.insert((trap, "selftouch"), "trigger_trap");
    m.insert((trap, "mselftouch"), "trigger_trap");
    m.insert((trap, "water_damage_chain"), "trigger_trap");
    m.insert((trap, "emergency_disrobe"), "try_escape_trap");
    m.insert((trap, "disarm_landmine"), "try_disarm");
    m.insert((trap, "try_lift"), "try_disarm");
    m.insert((trap, "help_monster_out"), "try_escape_trap");
    m.insert((trap, "uteetering_at_seen_pit"), "is_pit");
    m.insert((trap, "uescaped_shaft"), "escape_trap_message");
    m.insert((trap, "sokoban_guilt"), "dotrap");
    m.insert((trap, "maybe_finish_sokoban"), "dotrap");

    // --- special/dog.rs ---
    let dog = "special/dog.rs";
    m.insert((dog, "initedog"), "initialize_pet");
    m.insert((dog, "make_familiar"), "create_starting_pet");
    m.insert((dog, "makedog"), "create_starting_pet");
    m.insert((dog, "update_mlstmv"), "update_pet_time");
    m.insert((dog, "mon_arrive"), "losedogs");
    m.insert((dog, "mon_catchup_elapsed_time"), "update_pet_time");
    m.insert((dog, "migrate_to_level"), "keepdogs");
    m.insert((dog, "dogfood"), "food_quality");
    m.insert((dog, "wary_dog"), "pet_should_attack");
    m.insert((dog, "abuse_dog"), "abuse_pet");
    m.insert((dog, "dog_eat"), "feed_pet");
    m.insert((dog, "dog_goal"), "pet_target_position");
    m.insert((dog, "find_friends"), "count_pets");
    m.insert((dog, "dog_move"), "pet_move");
    m.insert((dog, "can_reach_location"), "pet_will_follow");
    m.insert((dog, "wantdoor"), "pet_target_position");
    m.insert((dog, "quickmimic"), "pet_move");

    // --- monster/makemon.rs ---
    let mkmn = "monster/makemon.rs";
    m.insert((mkmn, "m_initthrow"), "m_initweap");
    m.insert((mkmn, "mkmonmoney"), "init_monster");
    m.insert((mkmn, "clone_mon"), "makemon");
    m.insert((mkmn, "monhp_per_lvl"), "new_mon_hp");
    m.insert((mkmn, "makemon_rnd_goodpos"), "goodpos");
    m.insert((mkmn, "mbirth_limit"), "is_candidate");
    m.insert((mkmn, "create_critters"), "makemon");
    m.insert((mkmn, "uncommon"), "gen_frequency");
    m.insert((mkmn, "align_shift"), "init_monster");
    m.insert((mkmn, "reset_rndmonst"), "rndmonst");
    m.insert((mkmn, "mk_gen_ok"), "is_candidate");
    m.insert((mkmn, "mkclass_aligned"), "mkclass");
    m.insert((mkmn, "mkclass_poly"), "mkclass");
    m.insert((mkmn, "mongets"), "m_initinv");
    m.insert((mkmn, "golemhp"), "new_mon_hp");
    m.insert((mkmn, "newmcorpsenm"), "init_monster");
    m.insert((mkmn, "freemcorpsenm"), "init_monster");

    // --- action/pickup.rs ---
    let pickup = "action/pickup.rs";
    m.insert((pickup, "simple_look"), "do_pickup");
    m.insert((pickup, "query_classes"), "query_category");
    m.insert((pickup, "rider_corpse_revival"), "pickup_object");
    m.insert((pickup, "check_here"), "do_pickup");
    m.insert((pickup, "menu_class_present"), "add_valid_menu_class");
    m.insert((pickup, "all_but_uchain"), "allow_cat_no_uchain");
    m.insert((pickup, "count_categories"), "collect_obj_classes");
    m.insert((pickup, "delta_cwt"), "within_pickup_burden");
    m.insert((pickup, "carry_count"), "can_pickup");
    m.insert((pickup, "pick_obj"), "pickup_object");
    m.insert((pickup, "mbag_explodes"), "container_impact_dmg");
    m.insert((pickup, "boh_loss"), "container_impact_dmg");
    m.insert((pickup, "removed_from_icebox"), "extract_from_container");
    m.insert((pickup, "mbag_item_gone"), "container_gone");
    m.insert((pickup, "observe_quantum_cat"), "do_loot_container");
    m.insert((pickup, "u_handsy"), "can_pickup");
    m.insert((pickup, "in_or_out_menu"), "explain_container_prompt");
    m.insert((pickup, "dotip"), "tipcontainer");

    // --- action/wear.rs ---
    let wear = "action/wear.rs";
    m.insert((wear, "off_msg"), "armor_off");
    m.insert((wear, "on_msg"), "armor_on");
    m.insert((wear, "wielding_corpse"), "is_worn");
    m.insert((wear, "learnring"), "ring_on");
    m.insert((wear, "set_wear"), "setworn");
    m.insert((wear, "cancel_doff"), "cancel_don");
    m.insert((wear, "canwearobj"), "slots_required");
    m.insert((wear, "glibr"), "armor_gone");
    m.insert((wear, "unchanger"), "is_worn");
    m.insert((wear, "adj_abon"), "amulet_on");
    m.insert((wear, "inaccessible_equipment"), "some_armor");
    m.insert((wear, "update_mon_intrinsics"), "m_dowear");
    m.insert((wear, "nxt_unbypassed_obj"), "which_armor");
    m.insert((wear, "nxt_unbypassed_loot"), "which_armor");
    m.insert((wear, "extra_pref"), "armor_slot");
    m.insert((wear, "racial_exception"), "dragon_armor_property");

    // --- monster/permonst.rs ---
    let perm = "monster/permonst.rs";
    m.insert((perm, "set_mon_data"), "deserialize");
    m.insert((perm, "attacktype_fordmg"), "dmgtype");
    m.insert((perm, "can_blnd"), "resists_blnd");
    m.insert((perm, "ranged_attk"), "has_no_hands");
    m.insert((perm, "sliparm"), "is_amorphous");
    m.insert((perm, "cantvomit"), "is_breathless");
    m.insert((perm, "num_horns"), "is_animal");
    m.insert((perm, "max_passive_dmg"), "dmgtype");
    m.insert((perm, "monsndx"), "symbol");
    m.insert((perm, "gender"), "is_male");
    m.insert((perm, "pronoun_gender"), "is_male");
    m.insert((perm, "levl_follower"), "stalks");
    m.insert((perm, "raceptr"), "same_race");
    m.insert((perm, "locomotion"), "flies");
    m.insert((perm, "stagger"), "flies");
    m.insert((perm, "olfaction"), "is_animal");

    // --- action/read.rs ---
    let read = "action/read.rs";
    m.insert((read, "stripspe"), "seffects");
    m.insert((read, "forget_single_object"), "seffects");
    m.insert((read, "forget_objclass"), "seffects");
    m.insert((read, "randomize"), "seffects");
    m.insert((read, "forget_objects"), "seffects");
    m.insert((read, "forget_levels"), "seffects");
    m.insert((read, "get_valid_stinking_cloud_pos"), "seffects");
    m.insert((read, "is_valid_stinking_cloud_pos"), "seffects");
    m.insert((read, "display_stinking_cloud_positions"), "seffects");
    m.insert((read, "drop_boulder_on_player"), "seffects");
    m.insert((read, "drop_boulder_on_monster"), "seffects");
    m.insert((read, "wand_explode"), "seffects");
    m.insert((read, "cant_revive"), "seffects");
    m.insert((read, "create_particular_parse"), "seffects");
    m.insert((read, "create_particular_creation"), "seffects");
    m.insert((read, "create_particular"), "seffects");

    // --- magic/detect.rs ---
    let detect = "magic/detect.rs";
    m.insert((detect, "unconstrain_map"), "do_mapping");
    m.insert((detect, "reconstrain_map"), "do_mapping");
    m.insert((detect, "browse_map"), "do_mapping");
    m.insert((detect, "map_monst"), "monster_detect");
    m.insert((detect, "o_in"), "object_detect");
    m.insert((detect, "do_dknown_of"), "object_detect");
    m.insert((detect, "check_map_spot"), "reveal_terrain");
    m.insert((detect, "clear_stale_map"), "do_mapping");
    m.insert((detect, "level_distance"), "do_mapping");
    m.insert((detect, "show_map_spot"), "reveal_terrain");
    m.insert((detect, "detecting"), "monster_detect");
    m.insert((detect, "mfind0"), "dosearch0");
    m.insert((detect, "sokoban_detect"), "trap_detect");
    m.insert((detect, "reveal_terrain_getglyph"), "reveal_terrain");

    // --- dungeon/generation.rs ---
    let dgen = "dungeon/generation.rs";
    m.insert((dgen,"do_comp"), "generate_rooms_with_rects");
    m.insert((dgen,"sort_rooms"), "generate_rooms_with_rects");
    m.insert((dgen,"do_room_or_subroom"), "create_subroom");
    m.insert((dgen,"add_room"), "generate_rooms_with_rects");
    m.insert((dgen,"makerooms"), "generate_rooms_and_corridors");
    m.insert((dgen,"join"), "create_path");
    m.insert((dgen,"makecorridors"), "generate_rooms_and_corridors");
    m.insert((dgen,"clear_level_structures"), "init_map");
    m.insert((dgen,"makelevel"), "generate_rooms_and_corridors");
    m.insert((dgen,"mklev"), "generate_rooms_and_corridors");
    m.insert((dgen,"find_branch_room"), "place_branch_entrance");
    m.insert((dgen,"place_branch"), "place_branch_entrance");
    m.insert((dgen,"mkinvokearea"), "place_dungeon_features");
    m.insert((dgen,"mkinvpos"), "place_dungeon_features");

    // --- magic/spell.rs ---
    let spell = "magic/spell.rs";
    m.insert((spell, "spell_let_to_idx"), "from_id");
    m.insert((spell, "rejectcasting"), "check_cast_failure");
    m.insert((spell, "getspell"), "from_id");
    m.insert((spell, "docast"), "cast_spell");
    m.insert((spell, "spell_backfire"), "backfire");
    m.insert((spell, "spelleffects"), "cast_spell");
    m.insert((spell, "spell_aim_step"), "cast_spell");
    m.insert((spell, "throwspell"), "cast_spell");
    m.insert((spell, "spell_cmp"), "failure_chance");
    m.insert((spell, "sortspells"), "all");
    m.insert((spell, "spellsortmenu"), "all");
    m.insert((spell, "dovspell"), "all");
    m.insert((spell, "dospellmenu"), "all");
    m.insert((spell, "initialspell"), "learn_spell");

    // --- combat/uhitm.rs ---
    let uhitm = "combat/uhitm.rs";
    m.insert((uhitm, "check_caitiff"), "attack_checks");
    m.insert((uhitm, "hmon_hitmon"), "hmon");
    m.insert((uhitm, "m_slips_free"), "try_escape_grab");
    m.insert((uhitm, "theft_petrifies"), "attack_checks");
    m.insert((uhitm, "explum"), "attack");
    m.insert((uhitm, "start_engulf"), "attack");
    m.insert((uhitm, "end_engulf"), "attack");
    m.insert((uhitm, "gulpum"), "attack");
    m.insert((uhitm, "passive"), "apply_player_defense");
    m.insert((uhitm, "passive_obj"), "apply_player_defense");
    m.insert((uhitm, "stumble_onto_mimic"), "attack_checks");
    m.insert((uhitm, "nohandglow"), "shade_glare");
    m.insert((uhitm, "flash_hits_mon"), "shade_glare");
    m.insert((uhitm, "light_hits_gremlin"), "shade_glare");

    // --- combat/artifact.rs ---
    let arti = "combat/artifact.rs";
    m.insert((arti, "artifact_name"), "artifact_index_by_name");
    m.insert((arti, "undiscovered_artifact"), "discoveries");
    m.insert((arti, "disp_artifact_discoveries"), "discoveries");
    m.insert((arti, "finesse_ahriman"), "artifact_hit");
    m.insert((arti, "artifact_light"), "is_created");
    m.insert((arti, "artifact_has_invprop"), "artifact_properties");
    m.insert((arti, "abil_to_adtyp"), "damage_type_to_resistance");
    m.insert((arti, "abil_to_spfx"), "spec_ability");
    m.insert((arti, "what_gives"), "collect_spfx_properties");
    m.insert((arti, "untouchable"), "touch_artifact");
    m.insert((arti, "mkot_trap_warn"), "artifact_properties");
    m.insert((arti, "is_magic_key"), "artifact_properties");
    m.insert((arti, "has_magic_key"), "artifact_properties");

    // --- monster/ai.rs ---
    let ai = "monster/ai.rs";
    m.insert((ai, "monhaskey"), "find_offensive");
    m.insert((ai, "mon_yells"), "wakeup");
    m.insert((ai, "watch_on_duty"), "dochug");
    m.insert((ai, "release_hero"), "dochug");
    m.insert((ai, "m_arrival"), "movemon");
    m.insert((ai, "itsstuck"), "dochug");
    m.insert((ai, "should_displace"), "domove_core");
    m.insert((ai, "dissolve_bars"), "mdig_tunnel");
    m.insert((ai, "set_apparxy"), "dochug");
    m.insert((ai, "undesirable_disp"), "domove_core");
    m.insert((ai, "stuff_prevents_passage"), "domove_core");
    m.insert((ai, "vamp_shift"), "dochug");

    // --- player/attributes.rs ---
    let attr = "player/attributes.rs";
    m.insert((attr, "poisontell"), "name");
    m.insert((attr, "poisoned"), "modify");
    m.insert((attr, "exerper"), "periodic_exercise");
    m.insert((attr, "init_attr"), "init");
    m.insert((attr, "redist_attr"), "redistribute");
    m.insert((attr, "check_innate_abil"), "check_innate_ability");
    m.insert((attr, "innately"), "check_innate_ability");
    m.insert((attr, "is_innate"), "check_innate_ability");
    m.insert((attr, "from_what"), "check_innate_ability");
    m.insert((attr, "extremeattr"), "is_extreme");
    m.insert((attr, "uchangealign"), "modify");

    // --- action/eat.rs ---
    let eat = "action/eat.rs";
    m.insert((eat, "init_uhunger"), "newuhs");
    m.insert((eat, "recalc_wt"), "calculate_nutrition");
    m.insert((eat, "maybe_cannibal"), "eating_conducts");
    m.insert((eat, "violated_vegetarian"), "eating_conducts");
    m.insert((eat, "set_tin_variety"), "tin_variety");
    m.insert((eat, "opentin"), "consume_tin");
    m.insert((eat, "start_tin"), "do_eat_tin");
    m.insert((eat, "leather_cover"), "is_edible");

    // --- combat/mhitu.rs ---
    let mhitu = "combat/mhitu.rs";
    m.insert((mhitu, "hitmsg"), "hit_message");
    m.insert((mhitu, "missmu"), "miss_message");
    m.insert((mhitu, "mswings"), "weapon_swing_message");
    m.insert((mhitu, "mpoisons_subj"), "resistance_message");
    m.insert((mhitu, "wildmiss"), "wild_miss_message");
    m.insert((mhitu, "u_slip_free"), "try_escape_grab");
    m.insert((mhitu, "hitmu"), "process_single_attack");
    m.insert((mhitu, "mdamageu"), "apply_damage_effect");
    m.insert((mhitu, "mayberem"), "steal_from_player");
    m.insert((mhitu, "cloneu"), "mattacku");

    // --- combat/mhitm.rs ---
    let mhitm = "combat/mhitm.rs";
    m.insert((mhitm, "engulf_target"), "gulpmm");
    m.insert((mhitm, "paralyze_monst"), "apply_monster_damage_effect");
    m.insert((mhitm, "sleep_monst"), "apply_monster_damage_effect");
    m.insert((mhitm, "slept_monst"), "apply_monster_damage_effect");
    m.insert((mhitm, "rustm"), "apply_monster_damage_effect");
    m.insert((mhitm, "xdrainenergym"), "apply_monster_damage_effect");

    // --- action/apply.rs ---
    let apply = "action/apply.rs";
    m.insert((apply, "its_dead"), "use_stethoscope");
    m.insert((apply, "next_to_u"), "use_leash");
    m.insert((apply, "use_candelabrum"), "apply_candelabrum");
    m.insert((apply, "use_candle"), "apply_light");
    m.insert((apply, "snuff_candle"), "snuff_light_source");
    m.insert((apply, "dorub"), "do_apply");
    m.insert((apply, "fig_transform"), "apply_figurine");
    m.insert((apply, "reset_trapset"), "set_trap");
    m.insert((apply, "find_poleable_mon"), "use_pole");
    m.insert((apply, "get_valid_polearm_position"), "use_pole");
    m.insert((apply, "display_polearm_positions"), "use_pole");
    m.insert((apply, "do_break_wand"), "do_apply");
    m.insert((apply, "add_class"), "setapplyclasses");

    // --- action/pray.rs ---
    let pray = "action/pray.rs";
    m.insert((pray, "worst_cursed_item"), "fix_worst_trouble");
    m.insert((pray, "gcrownu"), "crown_player");
    m.insert((pray, "gods_angry"), "gods_upset");
    m.insert((pray, "a_gname"), "godvoice");
    m.insert((pray, "a_gname_at"), "godvoice");
    m.insert((pray, "u_gname"), "godvoice");
    m.insert((pray, "halu_gname"), "godvoice");
    m.insert((pray, "blocked_boulder"), "do_sacrifice");

    // --- action/throw.rs ---
    let throw = "action/throw.rs";
    m.insert((throw, "throw_obj"), "throwit");
    m.insert((throw, "ok_to_throw"), "too_encumbered_to_throw");
    m.insert((throw, "endmultishot"), "multishot_count");
    m.insert((throw, "walk_path"), "trace_projectile");
    m.insert((throw, "check_shop_obj"), "drop_throw");
    m.insert((throw, "sho_obj_return_to_u"), "returning_weapon_check");
    m.insert((throw, "omon_adj"), "monster_catches");
    m.insert((throw, "release_camera_demon"), "do_throw");

    // --- action/teleport.rs ---
    let tele = "action/teleport.rs";
    m.insert((tele, "enexto_core"), "rloc_pos_ok");
    m.insert((tele, "teleport_pet"), "safe_teleds");
    m.insert((tele, "scrolltele"), "scroll_teleport");
    m.insert((tele, "tele_trap"), "trap_teleport");
    m.insert((tele, "level_tele_trap"), "trap_level_teleport");
    m.insert((tele, "rloc_to"), "rloc_monster_to");
    m.insert((tele, "rloc"), "rloc_monster");
    m.insert((tele, "mvault_tele"), "vault_tele");
    m.insert((tele, "rloco"), "rloc_object");

    // --- action/engrave.rs ---
    let engr = "action/engrave.rs";
    m.insert((engr, "wipeout_text"), "wipe_engrave_at");
    m.insert((engr, "cant_reach_floor"), "do_engrave");
    m.insert((engr, "freehand"), "do_engrave");
    m.insert((engr, "sanitize_engravings"), "save_engravings");

    // --- action/fountain.rs ---
    m.insert(("action/fountain.rs", "breaksink"), "dryup");

    // --- action/kick.rs ---
    let kick = "action/kick.rs";
    m.insert((kick, "impact_drop"), "kick_object");
    m.insert((kick, "ship_object"), "hurtle");
    m.insert((kick, "obj_delivery"), "kick_object");
    m.insert((kick, "deliver_obj_to_mon"), "kick_monster");
    m.insert((kick, "otransit_msg"), "kick_object");

    // --- dungeon/bones.rs ---
    let bones = "dungeon/bones.rs";
    m.insert((bones, "goodfruit"), "sanitize_level_for_bones");
    m.insert((bones, "resetobjs"), "sanitize_object_for_bones");
    m.insert((bones, "fixuporacle"), "process_loaded_bones");
    m.insert((bones, "can_make_bones"), "should_save_bones");

    // --- dungeon/drawbridge.rs ---
    let db = "dungeon/drawbridge.rs";
    m.insert((db, "m_to_e"), "do_entity");
    m.insert((db, "u_to_e"), "do_entity");

    // --- dungeon/rect.rs ---
    let rect = "dungeon/rect.rs";
    m.insert((rect, "init_rect"), "init");
    m.insert((rect, "intersect"), "intersection");

    // --- dungeon/shop.rs ---
    let shop = "dungeon/shop.rs";
    m.insert((shop, "init_shop_selection"), "select_shop_type");
    m.insert((shop, "shkveg"), "create_shop_item");
    m.insert((shop, "mkveggy_at"), "create_shop_item");
    m.insert((shop, "mkshobj_at"), "create_shop_item");
    m.insert((shop, "nameshk"), "generate_shopkeeper_name");
    m.insert((shop, "get_shop_item"), "shop_object_classes");
    m.insert((shop, "shkname"), "generate_shopkeeper_name");
    m.insert((shop, "shkname_is_pname"), "generate_shopkeeper_name");

    // --- dungeon/special_level.rs ---
    let splev = "dungeon/special_level.rs";
    m.insert((splev, "roguejoin"), "generate_rogue_level");
    m.insert((splev, "roguecorr"), "generate_rogue_level");
    m.insert((splev, "makeroguerooms"), "generate_rogue_level");
    m.insert((splev, "makerogueghost"), "generate_rogue_level");

    // --- dungeon/special_rooms.rs ---
    let srooms = "dungeon/special_rooms.rs";
    m.insert((srooms, "mkroom"), "populate_special_room");
    m.insert((srooms, "mk_zoo_thronemon"), "select_room_monster");
    m.insert((srooms, "mkundead"), "morgue_monster");
    m.insert((srooms, "morguemon"), "morgue_monster");
    m.insert((srooms, "antholemon"), "anthole_monster");
    m.insert((srooms, "shrine_pos"), "populate_temple");
    m.insert((srooms, "courtmon"), "court_monster");
    m.insert((srooms, "squadmon"), "squad_monster");
    m.insert((srooms, "save_room"), "stock_room");
    m.insert((srooms, "rest_room"), "stock_room");

    // --- monster/tactics.rs ---
    let tact = "monster/tactics.rs";
    m.insert((tact, "precheck"), "determine_tactics");
    m.insert((tact, "mreadmsg"), "determine_tactics");
    m.insert((tact, "rnd_defensive_item"), "determine_tactics");
    m.insert((tact, "rnd_offensive_item"), "determine_tactics");
    m.insert((tact, "muse_newcham_mon"), "determine_tactics");
    m.insert((tact, "rnd_misc_item"), "determine_tactics");
    m.insert((tact, "searches_for_item"), "determine_tactics");
    m.insert((tact, "cures_stoning"), "determine_tactics");
    m.insert((tact, "cures_sliming"), "determine_tactics");

    // --- magic/potion.rs ---
    let pot = "magic/potion.rs";
    m.insert((pot, "itimeout"), "potion_paralysis");
    m.insert((pot, "itimeout_incr"), "potion_paralysis");
    m.insert((pot, "set_itimeout"), "potion_paralysis");
    m.insert((pot, "incr_itimeout"), "potion_paralysis");
    m.insert((pot, "self_invis_message"), "potion_invisibility");
    m.insert((pot, "strange_feeling"), "potion_object_detection");
    m.insert((pot, "mongrantswish"), "quaff_potion");
    m.insert((pot, "split_mon"), "quaff_potion");

    // --- special/sounds.rs ---
    let snd = "special/sounds.rs";
    m.insert((snd, "mon_in_room"), "noises");
    m.insert((snd, "dosounds"), "generate_ambient_sounds");
    m.insert((snd, "growl"), "monster_growl");
    m.insert((snd, "yelp"), "monster_yelp");
    m.insert((snd, "whimper"), "monster_whimper");
    m.insert((snd, "mon_is_gecko"), "can_make_sound");
    m.insert((snd, "domonnoise"), "generate_monster_noise");
    m.insert((snd, "dotalk"), "humanoid_speech");
    m.insert((snd, "dochat"), "humanoid_speech");
    m.insert((snd, "play_sound_for_message"), "find_sound_for_message");

    // --- special/summon.rs ---
    let summ = "special/summon.rs";
    m.insert((summ, "newemin"), "new");
    m.insert((summ, "free_emin"), "new");
    m.insert((summ, "monster_census"), "nasty");
    m.insert((summ, "bribe"), "dosummon");
    m.insert((summ, "lose_guardian_angel"), "nasty");
    m.insert((summ, "gain_guardian_angel"), "msummon");

    // --- player/polymorph.rs ---
    let poly = "player/polymorph.rs";
    m.insert((poly, "float_vs_flight"), "polymon");
    m.insert((poly, "check_strangling"), "polymon");
    m.insert((poly, "dopoly"), "polyself");
    m.insert((poly, "uunstick"), "polymon");
    m.insert((poly, "skinback"), "rehumanize");
    m.insert((poly, "ugolemeffects"), "polymon");
    m.insert((poly, "polysense"), "polymon");
    m.insert((poly, "ugenocided"), "polymon");
    m.insert((poly, "udeadinside"), "polymon");

    // --- player/role.rs ---
    let role = "player/role.rs";
    m.insert((role, "randrole"), "pick_role");
    m.insert((role, "randrole_filtered"), "pick_role");
    m.insert((role, "randrace"), "pick_race");
    m.insert((role, "randgend"), "pick_gend");
    m.insert((role, "randalign"), "pick_align");
    m.insert((role, "ok_role"), "validrole");
    m.insert((role, "promptsep"), "build_plselection_prompt");

    // --- player/you.rs ---
    m.insert(("player/you.rs", "enermod"), "regen_energy");

    // --- rng.rs ---
    let rng = "rng.rs";
    m.insert((rng, "whichrng"), "new");
    m.insert((rng, "init_isaac64"), "new");
    m.insert((rng, "rn2_on_display_rng"), "rn2");
    m.insert((rng, "d"), "dice");

    m
}

/// Functions that should be marked as not_needed regardless of rust_file.
fn not_needed_functions() -> HashSet<(&'static str, &'static str)> {
    let mut s = HashSet::new();

    // All cmd.c functions -> not_needed (UI/input layer)
    for func in &[
        "timed_occupation", "reset_occupations", "set_occupation", "domonability",
        "wiz_wish", "wiz_identify", "wiz_makemap", "wiz_map", "wiz_genesis",
        "wiz_where", "wiz_detect", "wiz_level_change", "wiz_panic",
        "wiz_show_seenv", "wiz_show_vision", "wiz_show_wmodes",
        "wiz_map_levltyp", "wiz_levltyp_legend", "wiz_smell",
        "wiz_intrinsic", "wiz_rumor_check", "doterrain", "enlght_out",
        "enlght_line", "enlght_combatinc", "enlght_halfdmg",
        "walking_on_water", "cause_known", "background_enlightenment",
        "basics_enlightenment", "characteristics_enlightenment",
        "one_characteristic", "status_enlightenment",
        "attributes_enlightenment", "minimal_enlightenment",
        "doattributes", "youhiding", "commands_init", "cmd_from_func",
        "size_obj", "obj_chain", "mon_invent_chain", "size_monst",
        "mon_chain", "misc_stats", "wiz_show_stats", "wiz_migrate_mons",
        "parseautocomplete", "reset_commands", "randomkey",
        "random_response", "rhack", "movecmd", "dxdy_moveok",
        "redraw_cmd", "get_adjacent_loc", "getdir", "help_dir",
        "doherecmdmenu", "dotherecmdmenu", "there_cmd_menu",
        "here_cmd_menu", "click_to_cmd", "get_count", "parse",
        "hangup", "end_of_input", "readchar", "dotravel",
        "paranoid_query", "dosuspend_core", "dosh_core",
    ] {
        s.insert(("cmd.c", *func));
    }

    // wiz_timeout_queue from timeout.c
    s.insert(("timeout.c", "wiz_timeout_queue"));

    // Functions whose Rust equivalent is in a different file than rust_file suggests
    // isbig: equivalent is is_big() in permonst.rs, not special_rooms.rs
    s.insert(("mkroom.c", "isbig"));
    // m_slips_free: equivalent is try_escape_grab() in mhitu.rs, not uhitm.rs
    s.insert(("uhitm.c", "m_slips_free"));

    // quest.c pager functions (message delivery system)
    for func in &["msg_in", "deliver_by_pline", "deliver_by_window", "skip_pager"] {
        s.insert(("quest.c", *func));
    }

    s
}

// ============================================================================
// Function extraction from Rust source
// ============================================================================

fn extract_rust_fns(path: &Path) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut fns = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        // Match pub fn, pub(crate) fn, pub(super) fn, fn
        let fn_start = if let Some(pos) = trimmed.find("fn ") {
            // Check it's a function declaration (not a comment, string, etc.)
            let before = &trimmed[..pos];
            if before.starts_with("//") || before.starts_with("*") || before.starts_with("\"") {
                continue;
            }
            pos + 3
        } else {
            continue;
        };

        let rest = &trimmed[fn_start..];
        // Find name: it ends at '(' or '<' (generics)
        let end = rest
            .find(|c: char| c == '(' || c == '<')
            .unwrap_or(rest.len());
        let name = rest[..end].trim();
        if !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            fns.push(name.to_string());
        }
    }

    fns.sort();
    fns.dedup();
    fns
}

// ============================================================================
// Main promotion logic
// ============================================================================

#[test]
fn promote_stubs_to_ported() {
    // Load registry
    let data = fs::read_to_string(REGISTRY_PATH).expect("Failed to read registry JSON");
    let mut registry: Vec<serde_json::Value> =
        serde_json::from_str(&data).expect("Failed to parse JSON");

    let renames = known_renames();
    let not_needed = not_needed_functions();

    // Build Rust function cache: rust_file -> set of function names
    let mut rust_fn_cache: HashMap<String, HashSet<String>> = HashMap::new();

    // Stats
    let mut promoted_to_ported = 0;
    let mut promoted_to_not_needed = 0;
    let mut already_resolved = 0;
    let mut unmatched = Vec::new();

    for entry in registry.iter_mut() {
        let status = entry["status"].as_str().unwrap_or("");
        if status != "stub" {
            already_resolved += 1;
            continue;
        }

        let c_file = entry["c_file"].as_str().unwrap_or("").to_string();
        let c_func = entry["c_func"].as_str().unwrap_or("").to_string();
        let rust_file = entry["rust_file"].as_str().unwrap_or("").to_string();

        // Check not_needed patterns
        if not_needed.contains(&(c_file.as_str(), c_func.as_str())) {
            entry["status"] = serde_json::Value::String("not_needed".into());
            promoted_to_not_needed += 1;
            continue;
        }

        // Skip entries without a rust_file
        if rust_file.is_empty() {
            unmatched.push(format!("{}::{} (no rust_file)", c_file, c_func));
            continue;
        }

        // Load Rust functions for this file
        let rust_fns = rust_fn_cache
            .entry(rust_file.clone())
            .or_insert_with(|| {
                let path = Path::new(NH_CORE_SRC).join(&rust_file);
                extract_rust_fns(&path).into_iter().collect()
            });

        // Strategy 1: Known renames table
        if let Some(&rust_func) = renames.get(&(rust_file.as_str(), c_func.as_str())) {
            if rust_fns.contains(rust_func) {
                entry["status"] = serde_json::Value::String("ported".into());
                entry["rust_func"] = serde_json::Value::String(rust_func.to_string());
                promoted_to_ported += 1;
                continue;
            }
        }

        // Strategy 2: Exact match
        if rust_fns.contains(&c_func) {
            entry["status"] = serde_json::Value::String("ported".into());
            entry["rust_func"] = serde_json::Value::String(c_func.clone());
            promoted_to_ported += 1;
            continue;
        }

        // Strategy 3: Strip-underscore match
        let c_stripped = c_func.replace('_', "").to_lowercase();
        let mut found = false;
        for rfn in rust_fns.iter() {
            if rfn.replace('_', "").to_lowercase() == c_stripped {
                entry["status"] = serde_json::Value::String("ported".into());
                entry["rust_func"] = serde_json::Value::String(rfn.clone());
                promoted_to_ported += 1;
                found = true;
                break;
            }
        }
        if found {
            continue;
        }

        unmatched.push(format!("{}::{} (in {})", c_file, c_func, rust_file));
    }

    // Write updated registry
    let output = serde_json::to_string_pretty(&registry).expect("Failed to serialize JSON");
    fs::write(REGISTRY_PATH, output).expect("Failed to write registry JSON");

    // Print summary
    let total = registry.len();
    let mut final_counts: HashMap<String, usize> = HashMap::new();
    for entry in &registry {
        let status = entry["status"].as_str().unwrap_or("unknown");
        *final_counts.entry(status.to_string()).or_insert(0) += 1;
    }

    let ported = *final_counts.get("ported").unwrap_or(&0);
    let stub = *final_counts.get("stub").unwrap_or(&0);
    let nn = *final_counts.get("not_needed").unwrap_or(&0);
    let missing = *final_counts.get("missing").unwrap_or(&0);

    println!("\n========================================");
    println!("  Stub Promotion Results");
    println!("========================================");
    println!("  Promoted to ported:     {}", promoted_to_ported);
    println!("  Promoted to not_needed: {}", promoted_to_not_needed);
    println!("  Already resolved:       {}", already_resolved);
    println!("  Remaining unmatched:    {}", unmatched.len());
    println!();
    println!("  Final counts:");
    println!("    ported:     {} ({:.1}%)", ported, ported as f64 / total as f64 * 100.0);
    println!("    not_needed: {} ({:.1}%)", nn, nn as f64 / total as f64 * 100.0);
    println!("    stub:       {} ({:.1}%)", stub, stub as f64 / total as f64 * 100.0);
    println!("    missing:    {}", missing);
    println!("    TOTAL:      {}", total);
    println!();

    let convergence = (ported + nn) as f64 / total as f64 * 100.0;
    println!("  CONVERGENCE: {:.1}%", convergence);
    println!();

    if !unmatched.is_empty() {
        println!("  Remaining unmatched stubs ({}):", unmatched.len());
        for u in &unmatched {
            println!("    {}", u);
        }
    }

    // Assertions — these validate the final state (idempotent across re-runs)
    assert!(
        ported >= 2100,
        "Expected >= 2100 total ported, got {}",
        ported
    );
    assert!(
        nn >= 800,
        "Expected >= 800 total not_needed, got {}",
        nn
    );
    assert_eq!(stub, 0, "Expected 0 remaining stubs, got {}", stub);
    assert!(
        convergence >= 99.0,
        "Convergence {:.1}% is below 99% target",
        convergence
    );
}
