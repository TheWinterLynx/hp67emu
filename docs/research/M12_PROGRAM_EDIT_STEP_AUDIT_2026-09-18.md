# M12 PROGRAM editing and stepping audit — 2026-09-18

## Scope

This audit fixes the source-backed behavior required to close the remaining M12 integration scope:

- PROGRAM-mode SST navigation;
- PROGRAM-mode h BST navigation;
- PROGRAM-mode h DEL editing;
- RUN-mode SST single-step execution;
- RUN-mode h BST back-step without execution.

The implementation target remains the real HP-67 firmware path. No host-side program editor, synthetic program counter or semantic single-step shortcut is permitted.

## Primary HP behavior source

The Hewlett-Packard HP-67 Quick Reference Card, part 00067-90001 Rev. B (April 1977), states:

- PROGRAM mode has active program-control functions rather than storing every key as an instruction;
- SST moves forward one program-memory step;
- h BST moves backward one program-memory step;
- h DEL deletes the current instruction and moves all subsequent instructions up one step;
- RUN-mode SST displays the current step while held, executes that instruction on release, displays the result and advances to the next step;
- RUN-mode BST moves to the previous step and, after release, restores the original X display; no program instruction is executed.

The same card identifies 224 program steps and the top-of-memory marker at step 000.

Source used for this audit:
- Hewlett-Packard, *HP-67 Quick Reference Card*, 00067-90001 Rev. B, April 1977.
- Hewlett-Packard, *HP-67 Owner's Handbook and Programming Guide*, 00067-90011.

## Pinned firmware evidence

Pinned Nonpareil source:
- repository: `brouhaha/nonpareil`
- commit: `c347bc1ab20170c253512042f7aac0d952f304ea`
- files: `ncd/67-97/67.asm`, `ncd/67-97/6797.asm`

### SST

In `67.asm`, label `sst`:

- sets S1, the firmware SST flag;
- obtains the user PC from RAM register 0x3D;
- in PROGRAM mode, advances the PC with `incpc`;
- routes through the program-step display path;
- in RUN mode, step 000 is first advanced to step 001, while a nonzero current step is retained for execution;
- after execution, `inc_pc_if_running` treats S1 like the running state and advances the user PC;
- `op_done` clears S1.

This gives a precise RUN-mode oracle: starting from user PC 000, one SST executes step 001 and finishes with the user PC at step 002.

### BST

In `67.asm`:

- the h-shifted key table maps physical h + SST to `op_bst`;
- `op_bst` branches to `bst`;
- `bst` sets S3 and calls `S1214` before returning through the program-step display path.

This is program-counter navigation and does not branch through the user-instruction executor.

### DEL

In `67.asm` the h-shifted CLx entry maps to `del_x`, which reaches `del` in `6797.asm`.

The `del` routine:

- checks S11 and therefore only deletes while firmware is in PROGRAM mode;
- refuses deletion at user PC 000;
- shifts the stored program contents across program RAM so subsequent instructions move upward;
- returns through the same program-navigation machinery with S3 clear, leaving the user PC one instruction earlier.

A critical test consequence is that deleting or inserting byte `0x00` cannot be detected merely by requiring RAM bits to change. User-PC movement and exact final program bytes are separate invariants.

## M12 regression plan

### PROGRAM editing regression

1. Boot real firmware and enter PROGRAM mode.
2. Store `1 ENTER 2 + R/S` and require exact bytes `11 1B 12 37 00`.
3. Verify PROGRAM SST moves from step 005 to step 006.
4. Verify h BST returns to step 005, then to 004 and 003.
5. At step 003, execute h DEL.
6. Require:
   - user PC becomes step 002;
   - program bytes become `11 1B 37 00` at the front of RAM 0x2F.
7. Insert physical Digit3 after step 002.
8. Require exact program bytes `11 1B 13 37 00`.
9. Switch to RUN, use real h RTN to step 000, run with physical R/S, and require the edited program to halt on physical `4.00` with the user PC at step 006.

This proves that editing changes the stored program that firmware subsequently executes.

### RUN SST/BST regression

1. Store the original `1 ENTER 2 + R/S` program.
2. Return to RUN and use real h RTN to set user PC 000.
3. Press SST four times.
4. Require each single step to:
   - assert S1 at least once;
   - enter the real user-instruction executor at `06021`;
   - leave S2 clear;
   - advance user PC respectively to steps 002, 003, 004 and 005.
5. After the fourth SST, require physical `3.00`.
6. Execute h BST.
7. Require:
   - user PC moves from step 005 back to 004;
   - S2 and S1 remain clear at settle;
   - the path never enters `06021`;
   - the physical X display returns to the unchanged `3.00` frame.

## Fidelity boundary

These regressions validate firmware-visible PROGRAM and RUN control behavior at instruction/word boundaries. They do not claim exact key-hold display duration, LED persistence or PHI-relative timing; those remain in the electrical timing milestones.


## Physical-display settle requirement

The first RUN-SST regression reached the correct arithmetic result but observed the physical frame as `3.` rather than `3.00`. This was not an arithmetic or firmware-formatting failure. The live display model updates one physical scan slot per structural word, so reaching the no-key firmware wait can precede completion of the first full 15-position refresh after the result is made visible.

The HP quick-reference and owner's handbook explicitly describe RUN-mode SST as displaying the executed result; published examples retain the active display format (for example results ending in `.00`). The permanent harness therefore keeps the `3.00` oracle and, after firmware has settled to the no-key wait, advances one complete `HP67_DISPLAY_SCAN_SLOTS` refresh before inspecting the physical frame.

This changes only the test observation boundary. It does not alter emulator display production or firmware timing.
