"""Phase 2 and 3 sprites: five floppies, the 18-column sheet, the tweet emote row. Same rules as ../skill-pack/sprites.py."""
import json, os
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
OLD = os.path.join(HERE, '..', 'skill-pack', 'sprites')
OUT = os.path.join(HERE, 'sprites')

def hx(h):
    h = h.lstrip('#'); return tuple(int(h[i:i+2], 16) for i in (0, 2, 4))
def mix(a, b, t):
    a, b = hx(a), hx(b); return '#%02X%02X%02X' % tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))

NEW = [  # id, dark body, light body, glyph
    ('sneak', '#E060EC', '#A82EB5', ["..######..", ".#..##..#.", "#...##...#", ".#..##..#.", "..######.."]),
    ('scrub', '#BFE6F7', '#7CC2E8', [".##....#..", "#..#..#.#.", "#..#...#..", ".##..##...", ".....##..."]),
    ('bigpeep', '#5560D8', '#2A2E8C', [".####.....", "######.#.#", "######....", "######.#.#", ".####....."]),
    ('tweet', '#DC4667', '#AE2348', ["#......#..", "###..#..#.", "####.#..#.", "###..#..#.", "#......#.."]),
    ('goose', '#A47552', '#6F4728', ["##......##", "###....###", "..##..##..", "...####...", "....##...."]),
]
PRIOR = {  # the nine already shipped, for the distance check
    'dark': ['#F7D64A', '#5FCF9A', '#FF8AB8', '#FF7350', '#6FA0FF', '#B08AF0', '#3FD0D8', '#B6E04A', '#F2A65E'],
    'light': ['#E3BC2C', '#2FA873', '#E0609A', '#E0502F', '#3F74E0', '#8A5FD6', '#13959E', '#76A01A', '#B8702E'],
}

BODY = [
    ".OOOOOOOOOOOOO..",
    "OBBOMMMMMMMOBBO.",
    "OBBOMMMMDDMOBBBO",
    "OBBOMMMMDDMOBBBO",
    "OBBOMMMMMMMOBBBO",
    "OBBBOOOOOOOBBBBO",
    "OBBBBBBBBBBBBBBO",
    "OBBTTTTTTTTTTBBO",
    "OBBLLLLLLLLLLBBO",
    None, None, None, None, None,
    "OBBLLLLLLLLLLBBO",
    ".OOOOOOOOOOOOOO.",
]
INK = '#221D2B'
def floppy(body, glyph, disabled=None):
    rows, gi = [], 0
    for r in BODY:
        if r is None:
            rows.append('OBB' + glyph[gi].replace('#', 'G').replace('.', 'L') + 'BBO'); gi += 1
        else: rows.append(r)
    if disabled:
        p2, mu = disabled
        pal = {'O': mu, 'B': p2, 'H': p2, 'S': p2, 'M': p2, 'D': mu, 'T': p2, 'L': p2, 'G': mix(p2, mu, .55)}
    else:
        pal = {'O': INK, 'B': body, 'H': mix(body, '#FFFFFF', .4), 'S': mix(body, '#000000', .22),
               'M': '#C9CCD6', 'D': '#5A5F70', 'T': mix(body, '#000000', .22), 'L': '#FFFCF6', 'G': INK}
    im = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    for y, row in enumerate(rows):
        for x, c in enumerate(row):
            if c == '.': continue
            if c == 'B' and x == 1: c = 'H'
            if c == 'B' and x == 14: c = 'S'
            if disabled and c == 'O' and (x + y) % 2: continue
            im.putpixel((x, y), hx(pal[c]) + (255,))
    return im

THEMES = {'dark': ('#262330', '#A49FB0'), 'light': ('#F1E9DB', '#645C70')}
PHASE1 = ['unpack', 'walk', 'reel']
for ti, theme in enumerate(('dark', 'light')):
    old = Image.open(f'{OLD}/floppies-{theme}.png').convert('RGBA')  # columns 0 to 9 stay as shipped
    sheet = Image.new('RGBA', (16 * 18, 32), (0, 0, 0, 0))
    sheet.paste(old, (0, 0))
    for i, fid in enumerate(PHASE1):
        sheet.paste(Image.open(f'{OUT}/floppy-{fid}-{theme}.png').convert('RGBA'), (16 * (10 + i), 0))
        sheet.paste(Image.open(f'{OUT}/floppy-{fid}-{theme}-disabled.png').convert('RGBA'), (16 * (10 + i), 16))
    for i, f in enumerate(NEW):
        on, off = floppy(f[1 + ti], f[3]), floppy(None, f[3], THEMES[theme])
        on.save(f'{OUT}/floppy-{f[0]}-{theme}.png'); off.save(f'{OUT}/floppy-{f[0]}-{theme}-disabled.png')
        sheet.paste(on, (16 * (13 + i), 0)); sheet.paste(off, (16 * (13 + i), 16))
    sheet.save(f'{OUT}/floppies-{theme}.png')

