#!/usr/bin/env python3
"""
OpenCenter OpenDeck / Stream Deck Icon Generator
Renders high-resolution stylized action icons inspired by Elgato Control Center.
"""

import math
import os
from PIL import Image, ImageDraw, ImageFilter, ImageFont

CANVAS = 1024

def create_base():
    """Create high-tech dark rounded-square button base plate."""
    img = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    
    margin = 56
    radius = 160
    box = [margin, margin, CANVAS - margin, CANVAS - margin]
    
    # Subtle drop shadow
    shadow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    sdraw = ImageDraw.Draw(shadow)
    sdraw.rounded_rectangle([margin + 12, margin + 24, CANVAS - margin - 12, CANVAS - margin + 24], 
                           radius=radius, fill=(0, 0, 0, 190))
    shadow = shadow.filter(ImageFilter.GaussianBlur(30))
    img = Image.alpha_composite(shadow, img)
    
    # Base button surface with dark bezel
    base_surf = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    bdraw = ImageDraw.Draw(base_surf)
    bdraw.rounded_rectangle(box, radius=radius, fill=(16, 20, 28, 255), outline=(45, 55, 75, 255), width=14)
    
    # Inner bevel
    inner_box = [margin + 18, margin + 18, CANVAS - margin - 18, CANVAS - margin - 18]
    bdraw.rounded_rectangle(inner_box, radius=radius - 14, fill=None, outline=(24, 30, 42, 255), width=8)
    
    img = Image.alpha_composite(img, base_surf)
    return img

