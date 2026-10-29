# HP-67 pin captures v1.4 — HDS/WPS reader inputs

M16I appends HDS and WPS at indices 16 and 17 after MTRS, for eighteen named pins.
Every prior net/driver index stays stable. Metadata remains hp67-pins-v1 because
optional named columns already permit extension. See PIN_CAPTURE_v1.3.md.

Both logical sources use the existing External driver. Grounded HDS is head active.
Grounded WPS is a closed write-permission contact, not a protection assertion. CRC
samples permission at head closure, retains it for that pass and requires WPS
open/reclose before accepting another grounded pass. Reset arming is a logical
bridge policy. High/open normalization does not establish a physical pull-up,
voltage, bounce interval or propagation time.

A complete logical pair is published before one capture observation. This avoids
an artificial intermediate snapshot; it does not claim simultaneous physical
contact movement. Publication consumes no PHI tick, and unchanged state adds no
row. Ordinal delta order remains distinct from measured nanoseconds.

Older sixteen-pin or sparse captures import omitted HDS/WPS level/ownership as
None, re-exporting unknown rather than open, inactive or Floating. The comparison
and timing CLI resolves both names through the existing pin inventory. A missing
observation cannot prove equality or real timing. Default builds compile capture
storage/hooks out while retaining the same input validation and 224 transitions.
