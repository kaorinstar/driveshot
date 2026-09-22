"""Draws the Driveshot icon, and writes every size and format the bundler needs.

    python3 tools/make-icon.py

That writes the whole of `src-tauri/icons`: the PNG sizes, `icon.ico` and `icon.icns`. Pass a
different directory as the first argument to write it somewhere else.

The Tauri CLI is no longer part of this. `npx tauri icon` takes one large PNG and resamples it
down to every other size, and resampling a mark this detailed to 16 or 32 pixels turns its strokes
into grey mush. Each size here is drawn at its own resolution instead, so a stroke is anti-aliased
once rather than drawn and then averaged away.

The mark is a cloud with an upward arrow inside it, held in a region-capture frame: the shot
leaving for the drive. It is drawn in code rather than kept as an image because the repository
then holds the shape itself, which can be adjusted, rather than a grid of pixels that cannot.

Below 40 pixels the frame is dropped and the cloud is filled with the arrow cut out of it. Three
white strokes inside 32 pixels cannot be told apart, and the tray icon is 16 pixels on Windows at
100% scaling. Both marks are the same shape; the small one says less of it.

No image library is used, because none is installed on the machines this is run on. Shapes are
described as scanline spans rather than as a per-pixel test, so anti-aliasing costs eight sub-rows
per pixel row instead of sixty-four samples per pixel, and the whole set takes about ten seconds.

The icon is a placeholder in the sense that it was drawn here rather than by a designer. It is
not a placeholder in the sense of being unfinished: it is the icon the application ships with
until a better one replaces it.
"""

import math
import os
import struct
import sys
import zlib

UNIT = 1024.0                # the coordinate space every shape below is written in
BG = (0x1E, 0x55, 0xA8)      # the blue of the application accent, one shade deeper
FG = (0xFF, 0xFF, 0xFF)
SUBROWS = 8
SIMPLIFY_BELOW = 40          # sizes under this get the mark without the frame

PNG_SIZES = {
    "32x32.png": 32,
    "64x64.png": 64,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
    "StoreLogo.png": 50,
    "Square30x30Logo.png": 30,
    "Square44x44Logo.png": 44,
    "Square71x71Logo.png": 71,
    "Square89x89Logo.png": 89,
    "Square107x107Logo.png": 107,
    "Square142x142Logo.png": 142,
    "Square150x150Logo.png": 150,
    "Square284x284Logo.png": 284,
    "Square310x310Logo.png": 310,
}
ICO_SIZES = (16, 24, 32, 48, 64, 128, 256)
ICNS_TYPES = (
    (b"icp4", 16), (b"icp5", 32), (b"ic11", 32), (b"ic12", 64),
    (b"ic07", 128), (b"ic13", 256), (b"ic08", 256), (b"ic14", 512),
    (b"ic09", 512), (b"ic10", 1024),
)


# --- shapes -------------------------------------------------------------------------------
# Each returns a function from a y coordinate to the horizontal spans it covers there.

def rounded_rect(x0, y0, x1, y1, r):
    r = min(r, (x1 - x0) / 2, (y1 - y0) / 2)

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


def circle(cx, cy, r):
    def spans(y):
        dy = abs(y - cy)
        if dy >= r:
            return []
        dx = math.sqrt(r * r - dy * dy)
        return [(cx - dx, cx + dx)]
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


def coverage(shapes, size):
    """Per-pixel coverage of the union of the shapes, as a list of rows of floats."""
    rows = []
    for py in range(size):
        acc = [0.0] * size
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
                b = min(b, float(size))
                if b <= a:
                    continue
                for px in range(int(a), min(int(math.ceil(b)), size)):
                    acc[px] += (min(b, px + 1) - max(a, px)) / SUBROWS
        rows.append([min(v, 1.0) for v in acc])
    return rows


# --- the mark -----------------------------------------------------------------------------

def cloud(k, box, inset):
    """The cloud as a union of blobs, each pulled in by `inset`.

    `box` scales the whole cloud about the middle of the plate, which is how the small mark
    grows to fill the space the frame leaves behind.

    The last blob spans the middle of the cloud and changes nothing about its silhouette. It is
    there for the inset copy: pulling each blob in by the stroke width shrinks the overlaps
    between them until gaps open, and those gaps would show as white wedges inside the outline.
    That blob is wide enough to bridge all of them.
    """
    def s(v):
        return k * (512 + (v - 512) * box)

    def r(v):
        return k * v * box

    i = inset
    return [
        rounded_rect(s(286 + i), s(500 + i), s(738 - i), s(672 - i), r(86)),
        circle(s(500), s(452), r(166 - i)),
        circle(s(358), s(546), r(122 - i)),
        circle(s(656), s(534), r(134 - i)),
        rounded_rect(s(350 + i), s(440 + i), s(680 - i), s(655 - i), r(96)),
    ]


