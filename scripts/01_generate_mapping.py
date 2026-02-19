#!/usr/bin/env python3
"""
Generate comprehensive mapping.json for nh-assets from nh-core Rust sources.

Parses Rust source files to extract all game entities (monsters, objects,
dungeon features, traps, artifacts, player roles) and generates a mapping
file that maps each entity to its visual representation (TUI character,
TUI color, and Bevy sprite path).

Usage:
    python scripts/generate_mapping.py [--output PATH]
"""

import json
import re
import argparse
import sys
from pathlib import Path


def find_project_root() -> Path:
    """Find the project root by looking for Cargo.toml relative to this script."""
    path = Path(__file__).resolve().parent.parent
    if (path / "Cargo.toml").exists():
        return path
    # Fallback to cwd
    cwd = Path.cwd()
    if (cwd / "Cargo.toml").exists():
        return cwd
    print("Error: Cannot find project root (no Cargo.toml found)", file=sys.stderr)
    sys.exit(1)


# ============================================================================
# Color mapping: Rust constant name -> TUI color string
# ============================================================================

COLOR_CONST_MAP: dict[str, str] = {
    # Base colors (CLR_*)
    "CLR_BLACK": "black",
    "CLR_RED": "red",
    "CLR_GREEN": "green",
    "CLR_BROWN": "brown",
    "CLR_BLUE": "blue",
    "CLR_MAGENTA": "magenta",
    "CLR_CYAN": "cyan",
    "CLR_GRAY": "gray",
    "NO_COLOR": "gray",
    "CLR_ORANGE": "orange",
    "CLR_BRIGHT_GREEN": "lightgreen",
    "CLR_YELLOW": "yellow",
    "CLR_BRIGHT_BLUE": "lightblue",
    "CLR_BRIGHT_MAGENTA": "lightmagenta",
    "CLR_BRIGHT_CYAN": "lightcyan",
    "CLR_WHITE": "white",
    # Material-based aliases (HI_*)
    "HI_DOMESTIC": "white",
    "HI_LORD": "magenta",
    "HI_OBJ": "magenta",
    "HI_METAL": "cyan",
    "HI_COPPER": "yellow",
    "HI_SILVER": "gray",
    "HI_GOLD": "yellow",
    "HI_LEATHER": "brown",
    "HI_CLOTH": "brown",
    "HI_ORGANIC": "brown",
    "HI_WOOD": "brown",
    "HI_PAPER": "white",
    "HI_GLASS": "lightcyan",
    "HI_MINERAL": "gray",
    "DRAGON_SILVER": "lightcyan",
    "HI_ZAP": "lightblue",
}


def resolve_color(const_name: str) -> str:
    """Resolve a Rust color constant name to a TUI color string."""
    return COLOR_CONST_MAP.get(const_name.strip(), "gray")


# ============================================================================
# ObjectClass -> TUI symbol mapping (from objclass.rs ObjectClass::symbol())
# ============================================================================

CLASS_SYMBOLS: dict[str, str] = {
    "Random": "?",
    "IllObj": "]",
    "Weapon": ")",
    "Armor": "[",
    "Ring": "=",
    "Amulet": "\"",
    "Tool": "(",
    "Food": "%",
    "Potion": "!",
    "Scroll": "?",
    "Spellbook": "+",
    "Wand": "/",
    "Coin": "$",
    "Gem": "*",
    "Rock": "`",
    "Ball": "0",
    "Chain": "_",
    "Venom": ".",
}

# ============================================================================
# CellType -> (tui_char, tui_color) mapping (from tile.rs + classic NetHack)
# ============================================================================

