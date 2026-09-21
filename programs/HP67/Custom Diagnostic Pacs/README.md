# Custom Diagnostic Pacs

Synthetic HP-67 program cards for emulator validation. They are not Hewlett-Packard programs.
Each card isolates a small advanced-programming path and halts on a known display value.

Use: power-cycle the calculator, leave RUN selected, load the card from the Program Library,
insert the opposite end if firmware shows `Crd`, then press `A`.

| Ref | Coverage | Expected after A |
| --- | --- | ---: |
| CD-01 | LBL, GSB, RTN, GTO and arithmetic after return | 7.00 |
| CD-02 | SF 0, CF 0, F? 0 true/false skip semantics | 6.00 |
| CD-03 | x=y? and x=0? conditional execution/skip | 7.00 |
| CD-04 | x<>I, STO (i), RCL (i) | 42.00 |
| CD-05 | two-level nested GSB/RTN return stack | 6.00 |
| CD-06 | two-pass program card and GTO label E in steps 113-224 | 67.00 |
| CD-07 | I-register ISZ loop, skipping GTO when I reaches zero | 3.00 |
| CD-08 | I-register DSZ loop, skipping GTO when I reaches zero | 3.00 |
| CD-09 | 2000-iteration DSZ/addition burn-in | 2000.00 |
| CD-10 | 500 iterations with two-level nested GSB/RTN | 500.00 |
| CD-11 | 100 iterations of ln/e^x, log/10^x and sqrt/x^2 identities | 1.00 |
| CD-12 | 250 loops calling a nested subroutine stored in steps 113-224 | 500.00 |

CD-09 through CD-12 are intentionally longer-running burn-ins. They exercise thousands of
stored-program dispatches instead of returning almost immediately.

Every custom diagnostic is checked in only in the project's native `.hp67card` format. The
Program Library loads those native containers directly; Teenix `.hpp` is not part of this
diagnostic path. The generator verifies the 28-bit card checksum and can reproduce every native
fixture with `--check`.

The Standard Pac directory also contains `SD1-15A-Diagnostic-Program.hp67card`, a lossless native
container conversion of the existing source-backed SD1-15A Teenix sides. The separately supplied
`SD-15C` fixture is kept unchanged under `HP-67 Diagnostic Cards`; it is not renamed or silently
substituted for SD1-15A.


## Automated Cargo suite

All twelve native cards can be executed end-to-end through the real firmware, card transport,
physical A-key dispatch and raw LED display path with:

```powershell
cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture
```

The suite keeps running after an individual failure so the output contains one `OK`/`KO` row
per card. The Cargo test fails at the end if any diagnostic is `KO`; a clean run ends with
`DIAGNOSTIC SUITE OK: 12/12 passed`.
