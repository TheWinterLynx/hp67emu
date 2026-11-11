"""Generate the full-resolution card PNG include map without altering source bytes."""

import argparse
import re
from pathlib import Path


def artwork_paths(root):
    source = (root / "src/program_library.rs").read_text(encoding="utf-8")
    paths = sorted(set(re.findall(r'artwork_path:\s*Some\(\s*"([^"]+)"\s*\)', source)))
    for path in paths:
        if (not path.startswith("assets/hp67-imported-artwork/")
                or not path.endswith(".png") or ".." in Path(path).parts):
            raise ValueError(f"Unexpected imported artwork path: {path}")
    return paths


def generate(root):
    paths = artwork_paths(root)
    rows = []
    for path in paths:
        relative = Path(path).relative_to("assets").as_posix()
        rows.append(f'        "{path}" => Some(include_bytes!("{relative}")),')
    text = ("// Full-resolution source PNGs, embedded once per distinct asset.\n"
            "pub fn embedded_card_artwork(path: &str) -> Option<&'static [u8]> {\n"
            "    match path {\n" + "\n".join(rows) + "\n        _ => None,\n    }\n}\n")
    return text, paths


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = args.repo.resolve()
    text, paths = generate(root)
    target = root / "assets/embedded-card-artwork.rs"
    if args.check:
        actual = re.findall(
            r'"([^"]+)"\s*=>\s*Some\(\s*include_bytes!\(\s*"([^"]+)"\s*\)\s*\)',
            target.read_text(encoding="utf-8"),
        )
        expected = [(path, Path(path).relative_to("assets").as_posix()) for path in paths]
        if actual != expected:
            raise ValueError("Embedded artwork map is stale; run tools/generate_embedded_card_artwork.py")
    else:
        target.write_text(text, encoding="utf-8")
    print(f"EMBEDDED HIRES ARTWORK OK: {len(paths)} distinct PNGs")


if __name__ == "__main__":
    main()
