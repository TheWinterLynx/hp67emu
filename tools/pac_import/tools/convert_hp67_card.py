#!/usr/bin/env python3
"""Convert HP-67 magnetic-card compatibility and native formats.

Canonical project media is .hp67card. Teenix .hpp support exists only for
import/export compatibility. This tool never changes the 34 x 28-bit logical
records when converting between HPP and HP67CARD.

Examples:
  py tools/convert_hp67_card.py hpp-to-card side1.hpp side2.hpp -o card.hp67card
  py tools/convert_hp67_card.py hpp-to-card --track1 a1.hpp --track2 a2.hpp -o card.hp67card
  py tools/convert_hp67_card.py card-to-hpp card.hp67card --out-dir out --stem card
  py tools/convert_hp67_card.py listing-to-card --track1 listing.txt --header1 0x03100222 -o card.hp67card
  py tools/convert_hp67_card.py inspect card.hp67card
  py tools/convert_hp67_card.py self-test
"""

from __future__ import annotations

import argparse
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

MAGIC = b"HP67CARD"
VERSION = 1
RECORDS = 34
WORD_BITS = 28
WORD_MASK = (1 << WORD_BITS) - 1
TRACK_BITS = RECORDS * WORD_BITS
TRACK_BYTES = TRACK_BITS // 8
CONTAINER_BYTES = 12 + 2 * TRACK_BYTES
TRACK_RECORDED = 0x01
TRACK_WRITE_PROTECTED = 0x02
TRACK_FLAG_MASK = TRACK_RECORDED | TRACK_WRITE_PROTECTED
PROGRAM_STEPS = 112
HPP_DUMMY_NIBBLES = 21
HPP_REAL_NIBBLES = RECORDS * 7
HPP_CHECKSUM_MIRROR_NIBBLES = 7
HPP_TOTAL_NIBBLES = (
    HPP_DUMMY_NIBBLES + HPP_REAL_NIBBLES + HPP_CHECKSUM_MIRROR_NIBBLES
)


@dataclass(frozen=True)
class HppTrack:
    calculator_id: int
    bitmap_name: str
    card_name: str
    header_id: int
    inferred_track: int
    words: tuple[int, ...]
    declared_body_len: int
    actual_body_len: int


@dataclass(frozen=True)
class NativeTrack:
    recorded: bool
    write_protected: bool
    words: tuple[int, ...] | None


def split_line(raw: bytes, field: str) -> tuple[bytes, bytes]:
    cr = raw.find(b"\r")
    lf = raw.find(b"\n")
    indexes = [value for value in (cr, lf) if value >= 0]
    if not indexes:
        raise ValueError(f"HPP is missing {field}")
    index = min(indexes)
    separator = 2 if raw[index:index + 2] == b"\r\n" else 1
    return raw[:index], raw[index + separator:]