# ---------------- tweet emote: row 7 under the shipped seven ----------------
def blank(): return [['.'] * 8 for _ in range(8)]
def stamp(g, rows, ox=0, oy=0):
    for y, r in enumerate(rows):
        for x, c in enumerate(r):
            if c != '.': g[y + oy][x + ox] = c
    return g
SHUT = ["I...", "III.", "III.", "I..."]
OPEN = ["..I.", ".II.", "II..", "II..", ".II.", "..I."]
NEAR = ["T", "T"]
FAR = ["T.", ".T", ".T", ".T", ".T", "T."]
def e_tweet():
    return [stamp(blank(), SHUT, 0, 2),
            stamp(stamp(stamp(blank(), OPEN, 0, 1), NEAR, 5, 3), FAR, 6, 1),
            stamp(stamp(blank(), SHUT, 0, 2), FAR, 6, 1),
            stamp(stamp(blank(), OPEN, 0, 1), NEAR, 5, 3)]
TWEET = {'name': 'tweet', 'row': 7, 'tint': {'onLightBubble': '#6A48C8', 'onDarkBubble': '#C7B2FF'},
         'play': 'twice then hold frame 1', 'use': 'posted or received a tweet'}
for name, ink, tint in (('on-light', '#221D2B', '#6A48C8'), ('on-dark', '#FFFCF6', '#C7B2FF'),
                        ('mono-ink', '#1B1921', '#1B1921'), ('mono-paper', '#FFFFFF', '#FFFFFF')):
    old = Image.open(f'{OLD}/emotes-{name}.png').convert('RGBA')
    sheet = Image.new('RGBA', (32, 64), (0, 0, 0, 0))
    sheet.paste(old, (0, 0))
    for fi, g in enumerate(e_tweet()):
        for y in range(8):
            for x in range(8):
                if g[y][x] != '.': sheet.putpixel((fi * 8 + x, 56 + y), hx(ink if g[y][x] == 'I' else tint) + (255,))
    sheet.save(f'{OUT}/emotes-{name}.png')
man = json.load(open(os.path.join(HERE, '..', 'skill-pack', 'emotes.json')))
man['emotes'].append(TWEET)
json.dump(man, open(os.path.join(HERE, 'emotes.json'), 'w'), indent=2)

# ---------------- closest pair of the 14 label colours, per theme (CIE76) ----------------
def lab(h):
    r, g, b = [(c / 255) for c in hx(h)]
    r, g, b = [(c / 12.92 if c <= .04045 else ((c + .055) / 1.055) ** 2.4) for c in (r, g, b)]
    x, y, z = (.4124 * r + .3576 * g + .1805 * b) / .95047, .2126 * r + .7152 * g + .0722 * b, (.0193 * r + .1192 * g + .9505 * b) / 1.08883
    f = lambda t: t ** (1 / 3) if t > .008856 else 7.787 * t + 16 / 116
    return 116 * f(y) - 16, 500 * (f(x) - f(y)), 200 * (f(y) - f(z))
for ti, theme in enumerate(('dark', 'light')):
    cols = PRIOR[theme] + [f[1 + ti] for f in NEW]
    d, a, b = min((sum((p - q) ** 2 for p, q in zip(lab(a), lab(b))) ** .5, a, b) for i, a in enumerate(cols) for b in cols[i + 1:])
    print(theme, 'closest pair %.1f' % d, a, b)

def up(im, s): return im.resize((im.width * s, im.height * s), Image.NEAREST)
cs = Image.new('RGB', (18 * 16 * 4 + 20, 560), hx('#1B1921'))
cs.paste(hx('#FFFCF6'), (0, 280, cs.width, 560))
for ti, th in enumerate(('dark', 'light')):
    f = up(Image.open(f'{OUT}/floppies-{th}.png'), 4); cs.paste(f, (10, 10 + ti * 280), f)
    cs.paste(hx('#F3EFF8' if ti == 0 else '#2A2233'), (10, 150 + ti * 280, 300, 250 + ti * 280))
    e = Image.open(f'{OUT}/emotes-{"on-light" if ti == 0 else "on-dark"}.png').crop((0, 0, 32, 8))
    t = Image.open(f'{OUT}/emotes-{"on-light" if ti == 0 else "on-dark"}.png').crop((0, 56, 32, 64))
    cs.paste(up(e, 4), (20, 160 + ti * 280), up(e, 4)); cs.paste(up(t, 4), (160, 160 + ti * 280), up(t, 4))
cs.save(os.path.join(HERE, 'contact.png'))
