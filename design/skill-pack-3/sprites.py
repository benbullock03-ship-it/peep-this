"""Skill pack 3 sprites: drive light, step pips and the empty tray slot, on 16 px cells. Colours are tokens.css values."""
import os
from PIL import Image

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'sprites')
os.makedirs(OUT, exist_ok=True)

def hx(h):
    h = h.lstrip('#'); return tuple(int(h[i:i+2], 16) for i in (0, 2, 4)) + (255,)

# theme: led off, led on, pip todo, pip done, pip now, slot outline, drive face
THEMES = {
    'dark': ('#4A4458', '#7DFFA6', '#4A4458', '#EDE9F2', '#F7D64A', '#A49FB0', '#1B1921'),
    'light': ('#C9BFAE', '#1F7A3D', '#C9BFAE', '#2A2233', '#8A5A00', '#645C70', '#FFFCF6'),
}
# Floppy silhouette from skill-pack/sprites.py BODY: first and last inked column per row.
SPAN = [(1, 13)] + [(0, 14)] + [(0, 15)] * 13 + [(1, 14)]

def rect(im, x, y, w, h, c):
    for yy in range(y, y + h):
        for xx in range(x, x + w): im.putpixel((xx, yy), c)

for theme, (off, on, todo, done, now, slot, face) in THEMES.items():
    sheet = Image.new('RGBA', (16 * 6, 16), (0, 0, 0, 0))
    rect(sheet, 7, 7, 2, 2, hx(off))
    rect(sheet, 16 + 7, 7, 2, 2, hx(on))
    rect(sheet, 32 + 7, 7, 2, 2, hx(todo))
    rect(sheet, 48 + 7, 7, 2, 2, hx(done))
    rect(sheet, 64 + 7, 6, 2, 3, hx(now))
    for y, (a, b) in enumerate(SPAN):
        xs = range(a, b + 1) if y in (0, 15) else (a, b)
        for x in xs:
            if (x + y) % 2 == 0: sheet.putpixel((80 + x, y), hx(slot))
    sheet.save(f'{OUT}/drive-{theme}.png')

    # Reference drive face at 1x: 26 x 4, light on, six steps (2 done, 1 now, 3 to do), centred in two cells.
    ref = Image.new('RGBA', (32, 16), (0, 0, 0, 0))
    rect(ref, 3, 6, 26, 4, hx(face))
    rect(ref, 5, 7, 2, 2, hx(on))
    for i, c in enumerate((done, done, now, todo, todo, todo)):
        x = 3 + 26 - 1 - 17 + i * 3
        rect(ref, x, 6 if c == now else 7, 2, 3 if c == now else 2, hx(c))
    ref.save(f'{OUT}/drive-face-{theme}.png')
    print(theme, 'ok')
