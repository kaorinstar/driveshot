"""Draws the Driveshot source icon, and writes it as a PNG.

Run it, then hand what it wrote to the Tauri CLI, which produces every size and format the
bundler needs:

    python3 tools/make-icon.py icon-source.png
    npx tauri icon icon-source.png -o src-tauri/icons
    rm -rf src-tauri/icons/android src-tauri/icons/ios icon-source.png

The last line removes what the CLI writes for the two mobile platforms. Driveshot targets Windows
and macOS, and those folders are around a hundred files that nothing here reads.

The mark is a region-capture frame with an upward arrow inside it: the shot leaving for the drive.
It is drawn in code rather than kept as an image because the repository then holds the shape
itself, which can be adjusted, rather than a grid of pixels that cannot.

No image library is used, because none is installed on the machines this is run on. Shapes are
described as scanline spans rather than as a per-pixel test, so anti-aliasing costs four sub-rows
per pixel row instead of sixteen samples per pixel, and the whole 1024x1024 image takes about
three seconds.

The icon is a placeholder in the sense that it was drawn here rather than by a designer. It is
not a placeholder in the sense of being unfinished: it is the icon the application ships with
until a better one replaces it.
"""

import math
import struct
import zlib
import sys

SIZE = 1024
BG = (0x1E, 0x55, 0xA8)      # the blue of the application accent, one shade deeper
FG = (0xFF, 0xFF, 0xFF)
SUBROWS = 4


def rounded_rect(x0, y0, x1, y1, r):
    def spans(y):
        if y < y0 or y > y1:
            return []
        if y < y0 + r:
            dy = (y0 + r) - y
        elif y > y1 - r:
            dy = y - (y1 - r)
        else:
            return [(x0, x1)]
        if dy > r:
            return []
        dx = math.sqrt(max(r * r - dy * dy, 0.0))
        return [(x0 + r - dx, x1 - r + dx)]
    return spans


def triangle(p0, p1, p2):
    pts = [p0, p1, p2]

    def spans(y):
        xs = []
        for i in range(3):
            (ax, ay), (bx, by) = pts[i], pts[(i + 1) % 3]
            if ay == by:
                continue
            lo, hi = (ay, by) if ay < by else (by, ay)
            if not (lo <= y < hi):
                continue
            xs.append(ax + (bx - ax) * (y - ay) / (by - ay))
        if len(xs) < 2:
            return []
        return [(min(xs), max(xs))]
    return spans


def coverage(shapes):
    """Per-pixel coverage of the union of the shapes, as a list of rows of floats."""
    rows = []
    for py in range(SIZE):
        acc = [0.0] * SIZE
        for sub in range(SUBROWS):
            y = py + (sub + 0.5) / SUBROWS
            spans = []
            for shape in shapes:
                spans.extend(shape(y))
            if not spans:
                continue
            # Merge overlapping spans so an overlap is not counted twice.
            spans.sort()
            merged = [list(spans[0])]
            for a, b in spans[1:]:
                if a <= merged[-1][1]:
                    merged[-1][1] = max(merged[-1][1], b)
                else:
                    merged.append([a, b])
            for a, b in merged:
                a = max(a, 0.0)
                b = min(b, float(SIZE))
                if b <= a:
                    continue
                for px in range(int(a), min(int(math.ceil(b)), SIZE)):
                    acc[px] += (min(b, px + 1) - max(a, px)) / SUBROWS
        rows.append([min(v, 1.0) for v in acc])
    return rows


def write_png(path, rows_rgba):
    raw = bytearray()
    for row in rows_rgba:
        raw.append(0)
        raw.extend(row)
    data = zlib.compress(bytes(raw), 9)

    def chunk(tag, payload):
        return (struct.pack(">I", len(payload)) + tag + payload
                + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF))

    png = (b"\x89PNG\r\n\x1a\n"
           + chunk(b"IHDR", struct.pack(">IIBBBBB", SIZE, SIZE, 8, 6, 0, 0, 0))
           + chunk(b"IDAT", data)
           + chunk(b"IEND", b""))
    with open(path, "wb") as handle:
        handle.write(png)


# The plate the mark sits on.
background = [rounded_rect(40, 40, SIZE - 40, SIZE - 40, 224)]

# The selection frame: four corner brackets, which is how a region capture shows itself.
FRAME_X0, FRAME_Y0, FRAME_X1, FRAME_Y1 = 212, 212, 812, 812
T = 52          # stroke width
ARM = 176       # how far each bracket runs along the edge
brackets = []
for (cx, cy, sx, sy) in ((FRAME_X0, FRAME_Y0, 1, 1), (FRAME_X1, FRAME_Y0, -1, 1),
                         (FRAME_X0, FRAME_Y1, 1, -1), (FRAME_X1, FRAME_Y1, -1, -1)):
    hx0, hx1 = sorted((cx, cx + sx * ARM))
    hy0, hy1 = sorted((cy, cy + sy * T))
    brackets.append(rounded_rect(hx0, hy0, hx1, hy1, T / 2))
    vx0, vx1 = sorted((cx, cx + sx * T))
    vy0, vy1 = sorted((cy, cy + sy * ARM))
    brackets.append(rounded_rect(vx0, vy0, vx1, vy1, T / 2))

# The upward arrow: the shot leaving for the drive.
ARROW_X = 512
arrow = [
    rounded_rect(ARROW_X - 40, 452, ARROW_X + 40, 700, 26),
    triangle((ARROW_X, 300), (ARROW_X + 150, 476), (ARROW_X - 150, 476)),
]

bg_cov = coverage(background)
fg_cov = coverage(brackets + arrow)

rows = []
for y in range(SIZE):
    row = bytearray()
    for x in range(SIZE):
        a = bg_cov[y][x]
        f = min(fg_cov[y][x], a)
        if a <= 0.0:
            row.extend((0, 0, 0, 0))
            continue
        # The mark is painted over the plate, so the blend happens inside the plate's alpha.
        mix = f / a
        for channel in range(3):
            value = BG[channel] * (1.0 - mix) + FG[channel] * mix
            row.append(int(round(value)))
        row.append(int(round(a * 255)))
    rows.append(bytes(row))

write_png(sys.argv[1], rows)
print("wrote", sys.argv[1])
