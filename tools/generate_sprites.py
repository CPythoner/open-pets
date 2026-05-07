#!/usr/bin/env python3
"""Generate bundled pixel sprites for open-pets.

The output is a deterministic set of transparent PNG frames under
assets/sprites/<state>/<species-id>-<frame>.png.
"""

from __future__ import annotations

import os
import struct
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "sprites"
SIZE = 96
SCALE = 6
GRID = SIZE // SCALE

STATES = ("idle", "running", "waiting", "thinking", "happy", "grumpy", "sleeping")

SPECIES = {
    "void-cat": {"body": (32, 30, 48, 255), "accent": (138, 92, 246, 255), "shape": "cat"},
    "code-hound": {"body": (128, 89, 56, 255), "accent": (244, 190, 120, 255), "shape": "hound"},
    "terminal-turtle": {"body": (52, 112, 82, 255), "accent": (96, 186, 132, 255), "shape": "turtle"},
    "pixel-parrot": {"body": (34, 139, 136, 255), "accent": (244, 91, 91, 255), "shape": "parrot"},
    "debug-dragon": {"body": (50, 103, 162, 255), "accent": (247, 178, 71, 255), "shape": "dragon"},
    "rust-fox": {"body": (202, 92, 39, 255), "accent": (247, 218, 159, 255), "shape": "fox"},
    "schema-spider": {"body": (62, 58, 76, 255), "accent": (99, 210, 190, 255), "shape": "spider"},
    "cache-crow": {"body": (26, 33, 43, 255), "accent": (91, 141, 239, 255), "shape": "bird"},
    "null-pointer-neko": {"body": (214, 186, 202, 255), "accent": (83, 75, 120, 255), "shape": "cat"},
    "lambda-lizard": {"body": (93, 150, 73, 255), "accent": (220, 245, 124, 255), "shape": "lizard"},
    "recursion-raccoon": {"body": (120, 124, 132, 255), "accent": (42, 45, 52, 255), "shape": "raccoon"},
    "stack-overflow-owl": {"body": (128, 92, 55, 255), "accent": (245, 177, 66, 255), "shape": "owl"},
    "memory-leak-kraken": {"body": (116, 70, 143, 255), "accent": (244, 116, 161, 255), "shape": "kraken"},
    "race-condition-chimera": {"body": (82, 94, 171, 255), "accent": (105, 220, 165, 255), "shape": "chimera"},
}


def write_png(path: Path, pixels: list[list[tuple[int, int, int, int]]]) -> None:
    raw = bytearray()
    for row in pixels:
        raw.append(0)
        for r, g, b, a in row:
            raw.extend((r, g, b, a))

    def chunk(kind: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + kind
            + data
            + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
        )

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", SIZE, SIZE, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def blank() -> list[list[tuple[int, int, int, int]]]:
    return [[(0, 0, 0, 0) for _ in range(SIZE)] for _ in range(SIZE)]


def rect(img, x, y, w, h, color):
    for gy in range(y, y + h):
        for gx in range(x, x + w):
            fill_cell(img, gx, gy, color)


def fill_cell(img, gx, gy, color):
    if gx < 0 or gy < 0 or gx >= GRID or gy >= GRID:
        return
    for py in range(gy * SCALE, (gy + 1) * SCALE):
        for px in range(gx * SCALE, (gx + 1) * SCALE):
            img[py][px] = color


def dot(img, gx, gy, color):
    fill_cell(img, gx, gy, color)


def state_offsets(state: str, frame: int) -> tuple[int, int]:
    bob = {
        "idle": (0, frame % 2),
        "running": ((frame * 2) - 1, 0),
        "waiting": (0, 0),
        "thinking": (0, -1 if frame else 0),
        "happy": (0, -1 if frame else 0),
        "grumpy": (-1 if frame else 1, 0),
        "sleeping": (0, 1),
    }
    return bob[state]


def expression(state: str):
    if state == "happy":
        return (">", "<", "smile")
    if state == "grumpy":
        return ("-", "-", "flat")
    if state == "sleeping":
        return ("z", "z", "flat")
    if state == "thinking":
        return ("o", "o", "small")
    return ("o", "o", "smile")


