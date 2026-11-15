"""Peep This floppy / emote / peep sprites. Writes PNG sheets + JSON manifests to ./assets."""
import json, os
from PIL import Image

OUT = os.path.join(os.path.dirname(__file__), 'assets')
os.makedirs(OUT, exist_ok=True)

def hx(h):
    h = h.lstrip('#'); return tuple(int(h[i:i+2], 16) for i in (0, 2, 4))
def mix(a, b, t):
    a, b = hx(a), hx(b); return '#%02X%02X%02X' % tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))

# ---------------- floppies ----------------
FLOPPIES = [  # id, name, dark body, light body, command
    ('ducks', 'Ducks', '#F7D64A', '#E3BC2C', '/ducks'),
    ('question-time', 'Question Time', '#5FCF9A', '#2FA873', '/question-time'),
    ('ticket', 'Ticket', '#FF8AB8', '#E0609A', '/ticket'),
    ('shoot', 'Shoot', '#FF7350', '#E0502F', '/shoot'),
    ('handoff', 'Handoff', '#6FA0FF', '#3F74E0', '/handoff'),
    ('coderot', 'Coderot', '#B08AF0', '#8A5FD6', '/coderot'),
]
GLYPH = {
    'ticket': ["##########", "#..#.....#", "...#......", "#..#.....#", "##########"],
    'ducks': [".....###..", "#....#.###", "########..", ".#######..", "..#####..."],
    'question-time': ["...####...", "......#...", "....##....", "..........", "....#....."],
    'handoff': ["......#...", ".......#..", "#########.", ".......#..", "......#..."],
    'shoot': ["....##....", "....##....", "##..##..##", "....##....", "....##...."],
    'coderot': [".#......#.", "..######..", "#.#.##.#.#", "..######..", ".#......#."],
    'blank': [".........."] * 5,
}
FONT = {  # 3x5 letters for initials on custom floppies
    'R': ["##.", "#.#", "##.", "#.#", "#.#"], 'V': ["#.#", "#.#", "#.#", "#.#", ".#."],
    'D': ["##.", "#.#", "#.#", "#.#", "##."], 'P': ["###", "#.#", "###", "#..", "#.."],
    'T': ["###", ".#.", ".#.", ".#.", ".#."], 'S': ["###", "#..", "###", "..#", "###"],
}
def initials(two):
    a, b = FONT[two[0]], FONT[two[1]]
    return ['.' + a[r] + '..' + b[r] + '.' for r in range(5)]
CUSTOM = [('review', 'RV', '#4FD1C5', '#1E9E93'), ('deploy', 'DP', '#FFA24A', '#D97A1E'), ('tests', 'TS', '#C9CCD6', '#8E93A6')]

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
    rows = []
    gi = 0
    for r in BODY:
        if r is None:
            rows.append('OBB' + glyph[gi].replace('#', 'G').replace('.', 'L') + 'BBO'); gi += 1
        else: rows.append(r)
    if disabled:  # disabled = (panel2, muted): flat, ghosted, no label
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
            if disabled and c == 'O' and (x + y) % 2: continue  # dashed outline
            im.putpixel((x, y), hx(pal[c]) + (255,))
    return im

THEMES = {'dark': ('#262330', '#A49FB0'), 'light': ('#F1E9DB', '#645C70')}
order = [f[0] for f in FLOPPIES] + ['blank'] + [c[0] for c in CUSTOM]
for ti, theme in enumerate(('dark', 'light')):
    sheet = Image.new('RGBA', (16 * len(order), 32), (0, 0, 0, 0))
    for i, f in enumerate(FLOPPIES):
        sheet.paste(floppy(f[2 + ti], GLYPH[f[0]]), (16 * i, 0))
        sheet.paste(floppy(None, GLYPH[f[0]], THEMES[theme]), (16 * i, 16))
        floppy(f[2 + ti], GLYPH[f[0]]).save(f'{OUT}/floppy-{f[0]}-{theme}.png')
        floppy(None, GLYPH[f[0]], THEMES[theme]).save(f'{OUT}/floppy-{f[0]}-{theme}-disabled.png')
    blank_body = '#DAD5E2' if theme == 'dark' else '#B9B2C4'
    sheet.paste(floppy(blank_body, GLYPH['blank']), (16 * 6, 0))
    sheet.paste(floppy(None, GLYPH['blank'], THEMES[theme]), (16 * 6, 16))
    floppy(blank_body, GLYPH['blank']).save(f'{OUT}/floppy-blank-{theme}.png')
    for j, c in enumerate(CUSTOM):
        sheet.paste(floppy(c[2 + ti], initials(c[1])), (16 * (7 + j), 0))
        sheet.paste(floppy(None, initials(c[1]), THEMES[theme]), (16 * (7 + j), 16))
    sheet.save(f'{OUT}/floppies-{theme}.png')

