#!/usr/bin/env python3
"""Generate a simple test image (silhouette of a person) for pipeline testing."""

from PIL import Image, ImageDraw
import sys
from pathlib import Path


def draw_person(img: Image.Image):
    """Draw a simple humanoid silhouette on a white background."""
    w, h = img.size
    draw = ImageDraw.Draw(img)

    # White background
    draw.rectangle([0, 0, w, h], fill=(255, 255, 255))

    cx = w // 2
    skin = (210, 180, 140)
    shirt = (50, 100, 180)
    pants = (60, 60, 80)
    shoe = (40, 30, 20)

    # Head
    head_r = int(h * 0.06)
    head_y = int(h * 0.10)
    draw.ellipse([cx - head_r, head_y - head_r, cx + head_r, head_y + head_r], fill=skin)

    # Neck
    neck_w = int(h * 0.02)
    neck_top = head_y + head_r
    neck_bot = int(h * 0.18)
    draw.rectangle([cx - neck_w, neck_top, cx + neck_w, neck_bot], fill=skin)

    # Torso
    torso_w = int(h * 0.10)
    torso_bot = int(h * 0.50)
    draw.rectangle([cx - torso_w, neck_bot, cx + torso_w, torso_bot], fill=shirt)

    # Arms
    arm_w = int(h * 0.03)
    arm_top = neck_bot
    arm_bot = int(h * 0.45)

    # Left arm
    draw.rectangle([cx - torso_w - arm_w * 2, arm_top, cx - torso_w, arm_bot], fill=shirt)
    # Hand
    draw.rectangle([cx - torso_w - arm_w * 2, arm_bot, cx - torso_w, arm_bot + int(h * 0.03)], fill=skin)

    # Right arm
    draw.rectangle([cx + torso_w, arm_top, cx + torso_w + arm_w * 2, arm_bot], fill=shirt)
    draw.rectangle([cx + torso_w, arm_bot, cx + torso_w + arm_w * 2, arm_bot + int(h * 0.03)], fill=skin)

    # Legs
    leg_w = int(h * 0.04)
    leg_gap = int(h * 0.02)
    leg_bot = int(h * 0.88)

    # Left leg
    draw.rectangle([cx - leg_gap - leg_w * 2, torso_bot, cx - leg_gap, leg_bot], fill=pants)
    # Right leg
    draw.rectangle([cx + leg_gap, torso_bot, cx + leg_gap + leg_w * 2, leg_bot], fill=pants)

    # Shoes
    shoe_h = int(h * 0.04)
    draw.rectangle([cx - leg_gap - leg_w * 2 - int(h * 0.01), leg_bot, cx - leg_gap + int(h * 0.02), leg_bot + shoe_h], fill=shoe)
    draw.rectangle([cx + leg_gap - int(h * 0.02), leg_bot, cx + leg_gap + leg_w * 2 + int(h * 0.01), leg_bot + shoe_h], fill=shoe)


def main():
    output = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("input/test_character.png")
    output.parent.mkdir(parents=True, exist_ok=True)

    img = Image.new("RGB", (512, 768), (255, 255, 255))
    draw_person(img)
    img.save(output)
    print(f"Test image saved to {output}")


if __name__ == "__main__":
    main()
