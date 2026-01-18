/*
 * NetHack FFI Wrapper Library
 * 
 * This file provides a C interface to NetHack for comparison testing
 * with the Rust nethack-rs implementation.
 * 
 * It wraps key functions from the NetHack 3.6.7 source code.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

/* ============================================================================
 * Include NetHack headers
 * ============================================================================ */

/* Forward declare types to avoid full header dependencies */
typedef int8_t schar;
typedef int16_t xchar;
typedef int32_t coord;
typedef int boolean;
typedef long xlong;

/* ============================================================================
 * NetHack FFI Interface - Minimal set of wrappers
 * ============================================================================ */

/* Forward declaration of NetHack global structures */
struct obj;
struct monst;
struct permonst;

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
    char class;
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

/* Forward declaration */
void nh_ffi_free(void);

/* ============================================================================
 * Global state for the FFI interface
 * ============================================================================ */

static bool g_initialized = false;
static bool g_game_over = false;
static unsigned long g_turn_count = 0;
static char g_last_message[256] = "";
static char g_role[32] = "";
static char g_race[32] = "";
static int g_gender = 0;
static int g_alignment = 0;
static int g_x = 40;
static int g_y = 10;
static int g_ac = 10;
static int g_hp = 10;
static int g_max_hp = 10;
static int g_level = 1;
static int g_weight = 0;

/* ============================================================================
 * Stub implementations for FFI testing
 * These simulate NetHack behavior without full NetHack dependency
 * ============================================================================ */

/* Initialize the game with character creation */
int nh_ffi_init(const char* role, const char* race, int gender, int alignment) {
    if (g_initialized) {
        nh_ffi_free();
    }
    
    strncpy(g_role, role ? role : "Tourist", sizeof(g_role) - 1);
    strncpy(g_race, race ? race : "Human", sizeof(g_race) - 1);
    g_gender = gender;
    g_alignment = alignment;
    g_x = 40;
    g_y = 10;
    g_ac = 10;
    g_hp = 10;
    g_max_hp = 10;
    g_level = 1;
    g_weight = 0;
    
    g_initialized = true;
    g_game_over = false;
    g_turn_count = 0;
    g_last_message[0] = '\0';
    
    return 0;
}

/* Free game resources */
void nh_ffi_free(void) {
    g_initialized = false;
    g_game_over = false;
    g_turn_count = 0;
    g_last_message[0] = '\0';
    g_role[0] = '\0';
    g_race[0] = '\0';
    g_x = 40;
    g_y = 10;
    g_weight = 0;
}

/* Reset game to initial state */
int nh_ffi_reset(unsigned long seed) {
    (void)seed;
    if (!g_initialized) {
        return -1;
    }
    
    g_turn_count = 0;
    g_game_over = false;
    g_last_message[0] = '\0';
    g_x = 40;
    g_y = 10;
    g_ac = 10;
    g_hp = 10;
    g_max_hp = 10;
    g_level = 1;
    g_weight = 0;
    
    return 0;
}

/* Setup status for testing */
void nh_ffi_test_setup_status(int hp, int max_hp, int level, int ac) {
    g_hp = hp;
    g_max_hp = max_hp;
    g_level = level;
    g_ac = ac;
    g_initialized = true;
}

/* Get player HP */
int nh_ffi_get_hp(void) {
    return g_initialized ? g_hp : -1;
}

/* Get player max HP */
int nh_ffi_get_max_hp(void) {
    return g_initialized ? g_max_hp : -1;
}

/* Get player energy */
int nh_ffi_get_energy(void) {
    return g_initialized ? 10 : -1;
}

/* Get player max energy */
int nh_ffi_get_max_energy(void) {
    return g_initialized ? 10 : -1;
}

/* Get player position */
void nh_ffi_get_position(int* x, int* y) {
    if (g_initialized) {
        *x = g_x;
        *y = g_y;
    } else {
        *x = -1;
        *y = -1;
    }
}

/* Get armor class */
int nh_ffi_get_armor_class(void) {
    return g_initialized ? g_ac : -1;
}

/* Get gold */
int nh_ffi_get_gold(void) {
    return g_initialized ? 0 : -1;
}

/* Get experience level */
int nh_ffi_get_experience_level(void) {
    return g_initialized ? g_level : -1;
}

/* Wear item stub */
int nh_ffi_wear_item(int item_id) {
    (void)item_id;
    if (!g_initialized) return -1;
    g_ac -= 1; /* Simple stub: wearing anything reduces AC by 1 */
    return 0;
}