def arrow(k, box):
    """The upward arrow that sits inside the cloud."""
    def s(v):
        return k * (512 + (v - 512) * box)

    return [
        triangle((s(508), s(388)), (s(580), s(484)), (s(436), s(484))),
        rounded_rect(s(481), s(476), s(535), s(592), k * 18 * box),
    ]


def brackets(k):
    """The selection frame: four corner brackets, which is how a region capture shows itself."""
    t, arm = 56, 148
    out = []
    for (cx, cy, sx, sy) in ((168, 168, 1, 1), (856, 168, -1, 1),
                             (168, 856, 1, -1), (856, 856, -1, -1)):
        hx0, hx1 = sorted((cx, cx + sx * arm))
        hy0, hy1 = sorted((cy, cy + sy * t))
        out.append(rounded_rect(k * hx0, k * hy0, k * hx1, k * hy1, k * t / 2))
        vx0, vx1 = sorted((cx, cx + sx * t))
        vy0, vy1 = sorted((cy, cy + sy * arm))
        out.append(rounded_rect(k * vx0, k * vy0, k * vx1, k * vy1, k * t / 2))
    return out


def render(size):
    """The icon at `size` pixels, as rows of RGBA bytes."""
    k = size / UNIT
    plate = [rounded_rect(k * 40, k * 40, k * 984, k * 984, k * 224)]

    if size >= SIMPLIFY_BELOW:
        # The full mark: the frame, the cloud as an outline, the arrow drawn inside it.
        solid = brackets(k) + arrow(k, 1.0)
        outer, inner = cloud(k, 1.0, 0), cloud(k, 1.0, 54)
        holes = []
    else:
        # The small mark: a filled cloud, larger, with the arrow cut out of it.
        box = 1.42
        solid = cloud(k, box, 0)
        outer, inner = [], []
        holes = arrow(k, box)

    plate_cov = coverage(plate, size)
    solid_cov = coverage(solid, size)
    outer_cov = coverage(outer, size) if outer else None
    inner_cov = coverage(inner, size) if inner else None
    hole_cov = coverage(holes, size) if holes else None

    rows = []
    for y in range(size):
        row = bytearray()
        for x in range(size):
            a = plate_cov[y][x]
            if a <= 0.0:
                row.extend((0, 0, 0, 0))
                continue
            f = solid_cov[y][x]
            if outer_cov is not None:
                f = max(f, outer_cov[y][x] - inner_cov[y][x])
            if hole_cov is not None:
                f -= hole_cov[y][x]
            f = min(max(f, 0.0), a)
            # The mark is painted over the plate, so the blend happens inside the plate's alpha.
            mix = f / a
            for channel in range(3):
                row.append(int(round(BG[channel] * (1.0 - mix) + FG[channel] * mix)))
            row.append(int(round(a * 255)))
        rows.append(bytes(row))
    return rows


# --- files --------------------------------------------------------------------------------

def png_bytes(size, rows_rgba):
    raw = bytearray()
    for row in rows_rgba:
        raw.append(0)
        raw.extend(row)
    data = zlib.compress(bytes(raw), 9)

    def chunk(tag, payload):
        return (struct.pack(">I", len(payload)) + tag + payload
                + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF))

    return (b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", data)
            + chunk(b"IEND", b""))


def ico_bytes(images):
    """A Windows icon holding one PNG per size. Vista and later read PNG entries directly."""
    header = struct.pack("<HHH", 0, 1, len(images))
    offset = len(header) + 16 * len(images)
    entries, payloads = bytearray(), bytearray()
    for size, blob in images:
        # A dimension of 256 is written as 0; the field is one byte wide.
        entries.extend(struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0,
                                   1, 32, len(blob), offset))
        payloads.extend(blob)
        offset += len(blob)
    return header + bytes(entries) + bytes(payloads)


def icns_bytes(images):
    """A macOS icon family. Every type used here takes its image as a PNG."""
    body = bytearray()
    for tag, size in ICNS_TYPES:
        blob = images[size]
        body.extend(tag + struct.pack(">I", len(blob) + 8) + blob)
    return b"icns" + struct.pack(">I", len(body) + 8) + bytes(body)


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else os.path.join("src-tauri", "icons")
    os.makedirs(out, exist_ok=True)

    wanted = sorted(set(PNG_SIZES.values()) | set(ICO_SIZES) | {s for _, s in ICNS_TYPES})
    blobs = {}
    for size in wanted:
        blobs[size] = png_bytes(size, render(size))
        print("drew", size)

    for name, size in sorted(PNG_SIZES.items()):
        with open(os.path.join(out, name), "wb") as handle:
            handle.write(blobs[size])

    with open(os.path.join(out, "icon.ico"), "wb") as handle:
        handle.write(ico_bytes([(s, blobs[s]) for s in ICO_SIZES]))

    with open(os.path.join(out, "icon.icns"), "wb") as handle:
        handle.write(icns_bytes(blobs))

    print("wrote", len(PNG_SIZES) + 2, "files to", out)


if __name__ == "__main__":
    main()
