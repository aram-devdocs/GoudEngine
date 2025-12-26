#!/usr/bin/env python3
"""Generate simple isometric placeholder sprites for the RPG demo."""

from PIL import Image, ImageDraw
import os

# Asset directory
ASSET_DIR = "assets/sprites"

def ensure_dirs():
    """Create asset directories if they don't exist."""
    dirs = [
        f"{ASSET_DIR}/player",
        f"{ASSET_DIR}/enemies",
        f"{ASSET_DIR}/npcs",
        f"{ASSET_DIR}/projectiles",
        f"{ASSET_DIR}/ui",
        f"{ASSET_DIR}/tiles",
    ]
    for d in dirs:
        os.makedirs(d, exist_ok=True)

def create_isometric_character(filename, body_color, outline_color, size=64):
    """Create a simple isometric character sprite (diamond body with circle head)."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    cx, cy = size // 2, size // 2

    # Body - isometric diamond shape (lower 2/3 of sprite)
    body_top = cy - 8
    body_bottom = cy + 20
    body_width = 20

    body_points = [
        (cx, body_top),           # Top
        (cx + body_width, cy + 6),  # Right
        (cx, body_bottom),          # Bottom
        (cx - body_width, cy + 6),  # Left
    ]
    draw.polygon(body_points, fill=body_color, outline=outline_color)

    # Head - circle at top
    head_radius = 10
    head_cy = cy - 16
    draw.ellipse([
        cx - head_radius, head_cy - head_radius,
        cx + head_radius, head_cy + head_radius
    ], fill=body_color, outline=outline_color)

    # Simple face - two eyes
    eye_color = (255, 255, 255, 255)
    draw.ellipse([cx - 5, head_cy - 3, cx - 2, head_cy + 2], fill=eye_color)
    draw.ellipse([cx + 2, head_cy - 3, cx + 5, head_cy + 2], fill=eye_color)

    # Shadow underneath
    shadow_color = (0, 0, 0, 80)
    shadow_points = [
        (cx, body_bottom + 2),
        (cx + 18, body_bottom - 6),
        (cx, body_bottom + 8),
        (cx - 18, body_bottom - 6),
    ]
    # Draw shadow behind (need to create new image and composite)
    shadow_img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    shadow_draw = ImageDraw.Draw(shadow_img)
    shadow_draw.polygon(shadow_points, fill=shadow_color)

    # Composite shadow behind character
    final = Image.alpha_composite(shadow_img, img)
    final.save(filename)
    print(f"Created: {filename}")

def create_isometric_tile(filename, color, size=64):
    """Create an isometric ground tile (diamond shape)."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    cx, cy = size // 2, size // 2

    # Diamond tile shape
    points = [
        (cx, cy - 16),      # Top
        (cx + 30, cy),      # Right
        (cx, cy + 16),      # Bottom
        (cx - 30, cy),      # Left
    ]

    # Darker shade for 3D effect
    dark_color = tuple(max(0, c - 40) for c in color[:3]) + (color[3],)
    light_color = tuple(min(255, c + 30) for c in color[:3]) + (color[3],)

    # Main tile surface
    draw.polygon(points, fill=color, outline=(50, 50, 50, 255))

    # Add slight depth lines
    draw.line([points[0], points[1]], fill=light_color, width=1)
    draw.line([points[0], points[3]], fill=light_color, width=1)
    draw.line([points[2], points[1]], fill=dark_color, width=1)
    draw.line([points[2], points[3]], fill=dark_color, width=1)

    img.save(filename)
    print(f"Created: {filename}")

