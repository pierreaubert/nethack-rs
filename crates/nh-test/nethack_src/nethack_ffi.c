#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <signal.h>
#include <execinfo.h>

static void ffi_crash_handler(int sig) {
    void *array[30];
    int size = backtrace(array, 30);
    fprintf(stderr, "\n=== FFI CRASH: signal %d ===\n", sig);
    backtrace_symbols_fd(array, size, STDERR_FILENO);
    fprintf(stderr, "=== END CRASH ===\n");
    fflush(stderr);
    _exit(128 + sig);
}

__attribute__((constructor))
static void install_crash_handler(void) {
    signal(SIGSEGV, ffi_crash_handler);
    signal(SIGBUS, ffi_crash_handler);
    signal(SIGABRT, ffi_crash_handler);
}

#ifdef REAL_NETHACK
#include "hack.h"
#include "dlb.h"
#include "func_tab.h"
#include "mfndpos.h"

/* External declarations for role and race tables and lookup functions */
extern const struct Role roles[];
extern const struct Race races[];
extern int FDECL(str2role, (const char *));
extern int FDECL(str2race, (const char *));


/* ISAAC64 seed function from isaac64_standalone.c */
extern void set_random_generator_seed(unsigned long seed);

/* Forward declaration for level cleanup */
static void nh_ffi_pre_generate_cleanup(void);

/* NetHack initialization functions */
extern void NDECL(init_objects);
extern void NDECL(role_init);
extern void NDECL(init_dungeons);
extern void NDECL(init_artifacts);

/* Stub out status_initialize to avoid window-related segfaults but satisfy checks */
static boolean blinit_stub = FALSE;
void status_initialize(int reassessment) { 
    if (!reassessment) blinit_stub = TRUE;
}
/* NetHack's botl.c uses a static 'blinit'. We can't see it, but we can 
   hope our stub 'status_initialize' is linked instead of the real one. 
   Actually, the real one is in botl.o which is in the library. 
   To override it, we need to make sure our symbol is stronger. */

/* Override nh_terminate to prevent exit() during test — player may die from
   starvation or monster attacks during the post-turn processing loop. */
#include <setjmp.h>
static jmp_buf ffi_terminate_jmp;
static volatile int ffi_in_post_command = 0;
static volatile int ffi_player_died = 0;

void nh_terminate(int status) {
    fprintf(stderr, "FFI: nh_terminate(%d) called — player died or game ended\n", status);
    fflush(stderr);
    ffi_player_died = 1;
    if (ffi_in_post_command) {
        longjmp(ffi_terminate_jmp, 1);
    }
    /* If not in post_command, we can't longjmp — just abort cleanly */
    _exit(status);
}

/* Missing symbols from unixmain.c that we need to provide since we skip it */
short ospeed = 0;

boolean check_user_string(char *optstr) {
    (void)optstr;
    return FALSE;
}

void sethanguphandler(void (*fn)(int)) {
    (void)fn;
}

static unsigned long g_seed = 42;

unsigned long sys_random_seed(void) {
    return g_seed;
}

/* Dummy window procs to avoid segfaults in u_init */
static void dummy_init_nhwindows(int* argc, char** argv) { (void)argc; (void)argv; }
static void dummy_player_selection(void) {}
static void dummy_askname(void) {}
static void dummy_get_nh_event(void) {}
static void dummy_exit_nhwindows(const char* s) { (void)s; }
static void dummy_suspend_nhwindows(const char* s) { (void)s; }
static void dummy_resume_nhwindows(void) {}
static winid dummy_create_nhwindow(int type) { (void)type; return 0; }
static void dummy_clear_nhwindow(winid window) { (void)window; }
static void dummy_display_nhwindow(winid window, int blocking) { (void)window; (void)blocking; }
static void dummy_destroy_nhwindow(winid window) { (void)window; }
static void dummy_curs(winid window, int x, int y) { (void)window; (void)x; (void)y; }
static void dummy_putstr(winid window, int attr, const char* str) { (void)window; (void)attr; (void)str; }
static void dummy_display_file(const char* str, int blocking) { (void)str; (void)blocking; }
static void dummy_start_menu(winid window) { (void)window; }
static void dummy_add_menu(winid window, int glyph, const ANY_P* identifier, int ch, int gch, int attr, const char* str, int presel) { (void)window; (void)glyph; (void)identifier; (void)ch; (void)gch; (void)attr; (void)str; (void)presel; }
static void dummy_end_menu(winid window, const char* prompt) { (void)window; (void)prompt; }
static int dummy_select_menu(winid window, int how, MENU_ITEM_P** selected) { (void)window; (void)how; (void)selected; return 0; }
static void dummy_update_inventory(void) {}
static void dummy_mark_synch(void) {}
static void dummy_wait_synch(void) {}
static char dummy_message_menu(int let, int def, const char* msg) { (void)let; (void)msg; return (char)def; }
static void dummy_print_glyph(winid window, int x, int y, int glyph, int bkglyph) { (void)window; (void)x; (void)y; (void)glyph; (void)bkglyph; }
static void dummy_raw_print(const char* str) { (void)str; }
static void dummy_raw_print_bold(const char* str) { (void)str; }
static int dummy_nhgetch(void) { return 0; }
static int dummy_nh_poskey(int* x, int* y, int* mod) { (void)x; (void)y; (void)mod; return 0; }
static void dummy_nhbell(void) {}
static int dummy_doprev_message(void) { return 0; }
static char dummy_yn_function(const char* ques, const char* choices, int def) { (void)ques; (void)choices; return (char)def; }
static void dummy_getlin(const char* ques, char* input) { (void)ques; if (input) input[0] = '\0'; }
static int dummy_get_ext_cmd(void) { return -1; }
static void dummy_number_pad(int state) { (void)state; }
static void dummy_delay_output(void) {}
static void dummy_start_screen(void) {}
static void dummy_end_screen(void) {}
static void dummy_outrip(winid window, int how, time_t when) { (void)window; (void)how; (void)when; }
static void dummy_preference_update(const char* pref) { (void)pref; }
static void dummy_status_init(void) {}
static void dummy_status_finish(void) {}
static void dummy_status_enablefield(int field, const char* nm, const char* fmt, int enable) { (void)field; (void)nm; (void)fmt; (void)enable; }
static void dummy_status_update(int idx, genericptr_t ptr, int chg, int cls, int color, unsigned long *mask) { (void)idx; (void)ptr; (void)chg; (void)cls; (void)color; (void)mask; }

static boolean dummy_can_suspend(void) { return TRUE; }

static struct window_procs dummy_procs = {
    .name = "dummy",
    .wincap = 0,
    .wincap2 = 0,
    .win_init_nhwindows = dummy_init_nhwindows,
    .win_player_selection = dummy_player_selection,
    .win_askname = dummy_askname,
    .win_get_nh_event = dummy_get_nh_event,
    .win_exit_nhwindows = dummy_exit_nhwindows,
    .win_suspend_nhwindows = dummy_suspend_nhwindows,
    .win_resume_nhwindows = dummy_resume_nhwindows,
    .win_create_nhwindow = dummy_create_nhwindow,
    .win_clear_nhwindow = dummy_clear_nhwindow,
    .win_display_nhwindow = dummy_display_nhwindow,
    .win_destroy_nhwindow = dummy_destroy_nhwindow,
    .win_curs = dummy_curs,
    .win_putstr = dummy_putstr,
    .win_putmixed = dummy_putstr,
    .win_display_file = dummy_display_file,
    .win_start_menu = dummy_start_menu,
    .win_add_menu = dummy_add_menu,
    .win_end_menu = dummy_end_menu,
    .win_select_menu = dummy_select_menu,
    .win_message_menu = dummy_message_menu,
    .win_update_inventory = dummy_update_inventory,
    .win_mark_synch = dummy_mark_synch,
    .win_wait_synch = dummy_wait_synch,
    .win_print_glyph = dummy_print_glyph,
    .win_raw_print = dummy_raw_print,
    .win_raw_print_bold = dummy_raw_print_bold,
    .win_nhgetch = dummy_nhgetch,
    .win_nh_poskey = dummy_nh_poskey,
    .win_nhbell = dummy_nhbell,
    .win_doprev_message = dummy_doprev_message,
    .win_yn_function = dummy_yn_function,
    .win_getlin = dummy_getlin,
    .win_get_ext_cmd = dummy_get_ext_cmd,
    .win_number_pad = dummy_number_pad,
    .win_delay_output = dummy_delay_output,
    .win_start_screen = dummy_start_screen,
    .win_end_screen = dummy_end_screen,
    .win_outrip = dummy_outrip,
    .win_preference_update = dummy_preference_update,
    .win_status_init = dummy_status_init,
    .win_status_finish = dummy_status_finish,
    .win_status_enablefield = dummy_status_enablefield,
    .win_status_update = dummy_status_update,
    .win_can_suspend = dummy_can_suspend
};
#endif

/* Stub types if not using real NetHack */
#ifndef REAL_NETHACK
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

