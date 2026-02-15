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

## Automation Script (Placeholder)
`scripts/generate_assets.py` can be used to batch generate icons from the `mapping.json` file.

```bash
python scripts/generate_assets.py --mapping crates/nh-assets/initial_mapping.json --output crates/nh-bevy/assets/items/
```
