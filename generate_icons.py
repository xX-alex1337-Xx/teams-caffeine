"""
Icon generator for Teams Caffeine
Generates high-resolution multi-size ICO files for active and paused states.
"""
from PIL import Image, ImageDraw
import os

def draw_caffeine_icon(size: int, active: bool = True) -> Image.Image:
    # 4x supersampling for ultra smooth rendering
    scale = 4
    dim = size * scale
    img = Image.new("RGBA", (dim, dim), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Color definitions
    if active:
        cup_color = (255, 255, 255, 245)
        coffee_color = (212, 138, 54, 255)       # warm espresso/coffee
        steam_color = (76, 217, 100, 220)        # green steam / energy indicator (Teams Green)
        badge_bg = (34, 197, 94, 255)            # glowing green active badge
        badge_border = (15, 23, 42, 255)
        rim_color = (240, 240, 245, 255)
    else:
        cup_color = (160, 160, 170, 200)
        coffee_color = (100, 100, 110, 200)
        steam_color = (130, 130, 140, 0)         # no steam when paused
        badge_bg = (156, 163, 175, 255)          # gray paused badge
        badge_border = (15, 23, 42, 255)
        rim_color = (140, 140, 150, 220)

    # Coffee cup dimensions
    cup_left = int(dim * 0.18)
    cup_top = int(dim * 0.36)
    cup_right = int(dim * 0.68)
    cup_bottom = int(dim * 0.82)
    cup_radius = int(dim * 0.12)

    # Saucer / Base
    saucer_left = int(dim * 0.12)
    saucer_top = int(dim * 0.82)
    saucer_right = int(dim * 0.74)
    saucer_bottom = int(dim * 0.89)
    saucer_r = int(dim * 0.04)
    draw.rounded_rectangle([saucer_left, saucer_top, saucer_right, saucer_bottom], radius=saucer_r, fill=rim_color)

    # Handle (arc on right side)
    handle_left = int(dim * 0.54)
    handle_top = int(dim * 0.44)
    handle_right = int(dim * 0.88)
    handle_bottom = int(dim * 0.74)
    handle_w = max(int(dim * 0.08), 2)
    draw.ellipse([handle_left, handle_top, handle_right, handle_bottom], outline=rim_color, width=handle_w)

    # Cup body
    draw.rounded_rectangle(
        [cup_left, cup_top, cup_right, cup_bottom],
        radius=cup_radius,
        fill=cup_color,
        corners=(False, False, True, True)
    )

    # Coffee surface inner ellipse
    inner_pad = int(dim * 0.03)
    rim_h = int(dim * 0.10)
    draw.ellipse([cup_left + inner_pad, cup_top - rim_h // 2, cup_right - inner_pad, cup_top + rim_h // 2], fill=coffee_color)

    # Steam / Aroma lines (if active)
    if active:
        steam_w = max(int(dim * 0.05), 2)
        # Left steam curve
        steam1_pts = [
            (int(dim * 0.30), int(dim * 0.30)),
            (int(dim * 0.26), int(dim * 0.22)),
            (int(dim * 0.32), int(dim * 0.14)),
        ]
        draw.line(steam1_pts, fill=steam_color, width=steam_w, joint="curve")
        
        # Middle steam curve (taller)
        steam2_pts = [
            (int(dim * 0.43), int(dim * 0.30)),
            (int(dim * 0.47), int(dim * 0.20)),
            (int(dim * 0.41), int(dim * 0.10)),
        ]
        draw.line(steam2_pts, fill=steam_color, width=steam_w, joint="curve")

        # Right steam curve
        steam3_pts = [
            (int(dim * 0.56), int(dim * 0.30)),
            (int(dim * 0.52), int(dim * 0.22)),
            (int(dim * 0.58), int(dim * 0.14)),
        ]
        draw.line(steam3_pts, fill=steam_color, width=steam_w, joint="curve")

    # Status Indicator Badge (Bottom right or top right)
    badge_size = int(dim * 0.34)
    bx2 = int(dim * 0.96)
    by2 = int(dim * 0.96)
    bx1 = bx2 - badge_size
    by1 = by2 - badge_size
    
    # Outer dark border to pop against tray dark/light backgrounds
    draw.ellipse([bx1 - 2, by1 - 2, bx2 + 2, by2 + 2], fill=badge_border)
    draw.ellipse([bx1, by1, bx2, by2], fill=badge_bg)

    # Inner symbol on badge: checkmark / dot if active, pause bars if paused
    if active:
        # Green status dot / lightning or inner bright dot
        inner_m = int(badge_size * 0.22)
        draw.ellipse([bx1 + inner_m, by1 + inner_m, bx2 - inner_m, by2 - inner_m], fill=(255, 255, 255, 240))
    else:
        # Pause bars ||
        bar_w = max(int(badge_size * 0.14), 2)
        bar_h = int(badge_size * 0.48)
        bar_y1 = by1 + int((badge_size - bar_h) / 2)
        bar_y2 = bar_y1 + bar_h
        mid_x = (bx1 + bx2) // 2
        gap = max(int(badge_size * 0.10), 2)
        
        draw.line([(mid_x - gap, bar_y1), (mid_x - gap, bar_y2)], fill=(255, 255, 255, 255), width=bar_w)
        draw.line([(mid_x + gap, bar_y1), (mid_x + gap, bar_y2)], fill=(255, 255, 255, 255), width=bar_w)

    # Downsample to target size with Lanczos resampling
    return img.resize((size, size), Image.Resampling.LANCZOS)

def main():
    sizes = [16, 24, 32, 48, 64, 128, 256]
    os.makedirs("assets", exist_ok=True)
    
    # Generate active icon
    active_images = [draw_caffeine_icon(s, active=True) for s in sizes]
    active_images[0].save(
        "assets/icon.ico",
        format="ICO",
        sizes=[(s, s) for s in sizes],
        append_images=active_images[1:]
    )
    print("Generated assets/icon.ico")

    # Generate paused icon
    paused_images = [draw_caffeine_icon(s, active=False) for s in sizes]
    paused_images[0].save(
        "assets/icon_paused.ico",
        format="ICO",
        sizes=[(s, s) for s in sizes],
        append_images=paused_images[1:]
    )
    print("Generated assets/icon_paused.ico")

    # Also save PNG preview for GitHub README
    preview_img = draw_caffeine_icon(256, active=True)
    preview_img.save("assets/preview.png", format="PNG")
    print("Generated assets/preview.png")

if __name__ == "__main__":
    main()