CELL_DISPLAY: dict[str, tuple[str, str]] = {
    "Stone": (" ", "gray"),
    "VWall": ("|", "gray"),
    "HWall": ("-", "gray"),
    "TLCorner": ("+", "gray"),
    "TRCorner": ("+", "gray"),
    "BLCorner": ("+", "gray"),
    "BRCorner": ("+", "gray"),
    "CrossWall": ("+", "gray"),
    "TUWall": ("+", "gray"),
    "TDWall": ("+", "gray"),
    "TLWall": ("+", "gray"),
    "TRWall": ("+", "gray"),
    "DBWall": ("+", "gray"),
    "Tree": ("#", "green"),
    "SecretDoor": ("+", "gray"),
    "SecretCorridor": ("#", "gray"),
    "Pool": ("}", "blue"),
    "Moat": ("}", "blue"),
    "Water": ("}", "blue"),
    "DrawbridgeUp": ("#", "brown"),
    "Lava": ("}", "red"),
    "IronBars": ("#", "cyan"),
    "Door": ("+", "brown"),
    "Corridor": ("#", "gray"),
    "Room": (".", "gray"),
    "Stairs": (">", "gray"),
    "Ladder": (">", "gray"),
    "Fountain": ("{", "lightblue"),
    "Throne": ("\\", "yellow"),
    "Sink": ("#", "gray"),
    "Grave": ("|", "gray"),
    "Altar": ("_", "gray"),
    "Ice": (".", "lightcyan"),
    "DrawbridgeDown": (".", "brown"),
    "Air": (" ", "lightcyan"),
    "Cloud": ("#", "gray"),
    "Wall": ("|", "gray"),
    "Vault": (".", "gray"),
}

# ============================================================================
# TrapType -> tui_color mapping (all traps display as '^')
# ============================================================================

TRAP_COLORS: dict[str, str] = {
    "Arrow": "cyan",
    "Dart": "cyan",
    "RockFall": "gray",
    "Squeaky": "brown",
    "BearTrap": "brown",
    "LandMine": "red",
    "RollingBoulder": "gray",
    "SleepingGas": "lightcyan",
    "RustTrap": "brown",
    "FireTrap": "red",
    "Pit": "gray",
    "SpikedPit": "gray",
    "Hole": "gray",
    "TrapDoor": "gray",
    "Teleport": "magenta",
    "LevelTeleport": "magenta",
    "MagicPortal": "lightmagenta",
    "Web": "gray",
    "Statue": "gray",
    "MagicTrap": "lightmagenta",
    "AntiMagic": "lightmagenta",
    "Polymorph": "lightgreen",
}


# ============================================================================
# Utility functions
# ============================================================================

def camel_to_snake(name: str) -> str:
    """Convert CamelCase to snake_case."""
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", s)
    return s.lower()


def safe_sprite_name(name: str) -> str:
    """Convert an arbitrary name to a safe file name component."""
    s = name.lower()
    s = re.sub(r"[^a-z0-9]+", "_", s)
    s = s.strip("_")
    return s


def parse_enum_variants(source: str, enum_name: str) -> list[tuple[str, int]]:
    """
    Parse a Rust enum and return list of (variant_name, value) tuples.

    Handles both explicit values (Foo = 5) and auto-incrementing variants.
    Skips comments, attributes, and the closing brace.
    """
    pattern = rf"pub enum {enum_name}\s*\{{(.*?)\}}"
    match = re.search(pattern, source, re.DOTALL)
    if not match:
        print(f"Warning: Could not find enum {enum_name}", file=sys.stderr)
        return []

    body = match.group(1)
    variants = []
    current_value = 0

    for line in body.split("\n"):
        line = line.strip()
        # Skip empty lines, comments, attributes
        if not line or line.startswith("//") or line.startswith("#"):
            continue

        # Variant with explicit value: Name = N,
        m = re.match(r"^(\w+)\s*=\s*(\d+)\s*,?", line)
        if m:
            name = m.group(1)
            current_value = int(m.group(2))
            variants.append((name, current_value))
            current_value += 1
            continue

        # Variant without explicit value: Name,
        m = re.match(r"^(\w+)\s*,?$", line)
        if m:
            name = m.group(1)
            variants.append((name, current_value))
            current_value += 1

    return variants


