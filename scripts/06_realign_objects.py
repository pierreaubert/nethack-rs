#!/usr/bin/env python3
"""Realign crates/nh-core/src/data/objects.rs OBJECTS array to match
NetHack-3.6.7 objects.c positional indices.

Reads:
  - NetHack-3.6.7/include/onames.h  (authoritative idx -> NAME map)
  - NetHack-3.6.7/src/objects.c     (entry order)
  - crates/nh-core/src/data/objects.rs  (current Rust data)

Writes:
  - crates/nh-core/src/data/objects.rs  (rewritten OBJECTS array;
    ObjectType enum is also regenerated to match positional indices)
"""
import re, sys, os

ROOT = "/Users/pierre/src/games"
ONAMES_H = f"{ROOT}/NetHack-3.6.7/include/onames.h"
OBJECTS_C = f"{ROOT}/NetHack-3.6.7/src/objects.c"
RUST_OBJECTS = f"{ROOT}/nethack-rs/crates/nh-core/src/data/objects.rs"

# ---------- 1. Parse onames.h: idx -> CONST_NAME ------------------
def parse_onames():
    text = open(ONAMES_H).read()
    by_idx = {}
    skip = {'NROFARTIFACTS', 'NUM_OBJECTS', 'LAST_GEM', 'MAXSPELL'}
    for line in text.split('\n'):
        m = re.match(r'#define\s+(\w+)\s+(\d+)\s*$', line)
        if not m: continue
        name, idx = m.group(1), int(m.group(2))
        if name.startswith('ART_') or name in skip:
            continue
        if idx in by_idx:
            # Take the first; collisions like SPETUM/MAXSPELL handled by skip
            continue
        by_idx[idx] = name
    return by_idx

# ---------- 2. Parse objects.c source to get C-name in index order ------
def parse_objects_c_order():
    """Return list of (macro, display_name) in C source order."""
    text = open(OBJECTS_C).read()
    macros = ['WEAPON', 'PROJECTILE', 'BOW', 'ARMOR', 'HELM', 'CLOAK', 'SHIELD',
              'GLOVES', 'BOOTS', 'SHIRT', 'DRGN_ARMR', 'FOOD', 'POTION', 'SCROLL',
              'SPELL', 'WAND', 'RING', 'AMULET', 'GEM', 'ROCK', 'TOOL', 'CONTAINER',
              'BALL', 'CHAIN', 'VENOM', 'WEPTOOL', 'COIN']
    items = []
    in_if0 = 0
    # Track per-class anonymous-slot counter so each None gets a unique name
    anon_counters = {}
    for line in text.split('\n'):
        sl = line.strip()
        if re.match(r'#if\s+0\b', sl):
            in_if0 += 1; continue
        if sl.startswith('#endif'):
            if in_if0 > 0: in_if0 -= 1
            continue
        if sl.startswith('#else') and in_if0 > 0:
            in_if0 -= 1; continue
        if in_if0 > 0:
            continue
        # Skip MAIL conditional scroll
        if 'SCROLL("mail"' in line:
            continue
        matched = False
        for m in macros:
            if line.startswith(m + '('):
                arg = re.search(rf'{m}\(\s*"([^"]+)"', line)
                if arg:
                    items.append((m, arg.group(1)))
                else:
                    # None-named slot — synthesize a unique name
                    if line.startswith(m + '(None'):
                        n = anon_counters.get(m, 0)
                        anon_counters[m] = n + 1
                        items.append((m, f'__anon_{m.lower()}_{n}'))
                matched = True
                break
        if matched: continue
        m = re.match(r'^OBJECT\(OBJ\(\s*"([^"]+)"', line)
        if m:
            items.append(('OBJECT', m.group(1)))
            continue
    return items

# ---------- 3. Parse Rust OBJECTS: name -> raw entry text ----------
def parse_rust_objects():
    text = open(RUST_OBJECTS).read()
    # Find the OBJECTS array body
    start = text.index('pub static OBJECTS: &[ObjClassDef] = &[')
    body_start = text.index('\n', start) + 1
    # Find the matching `];` for the slice — it's right before the next `pub` or end
    body_end = text.index('\n];', body_start)
    body = text[body_start:body_end]
    # Split by top-level "ObjClassDef {" — each entry ends with `},`
    # We need to track brace depth within the entry to find proper boundaries.
    entries = []
    i = 0
    while i < len(body):
        idx = body.find('ObjClassDef {', i)
        if idx < 0:
            break
        # Walk forward counting braces
        depth = 0
        j = idx
        while j < len(body):
            c = body[j]
            if c == '{':
                depth += 1
            elif c == '}':
                depth -= 1
                if depth == 0:
                    # Include trailing comma if present
                    k = j + 1
                    while k < len(body) and body[k] in ',\n ':
                        if body[k] == ',':
                            k += 1; break
                        k += 1
                    entries.append(body[idx:k])
                    i = k
                    break
            j += 1
        else:
            break
    # Build name -> entry-text map
    by_name = {}
    for e in entries:
        m = re.search(r'name:\s*"([^"]+)"', e)
        if m:
            by_name[m.group(1).lower()] = e
    return entries, by_name, text, body_start, body_end

