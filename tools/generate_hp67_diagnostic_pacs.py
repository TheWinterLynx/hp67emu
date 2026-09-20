#!/usr/bin/env python3
"""Regenerate/check the native HP-67 Standard diagnostic and Custom Diagnostic Pacs.

Pure standard-library tool. Native .hp67card is the canonical diagnostic format. The tool
regenerates synthetic native cards, verifies the exact supplied SD-15C native fixture, and uses
legacy Teenix .hpp only as an import source for the pre-existing SD1-15A compatibility corpus.
Program-card record 34 is the low 28 bits of the running sum of records 1-33.
"""

from __future__ import annotations

import argparse
import hashlib
from dataclasses import dataclass
from pathlib import Path

MASK_28 = (1 << 28) - 1
PROGRAM_STEPS = 112
RECORDS = 34
TRACK_BYTES = 119
SD15C_SHA256 = "f626047e875d919bb22bf7b25bb0d58218955b07ff504f2f546239a2f1fdf75a"


@dataclass(frozen=True)
class Diagnostic:
    ref: str
    slug: str
    title: str
    expected: str
    first: tuple[int, ...]
    second: tuple[int, ...] | None = None


DIAGNOSTICS = (
    Diagnostic("CD-01", "Flow-GSB-GTO-RTN", "Flow GSB GTO RTN", "7.00",
               (0xFA, 0x12, 0xB1, 0xD2, 0x19, 0x00, 0xF1, 0x1B, 0x39, 0x0E, 0xF2, 0x13, 0x37, 0x00)),
    Diagnostic("CD-02", "Flags-SF-CF-Test", "Flags SF CF Test", "6.00",
               (0xFA, 0x6A, 0x8A, 0x58, 0xD1, 0x19, 0x00, 0xF1, 0x6A, 0x58, 0xD2, 0x14, 0xD3, 0xF2, 0x18, 0x00, 0xF3, 0x12, 0x37, 0x00)),
    Diagnostic("CD-03", "Conditionals", "Conditionals", "7.00",
               (0xFA, 0x15, 0x1B, 0x15, 0x51, 0xD1, 0x19, 0x00, 0xF1, 0x32, 0x54, 0xD2, 0x18, 0x00, 0xF2, 0x17, 0x00)),
    Diagnostic("CD-04", "Indirect-STO-RCL", "Indirect STO RCL", "42.00",
               (0xFA, 0x12, 0x46, 0x14, 0x12, 0x3E, 0x32, 0x3F, 0x00)),
    Diagnostic("CD-05", "Nested-GSB", "Nested GSB", "6.00",
               (0xFA, 0x11, 0x1B, 0xB1, 0x00, 0xF1, 0x12, 0x37, 0xB2, 0x0E, 0xF2, 0x13, 0x37, 0x0E)),
    Diagnostic("CD-06", "Second-Half-Label-Search", "Second Half Label", "67.00",
               (0xFA, 0xDE, 0x19, 0x00), (0xFE, 0x16, 0x17, 0x00)),
    Diagnostic("CD-07", "ISZ-Loop", "ISZ Loop", "3.00",
               (0xFA, 0x13, 0x1C, 0x46, 0x32, 0xF0, 0x11, 0x37, 0x5C, 0xD0, 0x00)),
    Diagnostic("CD-08", "DSZ-Loop", "DSZ Loop", "3.00",
               (0xFA, 0x13, 0x46, 0x32, 0xF0, 0x11, 0x37, 0x5E, 0xD0, 0x00)),
    Diagnostic("CD-09", "Long-DSZ-Burn-In", "Long DSZ Burn In", "2000.00",
               (0xFA, 0x12, 0x10, 0x10, 0x10, 0x46, 0x32, 0xF0, 0x11, 0x37, 0x5E, 0xD0, 0x00)),
    Diagnostic("CD-10", "Nested-GSB-Burn-In", "Nested GSB Burn In", "500.00",
               (0xFA, 0x15, 0x10, 0x10, 0x46, 0x32, 0xF0, 0xB1, 0x5E, 0xD0, 0x00,
                0xF1, 0xB2, 0x0E, 0xF2, 0x11, 0x37, 0x0E)),
    Diagnostic("CD-11", "Function-Identity-Burn-In", "Function Identity Burn In", "1.00",
               (0xFA, 0x11, 0x10, 0x10, 0x46, 0x11, 0xF0, 0x07, 0x08, 0x27, 0x28,
                0x03, 0x02, 0x5E, 0xD0, 0x00)),
    Diagnostic("CD-12", "Cross-Half-GSB-Burn-In", "Cross Half GSB Burn In", "500.00",
               (0xFA, 0x12, 0x15, 0x10, 0x46, 0x32, 0xF0, 0xBE, 0x5E, 0xD0, 0x00),
               (0xFE, 0xBD, 0x0E, 0xFD, 0x12, 0x37, 0x0E)),
)


def checksum(words: list[int]) -> int:
    return sum(words[:33]) & MASK_28


