#!/usr/bin/env python3
"""HP67 card artwork extractor v1.5\n\nExtract HP-67/HP-97 magnetic-card artwork strips from PAC PDFs.

Workflow:
  1. Scan a PDF for wide dark magnetic-card strips.
  2. Review the generated HTML/candidate PNGs.
  3. Extract high-resolution PNGs, optionally naming them from a built-in
     PAC profile or a one-reference-per-line text file.
  4. Normalize every PDF crop at native DPI onto one exact physical-card canvas:
     missing PDF area is filled black, oversize PDF area is center-cropped, and
     only the physical chamfered silhouette is transparent outside.
  5. Suppress line-like PDF edge artifacts and explicitly solid-fill compact
     white registration marks at the top edge.
  6. Downsample canonical cards into the 499x80 atlas with BOX area sampling so
     tiny filled marks survive reduction without Lanczos ringing/hollowing.
  7. Optionally build/validate the 499x80-per-row RGBA atlas used by hp67emu.

Dependencies:
    py -m pip install pymupdf pillow

Examples:
    py tools/extract_hp67_card_artwork.py scan hp97-pac-standard-en.pdf --out .artwork-scan
    py tools/extract_hp67_card_artwork.py extract hp97-pac-standard-en.pdf --manifest .artwork-scan/candidates.csv --out programs/HP67/_artwork --profile standard
    py tools/extract_hp67_card_artwork.py auto hp6797-pac-games-en.pdf --out programs/HP67/_artwork --profile games1 --expected 20
    py tools/extract_hp67_card_artwork.py atlas --artwork programs/HP67/_artwork --out assets/hp67-card-artwork-atlas.png --profile current
"""

from __future__ import annotations

import argparse
import csv
import html
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

try:
    import pymupdf as fitz
except ImportError as exc:  # pragma: no cover - dependency guidance
    raise SystemExit("Missing PyMuPDF. Install with: py -m pip install pymupdf pillow") from exc

try:
    from PIL import Image, ImageDraw
except ImportError as exc:  # pragma: no cover - dependency guidance
    raise SystemExit("Missing Pillow. Install with: py -m pip install pymupdf pillow") from exc


STANDARD_REFS = [
    "SD1-01A", "SD1-02A", "SD1-03A", "SD1-04A", "SD1-05A",
    "SD1-06A", "SD1-07A", "SD1-08A", "SD1-09A", "SD1-10A",
    "SD1-11B", "SD1-12A", "SD1-13A", "SD1-14A", "SD1-15A",
]

GAMES1_REFS = [
    "GA1-01A", "GA1-02A", "GA1-03A", "GA1-04A", "GA1-05A",
    "GA1-06A1", "GA1-06A2", "GA1-07A", "GA1-08A", "GA1-09A",
    "GA1-10A", "GA1-11A", "GA1-12A", "GA1-13A", "GA1-14A",
    "GA1-15A", "GA1-16A", "GA1-17A", "GA1-18A", "GA1-19A",
]

PROFILES = {
    "standard": STANDARD_REFS,
    "games1": GAMES1_REFS,
}
ATLAS_PROFILES = {
    **PROFILES,
    "current": STANDARD_REFS + GAMES1_REFS,
}

CARD_WIDTH_MM = 71.1
CARD_HEIGHT_MM = 11.4
CARD_END_CHAMFER_MM = 4.2
CARD_EDGE_ARTIFACT_ZONE_MM = 0.4
CARD_WHITE_MARK_MAX_MM = 1.5
CARD_WHITE_MARK_MAX_ASPECT = 1.8
ATLAS_ROW_HEIGHT = 80
ATLAS_WIDTH = round(ATLAS_ROW_HEIGHT * CARD_WIDTH_MM / CARD_HEIGHT_MM)


@dataclass(frozen=True)
class Candidate:
    index: int
    page: int  # 1-based
    slot: int  # 1-based within page, top to bottom
    rect: fitz.Rect  # PDF points
    reference: str = ""


def _runs(values: Iterable[int]) -> list[tuple[int, int]]:
    vals = list(values)
    if not vals:
        return []
    out: list[tuple[int, int]] = []
    start = prev = vals[0]
    for value in vals[1:]:
        if value <= prev + 1:
            prev = value
            continue
        out.append((start, prev))
        start = prev = value
    out.append((start, prev))
    return out