def create_projectile(filename, color, size=16):
    """Create a small projectile sprite."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Simple diamond shape for projectile
    cx, cy = size // 2, size // 2
    r = 5
    points = [
        (cx, cy - r),
        (cx + r, cy),
        (cx, cy + r),
        (cx - r, cy),
    ]
    draw.polygon(points, fill=color, outline=(255, 255, 255, 255))

    img.save(filename)
    print(f"Created: {filename}")

def create_health_bar_bg(filename, width=150, height=20):
    """Create health bar background."""
    img = Image.new('RGBA', (width, height), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Dark background
    draw.rectangle([0, 0, width-1, height-1], fill=(40, 40, 40, 220), outline=(100, 100, 100, 255))

    img.save(filename)
    print(f"Created: {filename}")

def create_health_bar_fill(filename, width=150, height=20):
    """Create health bar fill (red)."""
    img = Image.new('RGBA', (width, height), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Red fill with gradient effect
    for i in range(height):
        shade = 200 - (i * 2)
        draw.line([(2, i), (width-3, i)], fill=(shade, 40, 40, 255))

    img.save(filename)
    print(f"Created: {filename}")

def create_dialogue_box(filename, width=700, height=150):
    """Create dialogue box background."""
    img = Image.new('RGBA', (width, height), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Semi-transparent dark box with border
    draw.rectangle([0, 0, width-1, height-1], fill=(20, 20, 40, 230), outline=(100, 150, 200, 255))
    draw.rectangle([2, 2, width-3, height-3], outline=(60, 80, 120, 255))

    img.save(filename)
    print(f"Created: {filename}")

def create_selection_arrow(filename, size=24):
    """Create dialogue selection arrow."""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Right-pointing arrow
    points = [
        (4, 4),
        (size - 4, size // 2),
        (4, size - 4),
    ]
    draw.polygon(points, fill=(255, 220, 100, 255), outline=(200, 180, 50, 255))

    img.save(filename)
    print(f"Created: {filename}")

def create_title_text(filename, text="ISOMETRIC RPG"):
    """Create title text image."""
    from PIL import ImageFont

    # Try to use a system font, fall back to default
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 48)
    except:
        font = ImageFont.load_default()

    # Calculate text size
    img = Image.new('RGBA', (400, 80), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Draw text with shadow
    draw.text((4, 4), text, font=font, fill=(50, 50, 50, 200))
    draw.text((2, 2), text, font=font, fill=(255, 200, 100, 255))

    img.save(filename)
    print(f"Created: {filename}")

def create_press_start_text(filename):
    """Create 'Press SPACE to Start' text."""
    from PIL import ImageFont

    try:
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 24)
    except:
        font = ImageFont.load_default()

    img = Image.new('RGBA', (300, 40), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    text = "Press SPACE to Start"
    draw.text((2, 2), text, font=font, fill=(200, 200, 200, 255))

    img.save(filename)
    print(f"Created: {filename}")

def create_ground_grid(filename, tile_size=64, grid_w=13, grid_h=10):
    """Create a full isometric ground grid."""
    # Calculate image size needed
    img_w = tile_size * grid_w
    img_h = tile_size * grid_h

    img = Image.new('RGBA', (img_w, img_h), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Isometric tile dimensions
    tile_w = 60
    tile_h = 30

    # Draw grid of isometric tiles
    for row in range(grid_h + 5):
        for col in range(grid_w + 5):
            # Convert grid coordinates to screen coordinates (isometric)
            screen_x = (col - row) * (tile_w // 2) + img_w // 2
            screen_y = (col + row) * (tile_h // 2) + 20

            # Checkerboard pattern
            if (row + col) % 2 == 0:
                color = (60, 80, 60, 255)
            else:
                color = (50, 70, 50, 255)

            # Diamond tile
            points = [
                (screen_x, screen_y - tile_h // 2),
                (screen_x + tile_w // 2, screen_y),
                (screen_x, screen_y + tile_h // 2),
                (screen_x - tile_w // 2, screen_y),
            ]

            draw.polygon(points, fill=color, outline=(40, 60, 40, 255))

    img.save(filename)
    print(f"Created: {filename}")

def main():
    ensure_dirs()

    # Colors (RGBA)
    BLUE = (70, 130, 200, 255)
    BLUE_OUTLINE = (40, 80, 150, 255)

    RED = (200, 70, 70, 255)
    RED_OUTLINE = (150, 40, 40, 255)

    GREEN = (70, 180, 100, 255)
    GREEN_OUTLINE = (40, 130, 60, 255)

    YELLOW = (255, 220, 100, 255)

    GRAY = (100, 100, 100, 255)

    # Create characters
    create_isometric_character(f"{ASSET_DIR}/player/player.png", BLUE, BLUE_OUTLINE)
    create_isometric_character(f"{ASSET_DIR}/enemies/enemy.png", RED, RED_OUTLINE)
    create_isometric_character(f"{ASSET_DIR}/npcs/npc.png", GREEN, GREEN_OUTLINE)

    # Create projectile
    create_projectile(f"{ASSET_DIR}/projectiles/projectile.png", YELLOW)

    # Create tiles
    create_isometric_tile(f"{ASSET_DIR}/tiles/ground.png", (80, 120, 80, 255))
    create_isometric_tile(f"{ASSET_DIR}/tiles/stone.png", GRAY)

    # Create UI elements
    create_health_bar_bg(f"{ASSET_DIR}/ui/health_bg.png")
    create_health_bar_fill(f"{ASSET_DIR}/ui/health_fill.png")
    create_dialogue_box(f"{ASSET_DIR}/ui/dialogue_box.png")
    create_selection_arrow(f"{ASSET_DIR}/ui/arrow.png")
    create_title_text(f"{ASSET_DIR}/ui/title.png")
    create_press_start_text(f"{ASSET_DIR}/ui/press_start.png")

    # Create ground grid
    create_ground_grid(f"{ASSET_DIR}/tiles/ground_grid.png")

    print("\nAll assets generated!")

if __name__ == "__main__":
    main()