# ---------------- emotes: 8x8, 4 frames, I = ink, T = tint ----------------
def blank(): return [['.'] * 8 for _ in range(8)]
def stamp(g, rows, ox=0, oy=0):
    for y, r in enumerate(rows):
        for x, c in enumerate(r):
            if c != '.' and 0 <= x + ox < 8 and 0 <= y + oy < 8: g[y + oy][x + ox] = c
    return g
BANG = ["II", "II", "II", "II", "II", "..", "II"]
DROP = [".T.", ".T.", "TTT", "TTT", ".T."]
Z3 = ["III", ".I.", "III"]
Z4 = ["IIII", "..I.", ".I..", "IIII"]
HAND = [".I.I.I.", ".I.I.I.", ".IIIII.", ".IIIII.", "..III..", "..III.."]
STAR_S = [".T.", "TTT", ".T."]
STAR_L = ["..T..", "..T..", "TTTTT", "..T..", "..T.."]
def e_startled():
    f = []
    for i, (oy, sp) in enumerate([(1, 0), (0, 1), (1, 0), (0, 1)]):
        g = stamp(blank(), BANG, 3, oy)
        if sp: stamp(g, ["T"], 0, 1); stamp(g, ["T"], 7, 1); stamp(g, ["T"], 1, 3); stamp(g, ["T"], 6, 3)
        f.append(g)
    return f
def e_sweat(): return [stamp(blank(), DROP, 4, oy) for oy in (-1, 0, 2, 4)]
def e_yawn():
    o1 = ["II", "II"]; o2 = [".II.", "I..I", "I..I", ".II."]; o3 = [".IIII.", "I....I", "I....I", "I....I", "I....I", ".IIII."]
    f = [stamp(blank(), o1, 3, 4), stamp(blank(), o2, 2, 3), stamp(blank(), o3, 1, 2), stamp(blank(), o3, 1, 2)]
    stamp(f[3], ["T", "T"], 7, 0); stamp(f[2], ["T"], 7, 1)
    return f
def e_sparkle():
    return [stamp(blank(), STAR_S, 2, 3), stamp(stamp(blank(), STAR_L, 1, 2), ["T"], 7, 0),
            stamp(stamp(stamp(blank(), STAR_L, 1, 2), STAR_S, 5, 0), ["T"], 0, 7), stamp(stamp(blank(), STAR_S, 2, 3), STAR_S, 5, 0)]
def e_wave():
    f = []
    for ox, m in [(0, 0), (1, 1), (0, 0), (-1, 2)]:
        g = stamp(blank(), HAND, 1 + (ox if ox > 0 else 0) - (1 if ox < 0 else 0), 1)
        if m == 1: stamp(g, ["T", ".", "T"], 0, 1)
        if m == 2: stamp(g, ["T", ".", "T"], 7, 1)
        f.append(g)
    return f
def e_worry():
    f = []
    for lens in [(5, 3, 4, 2), (4, 4, 3, 3), (3, 5, 2, 4), (4, 4, 3, 3)]:
        g = blank()
        for i, L in enumerate(lens):
            for y in range(L): g[1 + y][i * 2] = 'T'
        f.append(g)
    return f
def e_zzz():
    return [stamp(blank(), Z3, 0, 5), stamp(stamp(blank(), Z3, 0, 5), Z3, 3, 2),
            stamp(stamp(blank(), Z3, 0, 5), Z4, 4, 0), stamp(stamp(blank(), Z3, 1, 4), Z4, 4, 0)]
