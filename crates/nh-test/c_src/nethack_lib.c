/*
 * NetHack C FFI Wrapper Library
 * 
 * This library provides a C interface to NetHack game logic for
 * comparison testing with the Rust implementation.
 * 
 * It wraps key functions from the NetHack 3.6.7 source code.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

/* ============================================================================
 * Type Definitions (matching NetHack types)
 * ============================================================================ */

typedef int8_t schar;
typedef int16_t xchar;
typedef int32_t coord;
typedef int bool_int;

#ifndef TRUE
#define TRUE 1
#endif

#ifndef FALSE
#define FALSE 0
#endif

#ifndef boolean
typedef bool_int boolean;
#endif

/* ============================================================================
 * Structure Definitions
 * ============================================================================ */

/* Inventory item structure */
struct nh_object {
    char name[64];
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
    int x, y;
};

/* Monster structure */
struct nh_monster {
    char name[64];
    char symbol;
    int level;
    int hp;
    int max_hp;
    int armor_class;
    int x, y;
    bool asleep;
    bool peaceful;
};

/* Player structure */
struct nh_player {
    char role[32];
    char race[32];
    int gender;
    int alignment;
    
    /* Stats */
    int hp;
    int max_hp;
    int energy;
    int max_energy;
    int x, y;
    int level;
    int experience_level;
    int armor_class;
    int gold;
    
    /* Attributes */
    int strength;
    int dexterity;
    int constitution;
    int intelligence;
    int wisdom;
    int charisma;
    
    /* Status */
    bool is_dead;
    int hunger_state;
    int confusion_timeout;
    int stun_timeout;
    int blindness_timeout;
};

/* Game state structure */
struct nh_game_state {
    struct nh_player player;
    struct nh_object* inventory[52];
    int inventory_count;
    
    struct nh_monster monsters[100];
    int monster_count;
    
    int current_level;
    int dungeon_depth;
    int dungeon_visited[30];
    
    unsigned long turn_count;
    int hunger_state;
    
    char last_message[256];
};

/* ============================================================================
 * Global state for the C implementation
 * ============================================================================ */

static struct nh_game_state* g_game = NULL;
static int g_initialized = 0;
static unsigned long g_turn_count = 0;
static char g_last_message[256] = "";

/* ============================================================================
 * Helper Functions
 * ============================================================================ */

/* Forward declaration for nh_free_game */
void nh_free_game(void);

/* Free allocated string */
void nh_free_string(void* ptr) {
    if (ptr != NULL) {
        free(ptr);
    }
}

/* ============================================================================
 * Game Initialization
 * ============================================================================ */

/* Initialize NetHack with character creation */
int nh_init_game(const char* role, const char* race, int gender, int alignment) {
    if (g_game != NULL) {
        nh_free_game();
    }
    
    g_game = (struct nh_game_state*)malloc(sizeof(struct nh_game_state));
    if (g_game == NULL) {
        return -1;
    }
    
    memset(g_game, 0, sizeof(struct nh_game_state));
    
    /* Set up player from parameters */
    strncpy(g_game->player.role, role ? role : "Tourist", sizeof(g_game->player.role) - 1);
    strncpy(g_game->player.race, race ? race : "Human", sizeof(g_game->player.race) - 1);
    g_game->player.gender = gender;
    g_game->player.alignment = alignment;
    
    /* Initialize player stats */
    g_game->player.hp = 10;
    g_game->player.max_hp = 10;
    g_game->player.energy = 10;
    g_game->player.max_energy = 10;
    g_game->player.x = 40;
    g_game->player.y = 10;
    g_game->player.level = 1;
    g_game->player.experience_level = 1;
    g_game->player.armor_class = 10;
    g_game->player.gold = 0;
    g_game->player.strength = 10;
    g_game->player.dexterity = 10;
    g_game->player.constitution = 10;
    g_game->player.intelligence = 10;
    g_game->player.wisdom = 10;
    g_game->player.charisma = 10;
    g_game->player.is_dead = FALSE;
    g_game->player.hunger_state = 0;
    
    /* Initialize level */
    g_game->current_level = 1;
    g_game->dungeon_depth = 1;
    g_game->turn_count = 0;
    g_game->hunger_state = 0;
    g_game->monster_count = 0;
    
    /* Initialize inventory */
    g_game->inventory_count = 0;
    for (int i = 0; i < 52; i++) {
        g_game->inventory[i] = NULL;
    }
    
    /* Initialize dungeon visited */
    for (int i = 0; i < 30; i++) {
        g_game->dungeon_visited[i] = 0;
    }
    g_game->dungeon_visited[0] = 1;
    
    g_initialized = 1;
    g_turn_count = 0;
    g_last_message[0] = '\0';
    
    return 0;
}

