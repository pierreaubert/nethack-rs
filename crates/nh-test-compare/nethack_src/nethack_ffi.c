#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

#ifdef REAL_NETHACK
#include "hack.h"

/* Missing symbols from unixmain.c that we need to provide since we skip it */
short ospeed = 0;

boolean check_user_string(char *optstr) {
    (void)optstr;
    return FALSE;
}

void sethanguphandler(void (*fn)(int)) {
    (void)fn;
}

unsigned long sys_random_seed(void) {
    return 42; /* Constant for testing */
}

#else
/* Stub types if not using real NetHack */
typedef int8_t schar;
typedef int16_t xchar;
typedef int32_t coord;
typedef int boolean;
typedef long xlong;
#define TRUE 1
#define FALSE 0
#endif

/* ============================================================================
 * NetHack FFI Interface - Minimal set of wrappers
 * ============================================================================ */

/* Forward declaration of NetHack global structures */
struct obj;
struct monst;
struct permonst;

#ifndef REAL_NETHACK
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
    int is_dead;
    int hunger_state;
    int turn_count;
    int dungeon_depth;
    int monster_count;
};
#endif

/* Inventory item */
struct nh_ffi_object {
    char name[128];
    char class;
    int weight;
    int value;
    int quantity;
    int enchantment;
    boolean cursed;
    boolean blessed;
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
    boolean asleep;
    boolean peaceful;
};

/* Forward declaration */
void nh_ffi_free(void);

/* ============================================================================
 * Global state for the FFI interface
 * ============================================================================ */

#ifndef REAL_NETHACK
static boolean g_initialized = FALSE;
static boolean g_game_over = FALSE;
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
#else
static int g_weight_bonus = 0;
#endif

/* ============================================================================
 * Implementations
 * ============================================================================ */

/* Initialize the game with character creation */
int nh_ffi_init(const char* role, const char* race, int gender, int alignment) {
#ifdef REAL_NETHACK
    /* Minimal initialization of real NetHack globals for testing */
    u.uhp = u.uhpmax = 10;
    u.uen = u.uenmax = 10;
    u.ux = 40;
    u.uy = 10;
    u.uac = 10;
    u.ulevel = 1;
    u.umoney0 = 0;
    u.uz.dlevel = 1;
    moves = 0;
    flags.female = (gender > 0);
    u.ualign.type = alignment;

    /* These are normally set by u_init() */
    urole.name.m = role ? strdup(role) : strdup("Tourist");
    urace.noun = race ? strdup(race) : strdup("Human");

    return 0;
#else
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
    
    g_initialized = TRUE;
    g_game_over = FALSE;
    g_turn_count = 0;
    g_last_message[0] = '\0';
    
    return 0;
#endif
}

/* Free game resources */
void nh_ffi_free(void) {
#ifdef REAL_NETHACK
#else
    g_initialized = FALSE;
    g_game_over = FALSE;
    g_turn_count = 0;
    g_last_message[0] = '\0';
    g_role[0] = '\0';
    g_race[0] = '\0';
    g_x = 40;
    g_y = 10;
    g_weight = 0;
#endif
}

/* Reset game to initial state */
int nh_ffi_reset(unsigned long seed) {
#ifdef REAL_NETHACK
    (void)seed;
    return nh_ffi_init("Tourist", "Human", 0, 0);
#else
    (void)seed;
    if (!g_initialized) {
        return -1;
    }
    
    g_turn_count = 0;
    g_game_over = FALSE;
    g_last_message[0] = '\0';
    g_x = 40;
    g_y = 10;
    g_ac = 10;
    g_hp = 10;
    g_max_hp = 10;
    g_level = 1;
    g_weight = 0;
    
    return 0;
#endif
}

/* Setup status for testing */
void nh_ffi_test_setup_status(int hp, int max_hp, int level, int ac) {
#ifdef REAL_NETHACK
    u.uhp = hp;
    u.uhpmax = max_hp;
    u.ulevel = level;
    u.uac = ac;
#else
    g_hp = hp;
    g_max_hp = max_hp;
    g_level = level;
    g_ac = ac;
    g_initialized = TRUE;
#endif
}

/* Get player HP */
int nh_ffi_get_hp(void) {
#ifdef REAL_NETHACK
    return u.uhp;
#else
    return g_initialized ? g_hp : -1;
#endif
}

/* Get player max HP */
int nh_ffi_get_max_hp(void) {
#ifdef REAL_NETHACK
    return u.uhpmax;
#else
    return g_initialized ? g_max_hp : -1;
#endif
}

/* Get player energy */
int nh_ffi_get_energy(void) {
#ifdef REAL_NETHACK
    return u.uen;
#else
    return g_initialized ? 10 : -1;
#endif
}

/* Get player max energy */
int nh_ffi_get_max_energy(void) {
#ifdef REAL_NETHACK
    return u.uenmax;
#else
    return g_initialized ? 10 : -1;
#endif
}