def parse_hpp(path: Path) -> HppTrack:
    encoded = path.read_bytes()
    decoded = bytes(byte ^ 0x55 for byte in encoded)

    magic, rest = split_line(decoded, "magic")
    if magic.strip() != b"NeWe":
        raise ValueError(f"{path}: invalid HPP magic {magic!r}")

    length_line, body = split_line(rest, "declared body length")
    try:
        declared = int(length_line.strip())
    except ValueError as exc:
        raise ValueError(f"{path}: invalid HPP body length {length_line!r}") from exc

    calculator_line, rest = split_line(body, "calculator id")
    bitmap_line, rest = split_line(rest, "bitmap name")
    card_name_line, card_data = split_line(rest, "card name")

    try:
        calculator = int(calculator_line.strip())
    except ValueError as exc:
        raise ValueError(f"{path}: invalid calculator id") from exc
    if calculator not in (67, 97):
        raise ValueError(f"{path}: unsupported calculator id {calculator}")

    bitmap = bitmap_line.decode("utf-8").strip()
    card_name = card_name_line.decode("utf-8").strip()
    text = card_data.decode("ascii").strip()
    fields = text.split()
    if len(fields) > 1:
        try:
            nibbles = [int(field, 10) for field in fields]
        except ValueError as exc:
            raise ValueError(f"{path}: invalid decimal nibble token") from exc
    else:
        compact = fields[0] if fields else ""
        try:
            nibbles = [int(ch, 16) for ch in compact]
        except ValueError as exc:
            raise ValueError(f"{path}: invalid hexadecimal nibble data") from exc

    if len(nibbles) != HPP_TOTAL_NIBBLES:
        raise ValueError(
            f"{path}: expected {HPP_TOTAL_NIBBLES} HPP nibbles, got {len(nibbles)}"
        )
    if any(value < 0 or value > 0x0F for value in nibbles):
        raise ValueError(f"{path}: HPP nibble outside 0..15")

    real = nibbles[HPP_DUMMY_NIBBLES:HPP_DUMMY_NIBBLES + HPP_REAL_NIBBLES]
    mirror = nibbles[-HPP_CHECKSUM_MIRROR_NIBBLES:]
    if real[-7:] != mirror:
        raise ValueError(f"{path}: HPP duplicate checksum record does not match record 34")

    words: list[int] = []
    for record in range(RECORDS):
        word = 0
        for nibble in range(7):
            word |= real[record * 7 + nibble] << (nibble * 4)
        words.append(word)

    header = (words[0] >> 24) & 0x0F
    if header in (1, 3):
        inferred_track = 1
    elif header in (2, 4):
        inferred_track = 2
    else:
        raise ValueError(f"{path}: unsupported Teenix header id {header}")

    return HppTrack(
        calculator_id=calculator,
        bitmap_name=bitmap,
        card_name=card_name,
        header_id=header,
        inferred_track=inferred_track,
        words=tuple(words),
        declared_body_len=declared,
        actual_body_len=len(body),
    )