def parse_hpp(path: Path) -> list[int]:
    decoded = bytes(byte ^ 0x55 for byte in path.read_bytes()).decode("ascii")
    normalized = decoded.replace("\r\n", "\n").replace("\r", "\n")
    lines = normalized.split("\n")
    if lines[0].strip() != "NeWe":
        raise ValueError(f"{path}: invalid Teenix magic")
    card_data = "\n".join(lines[5:]).strip()
    fields = card_data.split()
    if len(fields) > 1:
        nibbles = [int(field, 10) for field in fields]
    else:
        nibbles = [int(ch, 16) for ch in (fields[0] if fields else "")]
    real = nibbles[21:21 + RECORDS * 7]
    if len(real) != RECORDS * 7:
        raise ValueError(f"{path}: invalid real nibble count {len(real)}")
    words: list[int] = []
    for record in range(RECORDS):
        word = 0
        for nibble in range(7):
            word |= real[record * 7 + nibble] << (nibble * 4)
        words.append(word)
    if checksum(words) != words[-1]:
        raise ValueError(f"{path}: source checksum mismatch")
    return words


def build_program_track(program: tuple[int, ...], header: int) -> list[int]:
    if len(program) > PROGRAM_STEPS:
        raise ValueError("program exceeds 112 steps")
    padded = list(program) + [0] * (PROGRAM_STEPS - len(program))
    nibbles: list[int] = []
    for code in padded:
        nibbles.extend((code & 0x0F, (code >> 4) & 0x0F))
    words = [0] * RECORDS
    words[0] = header
    for pair in range(16):
        first = nibbles[pair * 14:pair * 14 + 7]
        second = nibbles[pair * 14 + 7:pair * 14 + 14]
        words[2 + pair * 2] = sum(value << (4 * i) for i, value in enumerate(first))
        words[1 + pair * 2] = sum(value << (4 * i) for i, value in enumerate(second))
    words[33] = checksum(words)
    return words


def pack_track(words: list[int]) -> bytes:
    packed = bytearray(TRACK_BYTES)
    bit_index = 0
    for word in words:
        for bit in range(27, -1, -1):
            if word & (1 << bit):
                packed[bit_index // 8] |= 1 << (7 - bit_index % 8)
            bit_index += 1
    return bytes(packed)


def hp67card_bytes(first: list[int] | None, second: list[int] | None) -> bytes:
    out = bytearray(250)
    out[:8] = b"HP67CARD"
    out[8] = 1
    out[9] = 1 if first is not None else 0
    out[10] = 1 if second is not None else 0
    if first is not None:
        out[12:131] = pack_track(first)
    if second is not None:
        out[131:250] = pack_track(second)
    return bytes(out)


def unpack_track(payload: bytes) -> list[int]:
    if len(payload) != TRACK_BYTES:
        raise ValueError(f"invalid track byte count {len(payload)}")
    words: list[int] = []
    bit_index = 0
    for _ in range(RECORDS):
        word = 0
        for bit in range(27, -1, -1):
            if payload[bit_index // 8] & (1 << (7 - bit_index % 8)):
                word |= 1 << bit
            bit_index += 1
        words.append(word)
    return words


def parse_sd15c_native(path: Path) -> tuple[list[int], list[int]]:
    raw = path.read_bytes()
    if len(raw) != 250 or raw[:8] != b"HP67CARD" or raw[8] != 1 or raw[11] != 0:
        raise ValueError(f"{path}: invalid HP67CARD v1 container")
    if hashlib.sha256(raw).hexdigest() != SD15C_SHA256:
        raise ValueError(f"{path}: exact SD-15C fixture SHA-256 changed")
    if raw[9] & 1 == 0 or raw[10] & 1 == 0:
        raise ValueError(f"{path}: SD-15C must contain both recorded tracks")
    first = unpack_track(raw[12:131])
    second = unpack_track(raw[131:250])
    if checksum(first) != first[-1] or checksum(second) != second[-1]:
        raise ValueError(f"{path}: SD-15C record checksum mismatch")
    return first, second


def expected_files(root: Path) -> dict[Path, bytes]:
    result: dict[Path, bytes] = {}
    std = root / "programs" / "HP67" / "HP-67 Standard Pac"
    first = parse_hpp(std / "SD1-15A_1 Diagnostic Program.hpp")
    second = parse_hpp(std / "SD1-15A_2 Diagnostic Program.hpp")
    result[std / "SD1-15A-Diagnostic-Program.hp67card"] = hp67card_bytes(first, second)

    diagnostic_cards = root / "programs" / "HP67" / "HP-67 Diagnostic Cards"
    parse_sd15c_native(diagnostic_cards / "SD-15C-Diagnostic-Program.hp67card")

    out = root / "programs" / "HP67" / "Custom Diagnostic Pacs"
    for diagnostic in DIAGNOSTICS:
        first_header = 0x03000222 if diagnostic.second is not None else 0x03100222
        first = build_program_track(diagnostic.first, first_header)
        second = (
            build_program_track(diagnostic.second, 0x04000222)
            if diagnostic.second is not None
            else None
        )
        result[out / f"{diagnostic.ref}_{diagnostic.slug}.hp67card"] = hp67card_bytes(
            first, second
        )
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    mismatches: list[Path] = []
    for path, content in expected_files(root).items():
        if args.check:
            if not path.exists() or path.read_bytes() != content:
                mismatches.append(path)
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
            print(path.relative_to(root))
    if mismatches:
        for path in mismatches:
            print(f"MISMATCH: {path.relative_to(root)}")
        return 1
    if args.check:
        print("HP-67 diagnostic card assets are reproducible")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
