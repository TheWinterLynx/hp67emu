# M12 program execution audit — 2026-09-18

## Scope

This audit replaces the earlier M12 trial-and-error RUN sequence with a state-driven proof derived from HP-67 documentation, the pinned Nonpareil disassembly, the already-proven M11 physical keyboard path, and independent HP67u RAM observations.

The triggering failure was:

```text
held ClearX contact never reached firmware target 1417
```

That failure occurred only after all five PROGRAM keys had already been accepted and after the display-transport defects had been corrected. The failure was therefore re-analysed as a RUN-transition/harness problem rather than patched as another keyboard or display defect.

## Firmware RUN/PRGM state machine

Pinned Nonpareil `ncd/67-97/67.asm` shows the main wait loop at `L0167`:

- CRC flag 1 is tested as the physical PRGM/RUN input.
- In PRGM, firmware enters `L0315`, exchanges B/C and sets ACT status S11.
- S11 is therefore the firmware's internal PROGRAM-mode latch.
- When the external switch returns to RUN, the next main-loop transition clears S11, restores B/C and returns through the normal RUN cleanup path.

Consequently, changing the external flag is not itself a completed mode transition. A test must wait until a later no-key main-loop visit where S11 agrees with the requested switch state.

The previous M12 test violated this rule: it called `set_program_mode(false)` and immediately pressed CLX. That could inject CLX while firmware was still leaving PROGRAM mode.

## Why CLX was removed

CLX was never required to start a stored HP-67 program. It was an artificial test step used only to normalize the display before execution.

The HP-67 quick-reference behavior is more direct: in RUN mode, RTN sets the program counter to 000. The firmware implementation in pinned Nonpareil confirms this. With S2 (running) and S1 (single-step) both clear, `op_rtn` reaches `clear_return_stack`, writes an all-zero word to RAM register `0x3D`, and returns to the main loop.

M12 therefore uses the real physical `h -> RTN` path and requires RAM `0x3D == 0` before starting.

## User program counter

Pinned firmware `getpc` and `incpc0` select RAM block 3, register 13: physical RAM address `0x3D` (decimal 61).

Independent HP67u observations identify the same register and show normal user-PC encodings such as `62f` for step 001. In the project's least-significant-nibble-first `ActRegister`, the first five post-insert PC values must therefore end in:

| step | displayed low three nibbles | ActRegister [0..3] |
| ---: | --- | --- |
| 001 | 62F | F,2,6 |
| 002 | 52F | F,2,5 |
| 003 | 42F | F,2,4 |
| 004 | 32F | F,2,3 |
| 005 | 22F | F,2,2 |

The M12 helper now requires those exact values instead of accepting an arbitrary RAM change.

## Program RAM and exact stored bytes

HP67u observations place program memory in RAM `0x10..0x2F`. RAM `0x2F` (decimal 47) holds steps 001..007, and step 001 occupies nibbles 0 and 1 at the right-hand side of the register.

The HP-67/97 internal program-code table and firmware execution dispatch give:

- `1` = `0x11`
- `ENTER` = `0x1B`
- `2` = `0x12`
- `+` = `0x37`
- `R/S` = `0x00`

Therefore, after entering `1 ENTER 2 + R/S`, RAM `0x2F` nibbles 0..9 must be:

```text
1 1  B 1  2 1  7 3  0 0
\_/  \_/  \_/  \_/  \_/
 11    1B    12    37    00
```

M12 now asserts that exact packed representation.

## Physical keyboard dispatch

The old RUN helper only watched for an incidental ROM PC value while holding a key. It has been replaced with the already-proven M11 dispatch model:

1. close the physical contact;
2. wait for firmware `0120 keys -> A`;
3. verify A[2:1] equals the physical scan code;
4. continue holding until firmware executes `0220 A -> ROM address`;
5. verify the resulting dispatch-table target;
6. release the contact;
7. for non-running operations, wait for a later no-key main-loop visit.

PROGRAM entry now follows the same physical hold-through-dispatch rule before the insertion path is allowed to complete.

## Starting and stopping the program

With RUN settled and RTN having set RAM `0x3D` to step 000:

- physical R/S is dispatched through the real unshifted key table;
- firmware `run_stop` sets S2 (running);
- because the PC is zero, firmware calls `incpc` and begins at step 001;
- stored byte `0x00` at step 005 is another R/S;
- executing that R/S while S2 is set follows the halt path and clears S2.

The regression therefore requires that S2 was observed true and later false, in addition to requiring the canonical physical `3.00` display frame already established by M10/M11.

## Source set

- Hewlett-Packard, *HP-67 Owner's Handbook and Programming Guide*, part 00067-90011.
- HP-67 Quick Reference: RUN-mode RTN sets the program counter to 000.
- Pinned Nonpareil `c347bc1ab20170c253512042f7aac0d952f304ea`, especially `ncd/67-97/67.asm` and `ncd/67-97/6797.asm`.
- Greg Sydney-Smith HP67u notes: program RAM is registers 16..47; RAM[47] holds steps 001..007; step 001 is nibbles 0/1; user PC is RAM[61].
- Eric Smith HP-67/97 internal hexadecimal instruction table.