# ============================================================================
# Parsing: Monsters
# ============================================================================

def parse_monsters(root: Path) -> list[dict]:
    """
    Parse monster definitions from monsters.rs.

    Extracts MonsterType enum variants and correlates them with PerMonst
    entries to get display name, TUI symbol, and color.
    """
    source = (root / "crates/nh-core/src/data/monsters.rs").read_text()

    # Parse enum variants, excluding the sentinel
    variants = parse_enum_variants(source, "MonsterType")
    variants = [(n, v) for n, v in variants if n != "NumMonsters"]

    # Split on PerMonst entries to get per-monster data
    entries_raw = re.split(r"PerMonst\s*\{", source)[1:]

    monsters = []
    for i, (variant_name, variant_value) in enumerate(variants):
        tui_char = "M"
        tui_color = "gray"
        display_name = camel_to_snake(variant_name).replace("_", " ")

        if i < len(entries_raw):
            entry = entries_raw[i]

            name_match = re.search(r'name:\s*"([^"]*)"', entry)
            if name_match:
                display_name = name_match.group(1)

            sym_match = re.search(r"symbol:\s*'(.)'", entry)
            if sym_match:
                tui_char = sym_match.group(1)

            color_match = re.search(r"color:\s*(\w+)", entry)
            if color_match:
                tui_color = resolve_color(color_match.group(1))

        monsters.append({
            "monster_type": variant_value,
            "name": display_name,
            "icon": {
                "tui_char": tui_char,
                "tui_color": tui_color,
                "bevy_sprite": f"monsters/{camel_to_snake(variant_name)}.png",
            },
        })

    return monsters


# ============================================================================
# Parsing: Objects
# ============================================================================

def _parse_objclassdef(entry_text: str) -> dict[str, str | None]:
    """Extract fields from a single ObjClassDef entry."""
    result: dict[str, str | None] = {
        "name": None,
        "class": None,
        "material": None,
        "color": None,
    }

    name_match = re.search(r'name:\s*"([^"]*)"', entry_text)
    if name_match:
        result["name"] = name_match.group(1)

    class_match = re.search(r"class:\s*ObjectClass::(\w+)", entry_text)
    if class_match:
        result["class"] = class_match.group(1)

    mat_match = re.search(r"material:\s*Material::(\w+)", entry_text)
    if mat_match:
        result["material"] = mat_match.group(1)

    color_match = re.search(r"color:\s*(\w+)", entry_text)
    if color_match:
        result["color"] = color_match.group(1)

    return result


