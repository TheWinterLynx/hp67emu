# HP-67 pin captures v1.5 — PGMSW program selector

M16J appends PGMSW at net index 18 after WPS, giving nineteen named pins. Previous
net/driver indices are unchanged. Metadata remains hp67-pins-v1; optional named
columns already support extension. See PIN_CAPTURE_v1.4.md.

The grounded PROGRAM contact uses the existing External driver. Low selects
W-PRGM in the normalized bridge; High represents RUN. This does not determine the
physical idle bias, voltage, switch bounce or CRC/PHI sampling edge. The physical
selector survives electronic reset; capture/reset storage retains the existing
reset lifetime rather than fabricating a continuous pre/post-reset time axis.

Only changed positions add rows; publication advances no PHI tick. Both diagnostic
and normal builds validate the same input and execute 224 transitions per word.
Older eighteen-pin/sparse inputs leave PGMSW level/ownership None and re-export
unknown, not RUN/High/Floating. Comparison/timing CLI selects PGMSW through the
existing inventory; absent observations cannot establish equality or real timing.