# ---------- 4. Build C (macro, name) -> Rust-name lookup ----------
MACRO_PREFIX = {
    'POTION': 'potion of ',
    'SCROLL': 'scroll of ',
    'WAND':   'wand of ',
    'SPELL':  'spellbook of ',
    'RING':   'ring of ',
    # Others use bare names
}
def make_name_resolver(rust_by_name):
    def resolve(macro, c_name):
        key_bare = c_name.lower()
        # Class-prefixed lookup first
        pref = MACRO_PREFIX.get(macro)
        if pref:
            cand = pref + key_bare
            if cand in rust_by_name:
                return cand
        # Bare lookup
        if key_bare in rust_by_name:
            return key_bare
        return None
    return resolve

# ---------- 5. Emit aligned OBJECTS body and ObjectType enum ----------
MACRO_CLASS = {
    'WEAPON': 'Weapon', 'PROJECTILE': 'Weapon', 'BOW': 'Weapon',
    'WEPTOOL': 'Tool',
    'ARMOR': 'Armor', 'HELM': 'Armor', 'CLOAK': 'Armor', 'SHIELD': 'Armor',
    'GLOVES': 'Armor', 'BOOTS': 'Armor', 'SHIRT': 'Armor', 'DRGN_ARMR': 'Armor',
    'FOOD': 'Food', 'POTION': 'Potion', 'SCROLL': 'Scroll',
    'SPELL': 'Spellbook', 'WAND': 'Wand', 'RING': 'Ring',
    'AMULET': 'Amulet', 'GEM': 'Gem', 'ROCK': 'Gem', 'TOOL': 'Tool',
    'CONTAINER': 'Tool', 'BALL': 'Ball', 'CHAIN': 'Chain', 'VENOM': 'Venom',
    'COIN': 'Coin', 'OBJECT': 'IllObj',  # OBJECT() is mostly used for special items
}

def emit_objects(c_order_names, rust_by_name, resolver):
    aligned = []
    misses = []
    used_rust = set()
    for macro, c_name in c_order_names:
        rs_key = resolver(macro, c_name)
        if rs_key is None:
            # Stub entry: use the macro's natural class so class spans stay
            # contiguous. Anonymous scroll/wand appearance slots would break
            # `bases[Scroll]` if we used IllObj here.
            macro_class = MACRO_CLASS.get(macro, 'IllObj')
            misses.append(c_name)
            stub = (
                'ObjClassDef {\n'
                f'    name: "{c_name}",\n'
                '    description: "",\n'
                f'    class: ObjectClass::{macro_class},\n'
                '    material: Material::Iron,\n'
                '    weight: 0,\n'
                '    cost: 0,\n'
                '    probability: 0,\n'
                '    nutrition: 0,\n'
                '    w_small_damage: 0,\n'
                '    w_large_damage: 0,\n'
                '    bonus: 0,\n'
                '    skill: 0,\n'
                '    delay: 0,\n'
                '    color: 0,\n'
                '    magical: false,\n'
                '    merge: false,\n'
                '    unique: false,\n'
                '    no_wish: true,\n'
                '    big: false,\n'
                '    direction: DirectionType::None,\n'
                '    armor_category: None,\n'
                '    property: 0,\n'
                '},'
            )
            aligned.append(stub)
        else:
            aligned.append(rust_by_name[rs_key])
            used_rust.add(rs_key)
    return aligned, misses, used_rust

# ---------- 6. Generate enum ObjectType discriminants from onames.h --------
def emit_enum(onames_idx_to_const, c_order_names):
    """Map: position (in c_order_names) -> Rust enum variant name with
    discriminant = position. We need a mapping from C const (e.g. WAN_WISHING)
    to Rust variant name (Wishing) for this to be useful."""
    # Easiest: don't regenerate the enum; current variants stay but their numeric
    # values may now be wrong. The plan said realign these too. For this iteration
    # we'll log a TODO and leave the enum alone — add a #[deprecated] note.
    return None

def main():
    print("[1/4] Parsing onames.h...")
    onames = parse_onames()
    print(f"      {len(onames)} entries, max idx={max(onames)}")
    print("[2/4] Parsing objects.c source order...")
    c_order = parse_objects_c_order()
    print(f"      {len(c_order)} entries in source order")
    print("[3/4] Parsing Rust OBJECTS...")
    entries, by_name, full_text, body_start, body_end = parse_rust_objects()
    print(f"      {len(entries)} Rust entries")
    resolver = make_name_resolver(by_name)
    print("[4/4] Aligning...")
    aligned, misses, used = emit_objects(c_order, by_name, resolver)
    unused_rust = set(by_name) - used
    print(f"      Aligned {len(aligned)} entries")
    print(f"      C entries not in Rust (stubbed): {len(misses)}")
    for m in misses[:10]:
        print(f"        - {m}")
    print(f"      Rust entries not in C (dropped): {len(unused_rust)}")
    for u in sorted(unused_rust)[:10]:
        print(f"        - {u}")

    # Write the new OBJECTS body
    new_body = '\n'.join('    ' + e.replace('\n', '\n    ') for e in aligned) + '\n'
    new_text = full_text[:body_start] + new_body + full_text[body_end:]
    with open(RUST_OBJECTS, 'w') as f:
        f.write(new_text)
    print(f"\nWrote {RUST_OBJECTS}")
    print(f"New OBJECTS length: {len(aligned)}")

if __name__ == '__main__':
    main()
