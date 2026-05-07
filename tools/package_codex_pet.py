#!/usr/bin/env python3
"""Build and optionally install a Codex-compatible pet package."""

from __future__ import annotations

import argparse
import json
import os
import shutil
from pathlib import Path

from PIL import Image, ImageOps


ROOT = Path(__file__).resolve().parents[1]
SPRITES = ROOT / "assets" / "sprites"
OUT = ROOT / "assets" / "codex" / "open-pets-codex"

CELL = (192, 208)
ATLAS = (1536, 1872)
ROWS = [
    ("idle", "idle", 6, False),
    ("running-right", "running", 8, False),
    ("running-left", "running", 8, True),
    ("waving", "happy", 4, False),
    ("jumping", "happy", 5, False),
    ("failed", "grumpy", 8, False),
    ("waiting", "waiting", 6, False),
    ("running", "running", 6, False),
    ("review", "thinking", 6, False),
]


def codex_home() -> Path:
    return Path(os.environ.get("CODEX_HOME") or "~/.codex").expanduser().resolve()


def read_frame(species: str, state: str, index: int, mirror: bool) -> Image.Image:
    paths = sorted((SPRITES / state).glob(f"{species}-*.png"))
    if not paths:
        raise SystemExit(f"missing source frames for species={species} state={state}")
    frame = Image.open(paths[index % len(paths)]).convert("RGBA")
    if mirror:
        frame = ImageOps.mirror(frame)
    return frame


def frame_to_cell(frame: Image.Image) -> Image.Image:
    cell = Image.new("RGBA", CELL, (0, 0, 0, 0))
    scaled = frame.resize((192, 192), Image.Resampling.NEAREST)
    cell.alpha_composite(scaled, (0, 8))
    return cell


def compose_atlas(species: str) -> Image.Image:
    atlas = Image.new("RGBA", ATLAS, (0, 0, 0, 0))
    for row_index, (_codex_state, source_state, frame_count, mirror) in enumerate(ROWS):
        for col in range(frame_count):
            source = read_frame(species, source_state, col, mirror)
            cell = frame_to_cell(source)
            atlas.alpha_composite(cell, (col * CELL[0], row_index * CELL[1]))
    return atlas


def validate_atlas(atlas: Image.Image) -> None:
    if atlas.size != ATLAS:
        raise SystemExit(f"expected atlas size {ATLAS}, got {atlas.size}")

    for row_index, (_codex_state, _source_state, frame_count, _mirror) in enumerate(ROWS):
        for col in range(8):
            cell = atlas.crop(
                (
                    col * CELL[0],
                    row_index * CELL[1],
                    (col + 1) * CELL[0],
                    (row_index + 1) * CELL[1],
                )
            )
            alpha_bbox = cell.getchannel("A").getbbox()
            if col < frame_count and alpha_bbox is None:
                raise SystemExit(f"used cell is empty: row={row_index} col={col}")
            if col >= frame_count and alpha_bbox is not None:
                raise SystemExit(f"unused cell is not transparent: row={row_index} col={col}")


def write_package(species: str, output_dir: Path, install: bool) -> dict[str, str]:
    output_dir.mkdir(parents=True, exist_ok=True)
    atlas = compose_atlas(species)
    validate_atlas(atlas)

    png_path = output_dir / "spritesheet.png"
    webp_path = output_dir / "spritesheet.webp"
    manifest_path = output_dir / "pet.json"

    atlas.save(png_path)
    atlas.save(webp_path, format="WEBP", lossless=True, quality=100, method=6)
    manifest = {
        "id": "open-pets-codex",
        "displayName": "Open Pets Codex",
        "description": "A Codex-compatible open-pets companion.",
        "spritesheetPath": "spritesheet.webp",
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    install_dir = codex_home() / "pets" / "open-pets-codex"
    if install:
        install_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(manifest_path, install_dir / "pet.json")
        shutil.copy2(webp_path, install_dir / "spritesheet.webp")

    return {
        "package_dir": str(output_dir),
        "manifest": str(manifest_path),
        "spritesheet": str(webp_path),
        "installed_dir": str(install_dir) if install else "",
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--species", default="void-cat")
    parser.add_argument("--output-dir", default=str(OUT))
    parser.add_argument("--install", action="store_true")
    args = parser.parse_args()

    result = write_package(args.species, Path(args.output_dir).expanduser().resolve(), args.install)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
