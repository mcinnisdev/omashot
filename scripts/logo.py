#!/usr/bin/env python3
"""The Omashot mark, drawn as pixel art.

Everything lives on a 16x16 grid and is scaled by whole numbers, so every
edge lands on a pixel boundary and nothing is ever blurred. 16 divides into
every icon size that matters (32, 64, 128, 256, 512) exactly, which is the
whole reason the grid is that size.

Two marks:

  handoff   corner brackets with an arrow leaving through them: pick a
            region, hand it off. The default, and the whole product in one
            glyph.
  frame     the brackets alone, for places where the name is already beside
            the mark and the arrow is just noise.

Colour is an argument rather than a constant: the app re-tints the bar and
tray mark to the current Omarchy accent at runtime, so it belongs to
whatever theme you are running. See `theme.rs`.

The brackets run to the edge of the grid, which is why the icons ship as the
bare mark rather than knocked out of a tile -- inverted, the corners read as
notches instead of brackets. `--style tile` is kept for the favicon and the
social card, where a solid block earns its place.

    ./logo.py --out mark.png --size 512
    ./logo.py --out tray.png --size 32 --color '#ff9e64'
    ./logo.py --ascii --mark frame
"""

import argparse
import struct
import zlib

GRID = 16

# Each mark is 16 rows of 16 columns. '#' is on, anything else is off.
# Drawn by hand: at these sizes a rasterised circle looks worse than a
# letterform someone placed pixel by pixel.
MARKS = {
    # Corner brackets with an arrow leaving through them: select a region,
    # hand it off. The brackets are the one glyph everybody already reads as
    # "pick part of the screen", and they survive being 16 pixels wide.
    "handoff": """
    ######....######
    ######....######
    ##............##
    ##............##
    ##............##
    .........##.....
    ..........##....
    ....#########...
    ....#########...
    ..........##....
    .........##.....
    ##............##
    ##............##
    ##............##
    ######....######
    ######....######
    """,
    # The same brackets with nothing inside, for places where the mark sits
    # next to the name anyway and the arrow is just noise.
    "frame": """
    ######....######
    ######....######
    ##............##
    ##............##
    ##............##
    ................
    ................
    ................
    ................
    ................
    ................
    ##............##
    ##............##
    ##............##
    ######....######
    ######....######
    """,
}


def grid(mark: str) -> list[list[bool]]:
    rows = [ln.strip() for ln in MARKS[mark].strip().splitlines()]
    if len(rows) != GRID or any(len(r) != GRID for r in rows):
        raise ValueError(f"mark {mark!r} is not {GRID}x{GRID}")
    return [[ch == "#" for ch in row] for row in rows]


def rgba(color: str) -> tuple[int, int, int, int]:
    h = color.lstrip("#")
    if len(h) == 3:
        h = "".join(ch * 2 for ch in h)
    if len(h) == 6:
        h += "ff"
    if len(h) != 8:
        raise ValueError(f"colour must be #rgb, #rrggbb or #rrggbbaa, got {color!r}")
    return tuple(int(h[i : i + 2], 16) for i in (0, 2, 4, 6))


def png(pixels: list[list[tuple[int, int, int, int]]]) -> bytes:
    """Minimal RGBA PNG. No dependencies, so the mark builds anywhere."""
    height, width = len(pixels), len(pixels[0])
    raw = b"".join(
        b"\x00" + b"".join(struct.pack("BBBB", *px) for px in row) for row in pixels
    )

    def chunk(tag: bytes, body: bytes) -> bytes:
        return (
            struct.pack(">I", len(body))
            + tag
            + body
            + struct.pack(">I", zlib.crc32(tag + body) & 0xFFFFFFFF)
        )

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def tile_cell(row: int, col: int) -> bool:
    """A rounded square filling the grid: the corners are stepped, not curved,
    because a two-pixel step is what a rounded corner looks like at this size."""
    cut = (2, 1) + (0,) * (GRID - 4) + (1, 2)
    inset = cut[row]
    return inset <= col < GRID - inset


def render(
    mark: str, size: int, color: str, bg: str | None, pad: int, style: str, ink: str
) -> bytes:
    """Scales the grid to `size` by pixel replication, never interpolation.

    `pad` is in grid cells, so padding scales with the mark. A size that is
    not a whole multiple of the padded grid rounds down to one, keeping every
    pixel square; the caller gets the nearest exact size rather than a blur.

    In `tile` style the mark is punched out of a filled rounded square, which
    is what a launcher wants; `mark` style leaves it bare for the tray.
    """
    cells = GRID + pad * 2
    scale = max(1, size // cells)
    fg, back, knock = rgba(color), rgba(bg) if bg else (0, 0, 0, 0), rgba(ink)

    g = grid(mark)
    out = []
    for row in range(cells):
        line = []
        for col in range(cells):
            r, c = row - pad, col - pad
            inside = 0 <= r < GRID and 0 <= c < GRID
            on = inside and g[r][c]
            if style == "tile":
                px = knock if on else fg if inside and tile_cell(r, c) else back
            else:
                px = fg if on else back
            line.extend([px] * scale)
        out.extend([line] * scale)
    return png(out)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", help="PNG to write")
    ap.add_argument("--mark", default="handoff", choices=sorted(MARKS))
    ap.add_argument("--size", type=int, default=512, help="target edge in pixels")
    ap.add_argument("--color", default="#ff9e64", help="mark colour")
    ap.add_argument("--bg", default=None, help="background (default transparent)")
    ap.add_argument("--pad", type=int, default=0, help="padding in grid cells")
    ap.add_argument("--style", default="mark", choices=("mark", "tile"))
    ap.add_argument("--ink", default="#16181d", help="knockout colour for --style tile")
    ap.add_argument("--ascii", action="store_true", help="print the grid and exit")
    args = ap.parse_args()

    if args.ascii or not args.out:
        for row in grid(args.mark):
            print("".join("██" if on else "  " for on in row))
        return

    with open(args.out, "wb") as f:
        f.write(
            render(
                args.mark, args.size, args.color, args.bg, args.pad, args.style, args.ink
            )
        )


if __name__ == "__main__":
    main()