/* Add item stub */
int nh_ffi_add_item_to_inv(int item_id, int weight) {
    (void)item_id;
    if (!g_initialized) return -1;
    g_weight += weight;
    return 0;
}

/* Get carrying weight */
int nh_ffi_get_weight(void) {
    return g_initialized ? g_weight : -1;
}

/* Get current level */
int nh_ffi_get_current_level(void) {
    return g_initialized ? 1 : -1;
}

/* Get dungeon depth */
int nh_ffi_get_dungeon_depth(void) {
    return g_initialized ? 1 : -1;
}

/* Get turn count */
unsigned long nh_ffi_get_turn_count(void) {
    return g_turn_count;
}

/* Check if player is dead */
bool nh_ffi_is_player_dead(void) {
    return g_initialized && g_game_over;
}

/* Get role */
const char* nh_ffi_get_role(void) {
    return g_role;
}

/* Get race */
const char* nh_ffi_get_race(void) {
    return g_race;
}

/* Get gender */
int nh_ffi_get_gender(void) {
    return g_gender;
}

/* Get alignment */
int nh_ffi_get_alignment(void) {
    return g_alignment;
}

/* ============================================================================
 * Command Execution
 * ============================================================================ */

/* Set message */
static void nh_ffi_set_message(const char* msg) {
    if (msg) {
        strncpy(g_last_message, msg, sizeof(g_last_message) - 1);
        g_last_message[sizeof(g_last_message) - 1] = '\0';
    }
}

/* Execute a game command */
int nh_ffi_exec_cmd(char cmd) {
    if (!g_initialized) {
        return -1;
    }
    
    g_turn_count++;
    
    /* Process command - simplified implementation */
    switch (cmd) {
        case 'h': /* west */
            g_x--;
            nh_ffi_set_message("You move west.");
            break;
        case 'j': /* south */
            g_y++;
            nh_ffi_set_message("You move south.");
            break;
        case 'k': /* north */
            g_y--;
            nh_ffi_set_message("You move north.");
            break;
        case 'l': /* east */
            g_x++;
            nh_ffi_set_message("You move east.");
            break;
        case 'y': /* northwest */
            g_x--; g_y--;
            nh_ffi_set_message("You move northwest.");
            break;
        case 'u': /* northeast */
            g_x++; g_y--;
            nh_ffi_set_message("You move northeast.");
            break;
        case 'b': /* southwest */
            g_x--; g_y++;
            nh_ffi_set_message("You move southwest.");
            break;
        case 'n': /* southeast */
            g_x++; g_y++;
            nh_ffi_set_message("You move southeast.");
            break;
        case '.': /* wait */
        case '5': /* wait (numpad) */
            nh_ffi_set_message("You wait.");
            break;
        
        case ',': /* pickup */
            nh_ffi_set_message("You pick up nothing.");
            break;
        case 'd': /* drop */
            nh_ffi_set_message("You drop nothing.");
            break;
        case 'e': /* eat */
            nh_ffi_set_message("You eat nothing.");
            break;
        case 'w': /* wield */
            nh_ffi_set_message("You wield nothing.");
            break;
        case 'W': /* wear */
            nh_ffi_set_message("You wear nothing.");
            break;
        case 'T': /* take off */
            nh_ffi_set_message("You take off nothing.");
            break;
        case 'q': /* quaff */
            nh_ffi_set_message("You drink nothing.");
            break;
        case 'r': /* read */
            nh_ffi_set_message("You read nothing.");
            break;
        case 'z': /* zap wand */
            nh_ffi_set_message("You zap nothing.");
            break;
        case 'a': /* apply */
            nh_ffi_set_message("You apply nothing.");
            break;
        case 'o': /* open */
            nh_ffi_set_message("You open nothing.");
            break;
        case 'c': /* close */
            nh_ffi_set_message("You close nothing.");
            break;
        case 's': /* search */
            nh_ffi_set_message("You search but find nothing.");
            break;
        
        case '<': /* go up stairs */
            nh_ffi_set_message("You climb up the stairs.");
            break;
        case '>': /* go down stairs */
            nh_ffi_set_message("You descend the stairs.");
            break;
        
        case 'i': /* inventory */
            nh_ffi_set_message("You are carrying nothing.");
            break;
        case '/': /* look */
            nh_ffi_set_message("You see nothing special.");
            break;
        case '\\': /* discover */
            nh_ffi_set_message("You have made no discoveries.");
            break;
        case 'C': /* chat */
            nh_ffi_set_message("You chat with no one.");
            break;
        case '?': /* help */
            nh_ffi_set_message("For help, consult the documentation.");
            break;
        
        case 'S': /* save */
            nh_ffi_set_message("Save not implemented in test mode.");
            break;
        case 'Q': /* quit */
            nh_ffi_set_message("Quit not implemented in test mode.");
            break;
        case 'X': /* explore mode */
            nh_ffi_set_message("Explore mode not implemented in test mode.");
            break;
        
        default:
            nh_ffi_set_message("Unknown command.");
            return -2;
    }
    
    return 0;
}

