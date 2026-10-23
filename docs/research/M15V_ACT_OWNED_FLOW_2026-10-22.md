# M15V — retained ACT flow registers and own successor address

Date: 2026-10-22
Base: owner-validated M15U `8fe8b797ad449c6a303b9c3c2fc4fb469c76ceaf`, integrated into main.
Branch: `agent/m15v-act-owned-flow`
Status: owner FULL GATE GREEN; exact `e1f8e22160c671742b419ac2c1b7e5b8eb9164a2` integrated into main.
Diagnostic 12/12 with unchanged cycles; live median 2.701 us/word (118.47x hardware).

## Completed scope

PC, bank, delayed ROM, both return addresses and stack pointer now live in a
retained optional ACT endpoint state. Production installs it together with the
M15U received-instruction pipeline. Explicit semantic/injected fixtures disable
both. Reset reconstructs power-on latches and preserves the selected composition.

The flow decoder now accepts those owned latches separately from the frozen ACT
non-flow inputs. A poisoned host PC/bank/delayed selection/return stack cannot
supply a successor. Carry, A dispatch, key contact code and instruction/data
classification retain their existing snapshot ownership; this change does not
claim independent storage for every ACT register yet.

The owned successor request feeds the ROM address serializer, including the
initial fetch-only word. M14I preview is available before the b16..b27 address
window; completion at b55 retains the new flow state and applies the existing
low-page bank-zero normalization. Only after comparison does the architectural
mirror receive that completed result. Independent ROM bank latches retain their
own role and no provider-bank override is introduced.

Live compares every retained flow field before pipeline advancement or the
following-word DATA tail. A second check after commit catches failed handoffs.
No heap allocation or complete ACT clone is added per serial bit.

## Evidence boundary

This preserves M14I_CONTROL_FLOW_AUTHORITY's established behavioral contract:
sequential increment/wrap, JSB/GOTO, implied-GOTO, return, key/A dispatch,
immediate/delayed ROM selection and bank toggling. It retains the source-backed
b16..b27 address window and one-word pipeline, including the 0068 -> 0fc6 startup
landmark. Existing HARDWARE_SOURCES and M15U references apply unchanged.

Retained device ownership improves causality; it does not establish an internal
physical PC, bank or return-stack latch edge. Preview and b55 publication remain
WORKING APPROXIMATION. Exact PHI launch/sample timing, propagation, physical
RAM mapping/output stage, display pulses and CRC magnetic serialization remain
SOURCE-BLOCKED. No new hardware timing value is introduced.

## Acceptance

Six new regressions cover all 1024 words across multiple PC/carry/qualifier/
delayed-ROM contexts against the composed oracle, with all host flow fields
poisoned. Oracle-rejected words remain excluded rather than assigned semantics.
Received control instructions cover calls, returns, dispatch, ROM pages, banks
and implied GOTO; latches must remain unchanged through b54 and commit at b55.
Invalid coordinates cannot commit early.

Native tests compare reference with traced/untraced production through firmware
boot and physical key contacts, require actual ROM-received address agreement,
and check reset. Each poisoned flow mirror field fails before a pending DATA
tail can advance. A serializer-only test below the live guard demonstrates that
startup address bits come from the owned PC even with a wrong mirror PC/bank.

Actions is formatting-only. Compilation, runtime tests, release, 12/12 diagnostic
and all electrical/DATA/live benchmarks remain the owner's local gate.
