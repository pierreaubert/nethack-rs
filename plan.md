Plan: C-to-Rust NetHack Game Logic Convergence

 Context

 We're porting NetHack 3.6.7 (C) to Rust (nh-core) for WASM-compatible game logic. The UI (2D/3D) is deferred. 20 convergence phases (0-19) are already
 complete, yielding 2,621 unit tests and a working game loop. However, a function registry audit reveals 0 of 2,939 C functions are marked "ported", and
 416 TODOs remain in monster/ai.rs alone (~89% of all TODOs). The goal is provable feature parity with tests that verify behavior, not just existence.

 Current State (Measured)

 - C NetHack: ~192K LOC, ~2,277 public functions, 109 .c files
 - Rust nh-core: ~171K LOC, 2,621 tests passing
 - WASM: Already compiles (cargo check --target wasm32-unknown-unknown -p nh-core passes)
 - Function registry (nh-compare/data/c_function_registry.json): 2,418 stub, 399 missing, 122 not_needed, 0 ported
 - TODOs: 470 total (416 in monster/ai.rs, 54 scattered across 23 other files)
 - nh-compare: 17 test files, ~290 tests (7 passing, 1 failing)

 ---
 Convergence Score (tracked in nh-compare)

 Three dimensions, 100 points total:

 ┌────────────────────────┬───────────────────────────────────────────┬─────────┬────────┐
 │       Dimension        │                  Formula                  │ Current │ Target │
 ├────────────────────────┼───────────────────────────────────────────┼─────────┼────────┤
 │ Function Coverage (FC) │ (ported / (total - not_needed)) * 40      │ 0/40    │ 36/40  │
 ├────────────────────────┼───────────────────────────────────────────┼─────────┼────────┤
 │ Behavioral Tests (BTC) │ min(40, nh_compare_tests / 25)            │ 12/40   │ 40/40  │
 ├────────────────────────┼───────────────────────────────────────────┼─────────┼────────┤
 │ Structural (SC)        │ TODOs=0: 10pt, WASM: 5pt, 0 warnings: 5pt │ 5/20    │ 20/20  │
 ├────────────────────────┼───────────────────────────────────────────┼─────────┼────────┤
 │ Total                  │                                           │ ~17/100 │ 96/100 │
 └────────────────────────┴───────────────────────────────────────────┴─────────┴────────┘

 Each phase updates c_function_registry.json (promoting entries from stub/missing to ported) and adds behavioral tests to nh-compare. A test in nh-compare
  asserts the score never decreases.

 ---
 Phases 20-32

 Phase 20: Monster AI -- Ray Tracing and Wand Attacks

 The #1 gap: 416 TODOs in monster/ai.rs, mostly around wand/breath attacks.

 Implement:
 - buzz() in magic/zap.rs: full ray tracing with wall bounces, 8 ZapType variants, item destruction along path
 - mbhit(): monster-to-player beam (speed/sleep/poly beams without reflection)
 - Monster wand attack integration in monster/ai.rs: replace ~60 ray-related TODOs
 - Expand monster/item_usage.rs (203 LOC -> ~800 LOC): monster potion/scroll/wand selection and usage (C muse.c equivalent)

 Files: magic/zap.rs, monster/ai.rs, monster/item_usage.rs, combat/mod.rs

 Proof (10 tests in nh-compare):
 - test_buzz_fire_ray_hits_player -- fire ray applies damage
 - test_buzz_ray_stops_at_wall -- ray terminates at solid wall
 - test_buzz_ray_bounces -- diagonal ray reflects off wall
 - test_buzz_death_ray_kills -- death ray kills non-resistant target
 - test_buzz_cold_reflects_off_shield -- reflection bounces ray back
 - test_monster_selects_best_wand -- monster prefers death > fire > MM
 - test_monster_zaps_healing_on_self -- low-HP monster heals
 - test_monster_uses_scroll_teleport -- cornered monster teleports
 - test_mbhit_speed_beam -- speed beam changes target speed
 - test_monster_breath_weapon -- dragon breath uses correct type

 Gate: ai.rs TODO count < 350; item_usage.rs > 600 LOC; all 10 tests pass

 ---
 Phase 21: Monster AI -- Movement, Pathfinding, Special Behaviors

 Resolve the remaining ~300 TODOs in monster/ai.rs.

 Implement:
 - dochug()/m_move() completion: trap interaction, movement flags (ALLOW_WALL, ALLOW_WATER, OPEN_DOOR), position scoring, door interaction, tunnel/dig,
 displacement
 - Special AI: covetous monsters (Wizard seeking artifacts), worm segments, pet delegation to dog_move, shopkeeper/guard/priest movement delegation, tengu
  teleport
 - Awareness system: distfleece() awakening distance, stealth, aggravate, occupation detection
 - unblock_point() for terrain vision, grave disturbance, town guard reactions

 Files: monster/ai.rs, monster/tactics.rs, dungeon/level.rs

 Proof (10 tests in nh-compare):
 - test_monster_flees_when_low_hp
 - test_monster_trapped_in_pit
 - test_covetous_teleports_to_player
 - test_monster_opens_door
 - test_monster_breaks_locked_door
 - test_pet_follows_player
 - test_monster_picks_up_gold
 - test_monster_avoids_lava
 - test_town_guard_warns_vandal
 - test_monster_disturbs_grave

 Gate: ai.rs TODO count = 0; 500-turn stress tests still pass; all 10 tests pass

 ---
 Phase 22: Engulfed State and Reflection System

 Implement:
 - Engulfed state checks in all action dispatch paths: movement restricted, melee hits engulfer, zap hits engulfer, apply/eat/search restrictions,
 polymorph escape (size check)
 - Engulf/expel lifecycle verification in mhitu.rs
 - ureflects() in player/properties.rs: reflection from silver dragon scale mail, shield of reflection, amulet of reflection
 - Monster reflection from equipped items
 - Integrate reflection into buzz() and gaze attacks

 Files: gameloop.rs, combat/mhitu.rs, combat/uhitm.rs, player/properties.rs, action/movement.rs

 Proof (8 tests in nh-compare):
 - test_engulfed_cant_move_freely
 - test_engulfed_attacks_engulfer
 - test_expelled_when_engulfer_dies
 - test_reflect_magic_missile
 - test_reflect_death_ray
 - test_monster_reflect_gaze
 - test_polymorph_escapes_engulf
 - test_engulfed_zap_hits_engulfer

 Gate: player.swallowed checked in all action paths; all 8 tests pass

 ---
 Phase 23: Lock Picking, Trap Erosion, Equipment Damage

 Implement:
 - DC-based lock picking: tool quality (lock pick vs skeleton key vs credit card), cursed/blessed modifiers, skill rolls, multi-turn occupation, lock
 breaking
 - Chest traps (needle, gas, explosion)
 - Trap erosion: rust trap corrodes worn iron armor, water traps rust metal items, fire traps burn inventory, acid damage to equipment
 - Erosion levels (0-3) affecting AC; erosion-proofing from oilskin/greasing

 Files: action/open_close.rs, action/trap.rs, object/obj.rs

 Proof (10 tests in nh-compare):
 - test_pick_lock_skill_check -- success rate scales with tool+DEX
 - test_skeleton_key_better_than_lockpick
 - test_cursed_lockpick_can_break
 - test_force_chest_open
 - test_chest_trap_needle
 - test_rust_trap_corrodes_armor
 - test_elven_armor_resists_corrosion
 - test_fire_trap_burns_scrolls
 - test_erosion_reduces_ac
 - test_oilskin_resists_water

 Gate: Lock picking uses C probability table (statistical test, 1000 trials); all 10 tests pass

 ---
 Phase 24: Special Level Generation Fidelity

 Implement:
 - Verify/fix Sokoban levels (8 variants): boulder positions, prize items, no-teleport/no-dig flags
 - Mines Town: shop placement, temple, altar
 - Mines End: luckstone placement
 - Gehennom: Juiblex, Baalzebub, Asmodeus lairs; Wizard Tower (3 connected levels); Sanctum with high priest
 - Quest levels (per role): home, locate, goal with nemesis
 - Endgame planes: Astral (3 aligned altars), Earth/Air/Fire/Water

 Files: dungeon/special_level.rs, dungeon/endgame.rs, dungeon/quest.rs

 Proof (10 tests in nh-compare):
 - test_sokoban_1a_boulder_count
 - test_sokoban_prize_on_top
 - test_sokoban_no_teleport_flag
 - test_mines_town_has_shop
 - test_mines_end_has_luckstone
 - test_sanctum_has_high_priest
 - test_wizard_tower_connected
 - test_quest_goal_has_nemesis
 - test_astral_plane_three_altars
 - test_fire_plane_has_lava

 Gate: All 30+ special level variants generate without panic; 10 tests pass

 ---
 Phase 25: Vision/LOS System Fidelity

 Implement:
 - Room-based lighting: entering lit room reveals all cells (not circular raycasting)
 - Corridor darkness: 1-cell visibility without light source
 - Lamp/candelabrum extends corridor range
 - Infravision: see warm-blooded monsters in darkness
 - Telepathy: see monsters through walls when blind
 - See invisible: reveal invisible monsters
 - Underwater: very limited visibility

 Files: dungeon/level.rs (replace update_visibility), dungeon/room.rs, gameloop.rs

 Proof (7 tests in nh-compare):
 - test_lit_room_fully_visible
 - test_dark_room_limited_visibility
 - test_corridor_one_cell_visibility
 - test_infravision_sees_warm_monsters
 - test_telepathy_blind_sees_all
 - test_lamp_extends_corridor_range
 - test_see_invisible_reveals_stalker

 Gate: Room lighting matches C behavior; all 7 tests pass

 ---
 Phase 26: Timeout Effects and Multi-Turn Occupations

 Implement:
 - Verify all C timeout.c effect types: stoning (5 turns), sliming, illness, egg hatching, lamp burning, corpse aging, fumbling, wounded legs
 - Multi-turn occupation system: eating heavy food, digging walls, lock picking, extended search all take >1 turn with player.multi
 - Occupation interruption: monster attack cancels current occupation

 Files: world/timeout.rs, gameloop.rs (occupation in tick()), action/eat.rs, action/dig.rs

 Proof (10 tests in nh-compare):
 - test_stoning_countdown_5_turns
 - test_stoning_cured_by_lizard
 - test_sliming_cured_by_fire
 - test_egg_hatches_after_timeout
 - test_lamp_burns_out
 - test_eating_takes_multiple_turns
 - test_occupation_interrupted_by_attack
 - test_corpse_rots_after_timeout
 - test_illness_kills_without_cure
 - test_fumbling_causes_trip

 Gate: All C timeout types have Rust equivalents; occupation system works; all 10 tests pass

 ---
 Phase 27: Combat Edge Cases and Advanced Interactions

 Implement:
 - Silver damage: bonus to undead/demons/werecreatures
 - Artifact invocation effects for each artifact
 - Passive attacks: acid blob damages attacker, cockatrice stoning on touch
 - Two-weapon combat completion
 - Riding combat adjustments
 - Thrown weapon specials: Mjollnir returns, Grimtooth poison

 Files: combat/uhitm.rs, combat/artifact.rs, action/throw.rs

 Proof (15 tests in nh-compare):
 - Silver weapon tests (3), artifact invocation tests (3), passive attack tests (3), two-weapon tests (2), riding combat tests (2), thrown special tests
 (2)

 Gate: All combat TODOs resolved; all 15 tests pass

 ---
 Phase 28: Naming, Identification, and Object Interactions

 Implement:
 - Artifact creation from naming (naming "Sting" on elven dagger creates artifact, once per game)
 - Price identification in shops
 - Formal vs use-testing identification
 - Bag of holding weight reduction, bag of tricks, cancel bag of holding = explosion

 Files: action/name.rs, magic/identification.rs, object/container.rs

 Proof (12 tests in nh-compare):
 - Artifact naming tests (4), identification tests (4), container tests (4)

 Gate: Artifact naming works per C rules; all 12 tests pass

 ---
 Phase 29: Function Registry Promotion Sprint

 Goal: Systematically audit and promote "stub"/"missing" entries to "ported" by verifying implementations match C behavior.

 Process (by C file, descending stub count):
 1. sp_lev.c (131 stubs) - mark C-specific parser functions "not_needed", verify level gen
 2. cmd.c (109 stubs) - verify command dispatch coverage
 3. shk.c (106 stubs) - verify shop system coverage
 4. invent.c (95 stubs) - verify inventory operations
 5. mon.c (82 stubs) - verify monster lifecycle
 6. Mark display/TTY/platform functions "not_needed" (~200 entries)

 Files: nh-compare/data/c_function_registry.json, various nh-core files for gap-fills

 Proof (50 behavioral tests, one per major promoted function):
 - Each promoted function has at least one test verifying behavior matches C

 Gate: >500 entries promoted to "ported"; >200 marked "not_needed"; 50 new tests pass; convergence score FC > 30

 ---
 Phase 30: Remaining Scattered TODOs

 Resolve all non-ai.rs TODOs (54 across 23 files):
 - combat/uhitm.rs (6): armor-based defense, monster reflection
 - action/pray.rs (6): monster type checks
 - monster/casting.rs (6): spell casting edge cases
 - monster/mod.rs (5): minor state checks
 - gameloop.rs (4): minor edge cases
 - All others (1-3 each): search/engulfed, trap/rust, level_change/trap, dig/stuck

 Proof: stub_audit.rs reports 0 TODOs; gate test asserts total_todos == 0

 ---
 Phase 31: Save/Restore Completeness and Multi-Level Persistence

 Implement:
 - Verify multi-level save round-trip (all Level fields survive)
 - Bones file loading on level entry (integrate with generation)
 - Ghost monster from dead player's bones
 - WASM storage abstraction (localStorage/IndexedDB via trait)

 Files: save.rs, gameloop.rs, dungeon/bones.rs

 Proof (8 tests):
 - Multi-level save/restore round-trip tests (4), bones tests (2), WASM storage tests (2)

 Gate: Save/load preserves all state across multiple levels; all 8 tests pass

 ---
 Phase 32: Final Audit and Warning Cleanup

 Goal: Reach 95+ convergence score.

 Implement:
 - Final registry pass: audit remaining stub/missing entries
 - Fix all compiler warnings (currently 450)
 - 2000-turn stress test across 50 seeds
 - Determinism verification at turn boundaries

 Proof:
 - cargo clippy -p nh-core produces 0 errors
 - cargo check --target wasm32-unknown-unknown -p nh-core passes
 - Convergence score >= 95/100
 - 2000-turn x 50 seeds with no panics

 Gate: All metrics green; convergence score test passes with >= 95

 ---
 Phase Dependencies

 Phase 20 (Monster AI: Rays)
     |
     +---> Phase 21 (Monster AI: Movement)
     |         |
     |         +---> Phase 29 (Registry Sprint)
     |                   |
     |                   +---> Phase 32 (Final Audit)
     |
     +---> Phase 22 (Engulf & Reflection)
               |
               +---> Phase 27 (Combat Edge Cases)

 Independent (any order): Phase 23, 24, 25, 26, 28, 30, 31

 Summary Table

 ┌───────┬────────────────────────┬────────────┬──────────────────────────────┐
 │ Phase │      Description       │ Est. Tests │          Key Metric          │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 20    │ Monster AI: Rays/Wands │ +10        │ ai.rs TODOs < 350            │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 21    │ Monster AI: Movement   │ +10        │ ai.rs TODOs = 0              │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 22    │ Engulf & Reflection    │ +8         │ Engulf checks in all actions │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 23    │ Locks & Erosion        │ +10        │ Lock DC matches C            │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 24    │ Special Levels         │ +10        │ All special levels generate  │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 25    │ Vision/LOS             │ +7         │ Room lighting matches C      │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 26    │ Timeouts & Occupations │ +10        │ All C timeout types present  │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 27    │ Combat Edge Cases      │ +15        │ All combat TODOs gone        │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 28    │ Naming & Objects       │ +12        │ Artifact naming works        │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 29    │ Registry Sprint        │ +50        │ >500 entries "ported"        │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 30    │ Scattered TODOs        │ +0         │ Total TODOs = 0              │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 31    │ Save/Restore           │ +8         │ Multi-level round-trip       │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ 32    │ Final Audit            │ +10        │ Score >= 95/100              │
 ├───────┼────────────────────────┼────────────┼──────────────────────────────┤
 │ Total │                        │ +160       │ Convergence >= 95%           │
 └───────┴────────────────────────┴────────────┴──────────────────────────────┘