def detect_page_cards(
    page: fitz.Page,
    *,
    dpi: int,
    max_y_fraction: float,
    dark_threshold: int,
    min_row_fill: float,
    min_height_fraction: float,
    max_height_fraction: float,
    min_width_fraction: float,
    min_aspect: float,
) -> list[fitz.Rect]:
    """Return card-like black horizontal bands in PDF coordinates."""
    matrix = fitz.Matrix(dpi / 72.0, dpi / 72.0)
    pix = page.get_pixmap(matrix=matrix, colorspace=fitz.csGRAY, alpha=False)
    width, height = pix.width, pix.height
    max_y = max(1, min(height, int(height * max_y_fraction)))
    samples = memoryview(pix.samples)

    # Card faces in the PAC manuals are broad black strips. Count dark pixels
    # in each row without NumPy so the tool stays easy to install on Windows.
    row_counts: list[int] = []
    for y in range(max_y):
        base = y * width
        count = 0
        for x in range(width):
            if samples[base + x] < dark_threshold:
                count += 1
        row_counts.append(count)

    interesting_rows = [
        y for y, count in enumerate(row_counts) if count >= width * min_row_fill
    ]

    rects: list[fitz.Rect] = []
    for y0, y1 in _runs(interesting_rows):
        band_h = y1 - y0 + 1
        if band_h < height * min_height_fraction or band_h > height * max_height_fraction:
            continue

        x_min = width
        x_max = -1
        for y in range(y0, y1 + 1):
            base = y * width
            for x in range(width):
                if samples[base + x] < dark_threshold:
                    x_min = min(x_min, x)
                    x_max = max(x_max, x)
        if x_max < x_min:
            continue

        band_w = x_max - x_min + 1
        if band_w < width * min_width_fraction:
            continue
        if band_w / band_h < min_aspect:
            continue

        # Convert rendered pixel coordinates back to stable PDF-point coords.
        sx = page.rect.width / width
        sy = page.rect.height / height
        rects.append(
            fitz.Rect(
                page.rect.x0 + x_min * sx,
                page.rect.y0 + y0 * sy,
                page.rect.x0 + (x_max + 1) * sx,
                page.rect.y0 + (y1 + 1) * sy,
            )
        )

    rects.sort(key=lambda r: (r.y0, r.x0))
    return rects


def detect_document(pdf: Path, args: argparse.Namespace) -> list[Candidate]:
    doc = fitz.open(pdf)
    candidates: list[Candidate] = []
    index = 1
    for page_index, page in enumerate(doc):
        rects = detect_page_cards(
            page,
            dpi=args.scan_dpi,
            max_y_fraction=args.max_y,
            dark_threshold=args.dark,
            min_row_fill=args.min_row_fill,
            min_height_fraction=args.min_height,
            max_height_fraction=args.max_height,
            min_width_fraction=args.min_width,
            min_aspect=args.min_aspect,
        )
        for slot, rect in enumerate(rects, start=1):
            candidates.append(Candidate(index, page_index + 1, slot, rect))
            index += 1
    return candidates


def write_manifest(path: Path, candidates: list[Candidate]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["index", "page", "slot", "x0_pt", "y0_pt", "x1_pt", "y1_pt", "reference"])
        for c in candidates:
            writer.writerow([
                c.index,
                c.page,
                c.slot,
                f"{c.rect.x0:.3f}",
                f"{c.rect.y0:.3f}",
                f"{c.rect.x1:.3f}",
                f"{c.rect.y1:.3f}",
                c.reference,
            ])


def read_manifest(path: Path) -> list[Candidate]:
    out: list[Candidate] = []
    with path.open(newline="", encoding="utf-8-sig") as handle:
        for row in csv.DictReader(handle):
            out.append(
                Candidate(
                    int(row["index"]),
                    int(row["page"]),
                    int(row["slot"]),
                    fitz.Rect(
                        float(row["x0_pt"]),
                        float(row["y0_pt"]),
                        float(row["x1_pt"]),
                        float(row["y1_pt"]),
                    ),
                    (row.get("reference") or "").strip(),
                )
            )
    return out


def refs_from_args(args: argparse.Namespace, count: int) -> list[str] | None:
    if getattr(args, "profile", None):
        refs = list(PROFILES[args.profile])
    elif getattr(args, "references", None):
        refs = [
            line.strip()
            for line in Path(args.references).read_text(encoding="utf-8-sig").splitlines()
            if line.strip() and not line.lstrip().startswith("#")
        ]
    else:
        return None

    if len(refs) != count:
        raise SystemExit(f"Reference count mismatch: {len(refs)} names for {count} detected cards")
    if len(set(refs)) != len(refs):
        raise SystemExit("Duplicate references are not allowed")
    return refs


