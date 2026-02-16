import json
import argparse
import os
import base64
import time

# Common style block to ensure consistency across all icons
STYLE_MODIFIER = (
    "high-quality stylized digital illustration for a modern roguelike game, "
    "smooth shading and gradients, anti-aliased edges, vibrant professional color palette, "
    "clean centered composition, white background (to be transparent)."
)


# ============================================================================
# Prompt generation per entity category
# ============================================================================

def generate_item_prompt(entry, resolution):
    """Generate a prompt for an item (weapon, armor, potion, etc.)."""
    name = entry.get("name", "mysterious object")
    identifier = entry.get("identifier", {})
    material = identifier.get("material")
    item_class = identifier.get("class", "")

    parts = [f"a {name}"]
    if material:
        parts.append(f"made of {material.lower()}")

    # Add class-specific flavor
    class_hints = {
        "Weapon": "medieval fantasy weapon",
        "Armor": "piece of medieval armor",
        "Potion": "glowing magical potion bottle",
        "Scroll": "ancient parchment scroll",
        "Spellbook": "leather-bound magical spellbook",
        "Wand": "thin magical wand",
        "Ring": "ornate magical ring",
        "Amulet": "mystical amulet on a chain",
        "Food": "food item",
        "Gem": "precious gemstone",
        "Tool": "adventuring tool",
        "Coin": "pile of gold coins",
        "Rock": "stone or rock",
        "Ball": "heavy iron ball and chain",
        "Chain": "iron chain",
        "Venom": "splash of venom",
    }
    hint = class_hints.get(item_class)
    if hint:
        parts.append(hint)

    description = ", ".join(parts)
    return f"Create {description} as a {resolution}x{resolution} game item icon. {STYLE_MODIFIER}"


def generate_monster_prompt(entry, resolution):
    """Generate a prompt for a monster."""
    name = entry.get("name", "monster")
    return (
        f"Create a {name} creature as a {resolution}x{resolution} game icon, "
        f"fantasy RPG monster portrait, facing forward. {STYLE_MODIFIER}"
    )


def generate_dungeon_prompt(entry, resolution):
    """Generate a prompt for a dungeon tile."""
    cell_type = entry.get("cell_type", "floor")
    # Map cell types to descriptive tile prompts
    tile_descriptions = {
        "Stone": "solid stone wall texture, dark grey rock",
        "VWall": "vertical dungeon wall, stone bricks",
        "HWall": "horizontal dungeon wall, stone bricks",
        "TLCorner": "top-left corner of a stone dungeon wall",
        "TRCorner": "top-right corner of a stone dungeon wall",
        "BLCorner": "bottom-left corner of a stone dungeon wall",
        "BRCorner": "bottom-right corner of a stone dungeon wall",
        "CrossWall": "cross-shaped intersection of dungeon walls",
        "TUWall": "T-shaped wall junction pointing up",
        "TDWall": "T-shaped wall junction pointing down",
        "TLWall": "T-shaped wall junction pointing left",
        "TRWall": "T-shaped wall junction pointing right",
        "DBWall": "raised drawbridge, wooden planks with chains",
        "Tree": "gnarled underground tree with pale leaves",
        "SecretDoor": "stone wall with a hidden door outline",
        "SecretCorridor": "hidden passageway behind false wall",
        "Pool": "still pool of dark water on dungeon floor",
        "Moat": "deep moat of murky water",
        "Water": "underground river or lake, dark water",
        "DrawbridgeUp": "raised drawbridge with chains",
        "Lava": "pool of glowing molten lava",
        "IronBars": "vertical iron bars blocking passage",
        "Door": "wooden dungeon door with iron hinges",
        "Corridor": "narrow stone corridor, dim lighting",
        "Room": "stone dungeon floor tiles",
        "Stairs": "stone staircase leading down into darkness",
        "Ladder": "wooden ladder descending into darkness",
        "Fountain": "ornate stone fountain with magical water",
        "Throne": "ornate golden throne on a raised dais",
        "Sink": "stone basin with dripping water",
        "Grave": "stone gravestone with carved inscription",
        "Altar": "sacrificial stone altar with runes",
        "Ice": "slippery ice-covered dungeon floor",
        "DrawbridgeDown": "lowered drawbridge, wooden planks",
        "Air": "open sky seen from above, clouds below",
        "Cloud": "thick magical clouds, ethereal mist",
        "Wall": "solid stone dungeon wall",
        "Vault": "polished stone floor of a treasure vault",
    }
    desc = tile_descriptions.get(cell_type, f"dungeon {cell_type.lower()} tile")
    return f"Create a {desc} as a {resolution}x{resolution} top-down dungeon tile. {STYLE_MODIFIER}"


