/*
 * NetHack FFI Header File
 * 
 * This header provides the C interface for NetHack FFI operations.
 */

#ifndef NH_FFI_H
#define NH_FFI_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Game state */
struct nh_ffi_game_state {
    int hp;
    int hp_max;
    int energy;
    int energy_max;
    int x;
    int y;
    int level;
    int experience_level;
    int armor_class;
    int gold;
    int strength;
    int dexterity;
    int constitution;
    int intelligence;
    int wisdom;
    int charisma;
    bool is_dead;
    int hunger_state;
    int turn_count;
    int dungeon_depth;
    int monster_count;
};

/* Inventory item */
struct nh_ffi_object {
    char name[128];
    char obj_class;
    int weight;
    int value;
    int quantity;
    int enchantment;
    bool cursed;
    bool blessed;
    int armor_class;
    int damage;
    char inv_letter;
};

/* Monster info */
struct nh_ffi_monster {
    char name[128];
    char symbol;
    int level;
    int hp;
    int max_hp;
    int armor_class;
    int x;
    int y;
    bool asleep;
    bool peaceful;
};

/* ============================================================================
 * Initialization and Cleanup
 * ============================================================================ */

/* Initialize the game with character creation */
int nh_ffi_init(const char* role, const char* race, int gender, int alignment);

/* Free game resources */
void nh_ffi_free(void);

/* Reset game to initial state */
int nh_ffi_reset(unsigned long seed);

/* ============================================================================
 * State Queries
 * ============================================================================ */

int nh_ffi_get_hp(void);
int nh_ffi_get_max_hp(void);
int nh_ffi_get_energy(void);
int nh_ffi_get_max_energy(void);
void nh_ffi_get_position(int* x, int* y);
int nh_ffi_get_armor_class(void);
int nh_ffi_get_gold(void);
int nh_ffi_get_experience_level(void);
int nh_ffi_get_current_level(void);
int nh_ffi_get_dungeon_depth(void);
unsigned long nh_ffi_get_turn_count(void);
bool nh_ffi_is_player_dead(void);

const char* nh_ffi_get_role(void);
const char* nh_ffi_get_race(void);
int nh_ffi_get_gender(void);
int nh_ffi_get_alignment(void);

/* ============================================================================
 * Command Execution
 * ============================================================================ */

int nh_ffi_exec_cmd(char cmd);
int nh_ffi_exec_cmd_dir(char cmd, int dx, int dy);

/* ============================================================================
 * State Serialization
 * ============================================================================ */

char* nh_ffi_get_state_json(void);
void nh_ffi_free_string(void* ptr);

/* ============================================================================
 * Message Log
 * ============================================================================ */

char* nh_ffi_get_last_message(void);

/* ============================================================================
 * Inventory Management
 * ============================================================================ */

int nh_ffi_get_inventory_count(void);
char* nh_ffi_get_inventory_json(void);

/* ============================================================================
 * Monster Information
 * ============================================================================ */

char* nh_ffi_get_nearby_monsters_json(void);
int nh_ffi_count_monsters(void);

/* ============================================================================
 * Game Status
 * ============================================================================ */

bool nh_ffi_is_game_over(void);
bool nh_ffi_is_game_won(void);
char* nh_ffi_get_result_message(void);

#ifdef __cplusplus
}
#endif

#endif /* NH_FFI_H */
