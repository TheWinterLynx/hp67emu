#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "usage: build_nonpareil_uasm_wsl.sh <nonpareil-repo> <object-output-dir> <build-dir>" >&2
    exit 2
fi

repo="$1"
out="$2"
build="$3"

for cmd in gcc flex bison; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "missing WSL build dependency: $cmd" >&2
        echo "install with: sudo apt-get update && sudo apt-get install -y build-essential flex bison" >&2
        exit 4
    fi
done

rm -rf "$build"
mkdir -p "$build/src" "$out"
cp -a "$repo/src/." "$build/src/"
cd "$build/src"

# The pinned Nonpareil commit has a known upstream packaging defect: wasm.c and
# wasm_y.y include wasm.h, but that header was never committed. Upstream issue
# #24 documents the missing declaration. Generate the minimal compatibility
# header only in this disposable build tree; never modify or vendor upstream.
if [[ ! -f wasm.h ]]; then
    cat > wasm.h <<'EOF'
#ifndef NONPAREIL_WASM_H
#define NONPAREIL_WASM_H

#include "asm.h"

void pseudo_check(addr_t addr);

#endif
EOF
fi

for stem in asm asm_cond casm wasm nasm; do
    bison -d -o "${stem}_y.c" "${stem}_y.y"
    flex -o "${stem}_l.c" "${stem}_l.l"
done

gcc \
    -std=gnu99 -O2 -Wall -Wextra \
    -DNONPAREIL_RELEASE=0.79 \
    -I. \
    -o "$build/uasm" \
    asm.c symtab.c \
    asm_l.c asm_y.c \
    asm_cond.c asm_cond_l.c asm_cond_y.c \
    casm_l.c casm_y.c \
    wasm_l.c wasm_y.c wasm.c \
    nasm_l.c nasm_y.c \
    util.c arch.c release.c

probe="$({ "$build/uasm"; } 2>&1 || true)"
if [[ "$probe" != *"uasm microassembler"* ]]; then
    echo "built executable did not identify itself as Nonpareil uasm" >&2
    printf '%s\n' "$probe" >&2
    exit 5
fi

for name in 67 6797 67b1; do
    "$build/uasm" \
        -o "$out/$name.obj" \
        -l "$out/$name.lst" \
        "$repo/ncd/67-97/$name.asm"
done

printf '%s\n' "$build/uasm"
