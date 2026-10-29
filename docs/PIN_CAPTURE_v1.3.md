# HP-67 pin captures v1.3 — MTRS input

M16H appends MTRS at net index 15, following MOTOR. Full exports now contain sixteen
named pins, retaining every earlier net/driver index. Metadata remains hp67-pins-v1:
the named optional-column parser is already extensible. See PIN_CAPTURE_v1.2.md.

MTRS uses the existing External physical-stimulus slot. The normalized digital
bridge drives High for open and Low for card-insertion contact closed. Ground-on-
insertion is backed by the inspected service evidence; idle drive construction,
bounce, voltage and contact/PHI sampling edges are not established by this bridge.
The reader head switch HDS remains distinct. Insertion must not close HDS early.

Physical contact events update the source without advancing simulation time.
Unchanged publication does not create a capture row. Resolved levels and contributor
masks preserve missing/conflicting sources, including agreeing extra contributors.
Older fifteen-pin (and earlier sparse) captures import absent MTRS as None for both
level and ownership; re-export writes unknown rather than open, High or Floating.
The existing comparison/timing CLI accepts MTRS through the named inventory.

Capture-only storage stays compiled out by default. Both builds perform the same
contact validation, CRC reception and 224 PHI transitions per word. Capture event
coordinates and boundary policy do not prove physical contact or propagation timing.
