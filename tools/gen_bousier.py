#!/usr/bin/env python3
"""Genere le sprite 32x32 du bousier (dung beetle) + sa boule de pattes.
Sortie : assets/sprites/bousier.png (32x32) et bousier@12x.png (apercu).
Dependance : aucune (encodeur PNG via stdlib zlib)."""
import zlib, struct, math, os, json

W = H = 32

# palette : key -> (r,g,b,a)
PAL = {
    '.': (0, 0, 0, 0),
    'o': (28, 20, 30, 255),      # contour
    # boule (de pattes)
    'H': (203, 160, 98, 255),
    'B': (170, 117, 63, 255),
    'S': (133, 90, 50, 255),
    'D': (104, 68, 37, 255),
    'g': (205, 163, 109, 255),   # patte qui depasse (clair)
    'k': (120, 80, 47, 255),     # detail "patte" sur la boule
    # bousier
    'b': (52, 40, 63, 255),
    'm': (78, 59, 95, 255),
    'h': (128, 98, 163, 255),    # reflet iridescent violet
    't': (80, 140, 145, 255),    # reflet iridescent teal
    'l': (38, 28, 45, 255),      # pattes
    'e': (240, 237, 232, 255),   # blanc de l'oeil
    'p': (22, 16, 24, 255),      # pupille
}

grid = [['.' for _ in range(W)] for _ in range(H)]

def put(x, y, c):
    xi, yi = int(round(x)), int(round(y))
    if 0 <= xi < W and 0 <= yi < H:
        grid[yi][xi] = c

def line(x0, y0, x1, y1, c):
    n = int(max(abs(x1 - x0), abs(y1 - y0))) + 1
    for i in range(n):
        t = i / max(n - 1, 1)
        put(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, c)

def disc(cx, cy, r, fill, outline='o'):
    for y in range(H):
        for x in range(W):
            d = math.hypot(x - cx, y - cy)
            if d <= r + 0.5:
                grid[y][x] = outline if d > r - 0.8 else fill

# ---------- 1. la boule (domine le cadre) ----------
cx, cy, R = 19.0, 14.5, 11.5
for y in range(H):
    for x in range(W):
        dx, dy = x - cx, y - cy
        d = math.hypot(dx, dy)
        if d <= R + 0.5:
            if d > R - 0.85:
                grid[y][x] = 'o'
            else:
                nx, ny = dx / R, dy / R
                light = nx * -0.55 + ny * -0.62
                grid[y][x] = ('H' if light > 0.42 else
                              'B' if light > -0.12 else
                              'S' if light > -0.5 else 'D')

# details "pattes" a la surface
for (mx, my) in [(15, 9), (22, 11), (13, 16), (24, 16), (18, 20), (21, 7)]:
    if math.hypot(mx - cx, my - cy) < R - 1.5:
        put(mx, my, 'k'); put(mx + 1, my, 'k')

# pattes qui depassent (boule de pattes) sur le haut / les cotes
for ang in [-120, -90, -62, -34, -6, 26]:
    a = math.radians(ang)
    for r in (R - 1, R, R + 1):
        put(cx + math.cos(a) * r, cy + math.sin(a) * r, 'g')
    put(cx + math.cos(a) * (R + 2), cy + math.sin(a) * (R + 2), 'o')

# ---------- 2. pattes au sol (sous le corps) ----------
line(6, 26, 3, 29, 'l')
line(5, 25, 4, 28, 'l')
line(8, 27, 10, 30, 'l')

# ---------- 3. corps du bousier (bas-gauche, pousse la boule) ----------
bx, by, rx, ry = 8.0, 22.5, 6.0, 5.2
for y in range(H):
    for x in range(W):
        nx, ny = (x - bx) / rx, (y - by) / ry
        e = nx * nx + ny * ny
        if e <= 1.10:
            if e > 0.80:
                grid[y][x] = 'o'
            else:
                light = nx * -0.5 + ny * -0.72
                grid[y][x] = ('h' if light > 0.42 else
                              'm' if light > -0.05 else 'b')
# touche d'iridescence teal
put(6, 19, 't'); put(7, 18, 't')

# ---------- 4. tete (vers la boule) ----------
disc(12.2, 18.4, 2.7, 'b')
put(11, 17, 'm'); put(12, 17, 'm')

# antenne
line(13, 16, 14, 13, 'l')
put(14, 13, 'b'); put(15, 13, 'b')

# ---------- 5. visage (malicieux) ----------
# yeux
put(11, 17, 'e'); put(11, 18, 'p')
put(13, 17, 'e'); put(13, 18, 'p')
# sourcils en biais (sournois)
put(10, 16, 'o'); put(11, 16, 'o')
put(13, 16, 'o'); put(14, 16, 'o')
# petit rictus
put(11, 20, 'o'); put(12, 20, 'o'); put(13, 21, 'o')

# ---------- 6. pattes avant sur la boule (par-dessus) ----------
line(13, 19, 16, 15, 'l')
line(12, 20, 15, 17, 'l')

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
pix = to_pixels(grid)
write_png('assets/sprites/bousier.png', pix, W, H)
write_png('assets/sprites/bousier@12x.png', scale(pix, W, H, 12), W * 12, H * 12)

# dump pour l'apercu inline (grille + palette)
print("=== GRID ===")
for row in grid:
    print(''.join(row))
print("=== PAL ===")
print(json.dumps({k: '#%02x%02x%02x' % v[:3] if v[3] else 'transparent'
                  for k, v in PAL.items()}))
print("=== OK : assets/sprites/bousier.png (+ @12x) ===")