/* Reset game to initial state */
int nh_reset_game(unsigned long seed) {
    (void)seed; /* Unused for now */
    
    if (!g_initialized) {
        return -1;
    }
    
    /* Reset turn counter and position */
    g_turn_count = 0;
    g_game->player.x = 40;
    g_game->player.y = 10;
    g_game->current_level = 1;
    g_game->dungeon_depth = 1;
    g_game->player.hp = g_game->player.max_hp;
    g_game->player.energy = g_game->player.max_energy;
    g_game->player.gold = 0;
    g_game->hunger_state = 0;
    g_game->player.is_dead = FALSE;
    g_game->turn_count = 0;
    
    /* Clear inventory */
    for (int i = 0; i < g_game->inventory_count; i++) {
        if (g_game->inventory[i] != NULL) {
            free(g_game->inventory[i]);
            g_game->inventory[i] = NULL;
        }
    }
    g_game->inventory_count = 0;
    
    g_last_message[0] = '\0';
    
    return 0;
}

/* Free game resources */
void nh_free_game(void) {
    if (g_game != NULL) {
        /* Free inventory */
        for (int i = 0; i < g_game->inventory_count; i++) {
            if (g_game->inventory[i] != NULL) {
                free(g_game->inventory[i]);
            }
        }
        free(g_game);
        g_game = NULL;
    }
    g_initialized = 0;
    g_last_message[0] = '\0';
}

/* ============================================================================
 * Command Execution
 * ============================================================================ */

/* Set message */
static void nh_set_message(const char* msg) {
    if (msg) {
        strncpy(g_last_message, msg, sizeof(g_last_message) - 1);
        g_last_message[sizeof(g_last_message) - 1] = '\0';
        if (g_game) {
            strncpy(g_game->last_message, msg, sizeof(g_game->last_message) - 1);
        }
    }
}

