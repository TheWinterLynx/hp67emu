# src/machines/hp67/reader_contacts.rs

## Purpose
Resolved HDS/WPS contact source, strict receiver and retained CRC head/write permission.

## Why it exists
The transport must consume reader electronics, rather than use media metadata as its production write-inhibit authority. HDS and MTRS are separate contacts; WPS is evaluated at head arrival.

## Relationships
Uses existing External driver, appended Hds/Wps nets, atomic fabric/backplane publication, the owned transport port and the live card lifecycle. It does not parse media formats or UI state.

## Responsibilities
Validate both sole contributors before mutation, preserve faults, publish a complete logical pair, sample WPS on HDS closure and retain permission until the head opens.

## Implementation
Grounded HDS is active. Grounded WPS permits writing, backed by HP-97 service 2-37/2-39 and note 97-101. High/open normalization, joint publication and the existing logical arrival point are working approximations. No bounce, idle bias, geometry, physical contact ordering, voltage or PHI timing is inferred. The receiver reads pins; media metadata is not a receiver argument.

Service note 97-101 also requires opening/reclosing grounded WPS between passes. The receiver consumes the accepted WPS closure and requires an observed open before a later head closure can grant permission. A dedicated regression rejects held-ground reuse and accepts a real reopen/reclose. Reset begins armed as an explicit bridge initialization policy; no physical power-on waveform is claimed.
