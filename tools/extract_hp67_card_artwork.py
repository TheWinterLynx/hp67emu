#!/usr/bin/env python3
"""Extract HP-67 program-card face artwork from checked-in pack PDFs.

Requires PyMuPDF:
    python -m pip install pymupdf

The extractor searches each program title in the searchable scan, prefers the
second title hit on the first matching page (the first is usually the section
heading), and crops a 71.1 x 11.4 physical-card aspect rectangle around it.
Optional normalized crop overrides can be stored in:
    programs/HP67/_artwork/overrides.json

Override format:
{
  "SD1-14A": {"page": 122, "crop": [0.06, 0.08, 0.94, 0.2211]}
}
Page is zero-based. Crop coordinates are fractions of page width/height.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

try:
    import fitz  # type: ignore
except ImportError as exc:
    raise SystemExit(
        "PyMuPDF is required. Install it with: python -m pip install pymupdf"
    ) from exc

CARD_ASPECT = 71.1 / 11.4
DEFAULT_WIDTH_FRACTION = 0.88
TITLE_Y_FRACTION_ON_CARD = 0.31

PACKS = (
    ("HP-67 Standard Pac", "hp67-pac-standard-en.pdf"),
    ("HP-67 Games Pac 1", "hp6797-pac-games-en.pdf"),
)

FILE_RE = re.compile(r"^(?P<ref>[A-Z0-9-]+)_[12]\s+(?P<title>.+)\.hpp$", re.I)


@dataclass(frozen=True)
class Program:
    reference: str
    title: str
    pdf_path: Path


def normalize(text: str) -> str:
    return re.sub(r"[^A-Z0-9]+", "", text.upper())


def discover_programs(root: Path) -> list[Program]:
    programs: dict[str, Program] = {}
    for pack_dir_name, pdf_name in PACKS:
        pack_dir = root / pack_dir_name
        pdf_path = pack_dir / pdf_name
        if not pdf_path.is_file():
            continue
        for path in sorted(pack_dir.glob("*.hpp")):
            match = FILE_RE.match(path.name)
            if not match:
                continue
            reference = match.group("ref").upper()
            programs.setdefault(
                reference,
                Program(reference, match.group("title"), pdf_path),
            )
    return list(programs.values())


def load_overrides(path: Path) -> dict[str, dict[str, object]]:
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def find_page(document: fitz.Document, title: str) -> tuple[int, fitz.Page, list[fitz.Rect]]:
    exact_title = title.strip()
    normalized_title = normalize(exact_title)

    for page_index in range(document.page_count):
        page = document.load_page(page_index)
        hits = page.search_for(exact_title)
        if hits:
            return page_index, page, hits

        if normalized_title and normalized_title in normalize(page.get_text("text")):
            return page_index, page, []

    raise LookupError(f"title not found in PDF: {title}")


def crop_from_title(page: fitz.Page, hits: list[fitz.Rect]) -> fitz.Rect:
    page_rect = page.rect
    width = page_rect.width * DEFAULT_WIDTH_FRACTION
    height = width / CARD_ASPECT
    left = page_rect.x0 + (page_rect.width - width) * 0.5

    if hits:
        hit = hits[1] if len(hits) > 1 else hits[0]
        title_y = (hit.y0 + hit.y1) * 0.5
        top = title_y - height * TITLE_Y_FRACTION_ON_CARD
    else:
        top = page_rect.y0 + page_rect.height * 0.08

    top = max(page_rect.y0, min(top, page_rect.y1 - height))
    return fitz.Rect(left, top, left + width, top + height)


def override_crop(page: fitz.Page, values: list[float]) -> fitz.Rect:
    if len(values) != 4:
        raise ValueError("override crop must contain four normalized coordinates")
    x0, y0, x1, y1 = values
    rect = page.rect
    return fitz.Rect(
        rect.x0 + rect.width * x0,
        rect.y0 + rect.height * y0,
        rect.x0 + rect.width * x1,
        rect.y0 + rect.height * y1,
    )


def extract(program: Program, output_dir: Path, dpi: int, overrides: dict[str, dict[str, object]], force: bool) -> str:
    output = output_dir / f"{program.reference}.png"
    if output.exists() and not force:
        return f"SKIP {program.reference}: {output}"

    document = fitz.open(program.pdf_path)
    override = overrides.get(program.reference)

    if override is not None:
        page_index = int(override["page"])
        page = document.load_page(page_index)
        crop = override_crop(page, [float(value) for value in override["crop"]])
    else:
        page_index, page, hits = find_page(document, program.title)
        crop = crop_from_title(page, hits)

    scale = dpi / 72.0
    pixmap = page.get_pixmap(matrix=fitz.Matrix(scale, scale), clip=crop, alpha=False)
    output_dir.mkdir(parents=True, exist_ok=True)
    pixmap.save(output)
    return (
        f"OK   {program.reference}: page {page_index + 1}, "
        f"{pixmap.width}x{pixmap.height} -> {output}"
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        type=Path,
        default=Path("programs/HP67"),
        help="HP-67 programs root (default: programs/HP67)",
    )
    parser.add_argument("--dpi", type=int, default=400)
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()

    output_dir = args.root / "_artwork"
    overrides = load_overrides(output_dir / "overrides.json")
    programs = discover_programs(args.root)
    if not programs:
        print("No Standard/Games Pac programs with checked-in PDFs were found.", file=sys.stderr)
        return 2

    failures = 0
    for program in programs:
        try:
            print(extract(program, output_dir, args.dpi, overrides, args.force))
        except Exception as error:
            failures += 1
            print(f"FAIL {program.reference}: {error}", file=sys.stderr)

    print(f"Processed {len(programs)} programs; failures={failures}")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
