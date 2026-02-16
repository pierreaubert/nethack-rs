/*
 * Standalone ISAAC64 implementation for testing
 * Extracted from NetHack 3.6.7 - isaac64.c
 * Original by Timothy B. Terriberry (CC0 Public Domain)
 */

#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#define ISAAC64_SZ_LOG 8
#define ISAAC64_SZ (1 << ISAAC64_SZ_LOG)
#define ISAAC64_SEED_SZ_MAX (ISAAC64_SZ << 3)
#define ISAAC64_MASK ((uint64_t)0xFFFFFFFFFFFFFFFFULL)

typedef struct isaac64_ctx {
    unsigned n;
    uint64_t r[ISAAC64_SZ];
    uint64_t m[ISAAC64_SZ];
    uint64_t a;
    uint64_t b;
    uint64_t c;
} isaac64_ctx;

/* Extract ISAAC64_SZ_LOG bits (starting at bit 3). */
static inline uint32_t lower_bits(uint64_t x)
{
    return (x & ((ISAAC64_SZ-1) << 3)) >> 3;
}

/* Extract next ISAAC64_SZ_LOG bits (starting at bit ISAAC64_SZ_LOG+2). */
static inline uint32_t upper_bits(uint64_t y)
{
    return (y >> (ISAAC64_SZ_LOG+3)) & (ISAAC64_SZ-1);
}

static void isaac64_update(isaac64_ctx *_ctx)
{
    uint64_t *m;
    uint64_t *r;
    uint64_t a;
    uint64_t b;
    uint64_t x;
    uint64_t y;
    int i;

    m = _ctx->m;
    r = _ctx->r;
    a = _ctx->a;
    b = _ctx->b + (++_ctx->c);

    for (i = 0; i < ISAAC64_SZ/2; i++) {
        x = m[i];
        a = ~(a ^ a<<21) + m[i + ISAAC64_SZ/2];
        m[i] = y = m[lower_bits(x)] + a + b;
        r[i] = b = m[upper_bits(y)] + x;

        x = m[++i];
        a = (a ^ a>>5) + m[i + ISAAC64_SZ/2];
        m[i] = y = m[lower_bits(x)] + a + b;
        r[i] = b = m[upper_bits(y)] + x;

        x = m[++i];
        a = (a ^ a<<12) + m[i + ISAAC64_SZ/2];
        m[i] = y = m[lower_bits(x)] + a + b;
        r[i] = b = m[upper_bits(y)] + x;

        x = m[++i];
        a = (a ^ a>>33) + m[i + ISAAC64_SZ/2];
        m[i] = y = m[lower_bits(x)] + a + b;
        r[i] = b = m[upper_bits(y)] + x;
    }

    for (i = ISAAC64_SZ/2; i < ISAAC64_SZ; i++) {
        x = m[i];
        a = ~(a ^ a<<21) + m[i - ISAAC64_SZ/2];
        m[i] = y = m[lower_bits(x)] + a + b;
        r[i] = b = m[upper_bits(y)] + x;

        x = m[++i];
        a = (a ^ a>>5) + m[i - ISAAC64_SZ/2];
        m[i] = y = m[lower_bits(x)] + a + b;
        r[i] = b = m[upper_bits(y)] + x;

        x = m[++i];
        a = (a ^ a<<12) + m[i - ISAAC64_SZ/2];
        m[i] = y = m[lower_bits(x)] + a + b;
        r[i] = b = m[upper_bits(y)] + x;

        x = m[++i];
        a = (a ^ a>>33) + m[i - ISAAC64_SZ/2];
        m[i] = y = m[lower_bits(x)] + a + b;
        r[i] = b = m[upper_bits(y)] + x;
    }

    _ctx->b = b;
    _ctx->a = a;
    _ctx->n = ISAAC64_SZ;
}

static void isaac64_mix(uint64_t _x[8])
{
    static const unsigned char SHIFT[8] = {9, 9, 23, 15, 14, 20, 17, 14};
    int i;

    for (i = 0; i < 8; i++) {
        _x[i] -= _x[(i+4) & 7];
        _x[(i+5) & 7] ^= _x[(i+7) & 7] >> SHIFT[i];
        _x[(i+7) & 7] += _x[i];
        i++;
        _x[i] -= _x[(i+4) & 7];
        _x[(i+5) & 7] ^= _x[(i+7) & 7] << SHIFT[i];
        _x[(i+7) & 7] += _x[i];
    }
}

