# M14R — Moving Average data-card pass boundary

Date: 2026-10-04
Branch: `agent/m14r-data-card-pass-boundary`
Status: implemented; owner local validation pending.

## Source and new fixtures

The supplied HP-97 Standard Pac manual PDF page 15 / printed 01-02 was visually inspected. Its general Moving Average instructions specify shifted A initialization, A data input, B data save, D mean query, and two card passes for windows of ten or more. This is Tier B functional corroboration for the existing HP-67 SD1-01A media, not proof of electrical equivalence.

The new cases are independently constructed tests using those instructions; they are not printed official HP examples. Nine-point input 1 through 9 has mean 45/9 = 5.00; next input 10 replaces 1, giving 54/9 = 6.00. Ten-point input 1 through 10 has mean 55/10 = 5.50; next input 11 gives 65/10 = 6.50. Fixed expected strings exist only in the test oracle. Core code performs no host averaging. All initial data are distinct/nonzero to exercise saved-window recovery.

## Required path

Each writer starts from a fresh firmware boot and loads both program ends. Inputs traverse physical keyboard dispatch and stable raw-segment checks. B must produce firmware Crd/write mode. Blank End1 becomes data header 1 with a dirty primary track and an untouched blank opposite track. The nine-point writer must finish without another request. The ten-point writer must produce real Crd before the same returned card is inserted End2; header 2 and dirty secondary media are required, and all 34 primary words must remain identical. Last-pass idle and empty CRC write queue are required.

Only native physical-card bytes cross into a separately booted reader. Both program ends are reloaded. Data End1 and, for the ten-point case, firmware-requested End2 are read explicitly, with read mode asserted and the returned complete card equal to its pre-read image after each pass. Then the reader must settle without an extra prompt. D verifies the recovered mean, next A input verifies the rolling mean, and another D query verifies the result persists. No calculator registers, RAM, PC or display values are injected.

## Validation and limits

Production execution is unchanged. The existing card waits and one-million-word result bound apply. Actions is restricted to cargo fmt/check/save. Source diff and companion documentation contract were reviewed; Rust was not run in the agent environment. Owner gate runs warnings-denied full tests/release, diagnostics 12/12, existing M14P/M14Q and the explicit ignored M14R boundary test (2/2 cases). Successful execution is pending.

Fresh construction tests electronic-state isolation, not OFF-switch or analog reset timing. Native container equality tests host persistence, not physical serialization. Logical CRC/card transport, firmware results and preserved media do not close PHI/DATA/STR/RCD edges, physical RAM partitioning or magnetic sense/flux-order gaps. Next priority is other Standard/Games Pac applications after this boundary is validated.
