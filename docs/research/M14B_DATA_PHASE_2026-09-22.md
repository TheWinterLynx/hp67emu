# M14B DATA serial-phase foundation — 2026-09-22

## Scope

This slice begins M14B without connecting the HP-67 DATA pin electrically. It locks and implements only the DATA relationships already supported by direct HP-67 captures:

- one continuous 56-bit register stream;
- LSB-first within each 4-bit digit;
- serial DATA bit 0 at machine-word b2;
- serial bit 53 at b55;
- serial bits 54/55 at b0/b1 of the following machine word.

The goal is to make the cross-word DATA phase explicit before physical 1818-* RAM devices are wired to the dense electrical scheduler.

## Why this is separated from DATA electrical integration

The reviewed sources still do not justify:

- DATA passive level;
- active drive polarity;
- exact ACT/RAM driver-enable timing;
- PHI-relative launch edge;
- PHI-relative sample edge;
- propagation delay;
- exact physical 1818-* RAM chip/address mapping.

Encoding any of those now would turn a source-backed machine-bit phase into an invented electrical protocol.

## Implemented

`src/machines/hp67/data.rs` adds:

- `data_register_serial_bit()` for logical register serialization;
- `Hp67DataSerialSource` carrying bits 54/55 into the following word;
- `Hp67DataSerialSink` reconstructing a frame only after following-word b1;
- back-to-back frame handling where one frame completes at b1 and the next starts at b2;
- hard errors for missing/unexpected logical samples and out-of-order traversal;
- `Hp67DataTransferPlan` for the already-validated ACT RAM read/write instruction classes and address selection.

No production path reads or drives `Hp67Net::Data` in this slice.

## Performance rule

M14B changes must not silently spend the M14A realtime margin.

The established M14A validation reference on the same host is:

- dense staged scheduler: **1.171 us/word, 273.27x realtime**;
- full current structural path: **0.946 us/word, 338.35x realtime**;
- production dual path: **1.000 us/word, 320.14x realtime**;
- real firmware dual path: **0.955 us/word, 335.24x realtime**.

The new DATA logical-phase benchmark is appended after all established rows so it cannot change their measurement order. It is measured in isolation before any production integration. If established rows regress materially, the integration is rejected or redesigned before adding more fidelity.

## Exit criterion for this slice

- DATA source/sink round-trip is exact across at least two back-to-back frames;
- direct and register-select RAM read/write plans match the architectural address semantics;
- the full normal test gate remains green;
- the established M14A benchmark rows remain within normal host-run variance;
- the isolated DATA row establishes the incremental cost before electrical DATA integration.

## Next slice

After performance validation, the next step is to attach logical DATA ownership to ACT/peripheral endpoints while keeping polarity and PHI edges source-blocked. Physical 1818-* device/address mapping must be verified before the temporary architectural RAM image is partitioned into named chips.