def parse_objects(root: Path) -> tuple[list[dict], dict[str, str]]:
    """
    Parse object definitions from objects.rs.

    Iterates over the OBJECTS array entries by their array position (which is
    the authoritative object_type value used at runtime). The ObjectType enum
    has explicit non-contiguous values and gaps, so we cannot correlate by
    enum position — only by array index.

    Returns:
        (objects_list, type_to_class): The list of parsed objects and a mapping
        from ObjectType variant name to its ObjectClass name (used by artifacts).
    """
    source = (root / "crates/nh-core/src/data/objects.rs").read_text()

    # Parse ObjectType enum to build variant_name → enum_value map
    # (used for nicer sprite names when available, and for artifact class lookup)
    enum_variants = parse_enum_variants(source, "ObjectType")
    enum_value_to_name: dict[int, str] = {}
    for name, value in enum_variants:
        if name != "StrangeObject":
            enum_value_to_name[value] = name

    # Parse ALL ObjClassDef entries from the OBJECTS array by position.
    # Position 0 is StrangeObject (dummy), positions 1+ are real items.
    entries_raw = re.split(r"ObjClassDef\s*\{", source)[1:]

    type_to_class: dict[str, str] = {}
    objects = []

    for array_idx, entry_text in enumerate(entries_raw):
        if array_idx == 0:
            continue  # skip StrangeObject (position 0)

        fields = _parse_objclassdef(entry_text)
        obj_name = fields["name"]
        obj_class = fields["class"]
        material = fields["material"]
        raw_color = fields["color"]

        # Skip dummy/sentinel entries (last entry may have name="?")
        if not obj_name or obj_name == "?":
            continue

        tui_char = CLASS_SYMBOLS.get(obj_class, "?") if obj_class else "?"
        tui_color = resolve_color(raw_color) if raw_color else "gray"

        # Check if this position has a named ObjectType enum variant
        variant_name = enum_value_to_name.get(array_idx)
        if variant_name and obj_class:
            type_to_class[variant_name] = obj_class

        # Build identifier: object_type is the array position (runtime index)
        identifier: dict = {"object_type": array_idx}
        if obj_class:
            identifier["class"] = obj_class
        if material:
            identifier["material"] = material

        # Sprite path: use enum variant name (CamelCase→snake_case) if available,
        # otherwise derive from the display name
        if variant_name:
            sprite_name = camel_to_snake(variant_name)
        else:
            sprite_name = safe_sprite_name(obj_name)
        class_dir = camel_to_snake(obj_class) if obj_class else "misc"

        objects.append({
            "identifier": identifier,
            "name": obj_name,
            "icon": {
                "tui_char": tui_char,
                "tui_color": tui_color,
                "bevy_sprite": f"items/{class_dir}/{sprite_name}.png",
            },
        })

    # Gold pieces (Coin class) are not in the OBJECTS array in NetHack —
    # they're tracked as quantities, not full objects. Add a class-level
    # fallback entry (no object_type, matching by class only).
    objects.append({
        "identifier": {"class": "Coin"},
        "name": "gold piece",
        "icon": {
            "tui_char": "$",
            "tui_color": "yellow",
            "bevy_sprite": "items/coin/gold_piece.png",
        },
    })

    return objects, type_to_class


# ============================================================================
# Parsing: Dungeon tiles
# ============================================================================

def parse_dungeon(root: Path) -> list[dict]:
    """Parse CellType enum from cell.rs for dungeon tile mapping."""
    source = (root / "crates/nh-core/src/dungeon/cell.rs").read_text()
    variants = parse_enum_variants(source, "CellType")

    tiles = []
    for name, value in variants:
        char, color = CELL_DISPLAY.get(name, ("?", "gray"))
        tiles.append({
            "cell_type": name,
            "value": value,
            "icon": {
                "tui_char": char,
                "tui_color": color,
                "bevy_sprite": f"dungeon/{camel_to_snake(name)}.png",
            },
        })

    return tiles


# ============================================================================
# Parsing: Traps
# ============================================================================

def parse_traps(root: Path) -> list[dict]:
    """Parse TrapType enum from level.rs for trap mapping."""
    source = (root / "crates/nh-core/src/dungeon/level.rs").read_text()
    variants = parse_enum_variants(source, "TrapType")

    traps = []
    for name, value in variants:
        color = TRAP_COLORS.get(name, "gray")
        traps.append({
            "trap_type": name,
            "value": value,
            "icon": {
                "tui_char": "^",
                "tui_color": color,
                "bevy_sprite": f"traps/{camel_to_snake(name)}.png",
            },
        })

    return traps


# ============================================================================
# Parsing: Artifacts
# ============================================================================