def pack_track(words: tuple[int, ...] | list[int]) -> bytes:
    if len(words) != RECORDS:
        raise ValueError(f"expected {RECORDS} records, got {len(words)}")
    out = bytearray(TRACK_BYTES)
    bit_index = 0
    for index, word in enumerate(words):
        if word < 0 or word > WORD_MASK:
            raise ValueError(f"record {index + 1} exceeds 28 bits: 0x{word:x}")
        for bit in range(27, -1, -1):
            if word & (1 << bit):
                out[bit_index // 8] |= 1 << (7 - bit_index % 8)
            bit_index += 1
    assert bit_index == TRACK_BITS
    return bytes(out)


def unpack_track(payload: bytes) -> tuple[int, ...]:
    if len(payload) != TRACK_BYTES:
        raise ValueError(f"expected {TRACK_BYTES} track bytes, got {len(payload)}")
    words: list[int] = []
    bit_index = 0
    for _ in range(RECORDS):
        word = 0
        for bit in range(27, -1, -1):
            if payload[bit_index // 8] & (1 << (7 - bit_index % 8)):
                word |= 1 << bit
            bit_index += 1
        words.append(word)
    assert bit_index == TRACK_BITS
    return tuple(words)


def parse_native(path: Path) -> tuple[NativeTrack, NativeTrack]:
    raw = path.read_bytes()
    if len(raw) != CONTAINER_BYTES:
        raise ValueError(
            f"{path}: expected {CONTAINER_BYTES} bytes, got {len(raw)}"
        )
    if raw[:8] != MAGIC:
        raise ValueError(f"{path}: invalid HP67CARD magic")
    if raw[8] != VERSION:
        raise ValueError(f"{path}: unsupported HP67CARD version {raw[8]}")
    if raw[11] != 0:
        raise ValueError(f"{path}: reserved byte 11 must be zero")

    result: list[NativeTrack] = []
    for track_index, flag_offset, payload_offset in ((1, 9, 12), (2, 10, 131)):
        flags = raw[flag_offset]
        if flags & ~TRACK_FLAG_MASK:
            raise ValueError(
                f"{path}: Track {track_index} has invalid flags 0x{flags:02x}"
            )
        recorded = bool(flags & TRACK_RECORDED)
        protected = bool(flags & TRACK_WRITE_PROTECTED)
        payload = raw[payload_offset:payload_offset + TRACK_BYTES]
        words = unpack_track(payload) if recorded else None
        result.append(NativeTrack(recorded, protected, words))
    return result[0], result[1]


def native_bytes(
    track1: tuple[int, ...] | None,
    track2: tuple[int, ...] | None,
    *,
    protect1: bool = False,
    protect2: bool = False,
) -> bytes:
    out = bytearray(CONTAINER_BYTES)
    out[:8] = MAGIC
    out[8] = VERSION
    out[9] = (TRACK_RECORDED if track1 is not None else 0) | (
        TRACK_WRITE_PROTECTED if protect1 else 0
    )
    out[10] = (TRACK_RECORDED if track2 is not None else 0) | (
        TRACK_WRITE_PROTECTED if protect2 else 0
    )
    if track1 is not None:
        out[12:131] = pack_track(track1)
    if track2 is not None:
        out[131:250] = pack_track(track2)
    return bytes(out)


def hpp_bytes(
    words: tuple[int, ...],
    *,
    calculator: int,
    bitmap: str,
    card_name: str,
    eol: bytes,
    decimal: bool,
) -> bytes:
    if calculator not in (67, 97):
        raise ValueError("HPP calculator must be 67 or 97")

    nibbles = [0] * HPP_DUMMY_NIBBLES
    for word in words:
        nibbles.extend((word >> (4 * nibble)) & 0x0F for nibble in range(7))
    nibbles.extend(nibbles[-7:])

    if decimal:
        data = " ".join(str(value) for value in nibbles).encode("ascii")
    else:
        data = "".join(f"{value:x}" for value in nibbles).encode("ascii")

    body = eol.join(
        (
            str(calculator).encode("ascii"),
            bitmap.encode("utf-8"),
            card_name.encode("utf-8"),
            data,
            b"",
        )
    )
    clear = eol.join((b"NeWe", str(len(body)).encode("ascii"), body))
    return bytes(byte ^ 0x55 for byte in clear)


def card_header_track(words: tuple[int, ...]) -> tuple[int, int]:
    header = (words[0] >> 24) & 0x0F
    if header in (1, 3):
        return header, 1
    if header in (2, 4):
        return header, 2
    raise ValueError(f"unsupported HPP header id {header}")


def write_file(path: Path, data: bytes, overwrite: bool) -> None:
    if path.exists() and not overwrite:
        raise ValueError(f"{path} exists; use --overwrite to replace it")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def hpp_to_card(args: argparse.Namespace) -> int:
    if args.inputs and (args.track1 or args.track2):
        raise ValueError(
            "use positional HPP files for header-based auto placement OR --track1/--track2"
        )
    if not args.inputs and not args.track1 and not args.track2:
        raise ValueError("no HPP input supplied")

    words: dict[int, tuple[int, ...]] = {}
    if args.inputs:
        for raw_path in args.inputs:
            parsed = parse_hpp(Path(raw_path))
            target = parsed.inferred_track
            if target in words:
                raise ValueError(
                    f"multiple HPP files map to Track {target}; use explicit "
                    "--track1/--track2 for media such as two independent header-3 sides"
                )
            words[target] = parsed.words
            if parsed.declared_body_len != parsed.actual_body_len:
                print(
                    f"warning: {raw_path}: declared HPP body length "
                    f"{parsed.declared_body_len} != actual {parsed.actual_body_len}",
                    file=sys.stderr,
                )
    else:
        for target, raw_path in ((1, args.track1), (2, args.track2)):
            if raw_path is None:
                continue
            parsed = parse_hpp(Path(raw_path))
            words[target] = parsed.words
            if parsed.inferred_track != target:
                print(
                    f"warning: {raw_path}: header {parsed.header_id} normally maps to "
                    f"Track {parsed.inferred_track}; explicit --track{target} overrides it",
                    file=sys.stderr,
                )

    encoded = native_bytes(
        words.get(1),
        words.get(2),
        protect1=args.protect1,
        protect2=args.protect2,
    )
    write_file(Path(args.output), encoded, args.overwrite)
    print(
        f"wrote {args.output}: Track1={'recorded' if 1 in words else 'unrecorded'}, "
        f"Track2={'recorded' if 2 in words else 'unrecorded'}"
    )
    return 0


def card_to_hpp(args: argparse.Namespace) -> int:
    source = Path(args.input)
    tracks = parse_native(source)
    out_dir = Path(args.out_dir)
    stem = args.stem or source.stem
    eol = {"lf": b"\n", "crlf": b"\r\n", "cr": b"\r"}[args.line_ending]

    written = 0
    for track_number, track in enumerate(tracks, start=1):
        if not track.recorded or track.words is None:
            continue
        header, inferred = card_header_track(track.words)
        if track.write_protected:
            print(
                f"warning: Track {track_number} is write-protected; HPP cannot represent "
                "write protection",
                file=sys.stderr,
            )
        if inferred != track_number:
            print(
                f"warning: native Track {track_number} carries header {header}, which Teenix "
                f"re-import normally maps to Track {inferred}; use explicit --track{track_number} "
                "when converting it back",
                file=sys.stderr,
            )

        target = out_dir / f"{stem}_{track_number}.hpp"
        encoded = hpp_bytes(
            track.words,
            calculator=args.calculator,
            bitmap=args.bitmap or f"{stem}.bmp",
            card_name=args.name or stem,
            eol=eol,
            decimal=args.decimal_nibbles,
        )
        write_file(target, encoded, args.overwrite)
        print(f"wrote {target}")
        written += 1

    if written == 0:
        raise ValueError(f"{source}: no recorded tracks to export")
    return 0


def parse_listing(path: Path, base_step: int) -> list[int]:
    program = [0] * PROGRAM_STEPS
    seen: set[int] = set()
    matched = 0

    for line_number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        fields = raw.strip().split()
        if len(fields) < 2 or not fields[0].isdigit():
            continue
        try:
            code = int(fields[1], 16)
        except ValueError:
            continue
        step = int(fields[0])
        if code < 0 or code > 0xFF:
            raise ValueError(f"{path}:{line_number}: opcode must be one byte")
        offset = step - base_step
        if offset < 0 or offset >= PROGRAM_STEPS:
            raise ValueError(
                f"{path}:{line_number}: step {step} outside "
                f"{base_step}..{base_step + PROGRAM_STEPS - 1}"
            )
        if step in seen:
            raise ValueError(f"{path}:{line_number}: duplicate step {step}")
        seen.add(step)
        program[offset] = code
        matched += 1

    if matched == 0:
        raise ValueError(
            f"{path}: no '<step> <hex-byte>' listing lines were found, "
            "for example '001  FA  LBL A'"
        )
    return program


def build_program_track(program: list[int], header: int) -> tuple[int, ...]:
    if len(program) != PROGRAM_STEPS:
        raise ValueError(f"program must contain exactly {PROGRAM_STEPS} slots")
    if header < 0 or header > WORD_MASK:
        raise ValueError("header must be a 28-bit value")
    header_id = (header >> 24) & 0x0F
    if header_id not in (3, 4):
        raise ValueError(
            "listing-to-card requires a program header (high nibble 3 or 4)"
        )

    nibbles: list[int] = []
    for code in program:
        nibbles.extend((code & 0x0F, (code >> 4) & 0x0F))

    words = [0] * RECORDS
    words[0] = header
    for pair in range(16):
        first = nibbles[pair * 14:pair * 14 + 7]
        second = nibbles[pair * 14 + 7:pair * 14 + 14]
        words[2 + pair * 2] = sum(value << (4 * i) for i, value in enumerate(first))
        words[1 + pair * 2] = sum(value << (4 * i) for i, value in enumerate(second))
    words[33] = sum(words[:33]) & WORD_MASK
    return tuple(words)


def parse_header(text: str | None, option: str) -> int:
    if text is None:
        raise ValueError(f"{option} is required for the supplied listing")
    try:
        value = int(text, 0)
    except ValueError as exc:
        raise ValueError(f"{option}: invalid integer {text!r}") from exc
    if value < 0 or value > WORD_MASK:
        raise ValueError(f"{option}: header must fit in 28 bits")
    return value


def listing_to_card(args: argparse.Namespace) -> int:
    if not args.track1 and not args.track2:
        raise ValueError("supply --track1 and/or --track2 listing")

    word1: tuple[int, ...] | None = None
    word2: tuple[int, ...] | None = None

    if args.track1:
        header1 = parse_header(args.header1, "--header1")
        base1 = 113 if ((header1 >> 24) & 0x0F) == 4 else 1
        word1 = build_program_track(parse_listing(Path(args.track1), base1), header1)
    if args.track2:
        header2 = parse_header(args.header2, "--header2")
        base2 = 113 if ((header2 >> 24) & 0x0F) == 4 else 1
        word2 = build_program_track(parse_listing(Path(args.track2), base2), header2)

    encoded = native_bytes(
        word1,
        word2,
        protect1=args.protect1,
        protect2=args.protect2,
    )
    write_file(Path(args.output), encoded, args.overwrite)
    print(f"wrote {args.output}")
    return 0


def inspect(args: argparse.Namespace) -> int:
    path = Path(args.input)
    if path.suffix.lower() == ".hpp":
        parsed = parse_hpp(path)
        print(f"format: HPP compatibility")
        print(f"calculator: HP-{parsed.calculator_id}")
        print(f"bitmap: {parsed.bitmap_name}")
        print(f"name: {parsed.card_name}")
        print(f"header: {parsed.header_id}")
        print(f"inferred native track: {parsed.inferred_track}")
        print(
            f"declared body length: {parsed.declared_body_len} "
            f"(actual {parsed.actual_body_len})"
        )
        print(f"record 1: 0x{parsed.words[0]:07x}")
        print(f"record 34: 0x{parsed.words[33]:07x}")
        return 0

    track1, track2 = parse_native(path)
    print("format: HP67CARD v1")
    for number, track in enumerate((track1, track2), start=1):
        state = "recorded" if track.recorded else "unrecorded"
        protection = ", write-protected" if track.write_protected else ""
        print(f"Track {number}: {state}{protection}")
        if track.words is not None:
            header = (track.words[0] >> 24) & 0x0F
            print(
                f"  header={header} record1=0x{track.words[0]:07x} "
                f"record34=0x{track.words[33]:07x}"
            )
    return 0


def self_test(_: argparse.Namespace) -> int:
    root = Path(__file__).resolve().parents[1]

    side1 = parse_hpp(
        root / "programs/HP67/HP-67 Standard Pac/SD1-15A_1 Diagnostic Program.hpp"
    )
    side2 = parse_hpp(
        root / "programs/HP67/HP-67 Standard Pac/SD1-15A_2 Diagnostic Program.hpp"
    )
    converted = native_bytes(side1.words, side2.words)
    expected = (
        root / "programs/HP67/HP-67 Standard Pac/SD1-15A-Diagnostic-Program.hp67card"
    ).read_bytes()
    if converted != expected:
        raise ValueError("HPP -> HP67CARD self-test differs from committed SD1-15A")

    sd15c = root / "programs/HP67/HP-67 Diagnostic Cards/SD-15C-Diagnostic-Program.hp67card"
    native_tracks = parse_native(sd15c)
    with tempfile.TemporaryDirectory() as temporary:
        temp = Path(temporary)
        for number, track in enumerate(native_tracks, start=1):
            if track.words is None:
                continue
            encoded = hpp_bytes(
                track.words,
                calculator=67,
                bitmap="self-test.bmp",
                card_name="self-test",
                eol=b"\n",
                decimal=False,
            )
            target = temp / f"side{number}.hpp"
            target.write_bytes(encoded)
            parsed = parse_hpp(target)
            if parsed.words != track.words:
                raise ValueError(
                    f"HP67CARD -> HPP -> records self-test failed for Track {number}"
                )

    cd01_listing = """001 FA LBL A
002 12 2
003 B1 GSB 1
004 D2 GTO 2
005 19 9
006 00 R/S
007 F1 LBL 1
008 1B ENTER
009 39 *
010 0E RTN
011 F2 LBL 2
012 13 3
013 37 +
014 00 R/S
"""
    with tempfile.TemporaryDirectory() as temporary:
        temp = Path(temporary)
        listing = temp / "cd01.txt"
        listing.write_text(cd01_listing, encoding="utf-8")
        track = build_program_track(parse_listing(listing, 1), 0x03100222)
        built = native_bytes(track, None)
        expected_cd01 = (
            root / "programs/HP67/Custom Diagnostic Pacs/CD-01_Flow-GSB-GTO-RTN.hp67card"
        ).read_bytes()
        if built != expected_cd01:
            raise ValueError("listing -> HP67CARD self-test differs from committed CD-01")

    print("converter self-test passed: HPP <-> HP67CARD and listing -> HP67CARD")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Convert HP-67 HPP compatibility files and native HP67CARD media"
    )
    sub = parser.add_subparsers(dest="command", required=True)

    h2c = sub.add_parser("hpp-to-card", help="convert one/two HPP tracks to native .hp67card")
    h2c.add_argument("inputs", nargs="*", help="HPP files, auto-placed from header 1/3 or 2/4")
    h2c.add_argument("--track1", help="explicit HPP to place on native Track 1")
    h2c.add_argument("--track2", help="explicit HPP to place on native Track 2")
    h2c.add_argument("-o", "--output", required=True)
    h2c.add_argument("--protect1", action="store_true")
    h2c.add_argument("--protect2", action="store_true")
    h2c.add_argument("--overwrite", action="store_true")
    h2c.set_defaults(func=hpp_to_card)

    c2h = sub.add_parser("card-to-hpp", help="export recorded native tracks as compatibility HPP")
    c2h.add_argument("input")
    c2h.add_argument("--out-dir", default=".")
    c2h.add_argument("--stem")
    c2h.add_argument("--calculator", type=int, default=67, choices=(67, 97))
    c2h.add_argument("--bitmap")
    c2h.add_argument("--name")
    c2h.add_argument("--line-ending", choices=("lf", "crlf", "cr"), default="lf")
    c2h.add_argument("--decimal-nibbles", action="store_true")
    c2h.add_argument("--overwrite", action="store_true")
    c2h.set_defaults(func=card_to_hpp)

    l2c = sub.add_parser(
        "listing-to-card",
        help="build native program-card tracks from '<step> <hex-byte> ...' listings",
    )
    l2c.add_argument("--track1", help="listing for native Track 1")
    l2c.add_argument("--track2", help="listing for native Track 2")
    l2c.add_argument("--header1", help="28-bit Track 1 header, e.g. 0x03100222")
    l2c.add_argument("--header2", help="28-bit Track 2 header, e.g. 0x04000222")
    l2c.add_argument("-o", "--output", required=True)
    l2c.add_argument("--protect1", action="store_true")
    l2c.add_argument("--protect2", action="store_true")
    l2c.add_argument("--overwrite", action="store_true")
    l2c.set_defaults(func=listing_to_card)

    info = sub.add_parser("inspect", help="show structural HPP or HP67CARD information")
    info.add_argument("input")
    info.set_defaults(func=inspect)

    test = sub.add_parser("self-test", help="verify converter against committed fixtures")
    test.set_defaults(func=self_test)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return args.func(args)
    except (OSError, UnicodeError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