def generate_trap_prompt(entry, resolution):
    """Generate a prompt for a trap."""
    trap_type = entry.get("trap_type", "trap")
    trap_descriptions = {
        "Arrow": "hidden arrow trap with trigger mechanism",
        "Dart": "concealed dart trap in dungeon wall",
        "RockFall": "unstable ceiling ready to collapse rocks",
        "Squeaky": "squeaky floorboard trap",
        "BearTrap": "steel bear trap with jagged teeth",
        "LandMine": "hidden land mine buried in floor",
        "RollingBoulder": "large boulder ready to roll down a slope",
        "SleepingGas": "vent releasing sleeping gas clouds",
        "RustTrap": "trap that sprays corrosive rust liquid",
        "FireTrap": "fire jet trap shooting flames from floor",
        "Pit": "concealed pit trap in dungeon floor",
        "SpikedPit": "pit trap lined with sharp spikes",
        "Hole": "hole in the dungeon floor",
        "TrapDoor": "hidden trapdoor in the floor",
        "Teleport": "glowing magical teleportation rune on floor",
        "LevelTeleport": "swirling portal of magical energy",
        "MagicPortal": "shimmering dimensional portal",
        "Web": "giant spider web stretching across passage",
        "Statue": "stone statue that is actually a trap",
        "MagicTrap": "glowing magical rune trap on floor",
        "AntiMagic": "anti-magic field emanating from floor rune",
        "Polymorph": "chaotic polymorph energy trap on floor",
    }
    desc = trap_descriptions.get(trap_type, f"dungeon {trap_type.lower()} trap")
    return f"Create a {desc} as a {resolution}x{resolution} game icon. {STYLE_MODIFIER}"


def generate_artifact_prompt(entry, resolution):
    """Generate a prompt for a legendary artifact."""
    name = entry.get("name", "artifact")
    base_type = entry.get("base_type", "")
    base_desc = base_type.lower() if base_type else "weapon"
    # Convert CamelCase to readable
    import re
    base_desc = re.sub(r"([a-z])([A-Z])", r"\1 \2", base_desc).lower()
    return (
        f"Create the legendary artifact \"{name}\" (a magical {base_desc}) "
        f"as a {resolution}x{resolution} game icon, glowing with power, "
        f"ornate and legendary. {STYLE_MODIFIER}"
    )


def generate_player_prompt(entry, resolution):
    """Generate a prompt for a player role."""
    role = entry.get("role", "adventurer")
    return (
        f"Create a {role} character portrait as a {resolution}x{resolution} game icon, "
        f"fantasy RPG hero, facing forward. {STYLE_MODIFIER}"
    )


# Map category -> prompt generator
PROMPT_GENERATORS = {
    "items": generate_item_prompt,
    "monsters": generate_monster_prompt,
    "dungeon": generate_dungeon_prompt,
    "traps": generate_trap_prompt,
    "artifacts": generate_artifact_prompt,
    "player": generate_player_prompt,
}


# ============================================================================
# API / generation backends
# ============================================================================