def add_references(candidates: list[Candidate], refs: list[str] | None) -> list[Candidate]:
    if refs is None:
        return candidates
    return [
        Candidate(c.index, c.page, c.slot, c.rect, refs[i])
        for i, c in enumerate(candidates)
    ]


def expanded_rect(rect: fitz.Rect, page_rect: fitz.Rect, margin_pt: float) -> fitz.Rect:
    r = fitz.Rect(rect.x0 - margin_pt, rect.y0 - margin_pt, rect.x1 + margin_pt, rect.y1 + margin_pt)
    return r & page_rect


def card_pixel_size(dpi: int) -> tuple[int, int]:
    return (
        round(CARD_WIDTH_MM * dpi / 25.4),
        round(CARD_HEIGHT_MM * dpi / 25.4),
    )


def card_silhouette_mask(size: tuple[int, int], dpi: int) -> Image.Image:
    width, height = size
    chamfer = max(1, round(CARD_END_CHAMFER_MM * dpi / 25.4))
    chamfer = min(chamfer, width // 4, height - 1)
    mask = Image.new("L", size, 0)
    ImageDraw.Draw(mask).polygon(
        [
            (chamfer, 0),
            (width - 1, 0),
            (width - 1, height - 1 - chamfer),
            (width - 1 - chamfer, height - 1),
            (0, height - 1),
            (0, chamfer),
        ],
        fill=255,
    )
    return mask


def composite_native_card(dst: Image.Image, src: Image.Image) -> None:
    """Anchor PDF artwork at the canonical top-left and crop only right/bottom excess.

    The PAC scans show that the stable physical registration is the top/left card
    edge while right/bottom extents vary by a few pixels. Keeping native DPI and
    anchoring those stable edges preserves the printed marks exactly; shorter
    source crops simply expose the black canonical substrate to the right/bottom.
    """
    width = min(src.width, dst.width)
    height = min(src.height, dst.height)
    if width <= 0 or height <= 0:
        raise RuntimeError(
            f"PDF crop {src.size} does not overlap canonical card canvas {dst.size}"
        )
    dst.alpha_composite(src.crop((0, 0, width, height)), (0, 0))


def suppress_boundary_white_artifacts(
    image: Image.Image,
    silhouette: Image.Image,
    dpi: int,
) -> None:
    """Paint PDF edge artifacts black without hollowing compact white top marks."""
    width, height = image.size
    pixels = image.load()
    silhouette_px = silhouette.load()

    guard = max(1, round(CARD_EDGE_ARTIFACT_ZONE_MM * dpi / 25.4))
    chamfer = max(1, round(CARD_END_CHAMFER_MM * dpi / 25.4))
    inner = Image.new("L", (width, height), 0)
    ImageDraw.Draw(inner).polygon(
        [
            (chamfer + guard, guard),
            (width - 1 - guard, guard),
            (width - 1 - guard, height - 1 - chamfer - guard),
            (width - 1 - chamfer - guard, height - 1 - guard),
            (guard, height - 1 - guard),
            (guard, chamfer + guard),
        ],
        fill=255,
    )
    inner_px = inner.load()

    white = bytearray(width * height)
    for y in range(height):
        for x in range(width):
            if silhouette_px[x, y] == 0:
                continue
            r, g, b, a = pixels[x, y]
            if a and r >= 150 and g >= 150 and b >= 150 and max(r, g, b) - min(r, g, b) <= 35:
                white[y * width + x] = 1

    seen = bytearray(width * height)
    neighbours = (
        (-1, -1), (0, -1), (1, -1),
        (-1, 0),            (1, 0),
        (-1, 1),  (0, 1),  (1, 1),
    )

    for y0 in range(height):
        for x0 in range(width):
            index = y0 * width + x0
            if not white[index] or seen[index]:
                continue

            stack = [(x0, y0)]
            seen[index] = 1
            component: list[tuple[int, int]] = []
            touches_boundary_zone = False
            min_x = max_x = x0
            min_y = max_y = y0

            while stack:
                x, y = stack.pop()
                component.append((x, y))
                min_x = min(min_x, x)
                max_x = max(max_x, x)
                min_y = min(min_y, y)
                max_y = max(max_y, y)
                if inner_px[x, y] == 0:
                    touches_boundary_zone = True

                for ox, oy in neighbours:
                    nx = x + ox
                    ny = y + oy
                    if not (0 <= nx < width and 0 <= ny < height):
                        continue
                    ni = ny * width + nx
                    if white[ni] and not seen[ni]:
                        seen[ni] = 1
                        stack.append((nx, ny))

            if not touches_boundary_zone:
                continue

            component_width = max_x - min_x + 1
            component_height = max_y - min_y + 1
            max_mark_px = max(2, round(CARD_WHITE_MARK_MAX_MM * dpi / 25.4))
            aspect = max(component_width, component_height) / max(
                1, min(component_width, component_height)
            )
            compact_mark = (
                component_width <= max_mark_px
                and component_height <= max_mark_px
                and aspect <= CARD_WHITE_MARK_MAX_ASPECT
            )
            if compact_mark:
                # These are the small white registration/index blocks printed
                # along the top of the HP card. Some PAC PDF rasterizations
                # contain an outlined/hollow-looking centre after antialiasing.
                # The source manuals show them as solid blocks, so normalize the
                # entire compact component bbox to opaque white.
                for y in range(min_y, max_y + 1):
                    for x in range(min_x, max_x + 1):
                        if silhouette_px[x, y] != 0:
                            pixels[x, y] = (255, 255, 255, 255)
                continue

            for x, y in component:
                pixels[x, y] = (0, 0, 0, 255)


def normalize_card_artwork(image: Image.Image, dpi: int) -> Image.Image:
    """Normalize a variable PDF card crop to the exact physical HP card canvas."""
    source = image.convert("RGBA")
    target_size = card_pixel_size(dpi)
    silhouette = card_silhouette_mask(target_size, dpi)

    # The PDF cards differ by a few pixels in width/height. Do not stretch each
    # one independently: preserve native DPI, anchor the stable top/left edges,
    # crop only right/bottom oversize, and fill missing card body black.
    normalized = Image.new("RGBA", target_size, (0, 0, 0, 255))
    normalized.putalpha(silhouette)
    composite_native_card(normalized, source)

    # Enforce the canonical silhouette after compositing page pixels, then turn
    # edge-only PDF whites/antialias lines back into the black card substrate.
    normalized.putalpha(silhouette)
    suppress_boundary_white_artifacts(normalized, silhouette, dpi)
    return normalized



def clean_card_png(path: Path, dpi: int) -> None:
    with Image.open(path) as source:
        cleaned = normalize_card_artwork(source, dpi)
    cleaned.save(path, format="PNG", optimize=False)

    # Decode again after writing.  This catches malformed PNG output at the
    # extraction stage rather than later inside the Rust application.
    with Image.open(path) as check:
        check.load()
        if check.mode != "RGBA":
            raise RuntimeError(f"Artwork must be RGBA after cleanup: {path} ({check.mode})")
        expected = card_pixel_size(dpi)
        if check.size != expected:
            raise RuntimeError(
                f"Extracted card has {check.size}; expected canonical {expected}: {path}"
            )


def render_candidate(page: fitz.Page, rect: fitz.Rect, out: Path, dpi: int, margin_pt: float) -> None:
    clip = expanded_rect(rect, page.rect, margin_pt)
    pix = page.get_pixmap(matrix=fitz.Matrix(dpi / 72.0, dpi / 72.0), clip=clip, alpha=False)
    out.parent.mkdir(parents=True, exist_ok=True)
    pix.save(out)
    clean_card_png(out, dpi)


def write_review_html(out_dir: Path, pdf: Path, candidates: list[Candidate]) -> None:
    rows = []
    for c in candidates:
        name = f"candidate-{c.index:03d}-p{c.page:03d}-{c.slot}.png"
        rows.append(
            f"<tr><td>{c.index}</td><td>{c.page}</td><td>{c.slot}</td>"
            f"<td><img src='{html.escape(name)}'></td><td>{html.escape(c.reference)}</td></tr>"
        )
    doc = f"""<!doctype html>
<meta charset='utf-8'>
<title>HP-67 card artwork scan - {html.escape(pdf.name)}</title>
<style>
body{{font-family:system-ui;background:#171816;color:#eee;margin:20px}}
table{{border-collapse:collapse}} td,th{{border:1px solid #555;padding:8px;vertical-align:middle}}
img{{max-width:720px;background:white}} code{{color:#f0c674}}
</style>
<h1>{html.escape(pdf.name)}</h1>
<p>Detected {len(candidates)} magnetic-card strips. Edit <code>candidates.csv</code> if you want to assign references manually.</p>
<table><tr><th>#</th><th>PDF page</th><th>slot</th><th>preview</th><th>reference</th></tr>
{''.join(rows)}</table>
"""
    (out_dir / "index.html").write_text(doc, encoding="utf-8")


def scan_command(args: argparse.Namespace) -> list[Candidate]:
    pdf = Path(args.pdf)
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    candidates = detect_document(pdf, args)
    if args.expected is not None and len(candidates) != args.expected:
        raise SystemExit(f"Detected {len(candidates)} cards, expected {args.expected}")
    refs = refs_from_args(args, len(candidates))
    candidates = add_references(candidates, refs)
    write_manifest(out_dir / "candidates.csv", candidates)

    doc = fitz.open(pdf)
    for c in candidates:
        render_candidate(
            doc[c.page - 1],
            c.rect,
            out_dir / f"candidate-{c.index:03d}-p{c.page:03d}-{c.slot}.png",
            args.preview_dpi,
            args.margin_pt,
        )
    write_review_html(out_dir, pdf, candidates)
    print(f"Detected {len(candidates)} cards")
    print(f"Manifest: {out_dir / 'candidates.csv'}")
    print(f"Review:   {out_dir / 'index.html'}")
    return candidates


def extract_from_candidates(args: argparse.Namespace, candidates: list[Candidate]) -> None:
    pdf = Path(args.pdf)
    refs = refs_from_args(args, len(candidates))
    if refs is not None:
        candidates = add_references(candidates, refs)
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    doc = fitz.open(pdf)

    for c in candidates:
        stem = c.reference or f"candidate-{c.index:03d}-p{c.page:03d}-{c.slot}"
        out = out_dir / f"{stem}.png"
        render_candidate(doc[c.page - 1], c.rect, out, args.dpi, args.margin_pt)
        print(f"{c.index:02d}: page {c.page:03d}/{c.slot} -> {out}")


def extract_command(args: argparse.Namespace) -> None:
    candidates = read_manifest(Path(args.manifest))
    if args.expected is not None and len(candidates) != args.expected:
        raise SystemExit(f"Manifest contains {len(candidates)} cards, expected {args.expected}")
    extract_from_candidates(args, candidates)


def auto_command(args: argparse.Namespace) -> None:
    out_dir = Path(args.out)
    scan_dir = Path(args.scan_out) if args.scan_out else out_dir / "_scan"
    scan_args = argparse.Namespace(**vars(args))
    scan_args.out = str(scan_dir)
    candidates = scan_command(scan_args)
    extract_from_candidates(args, candidates)


def atlas_command(args: argparse.Namespace) -> None:
    if args.profile:
        refs = list(ATLAS_PROFILES[args.profile])
    elif args.references:
        refs = [
            line.strip()
            for line in Path(args.references).read_text(encoding="utf-8-sig").splitlines()
            if line.strip() and not line.lstrip().startswith("#")
        ]
    elif args.manifest:
        candidates = read_manifest(Path(args.manifest))
        refs = [c.reference for c in candidates]
        if any(not ref for ref in refs):
            raise SystemExit("Manifest contains blank references; fill them or use --profile/--references")
    else:
        raise SystemExit("Atlas needs --profile, --references, or --manifest")

    if len(set(refs)) != len(refs):
        raise SystemExit("Duplicate references are not allowed in an atlas")

    art_dir = Path(args.artwork)
    rows: list[Image.Image] = []
    for reference in refs:
        path = art_dir / f"{reference}.png"
        if not path.is_file():
            raise SystemExit(f"Missing artwork: {path}")
        with Image.open(path) as src:
            src = src.convert("RGBA")
            # Extraction already normalized the exact physical card canvas.
            # Scale that canonical canvas directly to the canonical atlas row:
            # no per-card contain/letterboxing and therefore no variable border.
            # BOX is intentional: this is mostly high-contrast printed artwork,
            # and area sampling preserves tiny filled registration blocks better
            # than Lanczos, whose ringing can make them look hollow.
            rows.append(
                src.resize(
                    (args.width, args.row_height),
                    Image.Resampling.BOX,
                )
            )

    atlas = Image.new(
        "RGBA",
        (args.width, args.row_height * len(rows)),
        (0, 0, 0, 0),
    )
    for i, row in enumerate(rows):
        atlas.paste(row, (0, i * args.row_height))
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(out, format="PNG", optimize=False)

    with Image.open(out) as check:
        check.verify()
    with Image.open(out) as check:
        check.load()
        expected = (args.width, args.row_height * len(rows))
        if check.size != expected:
            raise SystemExit(f"Atlas validation failed: got {check.size}, expected {expected}")
        if check.mode != "RGBA":
            raise SystemExit(f"Atlas validation failed: expected RGBA, got {check.mode}")
        alpha_min, alpha_max = check.getchannel("A").getextrema()
        if alpha_min != 0 or alpha_max != 255:
            raise SystemExit(
                "Atlas validation failed: expected both transparent outside-card pixels "
                "and fully opaque artwork"
            )
    print(
        f"Valid RGBA atlas: {out} "
        f"({args.width}x{args.row_height * len(rows)}, {len(rows)} rows)"
    )


def add_detection_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--scan-dpi", type=int, default=100, help="DPI used only for auto-detection (default: 100)")
    parser.add_argument("--max-y", type=float, default=0.50, help="scan top fraction of each page (default: 0.50)")
    parser.add_argument("--dark", type=int, default=70, help="grayscale threshold 0..255 (default: 70)")
    parser.add_argument("--min-row-fill", type=float, default=0.23, help="minimum dark fraction of a row (default: 0.23)")
    parser.add_argument("--min-height", type=float, default=0.025, help="minimum card-band height/page height (default: 0.025)")
    parser.add_argument("--max-height", type=float, default=0.12, help="maximum card-band height/page height (default: 0.12)")
    parser.add_argument("--min-width", type=float, default=0.35, help="minimum card width/page width (default: 0.35)")
    parser.add_argument("--min-aspect", type=float, default=3.5, help="minimum width/height ratio (default: 3.5)")
    parser.add_argument("--expected", type=int, help="fail unless exactly this many cards are detected")
    parser.add_argument("--margin-pt", type=float, default=0.0, help="extra PDF-point margin around detected card (default: 0.0; physical silhouette masking removes page edge pixels)")
    naming = parser.add_mutually_exclusive_group()
    naming.add_argument("--profile", choices=sorted(PROFILES), help="built-in reference ordering")
    naming.add_argument("--references", help="text file: one card reference per line")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    scan = sub.add_parser("scan", help="detect card strips and create review previews/CSV")
    scan.add_argument("pdf")
    scan.add_argument("--out", required=True)
    scan.add_argument("--preview-dpi", type=int, default=220)
    add_detection_args(scan)
    scan.set_defaults(func=scan_command)

    extract = sub.add_parser("extract", help="extract high-resolution cards from a reviewed CSV manifest")
    extract.add_argument("pdf")
    extract.add_argument("--manifest", required=True)
    extract.add_argument("--out", required=True)
    extract.add_argument("--dpi", type=int, default=600)
    extract.add_argument("--margin-pt", type=float, default=0.0)
    extract.add_argument("--expected", type=int)
    naming = extract.add_mutually_exclusive_group()
    naming.add_argument("--profile", choices=sorted(PROFILES))
    naming.add_argument("--references")
    extract.set_defaults(func=extract_command)

    auto = sub.add_parser("auto", help="scan, review-output, and high-resolution extract in one pass")
    auto.add_argument("pdf")
    auto.add_argument("--out", required=True)
    auto.add_argument("--scan-out", help="scan/review directory (default: OUT/_scan)")
    auto.add_argument("--preview-dpi", type=int, default=220)
    auto.add_argument("--dpi", type=int, default=600)
    add_detection_args(auto)
    auto.set_defaults(func=auto_command)

    atlas = sub.add_parser("atlas", help="build and validate a fixed-row PNG atlas from extracted artwork")
    atlas.add_argument("--manifest", help="CSV manifest whose reference column defines atlas row order")
    atlas.add_argument("--artwork", required=True)
    atlas.add_argument("--out", required=True)
    atlas.add_argument("--width", type=int, default=ATLAS_WIDTH)
    atlas.add_argument("--row-height", type=int, default=ATLAS_ROW_HEIGHT)
    naming = atlas.add_mutually_exclusive_group()
    naming.add_argument("--profile", choices=sorted(ATLAS_PROFILES))
    naming.add_argument("--references")
    atlas.set_defaults(func=atlas_command)

    return parser


def main() -> int:
    args = build_parser().parse_args()
    args.func(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
