# M15S — ACT RAM reads without a local DATA source

Base/main: owner-validated M15R 761a85833511b8d3133d97884b7fbf3c38a8adce,
integrated exactly after FULL GATE GREEN. Diagnostic 12/12, identical cycles;
owner live idle 2.198 us/word (145.60x), independent RAM 1.163 us/word. These
are M15R measurements, not M15S results.

## Executable change

M15Q/R already sourced actual read bits from independent RAM, but ACT's phase
path retained a dummy zero register to express frame presence. M15S removes that
local read source entirely. The RAM-capable transport binds a receive-only word
from the ACT transfer direction. Its current payload, source register and source
tail are absent for reads. Two small flags retain body and next-b0/b1 receive
expectations independently of source storage. Existing count/owner/level checks
read the real net, requiring RamAggregate.

Default live passes None for autonomous read payload. The frozen oracle result
still lives in pending authority and must match the completed net frame before
C commits. ACT write/address sources, RAM storage/commands/read proposal and the
joint source commit remain unchanged. Legacy no-RAM public APIs preserve their
reference payload source behavior. There is no new source, allocation or clock.

Completion retains the old read direction for b0/b1 even though no local source
tail exists. A following ACT write starts at b2 while the old RAM read finishes
at b1. Source presence now governs writes/reference transfers; receive presence
governs autonomous reads. Unexpected idle drivers still fail.

## Verification

New core coverage checks source/current payload/tail absence, actual net read
collection with no payload supplied, read-to-write handoff, correct completions,
missing RAM driver rejection without receiver mutation, idle release and 672
transitions across three words. Existing M15Q wrong-payload tests now exercise
the receive-only binding through both shared transports. Existing M15R owner
checks remain. A new real-firmware native test inserts a foreign DATA driver
before a pending read tail: it must fail before C, RAM, executed-word count or
boot cycle changes, retain pending authority and preserve all device registers.

Existing real-firmware differential, oracle-corruption, full all-targets, release,
diagnostic and benchmark gates remain required. Rust is absent here; static
source/document checks and formatting-only Actions are reviewed here. No runtime
pass or performance result is claimed. M15S gate pending.

## Evidence boundaries

This removes a software composition placeholder, not a new hardware timing
claim. M15N direct HP-67 DATA b2..next-b1 phase and existing 56 cells/224 PHI
transitions remain. Exact PHI launch/sample edges, physical output stage/bias,
per-IC mapping and propagation remain SOURCE-BLOCKED. CRC DATA/sense timing and
ACT internal mutation checkpoints retain their prior evidence boundaries.