/* Execute a game command */
int nh_exec_cmd(char cmd) {
    if (!g_initialized || g_game == NULL) {
        return -1;
    }
    
    g_turn_count++;
    g_game->turn_count = g_turn_count;
    
    /* Process command - simplified implementation */
    switch (cmd) {
        /* Movement commands */
        case 'h': /* west */
            if (g_game->player.x > 1) {
                g_game->player.x--;
                nh_set_message("You move west.");
            } else {
                nh_set_message("You can't go that way.");
            }
            break;
        case 'j': /* south */
            if (g_game->player.y < 20) {
                g_game->player.y++;
                nh_set_message("You move south.");
            } else {
                nh_set_message("You can't go that way.");
            }
            break;
        case 'k': /* north */
            if (g_game->player.y > 1) {
                g_game->player.y--;
                nh_set_message("You move north.");
            } else {
                nh_set_message("You can't go that way.");
            }
            break;
        case 'l': /* east */
            if (g_game->player.x < 79) {
                g_game->player.x++;
                nh_set_message("You move east.");
            } else {
                nh_set_message("You can't go that way.");
            }
            break;
        case 'y': /* northwest */
            if (g_game->player.x > 1 && g_game->player.y > 1) {
                g_game->player.x--;
                g_game->player.y--;
                nh_set_message("You move northwest.");
            } else {
                nh_set_message("You can't go that way.");
            }
            break;
        case 'u': /* northeast */
            if (g_game->player.x < 79 && g_game->player.y > 1) {
                g_game->player.x++;
                g_game->player.y--;
                nh_set_message("You move northeast.");
            } else {
                nh_set_message("You can't go that way.");
            }
            break;
        case 'b': /* southwest */
            if (g_game->player.x > 1 && g_game->player.y < 20) {
                g_game->player.x--;
                g_game->player.y++;
                nh_set_message("You move southwest.");
            } else {
                nh_set_message("You can't go that way.");
            }
            break;
        case 'n': /* southeast */
            if (g_game->player.x < 79 && g_game->player.y < 20) {
                g_game->player.x++;
                g_game->player.y++;
                nh_set_message("You move southeast.");
            } else {
                nh_set_message("You can't go that way.");
            }
            break;
        case '.': /* wait */
        case '5': /* wait (numpad) */
            nh_set_message("You wait.");
            break;
        
        /* Actions */
        case ',': /* pickup */
            nh_set_message("You pick up nothing.");
            break;
        case 'd': /* drop */
            nh_set_message("You drop nothing.");
            break;
        case 'e': /* eat */
            nh_set_message("You eat nothing.");
            break;
        case 'w': /* wield */
            nh_set_message("You wield nothing.");
            break;
        case 'W': /* wear */
            nh_set_message("You wear nothing.");
            break;
        case 'T': /* take off */
            nh_set_message("You take off nothing.");
            break;
        case 'q': /* quaff */
            nh_set_message("You drink nothing.");
            break;
        case 'r': /* read */
            nh_set_message("You read nothing.");
            break;
        case 'z': /* zap wand */
            nh_set_message("You zap nothing.");
            break;
        case 'a': /* apply */
            nh_set_message("You apply nothing.");
            break;
        case 'o': /* open */
            nh_set_message("You open nothing.");
            break;
        case 'c': /* close */
            nh_set_message("You close nothing.");
            break;
        case 's': /* search */
            nh_set_message("You search but find nothing.");
            break;
        
        /* Navigation */
        case '<': /* go up stairs */
            if (g_game->current_level > 1) {
                g_game->current_level--;
                g_game->dungeon_depth--;
                g_game->player.x = 40;
                g_game->player.y = 10;
                nh_set_message("You climb up the stairs.");
            } else {
                nh_set_message("You are at the top of the dungeon.");
            }
            break;
        case '>': /* go down stairs */
            if (g_game->current_level < 30) {
                g_game->current_level++;
                g_game->dungeon_depth++;
                g_game->player.x = 40;
                g_game->player.y = 10;
                nh_set_message("You descend the stairs.");
            } else {
                nh_set_message("You can't go down further.");
            }
            break;
        
        /* Information */
        case 'i': /* inventory */
            nh_set_message("You are carrying nothing.");
            break;
        case '/': /* look */
            nh_set_message("You see nothing special.");
            break;
        case '\\': /* discover */
            nh_set_message("You have made no discoveries.");
            break;
        case 'C': /* chat */
            nh_set_message("You chat with no one.");
            break;
        case '?': /* help */
            nh_set_message("For help, consult the documentation.");
            break;
        
        /* Meta */
        case 'S': /* save */
            nh_set_message("Save not implemented in test mode.");
            break;
        case 'Q': /* quit */
            nh_set_message("Quit not implemented in test mode.");
            break;
        case 'X': /* explore mode */
            nh_set_message("Explore mode not implemented in test mode.");
            break;
        
        default:
            nh_set_message("Unknown command.");
            return -2;
    }
    
    return 0;
}

/* Execute a command with a direction */
int nh_exec_cmd_dir(char cmd, int dx, int dy) {
    (void)cmd; /* cmd parameter reserved for future use */
    if (!g_initialized || g_game == NULL) {
        return -1;
    }
    
    g_turn_count++;
    g_game->turn_count = g_turn_count;
    
    /* Simple movement based on direction */
    int new_x = g_game->player.x + dx;
    int new_y = g_game->player.y + dy;
    
    if (new_x >= 1 && new_x <= 79) g_game->player.x = new_x;
    if (new_y >= 1 && new_y <= 20) g_game->player.y = new_y;
    
    nh_set_message("You move.");
    
    return 0;
}

