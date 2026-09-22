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

## Validated foundation result (2026-09-22)

The owner validation gate passed, including the 12/12 Custom Diagnostic Pac suite. Release benchmark results on the validation host were:

| Path | Median us/word | Realtime multiple |
| --- | ---: | ---: |
| dense staged scheduler | 1.176 | 272.05x |
| IS ACT<->ROM structural fetch | 0.897 | 356.71x |
| IS + ROM0 display + serial ACT execution | 0.982 | 326.01x |
| production dual architectural + structural | 1.038 | 308.26x |
| real firmware architectural + structural | 0.955 | 334.96x |
| DATA logical phase source + sink | 0.061 | 5233.55x |

The established M14A rows remained within ordinary host-run variance and the real-firmware row was effectively unchanged from the 0.955 us/word M14A reference. The isolated DATA phase engine costs only 0.061 us/word even when exercised continuously on every word, leaving substantial headroom for conditional integration. This is a computational-cost result, not an electrical-fidelity promotion.

## Validated conditional firmware shadow result (2026-09-22)

A second validation run kept the established paths green and measured the realistic conditional DATA shadow against the real-firmware control flow:

| Path | Median us/word | Realtime multiple |
| --- | ---: | ---: |
| real firmware architectural + structural | 1.017 | 314.67x |
| real firmware + conditional RAM DATA shadow | 1.040 | 307.73x |
| isolated DATA logical phase source + sink | 0.067 | 4757.80x |

The same-run incremental cost of the conditional shadow is **0.023 us/word (~2.3%)**. This is acceptable as an experiment but is not yet paid by production. The next experiment must fuse DATA sampling into the already-existing structural b0..b55 loop rather than running a second 56-bit loop for transfer words. Existing production rows remain the regression baseline.

## Validated fused-loop result (2026-09-22)

The full owner gate passed at commit `c8d4de6663c024d86700b6974341af0dd9f1ad55`: focused fused-DATA tests, 194 library tests, 73/74 desktop tests with only the intentionally ignored diagnostic, architecture/documentation/startup/reference tests, release build, and the explicit 12/12 Custom Diagnostic Pac suite all passed.

Same-run release benchmark medians were:

| Path | Median us/word | Realtime multiple |
| --- | ---: | ---: |
| real firmware architectural + structural | 0.964 | 331.84x |
| DATA logical phase source + sink | 0.062 | 5190.26x |
| conditional second-loop RAM DATA shadow | 0.932 | 343.24x |
| fused RAM DATA phase | 0.957 | 334.27x |

The fused path is effectively coincident with the untouched real-firmware baseline in the same run (`0.957` versus `0.964` us/word). The shadow row being faster than baseline in this run confirms that differences at this scale are dominated by host/codegen variance rather than a reliable negative cost. The acceptance criterion is therefore that fusion shows no material regression, not that any sub-percent ordering between these rows is meaningful.

This validates the fused logical DATA path as the production integration shape. It does **not** promote DATA polarity, electrical ownership, PHI edge placement, propagation or physical RAM-chip mapping beyond SOURCE-BLOCKED.

## Exit criterion for this slice

- DATA source/sink round-trip is exact across at least two back-to-back frames;
- direct and register-select RAM read/write plans match the architectural address semantics;
- the full normal test gate remains green;
- the established M14A benchmark rows remain within normal host-run variance;
- the isolated DATA row establishes the incremental cost before electrical DATA integration.

## Next slice

After performance validation, the next step is to attach logical DATA ownership to ACT/peripheral endpoints while keeping polarity and PHI edges source-blocked. Physical 1818-* device/address mapping must be verified before the temporary architectural RAM image is partitioned into named chips.


## Fused-loop experiment

The next benchmark-only slice adds a DATA-aware variant of the structural display/fetch cycle. It reuses the existing b0..b55 loop and invokes logical DATA visitation from that loop only on transfer or tail-completion words. Existing production callers still instantiate the same structural transport with a monomorphized no-op DATA visitor; therefore any material movement in the established benchmark rows is treated as a regression and blocks integration.


The conditional and fused RAM experiments deliberately ignore transfer plans whose address is not installed in `ActRamImage`. This keeps CRC/card data ports out of the RAM slice: CRC DATA behavior remains a separate peripheral/electrical milestone and is not inferred from RAM serialization.