#ifdef REAL_NETHACK
/* extern void initoptions(int, char **); */
#endif

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
    int recharged;
    int poisoned;
    int otyp;
};

/* Monster info */
struct nh_ffi_monster {
    char name[128];
    char symbol;
    int level;
    int hp;
    int max_hp;
    int armor_class;
    int x; int y;
    boolean asleep;
    boolean peaceful;
    unsigned long strategy;
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
static char g_last_role[32] = "Tourist";
static char g_last_race[32] = "Human";
static int g_last_gender = 0;
static int g_last_alignment = 0;
static char g_json_buffer[1024 * 1024]; /* 1MB for map/state serialization */
#endif

/* ============================================================================
 * Implementations
 * ============================================================================ */

/* Set current dungeon level */
void nh_ffi_set_dlevel(int dnum, int dlevel) {
    u.uz.dnum = dnum;
    u.uz.dlevel = dlevel;
}

/* Force generation of a standard maze */
void nh_ffi_generate_maze(void) {
#ifdef REAL_NETHACK
    fprintf(stderr, "FFI: nh_ffi_generate_maze()...\n");
    
    s_level *sp = Is_special(&u.uz);
    if (sp) {
        fprintf(stderr, "FFI: Is_special! proto=%s, rndlevs=%d\n", sp->proto, (int)sp->rndlevs);
    }
    
    /* Setup level flags for maze */
    level.flags.is_maze_lev = TRUE;
    
    fprintf(stderr, "FFI: x_maze_max=%d, y_maze_max=%d\n", x_maze_max, y_maze_max);
    makemaz("");
    
    fprintf(stderr, "FFI: nh_ffi_generate_maze() complete. is_maze_lev=%d, corrmaze=%d\n", 
            level.flags.is_maze_lev, level.flags.corrmaze);
    fflush(stderr);
#endif
}

/* Cleanup globals to allow re-initialization */
void nh_ffi_cleanup_globals(void) {
#ifdef REAL_NETHACK
    /* 1. Drastic zeroing of core structures to stop u_init from freeing garbage */
    memset(&u, 0, sizeof(u));
    memset(&level, 0, sizeof(level));
    memset(&context, 0, sizeof(context));
    
    moves = 1L; /* NetHack starts at turn 1 usually, or 0? moves++ happens in many places */
    multi = 0;
    
    invent = (struct obj *)0;
    fmon = (struct monst *)0;
    fobj = (struct obj *)0;
    migrating_objs = (struct obj *)0;
    billobjs = (struct obj *)0;

    /* Clear equipment pointers — these are separate globals, NOT part of
       struct u, so memset(&u, 0, ...) does not zero them.  Stale pointers
       cause u_init→ini_inv to misbehave on repeated init calls. */
    uarm = uarmc = uarmh = uarms = uarmg = uarmf = (struct obj *)0;
    uarmu = uskin = uamul = uleft = uright = ublindf = (struct obj *)0;
    uwep = uswapwep = uquiver = (struct obj *)0;
    
    /* 2. Reset rooms */
    nroom = 0;
    nsubroom = 0;
    for (int i = 0; i < MAXNROFROOMS; i++) {
        rooms[i].hx = -1;
    }
#endif
}

/* Initialize the game with character creation */
int nh_ffi_init(const char* role, const char* race, int gender, int alignment) {
#ifdef REAL_NETHACK
    fprintf(stderr, "FFI: nh_ffi_init(%s, %s)...\n", role ? role : "NULL", race ? race : "NULL");
    fflush(stderr);

    static boolean global_initialized = FALSE;
    if (!global_initialized) {
        fprintf(stderr, "FFI: Global NetHack initialization...\n");
        fflush(stderr);
        windowprocs = dummy_procs;
        
        /* Change directory to HACKDIR to find data files */
        if (chdir(HACKDIR) != 0) {
            perror("FFI: chdir failed");
        }

        /* Set RNG seed */
        init_isaac64(42, rn2);
        init_isaac64(42, rn2_on_display_rng);

        /* NetHack 3.6.7 initialization sequence */
        strncpy(plname, "Hero", sizeof(plname)-1);
        
        initoptions();
        choose_windows("tty");
        
        dlb_init();
        init_objects();
        init_dungeons();
        monst_init();
        vision_init();
        
        global_initialized = TRUE;
    } 
    
    /* ALWAYS zero core structures before u_init() to avoid double-free/SIGABRT */
    nh_ffi_cleanup_globals();

    /* Set default dungeon level */
    u.uz.dnum = 0;
    u.uz.dlevel = 1;
    
    /* Setup dummy window system if not already done */
    windowprocs = dummy_procs;

    /* Reset core engine globals for a fresh game */
    int role_idx = str2role(role ? role : "Tourist");
    int race_idx = str2race(race ? race : "Human");
    if (role_idx < 0) role_idx = 0;
    if (race_idx < 0) race_idx = 0;
    
    flags.initrole = role_idx;
    flags.initrace = race_idx;
    flags.initgend = gender;
    flags.initalign = alignment;

    /* Call role_init every time to ensure urole/urace and others are set up correctly */
    role_init();

    /* Store for reset */
    if (role) strncpy(g_last_role, role, sizeof(g_last_role)-1);
    if (race) strncpy(g_last_race, race, sizeof(g_last_race)-1);
    g_last_gender = gender;
    g_last_alignment = alignment;

    /* Initialize flags/options needed by u_init */
    flags.female = (gender > 0);
    flags.initgend = flags.female;
    flags.initalign = alignment;

    fprintf(stderr, "FFI: u_init()...\n");
    fflush(stderr);
    u_init();

    /* Fix ubirthday to a constant for reproducible antholemon() results */
    ubirthday = 0;

    fprintf(stderr, "C FFI Rolled: Role=%s Race=%s HP=%d, Energy=%d\n", role, race, u.uhp, u.uen);
    fflush(stderr);

    fprintf(stderr, "C FFI: Init complete, player at (%d,%d)\n", u.ux, u.uy);
    fflush(stderr);

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
    /* No-op: u_init() handles its own cleanup, manual cleanup here causes double-frees. */
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
    /* Reseed the RNG and re-initialize character creation.
       We call cleanup_globals then u_init to get fresh character stats
       from the new seed. */
    g_seed = seed;
    init_isaac64(seed, rn2);
    init_isaac64(seed, rn2_on_display_rng);
    { extern unsigned long rng_call_counter; rng_call_counter = 0; }

    /* Don't re-call u_init() — calling it twice causes massive inventory
       duplication due to stale C global state that leaks between u_init calls.
       Instead, just reseed RNG and keep the character from init().
       Tests sync stats from Rust via set_state() anyway. */
    moves = 1L;
    multi = 0;
    return 0;
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

/* Thorough cleanup of accumulated state between level generations.
   clear_level_structures() inside mklev->makelevel resets cells/rooms/doors/rects,
   but there is other global state that accumulates across calls.
   We just NULL out pointers (leaking memory is fine for test harness). */
static void nh_ffi_pre_generate_cleanup(void) {
    int x, y;

    /* NULL out monster/object map arrays and global lists */
    level.monlist = (struct monst *)0;
    fmon = (struct monst *)0;
    level.objlist = (struct obj *)0;
    fobj = (struct obj *)0;
    level.buriedobjlist = (struct obj *)0;
    ftrap = (struct trap *)0;

    for (x = 0; x < COLNO; x++)
        for (y = 0; y < ROWNO; y++) {
            level.monsters[x][y] = (struct monst *)0;
            level.objects[x][y] = (struct obj *)0;
        }

    /* Clear mapseenchn to prevent accumulation (just leak it) */
    {
        extern mapseen *mapseenchn;
        mapseenchn = (mapseen *)0;
    }
}

/* Generate level and place player on stairs (full newgame-like init) */
int nh_ffi_generate_and_place(void) {
#ifdef REAL_NETHACK
    fprintf(stderr, "FFI: nh_ffi_generate_and_place() dnum=%d, dlevel=%d...\n",
            u.uz.dnum, u.uz.dlevel);
    fflush(stderr);

    nh_ffi_pre_generate_cleanup();
    mklev();

    /* Place player on stairs or first walkable room cell.
       Avoids u_on_upstairs() which calls cliparound()/place_lregion()
       that can crash with dummy window procs. */
    if (xupstair) {
        u.ux = xupstair;
        u.uy = yupstair;
    } else if (xdnstair) {
        u.ux = xdnstair;
        u.uy = ydnstair;
    } else {
        /* Find first room cell as fallback */
        int found = 0;
        int fx, fy;
        for (fy = 0; fy < ROWNO && !found; fy++)
            for (fx = 1; fx < COLNO && !found; fx++)
                if (levl[fx][fy].typ == ROOM) {
                    u.ux = fx;
                    u.uy = fy;
                    found = 1;
                }
    }

    /* Initialize hero movement points (matches C moveloop line 79) */
    youmonst.movement = NORMAL_SPEED;

    /* Initialize monstermoves (matches C moveloop) */
    monstermoves = 1L;

    fprintf(stderr, "FFI: nh_ffi_generate_and_place() complete. Player at (%d,%d), upstair=(%d,%d), dnstair=(%d,%d)\n",
            u.ux, u.uy, xupstair, yupstair, xdnstair, ydnstair);
    fflush(stderr);
    return 0;
#else
    return 0;
#endif
}

/* Generate a new level */
int nh_ffi_generate_level(void) {
#ifdef REAL_NETHACK
    fprintf(stderr, "FFI: nh_ffi_generate_level() ledger=%d, dnum=%d, dlevel=%d, name=%s...\n",
            ledger_no(&u.uz), u.uz.dnum, u.uz.dlevel, dungeons[u.uz.dnum].dname);
    fflush(stderr);

    /* Thorough cleanup of accumulated state from prior generations */
    nh_ffi_pre_generate_cleanup();

    mklev();

    fprintf(stderr, "FFI: nh_ffi_generate_level() complete. is_maze=%d, corrmaze=%d\n",
            level.flags.is_maze_lev, level.flags.corrmaze);
    fflush(stderr);
    return 0;
#else
    return 0;
#endif
}

/* Stub for getbones to avoid non-deterministic RNG consumption */
int getbones(void) {
    return 0;
}

#ifdef REAL_NETHACK
static const char* typ_name(int t) {
    switch(t) {
        case STONE: return "Stone";
        case VWALL: return "VWall";
        case HWALL: return "HWall";
        case TLCORNER: return "TLCorner";
        case TRCORNER: return "TRCorner";
        case BLCORNER: return "BLCorner";
        case BRCORNER: return "BRCorner";
        case CROSSWALL: return "CrossWall";
        case TUWALL: return "TUWall";
        case TDWALL: return "TDWall";
        case TLWALL: return "TLWall";
        case TRWALL: return "TRWall";
        case DBWALL: return "DBWall";
        case SDOOR: return "SDoor";
        case SCORR: return "SCorr";
        case POOL: return "Pool";
        case MOAT: return "Moat";
        case WATER: return "Water";
        case DRAWBRIDGE_UP: return "DrawbridgeUp";
        case LAVAPOOL: return "LavaPool";
        case IRONBARS: return "IronBars";
        case DOOR: return "Door";
        case STAIRS: return "Stairs";
        case CORR: return "Corridor";
        case ROOM: return "Room";
        case AIR: return "Air";
        case CLOUD: return "Cloud";
        case ICE: return "Ice";
        case FOUNTAIN: return "Fountain";
        case THRONE: return "Throne";
        case SINK: return "Sink";
        case GRAVE: return "Grave";
        case ALTAR: return "Altar";
        case DRAWBRIDGE_DOWN: return "DrawbridgeDown";
        case TREE: return "Tree";
        default: {
            static char buf[32];
            sprintf(buf, "Unknown(%d)", t);
            return buf;
        }
    }
}
#endif

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

/* Get map layout and rooms as JSON */
const char* nh_ffi_get_map_json(void) {
#ifdef REAL_NETHACK
    char *p = g_json_buffer;
    p += sprintf(p, "{\"width\": %d, \"height\": %d, \"cells\": [", COLNO, ROWNO);
    
    for (int x = 0; x < COLNO; x++) {
        p += sprintf(p, "[");
        for (int y = 0; y < ROWNO; y++) {
            p += sprintf(p, "{\"t\": %d, \"type\": \"%s\", \"l\": %d}%s", 
                level.locations[x][y].typ, 
                typ_name(level.locations[x][y].typ),
                level.locations[x][y].lit,
                (y < ROWNO - 1) ? "," : "");
        }
        p += sprintf(p, "]%s", (x < COLNO - 1) ? "," : "");
    }
    
    p += sprintf(p, "], \"nroom\": %d, \"rooms\": [", nroom);
    for (int i = 0; rooms[i].hx >= 0 && i < MAXNROFROOMS; i++) {
        p += sprintf(p, "{\"lx\": %d, \"hx\": %d, \"ly\": %d, \"hy\": %d, \"type\": %d}%s",
            rooms[i].lx, rooms[i].hx, rooms[i].ly, rooms[i].hy, rooms[i].rtype,
            (rooms[i+1].hx >= 0 && i+1 < MAXNROFROOMS) ? "," : "");
    }
    p += sprintf(p, "]}");
    return strdup(g_json_buffer);
#else
    return strdup("{}");
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
#ifndef REAL_NETHACK
static void nh_ffi_set_message(const char* msg) {
    if (msg) {
        strncpy(g_last_message, msg, sizeof(g_last_message) - 1);
        g_last_message[sizeof(g_last_message) - 1] = '\0';
    }
}
#endif

/* Inline regen_hp (static in allmain.c, can't call directly) */
static void ffi_regen_hp(int wtcap) {
    int heal = 0;
    boolean reached_full = FALSE;
    boolean encumbrance_ok = (wtcap < MOD_ENCUMBER || !u.umoved);

    if (Upolyd) {
        if (u.mh < 1) {
            rehumanize();
        } else if (youmonst.data->mlet == S_EEL
                   && !is_pool(u.ux, u.uy) && !Is_waterlevel(&u.uz)) {
            if (u.mh > 1 && !Regeneration && rn2(u.mh) > rn2(8)
                && (!Half_physical_damage || !(moves % 2L)))
                heal = -1;
        } else if (u.mh < u.mhmax) {
            if (Regeneration || (encumbrance_ok && !(moves % 20L)))
                heal = 1;
        }
        if (heal) {
            context.botl = TRUE;
            u.mh += heal;
            reached_full = (u.mh == u.mhmax);
        }
    } else {
        if (u.uhp < u.uhpmax && (encumbrance_ok || Regeneration)) {
            if (u.ulevel > 9) {
                if (!(moves % 3L)) {
                    int Con = (int) ACURR(A_CON);
                    if (Con <= 12) {
                        heal = 1;
                    } else {
                        heal = rnd(Con);
                        if (heal > u.ulevel - 9)
                            heal = u.ulevel - 9;
                    }
                }
            } else {
                if (!(moves % (long) ((MAXULEV + 12) / (u.ulevel + 2) + 1)))
                    heal = 1;
            }
            if (Regeneration && !heal)
                heal = 1;
            if (heal) {
                context.botl = TRUE;
                u.uhp += heal;
                if (u.uhp > u.uhpmax)
                    u.uhp = u.uhpmax;
                reached_full = (u.uhp == u.uhpmax);
            }
        }
    }

    if (reached_full) {
        /* interrupt_multi is static in allmain.c, inline it */
        if (multi > 0 && !context.travel && !context.run) {
            nomul(0);
        }
    }
}

/* Global flag: when set, skip movemon() in post-command processing.
   This allows comparing non-AI per-turn RNG consumption between C and Rust. */
static int ffi_skip_movemon = 0;

void nh_ffi_set_skip_movemon(int skip) {
    ffi_skip_movemon = skip;
}

/* Post-command processing: replicates C moveloop after player takes an action.
   This runs monster turns, new-turn bookkeeping, and per-turn effects.
   Matches allmain.c:93-318 */
static void ffi_post_command(void) {
    int moveamt = 0, wtcap = 0;
    boolean monscanmove = FALSE;
    int change = 0;

    /* Hero spends movement (C moveloop line 95) */
    youmonst.movement -= NORMAL_SPEED;

    do { /* hero can't move this turn loop */
        wtcap = encumber_msg();

        context.mon_moving = TRUE;
        if (!ffi_skip_movemon) {
            extern unsigned long rng_call_counter;
            unsigned long rng_movemon_start = rng_call_counter;
            int movemon_rounds = 0;
            do {
                monscanmove = movemon();
                movemon_rounds++;
                if (youmonst.movement >= NORMAL_SPEED)
                    break; /* hero gained movement from speed */
            } while (monscanmove);
            fprintf(stderr, "  C SECTION movemon: %lu RNG calls (%d rounds)\n",
                rng_call_counter - rng_movemon_start, movemon_rounds);
            /* Print per-monster positions after movemon for debugging */
            {
                struct monst *mtmp;
                int mi = 0;
                for (mtmp = fmon; mtmp; mtmp = mtmp->nmon, mi++) {
                    if (DEADMONSTER(mtmp)) continue;
                    fprintf(stderr, "    C MON %d mnum=%d at (%d,%d) mux=(%d,%d) movement=%d sleeping=%d\n",
                        mi, mtmp->mnum, mtmp->mx, mtmp->my,
                        mtmp->mux, mtmp->muy, mtmp->movement,
                        (int)mtmp->msleeping);
                }
            }
        }
        context.mon_moving = FALSE;

        if (!monscanmove && youmonst.movement < NORMAL_SPEED) {
            /* Both hero and monsters are out of steam — new turn */
            struct monst *mtmp;
            extern unsigned long rng_call_counter;
            unsigned long rng_sect;

            rng_sect = rng_call_counter;
            mcalcdistress(); /* adjust monsters' trap, blind, etc */
            fprintf(stderr, "  C SECTION mcalcdistress: %lu RNG calls\n", rng_call_counter - rng_sect);

            /* Reallocate movement to monsters */
            rng_sect = rng_call_counter;
            for (mtmp = fmon; mtmp; mtmp = mtmp->nmon)
                mtmp->movement += mcalcmove(mtmp);
            fprintf(stderr, "  C SECTION mcalcmove: %lu RNG calls\n", rng_call_counter - rng_sect);

            /* Occasionally add another monster (C moveloop lines 124-128) */
            rng_sect = rng_call_counter;
            if (!rn2(u.uevent.udemigod ? 25
                     : (depth(&u.uz) > depth(&stronghold_level)) ? 50
                       : 70))
                (void) makemon((struct permonst *) 0, 0, 0, NO_MM_FLAGS);
            fprintf(stderr, "  C SECTION spawn: %lu RNG calls\n", rng_call_counter - rng_sect);

            /* Calculate hero movement for this turn (C moveloop lines 131-169) */
            rng_sect = rng_call_counter;
            if (u.usteed && u.umoved) {
                moveamt = mcalcmove(u.usteed);
            } else {
                moveamt = youmonst.data->mmove;

                if (Very_fast) {
                    if (rn2(3) != 0)
                        moveamt += NORMAL_SPEED;
                } else if (Fast) {
                    if (rn2(3) == 0)
                        moveamt += NORMAL_SPEED;
                }
            }
            fprintf(stderr, "  C SECTION speed: %lu RNG calls (Fast=%d VFast=%d)\n", rng_call_counter - rng_sect, (int)(!!Fast), (int)(!!Very_fast));

            switch (wtcap) {
            case UNENCUMBERED: break;
            case SLT_ENCUMBER: moveamt -= (moveamt / 4); break;
            case MOD_ENCUMBER: moveamt -= (moveamt / 2); break;
            case HVY_ENCUMBER: moveamt -= ((moveamt * 3) / 4); break;
            case EXT_ENCUMBER: moveamt -= ((moveamt * 7) / 8); break;
            }

            youmonst.movement += moveamt;
            if (youmonst.movement < 0)
                youmonst.movement = 0;
            settrack();

            monstermoves++;
            moves++;

            /********************************/
            /* once-per-turn things go here */
            /********************************/

            rng_sect = rng_call_counter;
            if (Glib)
                glibr();
            nh_timeout();
            run_regions();
            fprintf(stderr, "  C SECTION timeout+regions: %lu RNG calls\n", rng_call_counter - rng_sect);

            if (u.ublesscnt)
                u.ublesscnt--;

            /* HP regeneration (C moveloop lines 200-205) */
            rng_sect = rng_call_counter;
            if (!u.uinvulnerable) {
                if (!Upolyd ? (u.uhp < u.uhpmax)
                            : (u.mh < u.mhmax
                               || youmonst.data->mlet == S_EEL)) {
                    ffi_regen_hp(wtcap);
                }
            }
            fprintf(stderr, "  C SECTION hp_regen: %lu RNG calls (hp=%d/%d)\n", rng_call_counter - rng_sect, u.uhp, u.uhpmax);

            /* Moving while encumbered costs HP (C moveloop lines 208-223) */
            if (wtcap > MOD_ENCUMBER && u.umoved) {
                if (!(wtcap < EXT_ENCUMBER ? moves % 30
                                           : moves % 10)) {
                    if (Upolyd && u.mh > 1) {
                        u.mh--;
                    } else if (!Upolyd && u.uhp > 1) {
                        u.uhp--;
                    }
                }
            }

            /* Energy regeneration (C moveloop lines 225-237) */
            rng_sect = rng_call_counter;
            if (u.uen < u.uenmax
                && ((wtcap < MOD_ENCUMBER
                     && (!(moves % ((MAXULEV + 8 - u.ulevel)
                                    * (Role_if(PM_WIZARD) ? 3 : 4)
                                    / 6)))) || Energy_regeneration)) {
                u.uen += rn1(
                    (int) (ACURR(A_WIS) + ACURR(A_INT)) / 15 + 1, 1);
                if (u.uen > u.uenmax)
                    u.uen = u.uenmax;
            }
            fprintf(stderr, "  C SECTION energy_regen: %lu RNG calls (en=%d/%d moves=%ld freq=%d)\n",
                rng_call_counter - rng_sect, u.uen, u.uenmax, moves,
                (int)((MAXULEV + 8 - u.ulevel) * (Role_if(PM_WIZARD) ? 3 : 4) / 6));

            /* Teleportation check (C moveloop line 240) */
            rng_sect = rng_call_counter;
            if (!u.uinvulnerable) {
                if (Teleportation && !rn2(85)) {
                    xchar old_ux = u.ux, old_uy = u.uy;
                    tele();
                    if (u.ux != old_ux || u.uy != old_uy) {
                        if (!next_to_u()) {
                            check_leash(old_ux, old_uy);
                        }
                    }
                }
                /* Polymorph check (C moveloop lines 257-261) */
                if ((change == 1 && !Polymorph)
                    || (change == 2 && u.ulycn == NON_PM))
                    change = 0;
                if (Polymorph && !rn2(100))
                    change = 1;
                else if (u.ulycn >= LOW_PM && !Upolyd
                         && !rn2(80 - (20 * night())))
                    change = 2;
                if (change && !Unchanging) {
                    if (multi >= 0) {
                        stop_occupation();
                        if (change == 1)
                            polyself(0);
                        else
                            you_were();
                        change = 0;
                    }
                }
            }
            fprintf(stderr, "  C SECTION tele+poly: %lu RNG calls\n", rng_call_counter - rng_sect);

            rng_sect = rng_call_counter;
            if (Searching && multi >= 0)
                (void) dosearch0(1);
            fprintf(stderr, "  C SECTION search: %lu RNG calls\n", rng_call_counter - rng_sect);
            rng_sect = rng_call_counter;
            dosounds();
            fprintf(stderr, "  C SECTION dosounds: %lu RNG calls\n", rng_call_counter - rng_sect);
            rng_sect = rng_call_counter;
            do_storms();
            fprintf(stderr, "  C SECTION do_storms: %lu RNG calls\n", rng_call_counter - rng_sect);
            rng_sect = rng_call_counter;
            gethungry();
            fprintf(stderr, "  C SECTION gethungry: %lu RNG calls\n", rng_call_counter - rng_sect);
            rng_sect = rng_call_counter;
            age_spells();
            fprintf(stderr, "  C SECTION age_spells: %lu RNG calls\n", rng_call_counter - rng_sect);
            rng_sect = rng_call_counter;
            exerchk();
            fprintf(stderr, "  C SECTION exerchk: %lu RNG calls\n", rng_call_counter - rng_sect);
            rng_sect = rng_call_counter;
            invault();
            if (u.uhave.amulet)
                amulet();
            fprintf(stderr, "  C SECTION invault+amulet: %lu RNG calls\n", rng_call_counter - rng_sect);

            rng_sect = rng_call_counter;
            if (!rn2(40 + (int) (ACURR(A_DEX) * 3)))
                u_wipe_engr(rnd(3));
            fprintf(stderr, "  C SECTION engrave: %lu RNG calls (dex=%d threshold=%d)\n",
                rng_call_counter - rng_sect, (int)ACURR(A_DEX), 40 + (int)(ACURR(A_DEX) * 3));
            if (u.uevent.udemigod && !u.uinvulnerable) {
                if (u.udg_cnt)
                    u.udg_cnt--;
                if (!u.udg_cnt) {
                    intervene();
                    u.udg_cnt = rn1(200, 50);
                }
            }
            restore_attrib();

            /* Immobile count */
            if (multi < 0) {
                if (++multi == 0) {
                    unmul((char *) 0);
                    if (u.utotype)
                        deferred_goto();
                }
            }
        }
    } while (youmonst.movement < NORMAL_SPEED);
}

/* Execute a game command */
int nh_ffi_exec_cmd(char cmd) {
#ifdef REAL_NETHACK
    fprintf(stderr, "C FFI Exec: '%c' Start Pos: (%d,%d)\n", cmd, u.ux, u.uy);
    fflush(stderr);

    int is_movement = 1;

    /* Set direction for movement commands */
    switch (cmd) {
        case 'h': u.dx = -1; u.dy =  0; u.dz = 0; break; /* West */
        case 'j': u.dx =  0; u.dy =  1; u.dz = 0; break; /* South */
        case 'k': u.dx =  0; u.dy = -1; u.dz = 0; break; /* North */
        case 'l': u.dx =  1; u.dy =  0; u.dz = 0; break; /* East */
        case 'y': u.dx = -1; u.dy = -1; u.dz = 0; break; /* NW */
        case 'u': u.dx =  1; u.dy = -1; u.dz = 0; break; /* NE */
        case 'b': u.dx = -1; u.dy =  1; u.dz = 0; break; /* SW */
        case 'n': u.dx =  1; u.dy =  1; u.dz = 0; break; /* SE */
        case '.': case '5':
            /* Rest/wait: time passes but no movement */
            is_movement = 0;
            break;
        default:
            fprintf(stderr, "FFI: Unsupported command '%c'\n", cmd);
            fflush(stderr);
            return -1;
    }

    if (is_movement) {
        /* Call real domove() for movement commands */
        context.run = 0;  /* Normal walk, not running */
        context.nopick = 0;
        domove_attempting |= DOMOVE_WALK;
        u.umoved = TRUE;
        domove();
    } else {
        u.umoved = FALSE;
    }

    /* Post-command processing: monster turns, new turn, per-turn effects.
       This matches C's moveloop after the player takes an action.
       We use setjmp/longjmp to catch nh_terminate (player death) gracefully. */
    ffi_player_died = 0;
    ffi_in_post_command = 1;
    if (setjmp(ffi_terminate_jmp) == 0) {
        ffi_post_command();
    } else {
        /* Returned via longjmp from nh_terminate — player died */
        fprintf(stderr, "C FFI Exec: '%c' — player died during post-turn processing\n", cmd);
        fflush(stderr);
    }
    ffi_in_post_command = 0;

    fprintf(stderr, "C FFI Exec: '%c' End Pos: (%d,%d) moves=%ld died=%d\n",
            cmd, u.ux, u.uy, moves, ffi_player_died);
    fflush(stderr);
    return ffi_player_died ? -2 : 0;
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
#ifdef REAL_NETHACK
    /* Count items first to avoid buffer overflow */
    int count = 0;
    struct obj *otmp;
    for (otmp = invent; otmp && count < 1000; otmp = otmp->nobj) count++;
    
    fprintf(stderr, "FFI: nh_ffi_get_inventory_json() found %d items.\n", count);
    fflush(stderr);

    size_t buf_size = (count + 1) * 1024 + 10;
    char* json = (char*)malloc(buf_size);
    if (json == NULL) return NULL;
    
    strcpy(json, "[");
    boolean first = TRUE;
    int limit = 1000; 
    for (otmp = invent; otmp && limit-- > 0; otmp = otmp->nobj) {
        if (!first) strcat(json, ", ");
        char item_json[512];
        snprintf(item_json, 512, 
            "{\"otyp\": %d, \"name\": \"%s\", \"quantity\": %d, \"weight\": %d, \"buc\": %d, \"enchantment\": %d, \"recharged\": %d, \"poisoned\": %d}",
            otmp->otyp,
            "item",
            (int)otmp->quan,
            (int)otmp->owt,
            otmp->blessed ? 1 : (otmp->cursed ? -1 : 0),
            (int)otmp->spe,
            (int)otmp->recharged,
            (int)otmp->otrapped
        );
        strcat(json, item_json);
        first = FALSE;
    }
    strcat(json, "]");
    return json;
#else
    return strdup("[]");
#endif
}

/* Get all object indices and names as JSON (for index synchronization) */
char* nh_ffi_get_object_table_json(void) {
#ifdef REAL_NETHACK
    size_t buf_size = 65536;
    char* json = (char*)malloc(buf_size);
    if (json == NULL) return NULL;
    
    fprintf(stderr, "FFI: nh_ffi_get_object_table_json()...\n");
    fflush(stderr);

    strcpy(json, "[");
    boolean first = TRUE;
    for (int i = 0; i < 450; i++) { /* 450 is safe for 3.6.7 */
        const char* name = (char*)0;
        
        /* SAFER name lookup: obj_descr might not be initialized if init_objects failed */
        if (objects[i].oc_name_idx >= 0 && objects[i].oc_name_idx < 1000) {
             name = obj_descr[objects[i].oc_name_idx].oc_name;
        }
        
        if (!name) continue;
        
        if (!first) strcat(json, ", ");
        char obj_json[256];
        snprintf(obj_json, 256, "{\"index\": %d, \"name\": \"%s\"}", i, name);
        strcat(json, obj_json);
        first = FALSE;
    }
    strcat(json, "]");
    return json;
#else
    return strdup("[]");
#endif
}

/* ============================================================================
 * Monster Information
 * ============================================================================ */

#ifdef REAL_NETHACK
/* Stub for minimal_monnam since we don't want to link all of NetHack's UI dependencies */
char* minimal_monnam(struct monst *mtmp, int b) {
    (void)b;
    if (!mtmp || !mtmp->data) return "unknown";
    return (char *)mtmp->data->mname;
}
#endif

/* Get nearby monsters as JSON */
char* nh_ffi_get_nearby_monsters_json(void) {
#ifdef REAL_NETHACK
    char* json = (char*)malloc(16384);
    if (json == NULL) return NULL;
    
    strcpy(json, "[");
    struct monst *mtmp;
    boolean first = TRUE;
    for (mtmp = fmon; mtmp; mtmp = mtmp->nmon) {
        if (!first) strcat(json, ", ");
        char mon_json[512];
        snprintf(mon_json, 512, 
            "{\"name\": \"%s\", \"x\": %d, \"y\": %d, \"hp\": %d, \"hp_max\": %d, \"asleep\": %d, \"peaceful\": %d, \"strategy\": %lu}",
            minimal_monnam(mtmp, FALSE),
            mtmp->mx, mtmp->my,
            mtmp->mhp, mtmp->mhpmax,
            mtmp->msleeping ? 1 : 0,
            mtmp->mpeaceful ? 1 : 0,
            mtmp->mstrategy
        );
        strcat(json, mon_json);
        first = FALSE;
    }
    strcat(json, "]");
    return json;
#else
    return strdup("[]");
#endif
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

/* Synchronize engine state from external source */
void nh_ffi_set_state(int hp, int hpmax, int x, int y, int ac, long turn_count) {
#ifdef REAL_NETHACK
    /* Ensure coordinates are within bounds */
    if (x >= 1 && x < COLNO && y >= 0 && y < ROWNO) {
        /* If player position changed, we might need to update map-related state */
        if (u.ux != x || u.uy != y) {
            u.ux = x;
            u.uy = y;
            /* In NetHack, vision depends on u.ux, u.uy. 
               We should mark vision as dirty so it's recalculated. */
            vision_full_recalc = 1;
        }
    }
    
    u.uhp = hp;
    u.uhpmax = hpmax;
    u.uac = ac;
    moves = turn_count;
    
    /* Ensure HP doesn't exceed max */
    if (u.uhp > u.uhpmax) u.uhp = u.uhpmax;
#else
    (void)hp; (void)hpmax; (void)x; (void)y; (void)ac; (void)turn_count;
#endif
}

/* Set wizard mode */
void nh_ffi_set_wizard_mode(int enable) {
#ifdef REAL_NETHACK
    wizard = enable ? TRUE : FALSE;
#else
    (void)enable;
#endif
}

/* RNG wrapper */
void nh_ffi_reset_rng(unsigned long seed) {
#ifdef REAL_NETHACK
    g_seed = seed;
    init_isaac64(seed, rn2);
    init_isaac64(seed, rn2_on_display_rng);
    { extern unsigned long rng_call_counter; rng_call_counter = 0; }
#endif
}

/* Override reseed_random to keep things deterministic */
void reseed_random(int (*fn)(int)) {
#ifdef REAL_NETHACK
    init_isaac64(g_seed, fn);
#endif
}

/* Forward declaration for rng_trace_record (defined below) */
static void rng_trace_record(const char *func, unsigned long arg, unsigned long result);

/* Return the total number of RNG calls made since last reset */
unsigned long nh_ffi_get_rng_call_count(void) {
#ifdef REAL_NETHACK
    extern unsigned long rng_call_counter;
    return rng_call_counter;
#else
    return 0;
#endif
}

int nh_ffi_rng_rn2(int limit) {
#ifdef REAL_NETHACK
    int r = rn2(limit);
    rng_trace_record("rn2", (unsigned long)limit, (unsigned long)r);
    fprintf(stderr, "FFI: rn2(%d) = %d\n", limit, r);
    return r;
#else
    if (limit <= 0) return 0;
    rng_trace_record("rn2", (unsigned long)limit, 0);
    return 0;
#endif
}

int nh_ffi_rng_rnd(int limit) {
#ifdef REAL_NETHACK
    int r = rnd(limit);
    rng_trace_record("rnd", (unsigned long)limit, (unsigned long)r);
    fprintf(stderr, "FFI: rnd(%d) = %d\n", limit, r);
    return r;
#else
    if (limit <= 0) return 1;
    rng_trace_record("rnd", (unsigned long)limit, 1);
    return 1;
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

/* ============================================================================
 * RNG Trace Ring Buffer (Phase 4: Convergence Framework)
 * ============================================================================ */

#define RNG_TRACE_SIZE 4096

struct rng_trace_entry {
    unsigned long seq;
    char func[8]; /* "rn2", "rnd", "rne", "rnz" */
    unsigned long arg;
    unsigned long result;
};

static struct rng_trace_entry g_rng_trace[RNG_TRACE_SIZE];
static unsigned long g_rng_trace_count = 0;
static int g_rng_tracing = 0;

void nh_ffi_enable_rng_tracing(void) {
    g_rng_tracing = 1;
    g_rng_trace_count = 0;
}

void nh_ffi_disable_rng_tracing(void) {
    g_rng_tracing = 0;
}

static void rng_trace_record(const char *func, unsigned long arg, unsigned long result) {
    if (!g_rng_tracing) return;
    unsigned long idx = g_rng_trace_count % RNG_TRACE_SIZE;
    g_rng_trace[idx].seq = g_rng_trace_count;
    strncpy(g_rng_trace[idx].func, func, sizeof(g_rng_trace[idx].func) - 1);
    g_rng_trace[idx].func[sizeof(g_rng_trace[idx].func) - 1] = '\0';
    g_rng_trace[idx].arg = arg;
    g_rng_trace[idx].result = result;
    g_rng_trace_count++;
}

/* Get RNG trace as JSON array */
char* nh_ffi_get_rng_trace(void) {
    unsigned long count = g_rng_trace_count < RNG_TRACE_SIZE ? g_rng_trace_count : RNG_TRACE_SIZE;
    unsigned long start = g_rng_trace_count <= RNG_TRACE_SIZE ? 0 : (g_rng_trace_count % RNG_TRACE_SIZE);

    /* Estimate buffer: ~80 chars per entry */
    size_t buf_size = count * 80 + 16;
    char* json = (char*)malloc(buf_size);
    if (json == NULL) return strdup("[]");

    char *p = json;
    p += sprintf(p, "[");
    for (unsigned long i = 0; i < count; i++) {
        unsigned long idx = (start + i) % RNG_TRACE_SIZE;
        if (i > 0) p += sprintf(p, ",");
        p += sprintf(p, "{\"seq\":%lu,\"func\":\"%s\",\"arg\":%lu,\"result\":%lu}",
            g_rng_trace[idx].seq, g_rng_trace[idx].func,
            g_rng_trace[idx].arg, g_rng_trace[idx].result);
    }
    p += sprintf(p, "]");
    return json;
}

/* Clear the RNG trace buffer */
void nh_ffi_clear_rng_trace(void) {
    g_rng_trace_count = 0;
}

/* ============================================================================
 * Extended State Queries (Phase 2b: Convergence Framework)
 * ============================================================================ */

/* Get player nutrition */
int nh_ffi_get_nutrition(void) {
#ifdef REAL_NETHACK
    return u.uhunger;
#else
    return 900; /* Default starting nutrition */
#endif
}

/* Get player attributes as JSON */
char* nh_ffi_get_attributes_json(void) {
#ifdef REAL_NETHACK
    char* json = (char*)malloc(512);
    if (json == NULL) return NULL;
    snprintf(json, 512,
        "{\"str\": %d, \"int\": %d, \"wis\": %d, \"dex\": %d, \"con\": %d, \"cha\": %d}",
        ACURR(A_STR), ACURR(A_INT), ACURR(A_WIS),
        ACURR(A_DEX), ACURR(A_CON), ACURR(A_CHA));
    return json;
#else
    return strdup("{\"str\": 10, \"int\": 10, \"wis\": 10, \"dex\": 10, \"con\": 10, \"cha\": 10}");
#endif
}

/* Export C's viz_array as flat COLNO*ROWNO byte array (Rust [x][y] order) */
void nh_ffi_get_visibility(char* out) {
#ifdef REAL_NETHACK
    int x, y;
    for (x = 0; x < COLNO; x++) {
        for (y = 0; y < ROWNO; y++) {
            out[x * ROWNO + y] = (viz_array[y][x] & IN_SIGHT) ? 1 : 0;
        }
    }
#else
    memset(out, 0, COLNO * ROWNO);
#endif
}

void nh_ffi_get_couldsee(char* out) {
#ifdef REAL_NETHACK
    int x, y;
    for (x = 0; x < COLNO; x++) {
        for (y = 0; y < ROWNO; y++) {
            out[x * ROWNO + y] = (viz_array[y][x] & COULD_SEE) ? 1 : 0;
        }
    }
#else
    memset(out, 0, COLNO * ROWNO);
#endif
}

/* Export current level as JSON (cells, rooms, stairs, objects, monsters) */
char* nh_ffi_export_level(void) {
#ifdef REAL_NETHACK
    /* Use the 1MB global buffer */
    char *p = g_json_buffer;
    char *end = g_json_buffer + sizeof(g_json_buffer) - 256;

    p += sprintf(p, "{\"width\": %d, \"height\": %d, ", COLNO, ROWNO);
    p += sprintf(p, "\"dnum\": %d, \"dlevel\": %d, ", u.uz.dnum, u.uz.dlevel);

    /* Cells */
    p += sprintf(p, "\"cells\": [");
    for (int y = 0; y < ROWNO && p < end; y++) {
        p += sprintf(p, "[");
        for (int x = 0; x < COLNO && p < end; x++) {
            p += sprintf(p, "%d%s", level.locations[x][y].typ,
                (x < COLNO - 1) ? "," : "");
        }
        p += sprintf(p, "]%s", (y < ROWNO - 1) ? "," : "");
    }
    p += sprintf(p, "], ");

    /* Door states: array of {x, y, mask} for all door cells */
    p += sprintf(p, "\"doors\": [");
    {
        int first = 1;
        for (int y = 0; y < ROWNO && p < end; y++) {
            for (int x = 0; x < COLNO && p < end; x++) {
                if (level.locations[x][y].typ == DOOR) {
                    if (!first) p += sprintf(p, ",");
                    p += sprintf(p, "{\"x\":%d,\"y\":%d,\"mask\":%d}",
                        x, y, level.locations[x][y].doormask);
                    first = 0;
                }
            }
        }
    }
    p += sprintf(p, "], ");

    /* Rooms */
    p += sprintf(p, "\"rooms\": [");
    for (int i = 0; rooms[i].hx >= 0 && i < MAXNROFROOMS && p < end; i++) {
        if (i > 0) p += sprintf(p, ",");
        p += sprintf(p, "{\"lx\":%d,\"hx\":%d,\"ly\":%d,\"hy\":%d,\"type\":%d}",
            rooms[i].lx, rooms[i].hx, rooms[i].ly, rooms[i].hy, rooms[i].rtype);
    }
    p += sprintf(p, "], ");

    /* Stairs */
    p += sprintf(p, "\"stairs\": [");
    {
        int first = 1;
        if (xupstair) {
            p += sprintf(p, "{\"x\":%d,\"y\":%d,\"dir\":\"up\"}", xupstair, yupstair);
            first = 0;
        }
        if (xdnstair) {
            if (!first) p += sprintf(p, ",");
            p += sprintf(p, "{\"x\":%d,\"y\":%d,\"dir\":\"down\"}", xdnstair, ydnstair);
        }
    }
    p += sprintf(p, "], ");

    /* Objects on floor */
    p += sprintf(p, "\"objects\": [");
    {
        struct obj *otmp;
        int first = 1;
        for (otmp = fobj; otmp && p < end; otmp = otmp->nobj) {
            if (!first) p += sprintf(p, ",");
            p += sprintf(p, "{\"otyp\":%d,\"x\":%d,\"y\":%d,\"quan\":%d,\"spe\":%d,\"buc\":%d}",
                otmp->otyp, otmp->ox, otmp->oy, (int)otmp->quan, (int)otmp->spe,
                otmp->blessed ? 1 : (otmp->cursed ? -1 : 0));
            first = 0;
        }
    }
    p += sprintf(p, "], ");

    /* Monsters */
    /* Level flags for dosounds parity */
    p += sprintf(p, "\"nfountains\": %d, \"nsinks\": %d, ", level.flags.nfountains, level.flags.nsinks);
    p += sprintf(p, "\"has_court\": %d, \"has_swamp\": %d, \"has_vault\": %d, ",
        level.flags.has_court ? 1 : 0, level.flags.has_swamp ? 1 : 0, level.flags.has_vault ? 1 : 0);
    p += sprintf(p, "\"has_beehive\": %d, \"has_morgue\": %d, \"has_barracks\": %d, \"has_zoo\": %d, ",
        level.flags.has_beehive ? 1 : 0, level.flags.has_morgue ? 1 : 0,
        level.flags.has_barracks ? 1 : 0, level.flags.has_zoo ? 1 : 0);
    p += sprintf(p, "\"has_shop\": %d, \"has_temple\": %d, ",
        level.flags.has_shop ? 1 : 0, level.flags.has_temple ? 1 : 0);

    /* Monsters */
    p += sprintf(p, "\"monsters\": [");
    {
        struct monst *mtmp;
        int first = 1;
        for (mtmp = fmon; mtmp && p < end; mtmp = mtmp->nmon) {
            if (!first) p += sprintf(p, ",");
            p += sprintf(p, "{\"mnum\":%d,\"x\":%d,\"y\":%d,\"hp\":%d,\"hpmax\":%d,\"peaceful\":%d,\"asleep\":%d,\"mmove\":%d,\"mspeed\":%d}",
                mtmp->mnum, mtmp->mx, mtmp->my,
                mtmp->mhp, mtmp->mhpmax,
                mtmp->mpeaceful ? 1 : 0,
                mtmp->msleeping ? 1 : 0,
                mtmp->data->mmove,
                (int)mtmp->mspeed);
            first = 0;
        }
    }
    p += sprintf(p, "]");

    /* Engravings — iterate all cells via engr_at() since head_engr is static */
    p += sprintf(p, ", \"engravings\": [");
    {
        int ex, ey, first = 1;
        for (ex = 0; ex < COLNO && p < end; ex++) {
            for (ey = 0; ey < ROWNO && p < end; ey++) {
                struct engr *ep = engr_at(ex, ey);
                if (ep) {
                    char safe_txt[512];
                    int si = 0, di = 0;
                    /* Escape quotes and backslashes */
                    while (ep->engr_txt[si] && di < 510) {
                        char c = ep->engr_txt[si++];
                        if (c == '"' || c == '\\') safe_txt[di++] = '\\';
                        safe_txt[di++] = c;
                    }
                    safe_txt[di] = '\0';
                    if (!first) p += sprintf(p, ",");
                    p += sprintf(p, "{\"x\":%d,\"y\":%d,\"typ\":%d,\"txt\":\"%s\"}",
                        ex, ey, (int)ep->engr_type, safe_txt);
                    first = 0;
                }
            }
        }
    }
    p += sprintf(p, "]}");

    return strdup(g_json_buffer);
#else
    return strdup("{\"width\":80,\"height\":21,\"dnum\":0,\"dlevel\":1,\"cells\":[],\"rooms\":[],\"stairs\":[],\"objects\":[],\"monsters\":[],\"engravings\":[]}");
#endif
}

/* ============================================================================
 * Function-Level Isolation Testing (Phase 1: Parity Strategy)
 * ============================================================================ */

/* Re-implementation of finddpos() for isolation testing.
 * finddpos is STATIC_DCL in mklev.c, so we reproduce it here.
 * This is a direct copy of mklev.c:69-95 for test fidelity.
 */
void nh_ffi_test_finddpos(int xl, int yl, int xh, int yh, int *out_x, int *out_y) {
#ifdef REAL_NETHACK
    register xchar x, y;

    x = rn1(xh - xl + 1, xl);  /* rn2(range) + base */
    y = rn1(yh - yl + 1, yl);
    if (okdoor(x, y))
        goto gotit;

    for (x = xl; x <= xh; x++)
        for (y = yl; y <= yh; y++)
            if (okdoor(x, y))
                goto gotit;

    for (x = xl; x <= xh; x++)
        for (y = yl; y <= yh; y++)
            if (IS_DOOR(levl[x][y].typ) || levl[x][y].typ == SDOOR)
                goto gotit;
    /* cannot find something reasonable -- strange */
    x = xl;
    y = yh;
 gotit:
    *out_x = x;
    *out_y = y;
#else
    *out_x = xl;
    *out_y = yh;
#endif
}

/* Test dig_corridor() in isolation.
 * Returns 1 if corridor was dug successfully, 0 otherwise.
 * The level cells are modified in place — use export_level to read them.
 */
int nh_ffi_test_dig_corridor(int sx, int sy, int dx, int dy, int nxcor) {
#ifdef REAL_NETHACK
    coord org, dest;
    org.x = sx;
    org.y = sy;
    dest.x = dx;
    dest.y = dy;
    /* dig_corridor(org, dest, nxcor, ftyp=CORR, btyp=STONE) */
    return dig_corridor(&org, &dest, nxcor, CORR, STONE) ? 1 : 0;
#else
    (void)sx; (void)sy; (void)dx; (void)dy; (void)nxcor;
    return 0;
#endif
}

/* Test makecorridors() in isolation.
 * Call after generate_level or after manually setting up rooms.
 * Corridors are placed on the current level — use export_level to read them.
 */
void nh_ffi_test_makecorridors(void) {
#ifdef REAL_NETHACK
    makecorridors();
#endif
}

/* Export a rectangular region of level cells as a flat JSON array.
 * Returns JSON: [row0_col0_typ, row0_col1_typ, ...] (row-major, y then x).
 * Caller must free with nh_ffi_free_string().
 */
char* nh_ffi_get_cell_region(int x1, int y1, int x2, int y2) {
#ifdef REAL_NETHACK
    /* Clamp bounds */
    if (x1 < 0) x1 = 0;
    if (y1 < 0) y1 = 0;
    if (x2 >= COLNO) x2 = COLNO - 1;
    if (y2 >= ROWNO) y2 = ROWNO - 1;

    int w = x2 - x1 + 1;
    int h = y2 - y1 + 1;
    size_t buf_size = (size_t)(w * h) * 8 + 64;
    char* json = (char*)malloc(buf_size);
    if (!json) return strdup("[]");

    char *p = json;
    p += sprintf(p, "[");
    int first = 1;
    for (int y = y1; y <= y2; y++) {
        for (int x = x1; x <= x2; x++) {
            if (!first) p += sprintf(p, ",");
            p += sprintf(p, "%d", level.locations[x][y].typ);
            first = 0;
        }
    }
    p += sprintf(p, "]");
    return json;
#else
    (void)x1; (void)y1; (void)x2; (void)y2;
    return strdup("[]");
#endif
}

/* Set a single cell type on the current level (for isolation testing). */
void nh_ffi_set_cell(int x, int y, int typ) {
#ifdef REAL_NETHACK
    if (x >= 0 && x < COLNO && y >= 0 && y < ROWNO) {
        level.locations[x][y].typ = typ;
    }
#else
    (void)x; (void)y; (void)typ;
#endif
}

/* Debug: query cell state at (x,y) - returns JSON string with typ and doormask */
char* nh_ffi_debug_cell(int x, int y) {
#ifdef REAL_NETHACK
    static char buf[256];
    if (x >= 0 && x < COLNO && y >= 0 && y < ROWNO) {
        int typ = level.locations[x][y].typ;
        int mask = level.locations[x][y].doormask;
        snprintf(buf, sizeof(buf), "C_CELL(%d,%d) typ=%d (DOOR=%d) mask=0x%02x (D_CLOSED=%d D_LOCKED=%d D_ISOPEN=%d)",
            x, y, typ, (typ == DOOR), mask,
            (mask & D_CLOSED) != 0, (mask & D_LOCKED) != 0, (mask & D_ISOPEN) != 0);
    } else {
        snprintf(buf, sizeof(buf), "C_CELL(%d,%d) out of bounds", x, y);
    }
    return buf;
#else
    static char buf[64];
    snprintf(buf, sizeof(buf), "C_CELL_NOOP(%d,%d)", x, y);
    return buf;
#endif
}

/* Debug: dump mfndpos results for monster at given index - returns string */
char* nh_ffi_debug_mfndpos(int mon_index) {
#ifdef REAL_NETHACK
    static char buf[1024];
    char *p = buf;
    char *end = buf + sizeof(buf) - 64;
    struct monst *mtmp;
    int idx = 0;
    for (mtmp = fmon; mtmp; mtmp = mtmp->nmon, idx++) {
        if (idx == mon_index && !DEADMONSTER(mtmp)) {
            coord poss[9];
            long info[9];
            long flag = 0L;
            struct permonst *ptr = mtmp->data;
            boolean can_open = !(nohands(ptr) || verysmall(ptr));
            boolean can_unlock = ((can_open && monhaskey(mtmp, TRUE)) || mtmp->iswiz || is_rider(ptr));
            boolean doorbuster = is_giant(ptr);

            if (!mtmp->mpeaceful)
                flag |= ALLOW_U;
            if (can_open)
                flag |= OPENDOOR;
            if (can_unlock)
                flag |= UNLOCKDOOR;
            if (doorbuster)
                flag |= BUSTDOOR;

            int cnt = mfndpos(mtmp, poss, info, flag);
            p += snprintf(p, end - p, "C_MFNDPOS: mon %d '%s' at (%d,%d) can_open=%d nohands=%d verysmall=%d flag=0x%lx cnt=%d positions=[",
                mon_index, ptr->mname, mtmp->mx, mtmp->my, can_open,
                nohands(ptr) != 0, verysmall(ptr) != 0, flag, cnt);
            for (int i = 0; i < cnt && p < end; i++) {
                if (i > 0) p += snprintf(p, end - p, ", ");
                p += snprintf(p, end - p, "(%d,%d)", poss[i].x, poss[i].y);
            }
            p += snprintf(p, end - p, "]");
            /* Also show adjacent door cells */
            for (int dx = -1; dx <= 1 && p < end; dx++) {
                for (int dy = -1; dy <= 1 && p < end; dy++) {
                    if (dx == 0 && dy == 0) continue;
                    int nx = mtmp->mx + dx;
                    int ny = mtmp->my + dy;
                    if (nx >= 1 && nx < COLNO && ny >= 0 && ny < ROWNO) {
                        int typ = level.locations[nx][ny].typ;
                        if (IS_DOOR(typ)) {
                            int mask = level.locations[nx][ny].doormask;
                            p += snprintf(p, end - p, " adj(%d,%d)=DOOR mask=0x%02x", nx, ny, mask);
                        }
                    }
                }
            }
            return buf;
        }
    }
    snprintf(buf, sizeof(buf), "C_MFNDPOS: mon %d not found", mon_index);
    return buf;
#else
    static char buf[64];
    snprintf(buf, sizeof(buf), "C_MFNDPOS_NOOP(%d)", mon_index);
    return buf;
#endif
}

/* Clear the entire level to STONE (for isolation testing). */
void nh_ffi_clear_level(void) {
#ifdef REAL_NETHACK
    for (int x = 0; x < COLNO; x++) {
        for (int y = 0; y < ROWNO; y++) {
            level.locations[x][y].typ = STONE;
            level.locations[x][y].lit = 0;
        }
    }
    /* Also reset rooms */
    nroom = 0;
    nsubroom = 0;
    doorindex = 0;
    for (int i = 0; i < MAXNROFROOMS; i++) {
        rooms[i].hx = -1;
        smeq[i] = 0;
    }
#endif
}

/* Set up a room in C's rooms[] array (for isolation testing of corridors).
 * Returns the room index, or -1 if max rooms reached.
 */
int nh_ffi_add_room(int lx, int ly, int hx, int hy, int rtype) {
#ifdef REAL_NETHACK
    if (nroom >= MAXNROFROOMS) return -1;
    int idx = nroom;
    rooms[idx].lx = lx;
    rooms[idx].ly = ly;
    rooms[idx].hx = hx;
    rooms[idx].hy = hy;
    rooms[idx].rtype = rtype;
    rooms[idx].rlit = 0;
    rooms[idx].doorct = 0;
    rooms[idx].fdoor = 0;
    rooms[idx].nsubrooms = 0;
    smeq[idx] = idx;  /* Each room starts as its own equivalence class */
    nroom++;
    /* Sentinel */
    rooms[nroom].hx = -1;
    return idx;
#else
    (void)lx; (void)ly; (void)hx; (void)hy; (void)rtype;
    return -1;
#endif
}

/* Carve a room's interior and walls on the current level (like create_room).
 * Matches the wall/floor layout from mklev.c add_room / makelevel.
 */
void nh_ffi_carve_room(int lx, int ly, int hx, int hy) {
#ifdef REAL_NETHACK
    /* Interior: ROOM */
    for (int x = lx; x <= hx; x++) {
        for (int y = ly; y <= hy; y++) {
            level.locations[x][y].typ = ROOM;
        }
    }
    /* Walls around the room */
    /* Top and bottom walls */
    for (int x = lx - 1; x <= hx + 1; x++) {
        if (x >= 0 && x < COLNO) {
            if (ly - 1 >= 0) level.locations[x][ly-1].typ = HWALL;
            if (hy + 1 < ROWNO) level.locations[x][hy+1].typ = HWALL;
        }
    }
    /* Left and right walls */
    for (int y = ly; y <= hy; y++) {
        if (lx - 1 >= 0) level.locations[lx-1][y].typ = VWALL;
        if (hx + 1 < COLNO) level.locations[hx+1][y].typ = VWALL;
    }
    /* Corners */
    if (lx-1 >= 0 && ly-1 >= 0) level.locations[lx-1][ly-1].typ = TLCORNER;
    if (hx+1 < COLNO && ly-1 >= 0) level.locations[hx+1][ly-1].typ = TRCORNER;
    if (lx-1 >= 0 && hy+1 < ROWNO) level.locations[lx-1][hy+1].typ = BLCORNER;
    if (hx+1 < COLNO && hy+1 < ROWNO) level.locations[hx+1][hy+1].typ = BRCORNER;
#else
    (void)lx; (void)ly; (void)hx; (void)hy;
#endif
}

/* Get the rectangle list from C's rect.c (static variables exposed via helper in rect.c) */
extern int nh_ffi_rect_count(void);
extern void nh_ffi_get_rect_list(int *out_count, int *out_coords);

char* nh_ffi_get_rect_json(void) {
#ifdef REAL_NETHACK
    int count = 0;
    int coords[200]; /* 50 rects * 4 coords */
    nh_ffi_get_rect_list(&count, coords);

    char *buf = (char*)malloc(4096);
    char *p = buf;
    p += sprintf(p, "{\"count\":%d,\"rects\":[", count);
    for (int i = 0; i < count; i++) {
        if (i > 0) p += sprintf(p, ",");
        p += sprintf(p, "[%d,%d,%d,%d]", coords[i*4], coords[i*4+1], coords[i*4+2], coords[i*4+3]);
    }
    p += sprintf(p, "]}");
    return buf;
#else
    return strdup("{\"count\":0,\"rects\":[]}");
#endif
}

/* Re-implementation of C's join() for step-by-step isolation testing.
 * Uses our re-implemented finddpos (same as mklev.c), and the public
 * okdoor(), dodoor(), dig_corridor() functions.
 * Modifies level cells, smeq[], and doorindex — just like the real join().
 */
void nh_ffi_test_join(int a, int b, int nxcor) {
#ifdef REAL_NETHACK
    coord cc, tt, org, dest;
    register xchar tx, ty, xx, yy;
    register struct mkroom *croom, *troom;
    register int dx, dy;

    croom = &rooms[a];
    troom = &rooms[b];

    if (troom->hx < 0 || croom->hx < 0 || doorindex >= DOORMAX)
        return;

    /* Use temp ints for finddpos output (cc/tt fields are xchar/schar) */
    int cc_x, cc_y, tt_x, tt_y;

    if (troom->lx > croom->hx) {
        dx = 1;
        dy = 0;
        xx = croom->hx + 1;
        tx = troom->lx - 1;
        nh_ffi_test_finddpos(xx, croom->ly, xx, croom->hy, &cc_x, &cc_y);
        nh_ffi_test_finddpos(tx, troom->ly, tx, troom->hy, &tt_x, &tt_y);
    } else if (troom->hy < croom->ly) {
        dy = -1;
        dx = 0;
        yy = croom->ly - 1;
        nh_ffi_test_finddpos(croom->lx, yy, croom->hx, yy, &cc_x, &cc_y);
        ty = troom->hy + 1;
        nh_ffi_test_finddpos(troom->lx, ty, troom->hx, ty, &tt_x, &tt_y);
    } else if (troom->hx < croom->lx) {
        dx = -1;
        dy = 0;
        xx = croom->lx - 1;
        tx = troom->hx + 1;
        nh_ffi_test_finddpos(xx, croom->ly, xx, croom->hy, &cc_x, &cc_y);
        nh_ffi_test_finddpos(tx, troom->ly, tx, troom->hy, &tt_x, &tt_y);
    } else {
        dy = 1;
        dx = 0;
        yy = croom->hy + 1;
        ty = troom->ly - 1;
        nh_ffi_test_finddpos(croom->lx, yy, croom->hx, yy, &cc_x, &cc_y);
        nh_ffi_test_finddpos(troom->lx, ty, troom->hx, ty, &tt_x, &tt_y);
    }

    cc.x = (xchar)cc_x;
    cc.y = (xchar)cc_y;
    tt.x = (xchar)tt_x;
    tt.y = (xchar)tt_y;

    xx = cc.x;
    yy = cc.y;
    tx = tt.x - dx;
    ty = tt.y - dy;

    if (nxcor && levl[xx + dx][yy + dy].typ)
        return;

    if (okdoor(xx, yy) || !nxcor)
        dodoor(xx, yy, croom);

    org.x = xx + dx;
    org.y = yy + dy;
    dest.x = tx;
    dest.y = ty;

    if (!dig_corridor(&org, &dest, (boolean)nxcor,
                      level.flags.arboreal ? ROOM : CORR, STONE))
        return;

    if (okdoor(tt.x, tt.y) || !nxcor)
        dodoor(tt.x, tt.y, troom);

    if (smeq[a] < smeq[b])
        smeq[b] = smeq[a];
    else
        smeq[a] = smeq[b];
#else
    (void)a; (void)b; (void)nxcor;
#endif
}

/* Get smeq array as JSON for comparing connectivity state.
 * Returns JSON: [smeq[0], smeq[1], ...] for nroom entries.
 */
char* nh_ffi_get_smeq(void) {
#ifdef REAL_NETHACK
    char buf[1024];
    char *p = buf;
    p += sprintf(p, "[");
    for (int i = 0; i < nroom; i++) {
        if (i > 0) p += sprintf(p, ",");
        p += sprintf(p, "%d", smeq[i]);
    }
    p += sprintf(p, "]");
    return strdup(buf);
#else
    return strdup("[]");
#endif
}

/* Get doorindex for diagnostics. */
int nh_ffi_get_doorindex(void) {
#ifdef REAL_NETHACK
    return doorindex;
#else
    return 0;
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