/* Get player position */
void nh_ffi_get_position(int* x, int* y) {
#ifdef REAL_NETHACK
    *x = u.ux;
    *y = u.uy;
#else
    if (g_initialized) {
        *x = g_x;
        *y = g_y;
    } else {
        *x = -1;
        *y = -1;
    }
#endif
}

/* Get armor class */
int nh_ffi_get_armor_class(void) {
#ifdef REAL_NETHACK
    return u.uac;
#else
    return g_initialized ? g_ac : -1;
#endif
}

/* Get gold */
int nh_ffi_get_gold(void) {
#ifdef REAL_NETHACK
    return (int)u.umoney0;
#else
    return g_initialized ? 0 : -1;
#endif
}

/* Get experience level */
int nh_ffi_get_experience_level(void) {
#ifdef REAL_NETHACK
    return u.ulevel;
#else
    return g_initialized ? g_level : -1;
#endif
}

/* Wear item stub */
int nh_ffi_wear_item(int item_id) {
#ifdef REAL_NETHACK
    (void)item_id;
    u.uac -= 1;
    return 0;
#else
    (void)item_id;
    if (!g_initialized) return -1;
    g_ac -= 1;
    return 0;
#endif
}

/* Add item stub */
int nh_ffi_add_item_to_inv(int item_id, int weight) {
#ifdef REAL_NETHACK
    (void)item_id;
    g_weight_bonus += weight;
    return 0;
#else
    (void)item_id;
    if (!g_initialized) return -1;
    g_weight += weight;
    return 0;
#endif
}

/* Get carrying weight */
int nh_ffi_get_weight(void) {
#ifdef REAL_NETHACK
    return g_weight_bonus; /* For now, use our tracker */
#else
    return g_initialized ? g_weight : -1;
#endif
}

/* Get current level */
int nh_ffi_get_current_level(void) {
#ifdef REAL_NETHACK
    return (int)u.uz.dlevel;
#else
    return g_initialized ? 1 : -1;
#endif
}

/* Get dungeon depth */
int nh_ffi_get_dungeon_depth(void) {
#ifdef REAL_NETHACK
    return (int)depth(&u.uz);
#else
    return g_initialized ? 1 : -1;
#endif
}

/* Get turn count */
unsigned long nh_ffi_get_turn_count(void) {
#ifdef REAL_NETHACK
    return (unsigned long)moves;
#else
    return g_turn_count;
#endif
}

/* Check if player is dead */
boolean nh_ffi_is_player_dead(void) {
#ifdef REAL_NETHACK
    return FALSE;
#else
    return g_initialized && g_game_over;
#endif
}

/* Get role */
const char* nh_ffi_get_role(void) {
#ifdef REAL_NETHACK
    return urole.name.m;
#else
    return g_role;
#endif
}

/* Get race */
const char* nh_ffi_get_race(void) {
#ifdef REAL_NETHACK
    return urace.noun;
#else
    return g_race;
#endif
}

/* Get gender */
int nh_ffi_get_gender(void) {
#ifdef REAL_NETHACK
    return flags.female ? 1 : 0;
#else
    return g_gender;
#endif
}

/* Get alignment */
int nh_ffi_get_alignment(void) {
#ifdef REAL_NETHACK
    return (int)u.ualign.type;
#else
    return g_alignment;
#endif
}

/* ============================================================================
 * Command Execution
 * ============================================================================ */

/* Set message */
static void nh_ffi_set_message(const char* msg) {
#ifndef REAL_NETHACK
    if (msg) {
        strncpy(g_last_message, msg, sizeof(g_last_message) - 1);
        g_last_message[sizeof(g_last_message) - 1] = '\0';
    }
#else
    (void)msg;
#endif
}