void isaac64_init(isaac64_ctx *_ctx, const unsigned char *_seed, int _nseed)
{
    _ctx->a = _ctx->b = _ctx->c = 0;
    memset(_ctx->r, 0, sizeof(_ctx->r));

    /* Inline reseed logic */
    uint64_t *m = _ctx->m;
    uint64_t *r = _ctx->r;
    uint64_t x[8];
    int i, j;

    if (_nseed > ISAAC64_SEED_SZ_MAX)
        _nseed = ISAAC64_SEED_SZ_MAX;

    for (i = 0; i < _nseed >> 3; i++) {
        r[i] ^= (uint64_t)_seed[i<<3|7] << 56 | (uint64_t)_seed[i<<3|6] << 48 |
                (uint64_t)_seed[i<<3|5] << 40 | (uint64_t)_seed[i<<3|4] << 32 |
                (uint64_t)_seed[i<<3|3] << 24 | (uint64_t)_seed[i<<3|2] << 16 |
                (uint64_t)_seed[i<<3|1] << 8  | _seed[i<<3];
    }
    _nseed -= i << 3;

    if (_nseed > 0) {
        uint64_t ri = _seed[i<<3];
        for (j = 1; j < _nseed; j++)
            ri |= (uint64_t)_seed[i<<3|j] << (j<<3);
        r[i++] ^= ri;
    }

    x[0] = x[1] = x[2] = x[3] = x[4] = x[5] = x[6] = x[7] = (uint64_t)0x9E3779B97F4A7C13ULL;

    for (i = 0; i < 4; i++)
        isaac64_mix(x);

    for (i = 0; i < ISAAC64_SZ; i += 8) {
        for (j = 0; j < 8; j++)
            x[j] += r[i+j];
        isaac64_mix(x);
        memcpy(m+i, x, sizeof(x));
    }

    for (i = 0; i < ISAAC64_SZ; i += 8) {
        for (j = 0; j < 8; j++)
            x[j] += m[i+j];
        isaac64_mix(x);
        memcpy(m+i, x, sizeof(x));
    }

    isaac64_update(_ctx);
}

uint64_t isaac64_next_uint64(isaac64_ctx *_ctx)
{
    if (!_ctx->n)
        isaac64_update(_ctx);
    return _ctx->r[--_ctx->n];
}

uint64_t isaac64_next_uint(isaac64_ctx *_ctx, uint64_t _n)
{
    uint64_t r;
    uint64_t v;
    uint64_t d;

    do {
        r = isaac64_next_uint64(_ctx);
        v = r % _n;
        d = r - v;
    } while (((d + _n - 1) & ISAAC64_MASK) < d);

    return v;
}

/* Global context for the standalone RNG */
static isaac64_ctx g_isaac64_ctx;
static isaac64_ctx g_disp_rng_ctx;

void set_random_generator_seed(unsigned long seed) {
    unsigned char seed_bytes[8];
    for (int i = 0; i < 8; i++) {
        seed_bytes[i] = (unsigned char)((seed >> (i * 8)) & 0xFF);
    }
    isaac64_init(&g_isaac64_ctx, seed_bytes, 8);
    isaac64_init(&g_disp_rng_ctx, seed_bytes, 8);
}

/* Core RNG functions from NetHack's rnd.c */

int rn2(int n) {
    if (n <= 0) return 0;
    int res = (int)isaac64_next_uint(&g_isaac64_ctx, (uint64_t)n);
    return res;
}

int rnd(int n) {
    if (n <= 0) return 1;
    return rn2(n) + 1;
}

int d(int n, int x) {
    int res = n;
    if (x <= 0 || n <= 0) return n;
    while (n--) res += rn2(x);
    return res;
}

int rn2_on_display_rng(int x) {
    if (x <= 0) return 0;
    return (int)isaac64_next_uint(&g_disp_rng_ctx, (uint64_t)x);
}

/* Stubs/Simplified versions of other rnd.c functions */
/* These might need NetHack globals like 'u' and 'Luck' */

#ifdef REAL_NETHACK
#include "hack.h"

int rnl(int x) {
    int adjustment = Luck;
    if (x <= 15) {
        adjustment = (abs(adjustment) + 1) / 3 * (adjustment < 0 ? -1 : (adjustment > 0 ? 1 : 0));
    }
    int i = rn2(x);
    if (adjustment && rn2(37 + abs(adjustment))) {
        i -= adjustment;
        if (i < 0) i = 0;
        else if (i >= x) i = x - 1;
    }
    return i;
}

int rne(int x) {
    int utmp = (u.ulevel < 15) ? 5 : u.ulevel / 3;
    int tmp = 1;
    while (tmp < utmp && !rn2(x)) tmp++;
    return tmp;
}

int rnz(int i) {
    long x = (long) i;
    long tmp = 1000L;
    tmp += rn2(1000);
    tmp *= rne(4);
    if (rn2(2)) {
        x *= tmp;
        x /= 1000;
    } else {
        x *= 1000;
        x /= tmp;
    }
    return (int) x;
}
#endif