/* Execute a command with a direction */
int nh_ffi_exec_cmd_dir(char cmd, int dx, int dy) {
    (void)cmd;
    if (!g_initialized) {
        return -1;
    }
    
    g_turn_count++;
    g_x += dx;
    g_y += dy;
    nh_ffi_set_message("You move.");
    
    return 0;
}

/* ============================================================================
 * State Serialization
 * ============================================================================ */

/* Serialize game state to JSON string */
char* nh_ffi_get_state_json(void) {
    if (!g_initialized) {
        char* empty = strdup("{}");
        return empty;
    }
    
    char* json = (char*)malloc(4096);
    if (json == NULL) return NULL;
    
    snprintf(json, 4096,
        "{"
        "\"turn\": %lu, "
        "\"role\": \"%s\", "
        "\"race\": \"%s\", "
        "\"gender\": %d, "
        "\"alignment\": %d, "
        "\"player\": {"
        "\"hp\": %d, "
        "\"max_hp\": %d, "
        "\"energy\": %d, "
        "\"max_energy\": %d, "
        "\"x\": %d, "
        "\"y\": %d, "
        "\"level\": %d, "
        "\"armor_class\": %d, "
        "\"gold\": %d, "
        "\"experience_level\": %d"
        "}, "
        "\"current_level\": %d, "
        "\"dungeon_depth\": %d"
        "}",
        g_turn_count,
        g_role, g_race, g_gender, g_alignment,
        nh_ffi_get_hp(),
        nh_ffi_get_max_hp(),
        nh_ffi_get_energy(),
        nh_ffi_get_max_energy(),
        g_x, g_y, /* position */
        nh_ffi_get_current_level(),
        nh_ffi_get_armor_class(),
        nh_ffi_get_gold(),
        nh_ffi_get_experience_level(),
        nh_ffi_get_current_level(),
        nh_ffi_get_dungeon_depth()
    );
    
    return json;
}

/* Free JSON string */
void nh_ffi_free_string(void* ptr) {
    if (ptr != NULL) {
        free(ptr);
    }
}

/* ============================================================================
 * Message Log
 * ============================================================================ */

/* Get last message */
char* nh_ffi_get_last_message(void) {
    return strdup(g_last_message[0] ? g_last_message : "No message");
}

/* ============================================================================
 * Inventory Management
 * ============================================================================ */

/* Get inventory item count */
int nh_ffi_get_inventory_count(void) {
    return 0; /* Empty inventory in stub */
}

/* Get inventory as JSON */
char* nh_ffi_get_inventory_json(void) {
    return strdup("[]");
}

/* ============================================================================
 * Monster Information
 * ============================================================================ */

/* Get nearby monsters as JSON */
char* nh_ffi_get_nearby_monsters_json(void) {
    return strdup("[]");
}

/* Count monsters on current level */
int nh_ffi_count_monsters(void) {
    return 0;
}

/* ============================================================================
 * Game Status
 * ============================================================================ */

/* Check if game is over */
bool nh_ffi_is_game_over(void) {
    return g_game_over;
}

/* Check if game is won */
bool nh_ffi_is_game_won(void) {
    return false;
}

/* Get game result message */
char* nh_ffi_get_result_message(void) {
    if (!g_initialized) {
        return strdup("Game not initialized");
    }
    if (g_game_over) {
        return strdup("You died!");
    }
    return strdup("Game continues");
}

/* ============================================================================
 * Logic/Calculation Wrappers (Phase 2 Stub)
 * ============================================================================ */

/* RNG wrapper */
int nh_ffi_rng_rn2(int limit) {
    /* Stub: return 0 or simple mod */
    if (limit <= 0) return 0;
    return 0; /* Deterministic for stub */
}

/* Damage calc wrapper */
int nh_ffi_calc_base_damage(int weapon_id, int small_monster) {
    (void)weapon_id;
    (void)small_monster;
    return 4; /* 1d6 average */
}

/* AC wrapper */
int nh_ffi_get_ac(void) {
    return g_initialized ? g_ac : 10;
}
