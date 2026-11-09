# Native card persistence

## Purpose
Save serialized host card bytes without truncating the previous destination.

## Why it exists
An in-memory dirty flag cannot protect a file truncated by a failed direct write.

## Relationships
Native-only binary module called by `app.rs`; has no chip/firmware dependencies.
Uses the standard filesystem replacement API documented at
https://doc.rust-lang.org/std/fs/fn.rename.html .

## Responsibilities
Create an exclusive sibling temporary file, finish writing and syncing it, close
its handle and rename over the destination. Never delete the old destination as a
replacement workaround. Reject pre-existing links/directories/read-only targets.

## Implementation
PID plus a bounded collision loop selects a name using create_new, so stale names
are not overwritten. The guard cleans failed temporary writes and is disarmed on
successful rename. Existing regular-file permission bits are copied. Hard-linked
names are replaced independently; other links keep their old bytes. Extended ACLs,
metadata identity, hostile concurrent path replacement and power-loss directory
entry durability are not guaranteed. Windows sharing/permission failures return
an error; the UI keeps the card dirty. No directory sync or crash-durability claim.
The regression injects a partial-write error, checks existing/absent destination
success and forces rename failure using a directory created after preflight.
