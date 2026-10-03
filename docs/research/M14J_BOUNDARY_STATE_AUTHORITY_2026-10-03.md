# M14J — remaining boundary-state authority

Date: 2026-10-03
Branch: `agent/m14j-boundary-state-authority`
Status: owner reported `M14J FULL GATE GREEN` on 2026-10-03; integrated into main under the standing green-gate authorization.

## Scope

Move P-change history, previous-carry capture and non-arithmetic carry reset for words outside M14F onto completed-word structural authority. Move arithmetic Normal/ThenGoto selection onto independently decoded structural authority. Operations 0x16..0x1b enter ThenGoto; their arithmetic carry result stays with M14E.

## Ownership

- M14F selected control family: no M14J image, preserving one owner for P/status/conditions.
- Arithmetic words: M14J owns P-change history, previous carry and instruction-state selection; M14E owns carry/register results.
- Other words: M14J owns P-change history, previous carry and carry reset. M14I retains implied-GOTO completion. CRC status/electronics and other unrelated effects are excluded.

The live bridge captures the oracle's expected owned fields, restores them before transport and commits only a matching complete image after b55. Ownership masks are shared by restore, comparison and commit, so a boundary commit cannot overwrite another family's carry or instruction-state effects.

## Evidence boundary

These are established ACT instruction-boundary semantics independently recomputed from the same real pre-instruction inputs. The complete structural word remains the authority handoff, not a physical write-edge claim. Exact P-history, carry and instruction-state internal PHI timing remains SOURCE-BLOCKED. RAM address family effects, bulk RAM clear, CRC electronics, DATA ownership and true intra-word mutation remain outside this slice.

## Validation

All 1024 words are compared over both carry states, normal/implied-GOTO interpretation and three P-change histories. Unsupported composed-oracle operations remain unsupported. Dedicated tests lock M14F exclusion, M14E carry preservation, b55 completion, next-word ownership transition and live boot. Full owner formatting/warnings/all-targets/release plus diagnostic 12/12 is required before merge. GitHub Actions executes cargo fmt only.
