# HP-67 host-authority audit v1.0

Base: 60ad7ec, PAC integration branch. Static review of the desktop live path, reader and media adapters, PAC presentation, electrical timing boundaries and local gate. This is a targeted source audit, not hardware certification.

| Finding | Change | Boundary |
|---|---|---|
| The host looked at the other track's contents and write protection to decide whether to offer reinsertion on Crd. | Offer the labelled physical gesture on a firmware prompt. Allow protected and blank media; the firmware and reader handle them. | No automatic insertion. |
| Switching off always returned the card to Idle/End1, even in the passive holder. | Only move media when the reader actually returns a card. Keep its orientation. | Original power-off reader recovery retained. |
| Failed insertion of a new blank discarded the prepared media and face. | Attempt the physical insertion before committing the new slot state. | Rejection leaves the old slot intact. |
| Successive HPP drops were merged by title. Real Artillery sides have different titles. | A single drop is independent; an explicit two-file drop checks complementary headers, calculator and bitmap ID before publishing the pair. | Nonstandard side mappings remain an explicit converter operation. |
| A matching title alone selected a handcrafted Moon Rocket Lander face. | Match complete native media to a unique scanned catalog entry and use its original artwork. | Unknown or ambiguous media use the generic face. |
| Turbo treated an isolated CRC clear as a wait; the UI could spin for the rest of its 8 ms budget after the scheduler yielded. | Require a repeated empty test at the same observed site. Propagate the yield to the UI. | Host pacing only. No timeout override, forced motor or skipped PHI transitions. |
| A fatal live/selector error was sent to stderr and the machine object was dropped, including any inserted card. | Stop execution, retain the actual media for saving, and show the fault until a successful restart. | Recovery does not certify a partially written track. |
| The validation gate formatted the checkout before checking formatting. | Run the check without rewriting files. | Branch formatting remains a separate workflow. |

Reproduction: the two Artillery HPP headers identify complementary tracks and share GA1-05A.bmp; their titles include different side numbers. Offline conversion produces Git blob 49e7958abba7bb21f195b573b4f2c5382f38bef8, identical to the native catalog card. The Rust regression exercises both drop orders and rejects duplicate or mismatched parts.

## Unresolved hardware evidence

The head-arrival model still uses two startup buffer clears as its integration reference. It is not a measurement of switch geometry; replacing it requires correlated insertion, motor and head-contact evidence. PHI-relative sampling, DATA output-stage characteristics and magnetic serialization also remain unresolved. The boot PC checkpoints observed in the live adapter identify boot progress; reaching Idle does not stop instruction execution. None of these observations authorizes bypassing the ROM. The architectural oracle and explicitly labelled synthetic benchmarks were kept separate from hardware evidence.

## Validation

Review the branch diff and run the Windows gate (format check, focused tests, full Rust suite, release, diagnostics and isolated EXE). For a visual check: park a card in the holder, power-cycle and check orientation; then load a two-part card together and reinsert on Crd. Gate success and visual verification are not asserted by this source audit.