def call_api(url, headers, payload, max_retries=5):
    """Call the Google API with exponential backoff for rate limiting."""
    import requests

    last_response = None
    for i in range(max_retries):
        try:
            response = requests.post(url, headers=headers, json=payload, timeout=60)
            last_response = response

            if response.status_code == 429:
                wait_time = (2 ** i) + 5
                print(f"  Rate limited (429). Retrying in {wait_time}s...")
                time.sleep(wait_time)
                continue

            return response
        except requests.exceptions.RequestException as e:
            print(f"  Network error: {e}")
            if i == max_retries - 1:
                raise e
            time.sleep(2 ** i)

    return last_response


def generate_local(prompt, target_path, resolution, seed, flux_model):
    """Generate an image locally using mflux Flux1Schnell."""
    image = flux_model.generate_image(
        seed=seed,
        prompt=prompt,
        num_inference_steps=4,
        height=resolution,
        width=resolution,
    )
    image.save(target_path)


def generate_google(prompt, target_path, reference_b64, api_key, model):
    """Generate an image using Google Gemini/Imagen API."""
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
    headers = {
        "Content-Type": "application/json",
        "X-goog-api-key": api_key,
    }

    parts = []
    if reference_b64:
        parts.append({
            "inline_data": {
                "mime_type": "image/png",
                "data": reference_b64,
            }
        })
        parts.append({"text": f"Generate a new item icon matching the style of the provided reference image. The new item is: {prompt}"})
    else:
        parts.append({"text": prompt})

    if "imagen" in model.lower():
        payload = {
            "instances": [{"prompt": prompt}],
            "parameters": {
                "sampleCount": 1,
                "aspectRatio": "1:1",
                "includeSafetyAttributes": False,
            },
        }
    else:
        payload = {
            "contents": [{"parts": parts}],
            "generationConfig": {"response_modalities": ["IMAGE"]},
        }

    response = call_api(url, headers, payload)

    if response is None:
        print(f"  Error: No response from API (max retries reached)")
        return False

    if response.status_code != 200:
        print(f"  Error: {response.status_code}")
        print(f"  Response: {response.text}")
        return False

    result = response.json()

    candidates = result.get("candidates", [])
    if candidates:
        content_parts = candidates[0].get("content", {}).get("parts", [])
        for part in content_parts:
            if "inlineData" in part:
                img_data = part["inlineData"].get("data")
                if img_data:
                    image_bytes = base64.b64decode(img_data)
                    with open(target_path, 'wb') as img_f:
                        img_f.write(image_bytes)
                    return True

    if "error" in result:
        print(f"  API Error: {result['error'].get('message')}")
    return False


# ============================================================================
# Mapping loader: collect entries from all sections
# ============================================================================

def load_entries(mapping_data, categories):
    """
    Collect entries from the mapping.json, supporting both the new sectioned
    format and the legacy flat format.

    Each returned entry is a tuple of (category, entry_dict).
    """
    entries = []

    # New sectioned format: {"items": [...], "monsters": [...], ...}
    for category in categories:
        section = mapping_data.get(category, [])
        for entry in section:
            entries.append((category, entry))

    # Legacy flat format: {"mappings": [...]}
    if not entries:
        for entry in mapping_data.get("mappings", []):
            entries.append(("items", entry))

    return entries


# ============================================================================
# Main
# ============================================================================