/* Execute a game command */
int nh_ffi_exec_cmd(char cmd) {
#ifdef REAL_NETHACK
    /* Simplified movement for testing */
    switch (cmd) {
        case 'h': u.ux--; break;
        case 'j': u.uy++; break;
        case 'k': u.uy--; break;
        case 'l': u.ux++; break;
        case '.': break;
        default: return -1;
    }
    moves++;
    return 0;
#else
    if (!g_initialized) {
        return -1;
    }
    
    g_turn_count++;
    
    switch (cmd) {
        case 'h': g_x--; nh_ffi_set_message("You move west."); break;
        case 'j': g_y++; nh_ffi_set_message("You move south."); break;
        case 'k': g_y--; nh_ffi_set_message("You move north."); break;
        case 'l': g_x++; nh_ffi_set_message("You move east."); break;
        case 'y': g_x--; g_y--; nh_ffi_set_message("You move northwest."); break;
        case 'u': g_x++; g_y--; nh_ffi_set_message("You move northeast."); break;
        case 'b': g_x--; g_y++; nh_ffi_set_message("You move southwest."); break;
        case 'n': g_x++; g_y++; nh_ffi_set_message("You move southeast."); break;
        case '.': case '5': nh_ffi_set_message("You wait."); break;
        case ',': nh_ffi_set_message("You pick up nothing."); break;
        case 'd': nh_ffi_set_message("You drop nothing."); break;
        case 'e': nh_ffi_set_message("You eat nothing."); break;
        case 'w': nh_ffi_set_message("You wield nothing."); break;
        case 'W': nh_ffi_set_message("You wear nothing."); break;
        case 'T': nh_ffi_set_message("You take off nothing."); break;
        case 'q': nh_ffi_set_message("You drink nothing."); break;
        case 'r': nh_ffi_set_message("You read nothing."); break;
        case 'z': nh_ffi_set_message("You zap nothing."); break;
        case 'a': nh_ffi_set_message("You apply nothing."); break;
        case 'o': nh_ffi_set_message("You open nothing."); break;
        case 'c': nh_ffi_set_message("You close nothing."); break;
        case 's': nh_ffi_set_message("You search but find nothing."); break;
        case '<': nh_ffi_set_message("You climb up the stairs."); break;
        case '>': nh_ffi_set_message("You descend the stairs."); break;
        case 'i': nh_ffi_set_message("You are carrying nothing."); break;
        case '/': nh_ffi_set_message("You see nothing special."); break;
        case '\\': nh_ffi_set_message("You have made no discoveries."); break;
        case 'C': nh_ffi_set_message("You chat with no one."); break;
        case '?': nh_ffi_set_message("For help, consult the documentation."); break;
        case 'S': nh_ffi_set_message("Save not implemented in test mode."); break;
        case 'Q': nh_ffi_set_message("Quit not implemented in test mode."); break;
        case 'X': nh_ffi_set_message("Explore mode not implemented in test mode."); break;
        default: nh_ffi_set_message("Unknown command."); return -2;
    }
    
    return 0;
#endif
}

/* Execute a command with a direction */
int nh_ffi_exec_cmd_dir(char cmd, int dx, int dy) {
#ifdef REAL_NETHACK
    (void)cmd;
    u.ux += dx;
    u.uy += dy;
    moves++;
    return 0;
#else
    (void)cmd;
    if (!g_initialized) {
        return -1;
    }
    
    g_turn_count++;
    g_x += dx;
    g_y += dy;
    nh_ffi_set_message("You move.");
    
    return 0;
#endif
}

/* ============================================================================
 * State Serialization
 * ============================================================================ */

/* Serialize game state to JSON string */
char* nh_ffi_get_state_json(void) {
#ifndef REAL_NETHACK
    if (!g_initialized) {
        return strdup("{}");
    }
#endif
    
    char* json = (char*)malloc(4096);
    if (json == NULL) return NULL;
    
    int x, y;
    nh_ffi_get_position(&x, &y);
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
        nh_ffi_get_turn_count(),
        nh_ffi_get_role(), nh_ffi_get_race(), nh_ffi_get_gender(), nh_ffi_get_alignment(),
        nh_ffi_get_hp(),
        nh_ffi_get_max_hp(),
        nh_ffi_get_energy(),
        nh_ffi_get_max_energy(),
        x, y,
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
#ifdef REAL_NETHACK
    return strdup("Real message log not yet implemented");
#else
    return strdup(g_last_message[0] ? g_last_message : "No message");
#endif
}

/* ============================================================================
 * Inventory Management
 * ============================================================================ */

/* Get inventory item count */
int nh_ffi_get_inventory_count(void) {
#ifdef REAL_NETHACK
    int count = 0;
    struct obj *otmp;
    for (otmp = invent; otmp; otmp = otmp->nobj) count++;
    return count;
#else
    return 0;
#endif
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
#ifdef REAL_NETHACK
    int count = 0;
    struct monst *mtmp;
    for (mtmp = fmon; mtmp; mtmp = mtmp->nmon) count++;
    return count;
#else
    return 0;
#endif
}

/* ============================================================================
 * Game Status
 * ============================================================================ */

/* Check if game is over */
boolean nh_ffi_is_game_over(void) {
#ifdef REAL_NETHACK
    return FALSE; 
#else
    return g_game_over;
#endif
}

/* Check if game is won */
boolean nh_ffi_is_game_won(void) {
    return FALSE;
}

/* Get game result message */
char* nh_ffi_get_result_message(void) {
#ifdef REAL_NETHACK
    return strdup("Game continues");
#else
    if (!g_initialized) {
        return strdup("Game not initialized");
    }
    if (g_game_over) {
        return strdup("You died!");
    }
    return strdup("Game continues");
#endif
}

/* ============================================================================
 * Logic/Calculation Wrappers (Phase 2)
 * ============================================================================ */

/* RNG wrapper */
int nh_ffi_rng_rn2(int limit) {
#ifdef REAL_NETHACK
    return rn2(limit);
#else
    if (limit <= 0) return 0;
    return 0;
#endif
}

/* Damage calc wrapper */
int nh_ffi_calc_base_damage(int weapon_id, int small_monster) {
#ifdef REAL_NETHACK
    (void)weapon_id; (void)small_monster;
    return 4; 
#else
    (void)weapon_id;
    (void)small_monster;
    return 4;
#endif
}

/* AC wrapper */
int nh_ffi_get_ac(void) {
#ifdef REAL_NETHACK
    return u.uac;
#else
    return g_initialized ? g_ac : 10;
#endif
}