def draw_face(img, x, y, state, dark):
    left, right, mouth = expression(state)
    if left == "z":
        dot(img, x + 4, y + 5, dark)
        dot(img, x + 8, y + 5, dark)
    elif left == "-":
        rect(img, x + 3, y + 5, 2, 1, dark)
        rect(img, x + 8, y + 5, 2, 1, dark)
    elif left == ">":
        dot(img, x + 3, y + 5, dark)
        dot(img, x + 4, y + 6, dark)
        dot(img, x + 9, y + 5, dark)
        dot(img, x + 8, y + 6, dark)
    else:
        dot(img, x + 4, y + 5, dark)
        dot(img, x + 8, y + 5, dark)

    if mouth == "smile":
        dot(img, x + 6, y + 8, dark)
        dot(img, x + 7, y + 8, dark)
        dot(img, x + 5, y + 7, dark)
        dot(img, x + 8, y + 7, dark)
    elif mouth == "small":
        dot(img, x + 6, y + 8, dark)
    else:
        rect(img, x + 5, y + 8, 4, 1, dark)


def draw_base(img, shape, x, y, body, accent, state):
    dark = (24, 26, 34, 255)
    light = (245, 245, 235, 255)

    if shape in {"cat", "fox"}:
        rect(img, x + 3, y + 4, 8, 7, body)
        dot(img, x + 3, y + 3, body)
        dot(img, x + 10, y + 3, body)
        rect(img, x + 5, y + 9, 4, 2, accent)
        if shape == "fox":
            rect(img, x + 10, y + 8, 3, 2, body)
            dot(img, x + 12, y + 7, accent)
    elif shape == "hound":
        rect(img, x + 3, y + 5, 8, 6, body)
        rect(img, x + 2, y + 5, 2, 3, accent)
        rect(img, x + 10, y + 6, 3, 2, body)
    elif shape == "turtle":
        rect(img, x + 3, y + 6, 9, 5, body)
        rect(img, x + 5, y + 5, 5, 5, accent)
        rect(img, x + 11, y + 7, 2, 2, body)
    elif shape in {"parrot", "bird"}:
        rect(img, x + 4, y + 5, 7, 6, body)
        rect(img, x + 2, y + 6, 3, 3, accent)
        dot(img, x + 10, y + 6, accent)
    elif shape == "dragon":
        rect(img, x + 3, y + 5, 8, 6, body)
        dot(img, x + 4, y + 4, accent)
        dot(img, x + 9, y + 4, accent)
        rect(img, x + 10, y + 8, 3, 2, body)
    elif shape == "spider":
        rect(img, x + 4, y + 6, 6, 4, body)
        for leg_y in (6, 8):
            rect(img, x + 1, y + leg_y, 3, 1, accent)
            rect(img, x + 10, y + leg_y, 3, 1, accent)
    elif shape == "lizard":
        rect(img, x + 3, y + 6, 8, 4, body)
        rect(img, x + 10, y + 7, 3, 1, accent)
        rect(img, x + 1, y + 8, 3, 1, body)
    elif shape == "raccoon":
        rect(img, x + 3, y + 5, 8, 6, body)
        rect(img, x + 3, y + 6, 8, 2, accent)
        rect(img, x + 10, y + 9, 3, 1, body)
        dot(img, x + 12, y + 8, accent)
    elif shape == "owl":
        rect(img, x + 3, y + 4, 8, 7, body)
        dot(img, x + 3, y + 3, accent)
        dot(img, x + 10, y + 3, accent)
        dot(img, x + 4, y + 6, light)
        dot(img, x + 8, y + 6, light)
    elif shape == "kraken":
        rect(img, x + 4, y + 4, 7, 6, body)
        for tx in (3, 5, 7, 9, 11):
            rect(img, x + tx, y + 10, 1, 2, accent)
    elif shape == "chimera":
        rect(img, x + 3, y + 5, 8, 6, body)
        dot(img, x + 3, y + 4, accent)
        dot(img, x + 10, y + 4, accent)
        rect(img, x + 10, y + 8, 3, 2, accent)

    draw_face(img, x, y, state, dark)

    if state == "thinking":
        dot(img, x + 12, y + 1, accent)
        dot(img, x + 13, y, accent)
    elif state == "sleeping":
        dot(img, x + 11, y + 2, light)
        dot(img, x + 12, y + 1, light)
        dot(img, x + 13, y, light)


def render_sprite(species_id: str, state: str, frame: int):
    spec = SPECIES[species_id]
    img = blank()
    ox, oy = state_offsets(state, frame)
    x = 1 + ox
    y = 2 + oy
    draw_base(img, spec["shape"], x, y, spec["body"], spec["accent"], state)
    return img


def main() -> None:
    for state in STATES:
        state_dir = OUT / state
        state_dir.mkdir(parents=True, exist_ok=True)
        for old in state_dir.glob("*.png"):
            old.unlink()
        for species_id in SPECIES:
            for frame in range(2):
                write_png(state_dir / f"{species_id}-{frame}.png", render_sprite(species_id, state, frame))

    count = sum(1 for _ in OUT.glob("*/*.png"))
    print(f"generated {count} sprites in {OUT}")


if __name__ == "__main__":
    main()