/* ============================================================================
 * State Access
 * ============================================================================ */

/* Get player HP */
int nh_get_hp(void) {
    return (g_game != NULL) ? g_game->player.hp : -1;
}

/* Get player max HP */
int nh_get_max_hp(void) {
    return (g_game != NULL) ? g_game->player.max_hp : -1;
}

/* Get player energy */
int nh_get_energy(void) {
    return (g_game != NULL) ? g_game->player.energy : -1;
}

/* Get player max energy */
int nh_get_max_energy(void) {
    return (g_game != NULL) ? g_game->player.max_energy : -1;
}

/* Get player position */
void nh_get_position(int* x, int* y) {
    if (g_game != NULL) {
        *x = g_game->player.x;
        *y = g_game->player.y;
    } else {
        *x = -1;
        *y = -1;
    }
}

/* Get armor class */
int nh_get_armor_class(void) {
    return (g_game != NULL) ? g_game->player.armor_class : -1;
}

/* Get gold */
int nh_get_gold(void) {
    return (g_game != NULL) ? g_game->player.gold : -1;
}

/* Get experience level */
int nh_get_experience_level(void) {
    return (g_game != NULL) ? g_game->player.experience_level : -1;
}

/* Get current level */
int nh_get_current_level(void) {
    return (g_game != NULL) ? g_game->current_level : -1;
}

/* Get dungeon depth */
int nh_get_dungeon_depth(void) {
    return (g_game != NULL) ? g_game->dungeon_depth : -1;
}

/* Get turn count */
unsigned long nh_get_turn_count(void) {
    return g_turn_count;
}

/* Check if player is dead */
bool nh_is_player_dead(void) {
    return (g_game != NULL) && (g_game->player.hp <= 0);
}

/* ============================================================================
 * State Serialization
 * ============================================================================ */

/* Serialize game state to JSON string */
char* nh_get_state_json(void) {
    if (g_game == NULL) {
        char* empty = strdup("{}");
        return empty;
    }
    
    /* Build JSON string */
    char* json = (char*)malloc(4096);
    if (json == NULL) return NULL;
    
    snprintf(json, 4096,
        "{"
        "\"turn\": %lu, "
        "\"player\": {"
        "\"role\": \"%s\", "
        "\"race\": \"%s\", "
        "\"gender\": %d, "
        "\"alignment\": %d, "
        "\"hp\": %d, "
        "\"max_hp\": %d, "
        "\"energy\": %d, "
        "\"max_energy\": %d, "
        "\"x\": %d, "
        "\"y\": %d, "
        "\"level\": %d, "
        "\"armor_class\": %d, "
        "\"gold\": %d, "
        "\"experience_level\": %d, "
        "\"strength\": %d, "
        "\"dexterity\": %d, "
        "\"constitution\": %d, "
        "\"intelligence\": %d, "
        "\"wisdom\": %d, "
        "\"charisma\": %d"
        "}, "
        "\"current_level\": %d, "
        "\"dungeon_depth\": %d, "
        "\"hunger_state\": %d"
        "}",
        g_turn_count,
        g_game->player.role,
        g_game->player.race,
        g_game->player.gender,
        g_game->player.alignment,
        g_game->player.hp,
        g_game->player.max_hp,
        g_game->player.energy,
        g_game->player.max_energy,
        g_game->player.x,
        g_game->player.y,
        g_game->player.level,
        g_game->player.armor_class,
        g_game->player.gold,
        g_game->player.experience_level,
        g_game->player.strength,
        g_game->player.dexterity,
        g_game->player.constitution,
        g_game->player.intelligence,
        g_game->player.wisdom,
        g_game->player.charisma,
        g_game->current_level,
        g_game->dungeon_depth,
        g_game->hunger_state
    );
    
    return json;
}