def main():
    parser = argparse.ArgumentParser(description="Batch generate item icons for nethack-rs.")
    parser.add_argument("--mapping", required=True, help="Path to mapping.json")
    parser.add_argument("--output", required=True, help="Output base directory for sprites")
    parser.add_argument("--dry-run", action="store_true", help="Print prompts without executing")
    parser.add_argument("--limit", type=int, help="Limit the number of items to generate")
    parser.add_argument("--force", action="store_true", help="Force re-generation of existing files")
    parser.add_argument("--resolution", type=int, default=512, help="Output resolution (default: 512)")
    parser.add_argument("--seed", type=int, default=42, help="Base seed for local generation (default: 42)")
    parser.add_argument(
        "--category",
        choices=["items", "monsters", "dungeon", "traps", "artifacts", "player", "all"],
        default="all",
        help="Which category to generate (default: all)",
    )
    parser.add_argument(
        "--backend", choices=["local", "google"], default="local",
        help="Generation backend: 'local' uses mflux Flux1Schnell, 'google' uses Gemini/Imagen API (default: local)",
    )

    # Google-specific options
    google_group = parser.add_argument_group("google backend options")
    google_group.add_argument("--api-key", help="Google API Key (or set GOOGLE_API_KEY env var)")
    google_group.add_argument("--reference-image", help="Path to a reference image to maintain style")
    google_group.add_argument("--model", default="imagen-3.0-generate-001", help="Model name (default: imagen-3.0-generate-001)")

    args = parser.parse_args()

    # Backend-specific validation
    reference_b64 = None
    flux_model = None
    api_key = None

    if args.backend == "google":
        api_key = args.api_key or os.environ.get("GOOGLE_API_KEY")
        if not api_key and not args.dry_run:
            print("Error: Google API Key is required. Set GOOGLE_API_KEY or use --api-key.")
            return

        if args.reference_image:
            if not os.path.exists(args.reference_image):
                print(f"Error: Reference image {args.reference_image} not found.")
                return
            with open(args.reference_image, "rb") as f:
                reference_b64 = base64.b64encode(f.read()).decode("utf-8")
    else:
        if not args.dry_run:
            from mflux.models.flux.variants.txt2img.flux import Flux1  # type: ignore[import-not-found]

            print("Loading Flux1Schnell model (quantize=8)...")
            flux_model = Flux1(quantize=8)

    if not os.path.exists(args.mapping):
        print(f"Error: Mapping file {args.mapping} not found.")
        return

    with open(args.mapping, 'r') as f:
        mapping_data = json.load(f)

    os.makedirs(args.output, exist_ok=True)

    # Determine which categories to generate
    all_categories = ["items", "monsters", "dungeon", "traps", "artifacts", "player"]
    if args.category == "all":
        categories = all_categories
    else:
        categories = [args.category]

    entries = load_entries(mapping_data, categories)

    # Filter out existing sprites unless --force
    if not args.force:
        remaining = []
        for category, entry in entries:
            sprite_path = entry.get("icon", {}).get("bevy_sprite")
            if sprite_path:
                target_path = os.path.join(args.output, sprite_path)
                if not os.path.exists(target_path):
                    remaining.append((category, entry))
        entries = remaining

    if args.limit:
        entries = entries[:args.limit]

    # Print category breakdown
    category_counts: dict[str, int] = {}
    for category, _ in entries:
        category_counts[category] = category_counts.get(category, 0) + 1
    print(f"Generating {len(entries)} sprites (backend: {args.backend}):")
    for cat, count in sorted(category_counts.items()):
        print(f"  {cat}: {count}")
    print()

    for idx, (category, entry) in enumerate(entries):
        icon_def = entry.get("icon", {})
        sprite_path = icon_def.get("bevy_sprite")

        if not sprite_path:
            continue

        target_path = os.path.join(args.output, sprite_path)

        # Create subdirectories as needed
        os.makedirs(os.path.dirname(target_path), exist_ok=True)

        # Generate prompt using category-specific generator
        prompt_fn = PROMPT_GENERATORS.get(category, generate_item_prompt)
        prompt = prompt_fn(entry, args.resolution)

        name = entry.get("name", sprite_path)
        print(f"[{idx + 1}/{len(entries)}] {category}/{name}")
        print(f"  Prompt: {prompt}")
        print(f"  Target: {sprite_path}")

        if args.dry_run:
            continue

        try:
            if args.backend == "local":
                generate_local(prompt, target_path, args.resolution, args.seed + idx, flux_model)
            else:
                if not generate_google(prompt, target_path, reference_b64, api_key, args.model):
                    print(f"  Warning: No image data returned for {sprite_path}")
                    continue

            print(f"  Saved to {target_path}")
        except Exception as e:
            print(f"  Error generating {sprite_path}: {e}")

if __name__ == "__main__":
    main()
