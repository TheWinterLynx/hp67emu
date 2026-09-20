#!/usr/bin/env python3
"""HP67 card artwork extractor v1.3\n\nExtract HP-67/HP-97 magnetic-card artwork strips from PAC PDFs.

Workflow:
  1. Scan a PDF for wide dark magnetic-card strips.
  2. Review the generated HTML/candidate PNGs.
  3. Extract high-resolution PNGs, optionally naming them from a built-in
     PAC profile or a one-reference-per-line text file.
  4. Apply the physical HP card silhouette as an alpha mask. This removes PDF
     page-white/antialias edge pixels without flood-filling legitimate white
     card markings that touch the top edge.
  5. Optionally build/validate the 480x80-per-row RGBA atlas used by hp67emu.

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
    from PIL import Image, ImageChops, ImageDraw, ImageOps
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
CARD_EDGE_INSET_WIDTH_FRACTION = 1.0 / 700.0
CARD_TRANSPARENT_PADDING_WIDTH_FRACTION = 1.0 / 700.0


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


def apply_card_silhouette(image: Image.Image) -> Image.Image:
    """Clip a PDF crop to the physical HP magnetic-card silhouette.

    A connectivity flood-fill is intentionally not used here. The PAC artwork
    contains legitimate white registration/top marks that are open to the top
    edge of the black card; flood-filling page white therefore hollows those
    marks out. The physical card geometry is known independently by the
    emulator, so use the same chamfer dimensions as a deterministic alpha mask.

    A tiny inward mask offset removes the one-pixel white antialias fringe left
    by rasterising the black card against the white PDF page. A transparent
    black padding ring is added afterwards so linear GPU sampling cannot pull
    pale RGB values back onto the edge.
    """
    rgba = image.convert("RGBA")
    width, height = rgba.size
    if width < 100 or height < 20:
        raise RuntimeError(f"Suspiciously small card crop: {rgba.size}")

    px_per_mm_x = width / CARD_WIDTH_MM
    px_per_mm_y = height / CARD_HEIGHT_MM
    px_per_mm = min(px_per_mm_x, px_per_mm_y)
    chamfer = max(1, round(CARD_END_CHAMFER_MM * px_per_mm))
    inset = max(1, round(width * CARD_EDGE_INSET_WIDTH_FRACTION))

    left = inset
    top = inset
    right = width - 1 - inset
    bottom = height - 1 - inset
    if right <= left or bottom <= top or chamfer * 2 >= min(width, height):
        raise RuntimeError(f"Invalid card mask geometry for crop {rgba.size}")

    mask = Image.new("L", (width, height), 0)
    draw = ImageDraw.Draw(mask)
    draw.polygon(
        [
            (left + chamfer, top),
            (right, top),
            (right, bottom - chamfer),
            (right - chamfer, bottom),
            (left, bottom),
            (left, top + chamfer),
        ],
        fill=255,
    )

    source_alpha = rgba.getchannel("A")
    rgba.putalpha(ImageChops.multiply(source_alpha, mask))

    bbox = rgba.getchannel("A").getbbox()
    if bbox is None:
        raise RuntimeError("Artwork silhouette removed the complete crop")
    rgba = rgba.crop(bbox)

    padding = max(1, round(width * CARD_TRANSPARENT_PADDING_WIDTH_FRACTION))
    padded = Image.new(
        "RGBA",
        (rgba.width + 2 * padding, rgba.height + 2 * padding),
        (0, 0, 0, 0),
    )
    padded.alpha_composite(rgba, (padding, padding))
    return padded



def clean_card_png(path: Path) -> None:
    with Image.open(path) as source:
        cleaned = apply_card_silhouette(source)
    cleaned.save(path, format="PNG", optimize=False)

    # Decode again after writing.  This catches malformed PNG output at the
    # extraction stage rather than later inside the Rust application.
    with Image.open(path) as check:
        check.load()
        if check.mode != "RGBA":
            raise RuntimeError(f"Artwork must be RGBA after cleanup: {path} ({check.mode})")
        if check.width < 100 or check.height < 20:
            raise RuntimeError(f"Suspiciously small extracted card {path}: {check.size}")


def render_candidate(page: fitz.Page, rect: fitz.Rect, out: Path, dpi: int, margin_pt: float) -> None:
    clip = expanded_rect(rect, page.rect, margin_pt)
    pix = page.get_pixmap(matrix=fitz.Matrix(dpi / 72.0, dpi / 72.0), clip=clip, alpha=False)
    out.parent.mkdir(parents=True, exist_ok=True)
    pix.save(out)
    clean_card_png(out)


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
            # The extracted PNG has already been silhouette-masked. Never
            # flood-fill or remask it here: top-edge white card markings are
            # legitimate artwork and must stay filled.
            # Fit inside the atlas cell without distorting the physical card.
            # Any unused area stays transparent black so LINEAR sampling in the
            # emulator cannot manufacture a white halo around the card.
            fitted = ImageOps.contain(src, (args.width, args.row_height), Image.Resampling.LANCZOS)
            cell = Image.new("RGBA", (args.width, args.row_height), (0, 0, 0, 0))
            x = (args.width - fitted.width) // 2
            y = (args.row_height - fitted.height) // 2
            cell.alpha_composite(fitted, (x, y))
            rows.append(cell)

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
    atlas.add_argument("--width", type=int, default=480)
    atlas.add_argument("--row-height", type=int, default=80)
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
