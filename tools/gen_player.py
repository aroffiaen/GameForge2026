#!/usr/bin/env python3
"""Base perso NEUTRE en vraie vue 3/4 top-down (sans chara design).
On voit le dessus de la tete (cheveux) + la face, corps vu de haut.
4 directions : bas (face), droite, gauche (miroir), haut (dos).
Sortie : assets/sprites/player_{down,right,left,up}.png + player_sheet.png.
Contour auto. Aucune dependance (PNG via stdlib)."""
import zlib, struct, math, os, json

W = H = 32

PAL = {
    '.': (0, 0, 0, 0),
    'o': (40, 38, 46, 255),    # contour neutre
    'K': (232, 184, 143, 255), # peau
    'k': (200, 148, 108, 255), # peau ombre
    'r': (92, 72, 50, 255),    # cheveux
    'R': (112, 90, 64, 255),   # cheveux clair (dessus)
    'B': (138, 144, 154, 255), # corps (gris neutre)
    'b': (108, 114, 124, 255), # corps ombre
    'P': (84, 88, 96, 255),    # jambes
    'S': (52, 54, 60, 255),    # pieds
    'e': (44, 36, 40, 255),    # yeux
}

def new_grid():
    return [['.' for _ in range(W)] for _ in range(H)]

def rect(g, x0, y0, x1, y1, c):
    for y in range(int(y0), int(y1) + 1):
        for x in range(int(x0), int(x1) + 1):
            if 0 <= x < W and 0 <= y < H:
                g[y][x] = c

def ell(g, cx, cy, rx, ry, c):
    for y in range(H):
        for x in range(W):
            nx, ny = (x - cx) / rx, (y - cy) / ry
            if nx * nx + ny * ny <= 1.0:
                g[y][x] = c

def outline_pass(g, col='o'):
    adds = []
    for y in range(H):
        for x in range(W):
            if g[y][x] == '.':
                for dx, dy in ((1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,-1),(1,-1),(-1,1)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < W and 0 <= ny < H and g[ny][nx] not in ('.', col):
                        adds.append((x, y)); break
    for (x, y) in adds:
        g[y][x] = col

def mirror(g):
    return [list(reversed(row)) for row in g]

# ---- corps vu de haut (commun) ----
def torso(g):
    rect(g, 10, 14, 21, 22, 'B')      # tronc
    rect(g, 20, 14, 21, 22, 'b')      # ombre cote
    rect(g, 7, 15, 9, 21, 'B')        # bras g
    rect(g, 22, 15, 24, 21, 'B')      # bras d
    rect(g, 7, 21, 9, 22, 'K')        # mains
    rect(g, 22, 21, 24, 22, 'K')
    rect(g, 11, 23, 14, 29, 'P')      # jambe g
    rect(g, 17, 23, 20, 29, 'P')      # jambe d
    rect(g, 11, 30, 14, 31, 'S')      # pieds
    rect(g, 17, 30, 20, 31, 'S')

def make_down():
    g = new_grid()
    torso(g)
    ell(g, 16, 8, 5.2, 5.6, 'K')      # tete
    rect(g, 14, 13, 17, 14, 'k')      # cou
    ell(g, 16, 5.2, 5.4, 3.8, 'r')    # cheveux (dessus, plonge 3/4)
    rect(g, 12, 4, 20, 4, 'R')        # eclat dessus du crane
    rect(g, 11, 9, 20, 9, 'k')        # ombre cheveux/front
    g[11][14] = 'e'; g[11][18] = 'e'  # yeux
    outline_pass(g)
    return g

def make_right():
    g = new_grid()
    torso(g)
    rect(g, 21, 15, 24, 18, 'B')      # bras avant tendu (droite)
    rect(g, 24, 18, 24, 18, 'K')      # main avant
    ell(g, 16.5, 8, 5.0, 5.6, 'K')    # tete
    rect(g, 14, 13, 17, 14, 'k')
    ell(g, 15.0, 5.4, 5.4, 3.9, 'r')  # cheveux (dessus, decales)
    rect(g, 11, 7, 13, 12, 'r')       # cheveux arriere (cote gauche)
    rect(g, 12, 4, 19, 4, 'R')
    rect(g, 13, 9, 20, 9, 'k')
    g[11][17] = 'e'; g[11][19] = 'e'  # yeux vers la droite
    g[12][20] = 'k'                   # nez
    outline_pass(g)
    return g

def make_up():
    g = new_grid()
    torso(g)
    ell(g, 16, 8, 5.2, 5.6, 'K')      # tete
    ell(g, 16, 7.4, 5.4, 5.0, 'r')    # cheveux (vu de dos : presque tout)
    rect(g, 12, 4, 20, 4, 'R')
    rect(g, 14, 12, 17, 13, 'K')      # nuque
    outline_pass(g)
    return g

frames = {
    'down':  make_down(),
    'right': make_right(),
    'left':  mirror(make_right()),
    'up':    make_up(),
}

# ================= sortie PNG =================
def to_pixels(g):
    return [PAL[g[y][x]] for y in range(H) for x in range(W)]

def write_png(path, pix, w, h):
    raw = bytearray()
    for y in range(h):
        raw.append(0)
        for x in range(w):
            raw += bytes(pix[y * w + x])
    comp = zlib.compress(bytes(raw), 9)
    def chunk(typ, data):
        return (struct.pack(">I", len(data)) + typ + data +
                struct.pack(">I", zlib.crc32(typ + data) & 0xffffffff))
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    with open(path, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n' +
                chunk(b'IHDR', ihdr) + chunk(b'IDAT', comp) + chunk(b'IEND', b''))

def scale(pix, w, h, s):
    return [pix[(y // s) * w + (x // s)] for y in range(h * s) for x in range(w * s)]

os.makedirs('assets/sprites', exist_ok=True)
order = ['down', 'right', 'left', 'up']
for name in order:
    write_png('assets/sprites/player_%s.png' % name, to_pixels(frames[name]), W, H)

sheet = [(0, 0, 0, 0)] * (W * 4 * H)
for fi, name in enumerate(order):
    g = frames[name]
    for y in range(H):
        for x in range(W):
            sheet[y * (W * 4) + fi * W + x] = PAL[g[y][x]]
write_png('assets/sprites/player_sheet.png', sheet, W * 4, H)
write_png('assets/sprites/player_sheet@8x.png', scale(sheet, W * 4, H, 8), W * 4 * 8, H * 8)

print("=== PAL ===")
print(json.dumps({k: '#%02x%02x%02x' % v[:3] if v[3] else 'transparent' for k, v in PAL.items()}))
for name in order:
    print("=== %s ===" % name.upper())
    for row in frames[name]:
        print(''.join(row))
print("=== OK ===")