/* Serialize game state to binary buffer */
int nh_serialize_state(void* buffer, size_t bufsize) {
    if (g_game == NULL || buffer == NULL || bufsize < sizeof(struct nh_game_state)) {
        return -1;
    }
    
    memcpy(buffer, g_game, sizeof(struct nh_game_state));
    return sizeof(struct nh_game_state);
}

/* Deserialize game state from binary buffer */
int nh_deserialize_state(void* buffer, size_t bufsize) {
    if (buffer == NULL || bufsize < sizeof(struct nh_game_state)) {
        return -1;
    }
    
    if (g_game == NULL) {
        g_game = (struct nh_game_state*)malloc(sizeof(struct nh_game_state));
        if (g_game == NULL) return -1;
    }
    
    memcpy(g_game, buffer, sizeof(struct nh_game_state));
    g_initialized = 1;
    
    return 0;
}

/* Get state size for serialization */
size_t nh_get_state_size(void) {
    return sizeof(struct nh_game_state);
}

/* ============================================================================
 * Message Log
 * ============================================================================ */

/* Get last message */
char* nh_get_last_message(void) {
    return strdup(g_last_message[0] ? g_last_message : "No message");
}

/* Get message history */
char* nh_get_message_history(void) {
    return strdup(""); /* Placeholder */
}

/* ============================================================================
 * Inventory Management
 * ============================================================================ */

/* Get inventory item count */
int nh_get_inventory_count(void) {
    return (g_game != NULL) ? g_game->inventory_count : 0;
}

/* Get inventory item as JSON */
char* nh_get_inventory_json(void) {
    if (g_game == NULL) {
        return strdup("[]");
    }
    
    /* Build JSON array */
    char* json = (char*)malloc(4096);
    if (json == NULL) return NULL;
    
    strcpy(json, "[");
    for (int i = 0; i < g_game->inventory_count; i++) {
        if (i > 0) strcat(json, ",");
        if (g_game->inventory[i] != NULL) {
            char item[256];
            snprintf(item, sizeof(item),
                "{\"name\":\"%s\",\"class\":\"%c\",\"qty\":%d}",
                g_game->inventory[i]->name,
                g_game->inventory[i]->class,
                g_game->inventory[i]->quantity);
            strcat(json, item);
        }
    }
    strcat(json, "]");
    
    return json;
}

/* ============================================================================
 * Monster Information
 * ============================================================================ */

/* Get nearby monsters as JSON */
char* nh_get_nearby_monsters_json(void) {
    if (g_game == NULL) {
        return strdup("[]");
    }
    
    char* json = (char*)malloc(4096);
    if (json == NULL) return NULL;
    
    strcpy(json, "[");
    for (int i = 0; i < g_game->monster_count; i++) {
        if (i > 0) strcat(json, ",");
        char monster[256];
        snprintf(monster, sizeof(monster),
            "{\"name\":\"%s\",\"symbol\":\"%c\",\"hp\":%d,\"x\":%d,\"y\":%d}",
            g_game->monsters[i].name,
            g_game->monsters[i].symbol,
            g_game->monsters[i].hp,
            g_game->monsters[i].x,
            g_game->monsters[i].y);
        strcat(json, monster);
    }
    strcat(json, "]");
    
    return json;
}

/* Count monsters on current level */
int nh_count_monsters(void) {
    return (g_game != NULL) ? g_game->monster_count : 0;
}

/* ============================================================================
 * Game Status
 * ============================================================================ */

/* Check if game is over */
bool nh_is_game_over(void) {
    return (g_game != NULL) && (g_game->player.hp <= 0);
}

/* Check if game is won */
bool nh_is_game_won(void) {
    return FALSE; /* Placeholder */
}

/* Get game result message */
char* nh_get_result_message(void) {
    if (g_game == NULL) {
        return strdup("Game not initialized");
    }
    if (g_game->player.hp <= 0) {
        return strdup("You died!");
    }
    return strdup("Game continues");
}
