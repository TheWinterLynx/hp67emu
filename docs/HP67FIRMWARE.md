# HP-67 firmware image

## Canonical project firmware

The emulator's canonical runtime firmware is versioned directly in this repository under:

- `src/machines/hp67/hp67firmware.00` through `hp67firmware.07`: bank 0, logical PC `0x000..0xfff`.
- `src/machines/hp67/hp67firmware.08` through `hp67firmware.09`: bank 1, logical PC `0x400..0x7ff`.
- `src/machines/hp67/hp67firmware.rs`: typed ROM source and bank mapping used by the emulator.

The image contains 5120 populated 10-bit words. Each data block contains 512 octal words. The firmware data is part of Git history; cloning the repository is sufficient to compile the emulator.

`hp67firmware.rs` uses `include_str!` to place these versioned blocks into the compiled executable. There is no runtime firmware pathname, environment variable, download, generated source file, or `.research` dependency.

## Address mapping

Bank 0 contains the full 4096-word logical address space. Bank 1 contains the physical `0x400..0x7ff` window only. When bank 1 is requested outside that populated window, the ROM source returns the corresponding bank-0 word, matching the machine's physical page-bank behaviour.

## Validation

The accepted image was established during the research phase by comparing independently available HP-67 firmware representations. The 5120 populated words agreed bit-for-bit across the accepted corpora before being promoted to `hp67firmware`.

The project regression path now validates the versioned image directly. `hp67_arithmetic_smoke` boots the real firmware to `0.00`, sends physical key contacts for `1 ENTER 2 +`, requires the firmware keyboard-dispatch path, and verifies the ROM0/1820-1749 physical display pattern for `3.00`.

## Provenance references

Historical acquisition, comparison and disassembly evidence remains documentation-only. Relevant research documents include `MICROCODE_PROVENANCE.md`, `ROM_CORPUS_WORKFLOW.md`, `NONPAREIL_ANALYSIS.md`, `X11_CALC_ANALYSIS.md`, and the dated comparison notes under `docs/research/`.

No production Rust module, firmware data filename, executable workflow or runtime path depends on those external project names or sources.