EMOTES = [  # name, frames, tint on-light-bubble (dark theme), tint on-dark-bubble (light theme), loop, used for
    ('startled', e_startled(), '#A85B00', '#FFB23E', 'twice then hold frame 1', 'needs you: permission or question'),
    ('sweat', e_sweat(), '#0B7391', '#7FE6FF', 'loop', 'long run, retries'),
    ('yawn', e_yawn(), '#0B7391', '#7FE6FF', 'once then hold frame 0', 'context past the threshold'),
    ('sparkle', e_sparkle(), '#9A6A00', '#F7D64A', 'twice then hide', 'success'),
    ('wave', e_wave(), '#6A48C8', '#C7B2FF', 'twice then hide', 'new session, Jump copied'),
    ('worry', e_worry(), '#5A6477', '#A9B3C6', 'loop', 'clash, error'),
    ('zzz', e_zzz(), '#6A48C8', '#C7B2FF', 'loop', 'idle for a while'),
]
for name, ink, col in (('on-light', '#221D2B', 2), ('on-dark', '#FFFCF6', 3), ('mono-ink', '#1B1921', None), ('mono-paper', '#FFFFFF', None)):
    sheet = Image.new('RGBA', (32, 8 * len(EMOTES)), (0, 0, 0, 0))
    for r, e in enumerate(EMOTES):
        tint = e[col] if col else ink
        for fi, g in enumerate(e[1]):
            for y in range(8):
                for x in range(8):
                    c = g[y][x]
                    if c != '.': sheet.putpixel((fi * 8 + x, r * 8 + y), hx(ink if c == 'I' else tint) + (255,))
    sheet.save(f'{OUT}/emotes-{name}.png')
json.dump({'cell': 8, 'frames': 4, 'fps': 8, 'tones': ['ink', 'tint'],
           'emotes': [{'name': e[0], 'row': i, 'tint': {'onLightBubble': e[2], 'onDarkBubble': e[3]}, 'play': e[4], 'use': e[5]} for i, e in enumerate(EMOTES)]},
          open(f'{OUT}/emotes.json', 'w'), indent=2)

# ---------------- stand-in peeps (the build uses <peep-bot>) ----------------
PEEP = [
    "................", ".....OOOOOO.....", "....OHBBBBBO....", "...OHBBBBBBBO...", "..OHBBBBBBBBBO..",
    "..OBBBBBBBBBBO..", "..OBBBBBBBBBBO..", "..OBBBBKKBBBBO..", "..OCCBBBBBBCCO..", "..OBBBBBBBBBBO..",
    "..OSBBBBBBBBSO..", "...OSBBBBBBSO...", "....OSSSSSSO....", ".....OOOOOO.....", "....FF....FF....", "................",
]
COLS = [('yellow', '#F7D64A'), ('lavender', '#C7B2FF'), ('mint', '#8DE8C0')]
EYES = {'ahead': 5, 'up': 4, 'down': 6}
sheet = Image.new('RGBA', (16 * 9, 16), (0, 0, 0, 0))
k = 0
for cn, c in COLS:
    pal = {'O': mix(c, '#2A1E12', .68), 'B': c, 'H': mix(c, '#FFFFFF', .5), 'S': mix(c, '#000000', .2), 'K': '#FA8A42', 'C': '#FAA262', 'F': '#FA8A42', 'E': '#3A2418'}
    for en, ey in EYES.items():
        for y, row in enumerate(PEEP):
            for x, ch in enumerate(row):
                if ch != '.': sheet.putpixel((k * 16 + x, y), hx(pal[ch]) + (255,))
        for ex in (5, 10):
            for dy in (0, 1): sheet.putpixel((k * 16 + ex, ey + dy), hx(pal['E']) + (255,))
        k += 1
sheet.save(f'{OUT}/peeps.png')

# contact sheet for checking
def up(im, s): return im.resize((im.width * s, im.height * s), Image.NEAREST)
cs = Image.new('RGB', (1000, 760), hx('#1B1921'))
cs.paste(hx('#FFFCF6'), (0, 380, 1000, 760))
for ti, th in enumerate(('dark', 'light')):
    f = up(Image.open(f'{OUT}/floppies-{th}.png'), 6); cs.paste(f, (10, 10 + ti * 380), f)
    p = up(Image.open(f'{OUT}/peeps.png'), 5); cs.paste(p, (10, 215 + ti * 380), p)
cs.paste(hx('#F3EFF8'), (740, 10, 990, 370)); cs.paste(hx('#2A2233'), (740, 390, 990, 750))
e = up(Image.open(f'{OUT}/emotes-on-light.png'), 6); cs.paste(e, (760, 20), e)
e = up(Image.open(f'{OUT}/emotes-on-dark.png'), 6); cs.paste(e, (760, 400), e)
cs.save(os.path.join(os.path.dirname(__file__), 'contact.png'))
print('ok', order)
