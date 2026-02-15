# Asset Generation Workflow (Banana)

This document describes the process for generating 2D item icons using the "banana" generation tool (or equivalent AI image generator).

## Aesthetic Parameters
To ensure visual consistency across all items, use the following parameters for all generation prompts:

- **Style:** 32x32 pixel art, roguelike aesthetic.
- **Palette:** Consistent 16-color palette (refer to `palette.png`).
- **Background:** Transparent.
- **View:** Top-down or slight isometric perspective.

## Prompt Templates

### Base Item
"A [Item Name] for a roguelike game, 32x32 pixel art, [Material] texture, transparent background."

### Identified Item
"A [Identified Item Name], distinct visual features indicating its properties, 32x32 pixel art, transparent background."

## Automation Script
The `scripts/generate_assets.py` script automates the process of calling the Google Nano Banana API (Gemini 2.5 Flash Image) for all mappings defined in a JSON file.

### Prerequisites
- Python 3.x
- `requests` library: `pip install requests`
- A Google API Key with access to Gemini 2.0 Flash image generation.

### Usage
```bash
# Preview generation prompts
python scripts/generate_assets.py --mapping crates/nh-assets/initial_mapping.json --output crates/nh-bevy/assets/items/ --dry-run

# Run actual generation (requires GOOGLE_API_KEY environment variable)
export GOOGLE_API_KEY="your_key_here"
python scripts/generate_assets.py --mapping crates/nh-assets/initial_mapping.json --output crates/nh-bevy/assets/items/
```