def draw_stand(draw, cx, cy, top_y, width=44, bottom_y=860):
    """Draws the sleek Key Light pole mount & socket."""
    draw.rounded_rectangle([cx - width, top_y, cx + width, top_y + 36], radius=12, fill=(35, 42, 56, 255), outline=(55, 65, 85, 255), width=4)
    draw.rectangle([cx - width//2, top_y + 30, cx + width//2, bottom_y], fill=(25, 30, 40, 255), outline=(48, 58, 76, 255), width=4)

def draw_power_glyph(draw, cx, cy, radius, width, color, gap_deg=60):
    """Draws the power icon symbol (circle with top gap + vertical line)."""
    start_angle = -90 + gap_deg / 2
    end_angle = 270 - gap_deg / 2
    bbox = [cx - radius, cy - radius, cx + radius, cy + radius]
    draw.arc(bbox, start=start_angle, end=end_angle, fill=color, width=width)
    line_len = radius * 0.95
    draw.line([cx, cy - line_len, cx, cy + radius * 0.1], fill=color, width=width)

def generate_icon_main():
    """Main OpenCenter / Key Light Plugin Icon with dual warm-to-cool Kelvin gradient."""
    img = create_base()
    
    px0, py0, px1, py1 = 180, 220, 844, 690
    cx = (px0 + px1) // 2
    
    # Ambient glow behind panel (split warm & cool)
    glow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    gdraw = ImageDraw.Draw(glow)
    gdraw.rounded_rectangle([px0 - 30, py0 - 30, cx + 20, py1 + 30], radius=60, fill=(255, 145, 0, 90))
    gdraw.rounded_rectangle([cx - 20, py0 - 30, px1 + 30, py1 + 30], radius=60, fill=(0, 229, 255, 90))
    glow = glow.filter(ImageFilter.GaussianBlur(50))
    img = Image.alpha_composite(img, glow)
    
    draw = ImageDraw.Draw(img)
    draw_stand(draw, cx, (py0 + py1)//2, py1, width=44, bottom_y=860)
    
    # Panel outer chassis
    draw.rounded_rectangle([px0, py0, px1, py1], radius=44, fill=(26, 32, 44, 255), outline=(58, 70, 92, 255), width=12)
    
    # Inner diffuser surface with horizontal Kelvin gradient
    diffuser_box = [px0 + 24, py0 + 24, px1 - 24, py1 - 24]
    dw = diffuser_box[2] - diffuser_box[0]
    dh = diffuser_box[3] - diffuser_box[1]
    
    diffuser_img = Image.new("RGBA", (dw, dh), (0, 0, 0, 0))
    ddraw = ImageDraw.Draw(diffuser_img)
    
    for x in range(dw):
        t = x / float(dw)
        if t < 0.5:
            t_sub = t / 0.5
            r = int(255 + (255 - 255) * t_sub)
            g = int(145 + (245 - 145) * t_sub)
            b = int(20 + (240 - 20) * t_sub)
        else:
            t_sub = (t - 0.5) / 0.5
            r = int(255 + (0 - 255) * t_sub)
            g = int(245 + (225 - 245) * t_sub)
            b = int(240 + (255 - 240) * t_sub)
        ddraw.line([(x, 0), (x, dh)], fill=(r, g, b, 255))
        
    mask = Image.new("L", (dw, dh), 0)
    mdraw = ImageDraw.Draw(mask)
    mdraw.rounded_rectangle([0, 0, dw, dh], radius=28, fill=255)
    img.paste(diffuser_img, (diffuser_box[0], diffuser_box[1]), mask)
    
    draw.rounded_rectangle(diffuser_box, radius=28, fill=None, outline=(255, 255, 255, 80), width=4)
    
    pcy = (py0 + py1) // 2
    pglow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    pgdraw = ImageDraw.Draw(pglow)
    draw_power_glyph(pgdraw, cx, pcy, radius=90, width=28, color=(20, 26, 36, 180))
    pglow = pglow.filter(ImageFilter.GaussianBlur(8))
    img = Image.alpha_composite(img, pglow)
    
    draw = ImageDraw.Draw(img)
    draw_power_glyph(draw, cx, pcy, radius=90, width=22, color=(22, 28, 38, 240))
    
    draw.ellipse([px0 + 50, py1 - 56, px0 + 74, py1 - 32], fill=(255, 120, 0, 255))
    draw.ellipse([px1 - 74, py1 - 56, px1 - 50, py1 - 32], fill=(0, 210, 255, 255))
    
    return img

def generate_toggle_on():
    """Toggle Action State 1: Power ON (Bright illuminated warm/white panel + radiance)."""
    img = create_base()
    
    px0, py0, px1, py1 = 180, 240, 844, 690
    cx = (px0 + px1) // 2
    cy = (py0 + py1) // 2
    
    rays = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    rdraw = ImageDraw.Draw(rays)
    
    top_y = py0 - 15
    for x in range(px0 + 60, px1 - 50, 90):
        rdraw.line([(x, top_y), (x, top_y - 75)], fill=(255, 190, 60, 220), width=18)
    for y in range(py0 + 80, py1 - 60, 90):
        rdraw.line([(px0 - 15, y), (px0 - 85, y)], fill=(255, 190, 60, 220), width=18)
    for y in range(py0 + 80, py1 - 60, 90):
        rdraw.line([(px1 + 15, y), (px1 + 85, y)], fill=(255, 190, 60, 220), width=18)
    rdraw.line([(px0 - 15, py1 - 10), (px0 - 65, py1 + 40)], fill=(255, 190, 60, 220), width=18)
    rdraw.line([(px1 + 15, py1 - 10), (px1 + 65, py1 + 40)], fill=(255, 190, 60, 220), width=18)
    
    bloom = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    bdraw = ImageDraw.Draw(bloom)
    bdraw.rounded_rectangle([px0 - 40, py0 - 40, px1 + 40, py1 + 40], radius=70, fill=(255, 185, 40, 160))
    bloom = bloom.filter(ImageFilter.GaussianBlur(60))
    
    img = Image.alpha_composite(img, bloom)
    img = Image.alpha_composite(img, rays)
    
    draw = ImageDraw.Draw(img)
    draw_stand(draw, cx, cy, py1, width=44, bottom_y=860)
    draw.rounded_rectangle([px0, py0, px1, py1], radius=44, fill=(40, 32, 20, 255), outline=(255, 200, 80, 255), width=12)
    
    diffuser_box = [px0 + 24, py0 + 24, px1 - 24, py1 - 24]
    draw.rounded_rectangle(diffuser_box, radius=28, fill=(255, 250, 230, 255), outline=(255, 255, 255, 255), width=6)
    draw_power_glyph(draw, cx, cy, radius=96, width=24, color=(210, 105, 0, 255))
    draw.ellipse([cx - 16, py1 - 48, cx + 16, py1 - 16], fill=(0, 255, 180, 255), outline=(255, 255, 255, 200), width=3)
    
    return img

def generate_toggle_off():
    """Toggle Action State 0: Power OFF (Dark standby panel + red standby LED)."""
    img = create_base()
    
    px0, py0, px1, py1 = 180, 240, 844, 690
    cx = (px0 + px1) // 2
    cy = (py0 + py1) // 2
    
    draw = ImageDraw.Draw(img)
    draw_stand(draw, cx, cy, py1, width=44, bottom_y=860)
    draw.rounded_rectangle([px0, py0, px1, py1], radius=44, fill=(20, 24, 34, 255), outline=(42, 50, 68, 255), width=12)
    
    diffuser_box = [px0 + 24, py0 + 24, px1 - 24, py1 - 24]
    draw.rounded_rectangle(diffuser_box, radius=28, fill=(14, 17, 24, 255), outline=(30, 36, 50, 255), width=6)
    draw_power_glyph(draw, cx, cy, radius=96, width=22, color=(68, 82, 108, 255))
    
    red_glow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    rgdraw = ImageDraw.Draw(red_glow)
    rgdraw.ellipse([cx - 24, py0 + 40, cx + 24, py0 + 88], fill=(255, 30, 30, 160))
    red_glow = red_glow.filter(ImageFilter.GaussianBlur(15))
    img = Image.alpha_composite(img, red_glow)
    
    draw = ImageDraw.Draw(img)
    draw.ellipse([cx - 14, py0 + 50, cx + 14, py0 + 78], fill=(255, 60, 60, 255), outline=(255, 180, 180, 220), width=3)
    
    return img

def generate_toggle():
    """Toggle Action Icon (Split state: Left Dark / Right Bright)."""
    img = create_base()
    
    px0, py0, px1, py1 = 180, 240, 844, 690
    cx = (px0 + px1) // 2
    cy = (py0 + py1) // 2
    
    glow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    gdraw = ImageDraw.Draw(glow)
    gdraw.rounded_rectangle([cx, py0 - 20, px1 + 30, py1 + 20], radius=50, fill=(255, 170, 30, 120))
    glow = glow.filter(ImageFilter.GaussianBlur(40))
    img = Image.alpha_composite(img, glow)
    
    draw = ImageDraw.Draw(img)
    draw_stand(draw, cx, cy, py1, width=44, bottom_y=860)
    draw.rounded_rectangle([px0, py0, px1, py1], radius=44, fill=(22, 27, 38, 255), outline=(50, 62, 84, 255), width=12)
    
    diffuser_box = [px0 + 24, py0 + 24, px1 - 24, py1 - 24]
    dw = diffuser_box[2] - diffuser_box[0]
    dh = diffuser_box[3] - diffuser_box[1]
    
    diffuser_img = Image.new("RGBA", (dw, dh), (0, 0, 0, 0))
    ddraw = ImageDraw.Draw(diffuser_img)
    ddraw.rectangle([0, 0, dw//2, dh], fill=(16, 20, 28, 255))
    ddraw.rectangle([dw//2, 0, dw, dh], fill=(255, 245, 215, 255))
    ddraw.line([(dw//2, 0), (dw//2, dh)], fill=(40, 50, 70, 255), width=6)
    
    mask = Image.new("L", (dw, dh), 0)
    mdraw = ImageDraw.Draw(mask)
    mdraw.rounded_rectangle([0, 0, dw, dh], radius=28, fill=255)
    img.paste(diffuser_img, (diffuser_box[0], diffuser_box[1]), mask)
    draw.rounded_rectangle(diffuser_box, radius=28, fill=None, outline=(55, 68, 92, 255), width=4)
    draw_power_glyph(draw, cx, cy, radius=96, width=24, color=(255, 175, 40, 255))
    
    return img

def generate_brightness():
    """Adjust Brightness Icon (Radiant Elgato Sun + Stepper / Radial Arc Meter)."""
    img = create_base()
    
    cx, cy = 512, 512
    sun_r = 110
    
    glow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    gdraw = ImageDraw.Draw(glow)
    gdraw.ellipse([cx - 240, cy - 240, cx + 240, cy + 240], fill=(255, 180, 20, 100))
    glow = glow.filter(ImageFilter.GaussianBlur(50))
    img = Image.alpha_composite(img, glow)
    
    draw = ImageDraw.Draw(img)
    arc_r = 310
    
    num_ticks = 14
    start_deg = 135
    end_deg = 405
    for i in range(num_ticks):
        frac = i / (num_ticks - 1)
        angle_deg = start_deg + frac * (end_deg - start_deg)
        rad = math.radians(angle_deg)
        t_len = 24 + frac * 28
        t_width = int(8 + frac * 10)
        r = int(100 + frac * 155)
        g = int(140 + frac * 115)
        b = int(180 + frac * 75)
        
        x0 = cx + (arc_r - t_len) * math.cos(rad)
        y0 = cy + (arc_r - t_len) * math.sin(rad)
        x1 = cx + arc_r * math.cos(rad)
        y1 = cy + arc_r * math.sin(rad)
        draw.line([(x0, y0), (x1, y1)], fill=(r, g, b, 255), width=t_width)
        
    num_rays = 8
    for i in range(num_rays):
        angle = (i * 360 / num_rays)
        rad = math.radians(angle)
        r0 = sun_r + 28
        r1 = sun_r + 88
        x0 = cx + r0 * math.cos(rad)
        y0 = cy + r0 * math.sin(rad)
        x1 = cx + r1 * math.cos(rad)
        y1 = cy + r1 * math.sin(rad)
        draw.line([(x0, y0), (x1, y1)], fill=(255, 195, 45, 255), width=20)
        
    draw.ellipse([cx - sun_r, cy - sun_r, cx + sun_r, cy + sun_r], 
                 fill=(255, 170, 0, 255), outline=(255, 225, 120, 255), width=10)
    draw.ellipse([cx - sun_r + 30, cy - sun_r + 30, cx + sun_r - 30, cy + sun_r - 30], 
                 fill=(255, 245, 200, 255))
                 
    draw.rounded_rectangle([260, 780, 360, 830], radius=16, fill=(28, 36, 48, 255), outline=(50, 62, 85, 255), width=4)
    draw.line([(290, 805), (330, 805)], fill=(160, 180, 210, 255), width=8)
    
    draw.rounded_rectangle([CANVAS - 360, 780, CANVAS - 260, 830], radius=16, fill=(28, 36, 48, 255), outline=(50, 62, 85, 255), width=4)
    draw.line([(CANVAS - 330, 805), (CANVAS - 290, 805)], fill=(255, 195, 45, 255), width=8)
    draw.line([(CANVAS - 310, 785), (CANVAS - 310, 825)], fill=(255, 195, 45, 255), width=8)
    
    return img

def generate_temperature():
    """Adjust Temperature Icon (Kelvin Dual Sun: Warm Amber 2900K vs Ice Cyan 7000K)."""
    img = create_base()
    
    cx, cy = 512, 480
    r = 170
    
    glow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    gdraw = ImageDraw.Draw(glow)
    gdraw.ellipse([cx - r - 100, cy - r - 50, cx + 20, cy + r + 50], fill=(255, 120, 0, 120))
    gdraw.ellipse([cx - 20, cy - r - 50, cx + r + 100, cy + r + 50], fill=(0, 220, 255, 120))
    glow = glow.filter(ImageFilter.GaussianBlur(50))
    img = Image.alpha_composite(img, glow)
    
    draw = ImageDraw.Draw(img)
    
    for angle in [135, 180, 225]:
        rad = math.radians(angle)
        x0 = cx + (r + 25) * math.cos(rad)
        y0 = cy + (r + 25) * math.sin(rad)
        x1 = cx + (r + 85) * math.cos(rad)
        y1 = cy + (r + 85) * math.sin(rad)
        draw.line([(x0, y0), (x1, y1)], fill=(255, 140, 0, 255), width=20)
        
    for angle in [-45, 0, 45]:
        rad = math.radians(angle)
        x0 = cx + (r + 25) * math.cos(rad)
        y0 = cy + (r + 25) * math.sin(rad)
        x1 = cx + (r + 85) * math.cos(rad)
        y1 = cy + (r + 85) * math.sin(rad)
        draw.line([(x0, y0), (x1, y1)], fill=(0, 220, 255, 255), width=20)
        
    draw.line([(cx, cy - r - 25), (cx, cy - r - 85)], fill=(255, 240, 220, 255), width=20)
    draw.line([(cx, cy + r + 25), (cx, cy + r + 85)], fill=(255, 240, 220, 255), width=20)
    
    draw.pieslice([cx - r, cy - r, cx + r, cy + r], start=90, end=270, fill=(255, 120, 0, 255), outline=(255, 175, 50, 255), width=8)
    draw.pieslice([cx - r, cy - r, cx + r, cy + r], start=-90, end=90, fill=(0, 200, 255, 255), outline=(140, 240, 255, 255), width=8)
    draw.line([(cx, cy - r), (cx, cy + r)], fill=(24, 30, 42, 255), width=10)
    
    k_r = 60
    draw.ellipse([cx - k_r, cy - k_r, cx + k_r, cy + k_r], fill=(20, 26, 36, 255), outline=(60, 72, 95, 255), width=6)
    draw.line([(cx - 20, cy - 34), (cx - 20, cy + 34)], fill=(255, 255, 255, 255), width=10)
    draw.line([(cx + 22, cy - 34), (cx - 18, cy + 2)], fill=(255, 255, 255, 255), width=10)
    draw.line([(cx - 6, cy - 8), (cx + 24, cy + 34)], fill=(255, 255, 255, 255), width=10)
    
    bar_y = 780
    bar_h = 36
    bar_w = 540
    bx0 = cx - bar_w // 2
    bx1 = cx + bar_w // 2
    
    draw.rounded_rectangle([bx0 - 6, bar_y - 6, bx1 + 6, bar_y + bar_h + 6], radius=16, fill=(16, 20, 28, 255), outline=(45, 55, 75, 255), width=4)
    
    gbar = Image.new("RGBA", (bar_w, bar_h), (0, 0, 0, 0))
    gbdraw = ImageDraw.Draw(gbar)
    for x in range(bar_w):
        t = x / float(bar_w)
        r = int(255 * (1 - t * 0.9))
        g = int(120 + 100 * t)
        b = int(255 * t)
        gbdraw.line([(x, 0), (x, bar_h)], fill=(r, g, b, 255))
        
    mask = Image.new("L", (bar_w, bar_h), 0)
    mdraw = ImageDraw.Draw(mask)
    mdraw.rounded_rectangle([0, 0, bar_w, bar_h], radius=12, fill=255)
    img.paste(gbar, (bx0, bar_y), mask)
    
    return img

def generate_preset():
    """Lighting Preset Icon (Sleek Preset Scene Card with star badge + sliders)."""
    img = create_base()
    
    card_x0, card_y0, card_x1, card_y1 = 180, 210, 844, 790
    
    glow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    gdraw = ImageDraw.Draw(glow)
    gdraw.rounded_rectangle([card_x0 - 20, card_y0 - 20, card_x1 + 20, card_y1 + 20], radius=50, fill=(0, 210, 255, 70))
    gdraw.rounded_rectangle([card_x0, card_y0 + 200, card_x1 + 20, card_y1 + 20], radius=50, fill=(255, 160, 20, 70))
    glow = glow.filter(ImageFilter.GaussianBlur(40))
    img = Image.alpha_composite(img, glow)
    
    draw = ImageDraw.Draw(img)
    draw.rounded_rectangle([card_x0 + 30, card_y0 - 40, card_x1 - 30, card_y0 + 10], radius=24, fill=(20, 26, 36, 255), outline=(40, 50, 70, 255), width=6)
    draw.rounded_rectangle([card_x0 + 15, card_y0 - 20, card_x1 - 15, card_y0 + 20], radius=28, fill=(25, 32, 45, 255), outline=(50, 62, 85, 255), width=6)
    draw.rounded_rectangle([card_x0, card_y0, card_x1, card_y1], radius=36, fill=(28, 35, 48, 255), outline=(65, 80, 110, 255), width=10)
    draw.rounded_rectangle([card_x0 + 20, card_y0 + 20, card_x1 - 20, card_y0 + 150], radius=24, fill=(18, 23, 32, 255), outline=(45, 55, 78, 255), width=6)
    
    star_cx, star_cy = card_x0 + 90, card_y0 + 85
    star_r_out = 42
    star_r_in = 20
    points = []
    for i in range(10):
        r = star_r_out if i % 2 == 0 else star_r_in
        ang = math.radians(-90 + i * 36)
        points.append((star_cx + r * math.cos(ang), star_cy + r * math.sin(ang)))
    draw.polygon(points, fill=(255, 195, 25, 255), outline=(255, 235, 120, 255))
    
    draw.rounded_rectangle([card_x0 + 160, card_y0 + 60, card_x1 - 60, card_y0 + 80], radius=10, fill=(255, 255, 255, 220))
    draw.rounded_rectangle([card_x0 + 160, card_y0 + 95, card_x1 - 180, card_y0 + 115], radius=10, fill=(100, 120, 150, 220))
    
    s1_y = card_y0 + 250
    draw.rounded_rectangle([card_x0 + 40, s1_y, card_x1 - 40, s1_y + 24], radius=12, fill=(16, 20, 28, 255), outline=(40, 50, 70, 255), width=4)
    knob1_x = card_x0 + 420
    draw.rounded_rectangle([card_x0 + 40, s1_y, knob1_x, s1_y + 24], radius=12, fill=(0, 225, 255, 255))
    draw.ellipse([knob1_x - 30, s1_y - 20, knob1_x + 30, s1_y + 44], fill=(255, 255, 255, 255), outline=(0, 200, 255, 255), width=8)
    
    s2_y = card_y0 + 390
    draw.rounded_rectangle([card_x0 + 40, s2_y, card_x1 - 40, s2_y + 24], radius=12, fill=(16, 20, 28, 255), outline=(40, 50, 70, 255), width=4)
    knob2_x = card_x0 + 260
    draw.rounded_rectangle([card_x0 + 40, s2_y, knob2_x, s2_y + 24], radius=12, fill=(255, 160, 20, 255))
    draw.ellipse([knob2_x - 30, s2_y - 20, knob2_x + 30, s2_y + 44], fill=(255, 255, 255, 255), outline=(255, 140, 0, 255), width=8)
    
    s3_y = card_y0 + 510
    draw.rounded_rectangle([card_x0 + 40, s3_y, card_x1 - 40, s3_y + 20], radius=10, fill=(16, 20, 28, 255), outline=(40, 50, 70, 255), width=4)
    knob3_x = card_x0 + 480
    draw.rounded_rectangle([card_x0 + 40, s3_y, knob3_x, s3_y + 20], radius=10, fill=(240, 245, 255, 255))
    draw.ellipse([knob3_x - 26, s3_y - 18, knob3_x + 26, s3_y + 38], fill=(255, 255, 255, 255), outline=(180, 200, 230, 255), width=6)
    
    return img

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    target_dirs = [
        os.path.join(root_dir, "crates", "opencenter-opendeck", "assets"),
        os.path.join(root_dir, "com.akitafuki.opencenter.sdPlugin", "assets"),
    ]

    for d in target_dirs:
        os.makedirs(d, exist_ok=True)

    icons = {
        "icon": generate_icon_main,
        "category": generate_icon_main,
        "toggle": generate_toggle,
        "toggle_on": generate_toggle_on,
        "toggle_off": generate_toggle_off,
        "brightness": generate_brightness,
        "temperature": generate_temperature,
        "preset": generate_preset,
    }

    print("🎨 Rendering stylized high-res icons...")
    for name, gen_fn in icons.items():
        print(f"  - Generating {name}.png and {name}@2x.png...")
        img = gen_fn()
        img_2x = img.resize((288, 288), Image.Resampling.LANCZOS)
        img_1x = img.resize((144, 144), Image.Resampling.LANCZOS)
        for d in target_dirs:
            img_1x.save(os.path.join(d, f"{name}.png"), "PNG")
            img_2x.save(os.path.join(d, f"{name}@2x.png"), "PNG")

    print("✨ All icons generated successfully!")

if __name__ == "__main__":
    main()