def parse_artifacts(root: Path, type_to_class: dict[str, str]) -> list[dict]:
    """
    Parse artifact definitions from artifacts.rs.

    Uses type_to_class to determine the correct TUI symbol for each
    artifact based on its base object type.
    """
    source = (root / "crates/nh-core/src/data/artifacts.rs").read_text()

    # Split on Artifact { to get individual entries
    entries_raw = re.split(r"Artifact\s*\{", source)[1:]

    artifacts = []
    for i, entry in enumerate(entries_raw):
        name_match = re.search(r'name:\s*"([^"]*)"', entry)
        otyp_match = re.search(r"otyp:\s*ObjectType::(\w+)", entry)
        color_match = re.search(r"color:\s*(\w+)", entry)

        if not name_match:
            continue

        name = name_match.group(1)
        base_type = otyp_match.group(1) if otyp_match else None

        # Determine TUI color: use artifact color if specified, else white
        raw_color = color_match.group(1) if color_match else "NO_COLOR"
        tui_color = "white" if raw_color == "NO_COLOR" else resolve_color(raw_color)

        # Determine TUI char from the base object's class
        tui_char = ")"  # most artifacts are weapons
        if base_type and base_type in type_to_class:
            obj_class = type_to_class[base_type]
            tui_char = CLASS_SYMBOLS.get(obj_class, ")")

        sprite_name = safe_sprite_name(name)

        artifacts.append({
            "artifact_index": i + 1,  # 1-based (artifact 0 = "not an artifact")
            "name": name,
            "base_type": base_type,
            "icon": {
                "tui_char": tui_char,
                "tui_color": tui_color,
                "bevy_sprite": f"artifacts/{sprite_name}.png",
            },
        })

    return artifacts


# ============================================================================
# Parsing: Player roles
# ============================================================================

def parse_player(root: Path) -> list[dict]:
    """Parse player role names from roles.rs."""
    source = (root / "crates/nh-core/src/data/roles.rs").read_text()

    # Extract unique role names from RoleName::new("Name", ...)
    role_names = re.findall(r'name:\s*RoleName::new\("(\w+)"', source)

    # Deduplicate while preserving order
    seen: set[str] = set()
    unique_roles: list[str] = []
    for name in role_names:
        if name not in seen:
            seen.add(name)
            unique_roles.append(name)

    players = []
    for role in unique_roles:
        players.append({
            "role": role,
            "icon": {
                "tui_char": "@",
                "tui_color": "white",
                "bevy_sprite": f"player/{camel_to_snake(role)}.png",
            },
        })

    return players


# ============================================================================
# Main
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Generate mapping.json for nh-assets from nh-core Rust sources."
    )
    parser.add_argument(
        "--output",
        "-o",
        default=None,
        help="Output file path (default: assets/mapping.json)",
    )
    args = parser.parse_args()

    root = find_project_root()
    output = Path(args.output) if args.output else root / "assets" / "mapping.json"

    print(f"Project root: {root}")
    print(f"Output:       {output}")
    print()

    # Parse objects first since artifacts need the type->class mapping
    print("Parsing objects...")
    objects, type_to_class = parse_objects(root)
    print(f"  {len(objects)} objects")

    print("Parsing monsters...")
    monsters = parse_monsters(root)
    print(f"  {len(monsters)} monsters")

    print("Parsing dungeon tiles...")
    dungeon = parse_dungeon(root)
    print(f"  {len(dungeon)} dungeon tile types")

    print("Parsing traps...")
    traps = parse_traps(root)
    print(f"  {len(traps)} trap types")

    print("Parsing artifacts...")
    artifacts = parse_artifacts(root, type_to_class)
    print(f"  {len(artifacts)} artifacts")

    print("Parsing player roles...")
    player = parse_player(root)
    print(f"  {len(player)} player roles")

    mapping = {
        "items": objects,
        "monsters": monsters,
        "dungeon": dungeon,
        "traps": traps,
        "artifacts": artifacts,
        "player": player,
    }

    total = sum(len(v) for v in mapping.values())
    print(f"\nTotal entries: {total}")

    output.parent.mkdir(parents=True, exist_ok=True)
    with open(output, "w") as f:
        json.dump(mapping, f, indent=2, ensure_ascii=False)
    print(f"Written to {output}")


if __name__ == "__main__":
    main()
