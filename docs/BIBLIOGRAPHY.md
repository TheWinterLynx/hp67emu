# HP-67 emulation bibliography

Research snapshot: 2026-09-14.

This bibliography is the working source set for the cycle-accurate, electrically modelled HP-67 emulator. `HARDWARE_SOURCES.md` defines the evidence hierarchy; this file records the concrete references and what each is useful for.

## 1. Original HP-67 microcode / ROM provenance

### Tony Nixon / Teenix ROM reader project

- ROM reader discussion: https://www.hpmuseum.org/forum/thread-18327-page-2.html
- ROM reader archive: https://www.teenix.org/ROMreader.zip
- Teenix calculator emulator home: https://www.teenix.org/
- HP-67 module: https://www.teenix.org/HP67.zip

Tony Nixon states that `ROMreader.zip` contains ROM files for both HP-97 and HP-67 read with his hardware ROM reader. This is currently the strongest publicly downloadable candidate for our canonical raw HP-67 ROM corpus because its provenance is tied to reads of physical calculator ICs rather than a transcription or reconstructed listing.

Do not commit those ROM bytes to this repository until we have inspected the archive, identified every image/part/bank, recorded SHA-256 values and made a separate redistribution decision.

### Independent original-microcode implementations for cross-checking

- Panamatik HP-67 emulator: https://www.panamatik.de/html/hp-67.html
- Panamatik downloads / ACT material: https://www.panamatik.de/html/download.html
- Sydney Smith HP-67 overview: https://www.sydneysmith.com/wordpress/hp67-main/
- Sydney Smith HP67u/HP67w background: https://www.sydneysmith.com/wordpress/hp67/
- Sydney Smith research articles: https://www.sydneysmith.com/wordpress/articles/

Panamatik explicitly describes its HP-67 emulator as executing the original HP-67 microcode. Sydney Smith provides a separate HP-67 microcode emulator and extensive address-level analyses. Neither should be copied blindly; they are independent cross-checks for the raw ROM dump and our own decoder.

## 2. HP-67 electrical hardware

### HP-67 logic-board schematic

- https://www.hpcc.org/cdroms/schematics5.1/handhelds/classic/hp67.pdf
- Schematics collection index: https://www.hpcc.org/cdroms/schematics5.1/index.html

Primary working reference for HP-67 board connectivity and named signals. Use it when transcribing the machine netlist and chip pin connections.

### HP-67 physical board and IC inventory

- https://www.keesvandersanden.nl/calculators/hp67_inside.php

Confirms the physical chip set and board layout, including 1820-2530 ACT, 1818-0268 ROM/display-anode driver, 1820-1749 cathode driver, 1820-1751 CRC, ROM/RAM devices and card-reader electronics.

## 3. Timing, serial buses, display scan and HP-67-specific reverse engineering

### Tony Nixon, *Notes on HP's Classic Calculators*

- Archive record: https://literature.hpcalc.org/items/2123
- PDF: https://literature.hpcalc.org/community/classic-notes.pdf

Despite the title, this contains substantial HP-67/Woodstock material: ACT/ROM serial traffic, 56-bit cycles, instruction/address flow, ROM bank selection, display RCD/STR behaviour, card-reader/CRC details and scope-derived waveforms. It also records that the HP-67 and HP-97 microcode is identical from ROM address `$400` through `$FFF`.

This is one of the most important practical sources for cycle timing, but it is reverse engineering rather than an HP internal design specification; measurements and uncertain statements should be corroborated whenever possible.

### HP-97 Service Manual

- Archive record: https://literature.hpcalc.org/items/1127
- PDF: https://literature.hpcalc.org/community/hp97-sm-en.pdf

The HP-97 is the desktop sibling of the HP-67 and shares a large amount of architecture and microcode. The service manual contains unusually detailed service-level theory, ROM/RAM organisation, display and card-reader information. Use only where HP-67 equivalence is established; printer-only HP-97 behaviour must remain machine-specific.

## 4. HP engineering publications

### *Inside the New Pocket Calculators* — Hewlett-Packard Journal, November 1975

- https://www.keesvandersanden.nl/calculators/hp_journals/HP_Journal_7511_Three_New_Pocket_Calculators_Smaller_Less_Costly_More_Powerful.pdf

Contemporary HP description of the second-generation "Woodstock" architecture used by the HP-21/22/25 family and forming the architectural basis relevant to the HP-67.

### *A Pair of Program-Compatible Personal Programmable Calculators* — Hewlett-Packard Journal, November 1976

- https://www.keesvandersanden.nl/calculators/hp_journals/HP_Journal_7611_A_Pair_of_Program-Compatible_Personal_Programmable_Calculators.pdf

Original HP article introducing the HP-67 and HP-97. Useful for design intent, memory/program organisation, card-reader behaviour and the documented relationship between both machines.

### HP Journal calculator article index

- https://www.keesvandersanden.nl/calculators/hpjournal.php

Useful index to other contemporary calculator and magnetic-card articles.

## 5. Historical reverse engineering of HP-67 buses and ROM flow

### Jacques Laporte archive — ROM dump

- ROM dump article: https://archived.hpcalc.org/laporte/ROM%20DUMP.htm
- Complete archived site: https://archived.hpcalc.org/laporte.zip

Documents reconstruction of ROM instruction/address flow from serial buses, including packing/inversion and SYNC use. It also points back to Tom Napier's 1978 physical HP-67 bus-capture work.

### Tom Napier, *An HP-67 Anatomy Lesson*, PPC Journal (1978)

Bibliographic trail:

- PPC Journal V5 N7, pp. 7-8
- continuation V5 N8, pp. 14-17
- continuation V5 N10, pp. 25-27
- Jacques Laporte reference: https://archived.hpcalc.org/laporte/ROM%20DUMP.htm

Napier interfaced a real HP-67 to an 8080 system and captured ISA/DATA serial traffic. We should locate scans of the actual PPC issues before treating any secondary summary as implementation evidence.

### Tony Duell, *HP67 Internals*, Datafile V23 N4 (July/August 2004), p. 9

- HPCC Datafile volume index: https://www.hpcc.org/datafile/datafilev23.html

Important repair/internal-hardware article. The index proves the article exists, but an openly accessible article PDF has not yet been located in this research pass.

## 6. Reference emulator implementation — Woodstock family

### Nonpareil / Eric Smith

- Repository: https://github.com/brouhaha/nonpareil
- Key source files: `src/proc_woodstock.c`, `src/proc_woodstock.h`, `src/dis_woodstock.c`, `src/crc.c`, `src/pick.c`

Nonpareil is a mature microcode-level HP calculator simulator and an excellent architecture/reference implementation for Woodstock processor semantics and peripheral modelling. Its currently advertised model set does not make it our canonical HP-67 ROM source. We use it to challenge our interpretation, not to substitute for physical HP-67 evidence.

## 7. HP-67 behavioural acceptance references

### HP-67 Owner's Handbook and Programming Guide

- Archive record: https://literature.hpcalc.org/items/979
- PDF: https://literature.hpcalc.org/community/hp67-oh-en.pdf

Use for functional acceptance tests, documented user-visible timing/behaviour, programming semantics and magnetic-card workflows. It is not a hardware timing source.

### HP-67 Quick Reference Card

- https://literature.hpcalc.org/items/983

Useful for keyboard/function coverage tests.

## 8. Background patents

- US 3,863,060: https://patents.google.com/patent/US3863060A/en
- US 4,001,569: https://patents.google.com/patent/US4001569A/en

These document earlier HP calculator architecture and are useful background for serial arithmetic/register concepts and processor lineage. They are not sufficient to establish HP-67-specific timing or wiring by themselves.

## 9. Source priority for implementation decisions

1. Physical HP-67 ROM dumps and HP-67 schematic/board evidence.
2. Scope-derived HP-67 traces and HP-67-specific reverse engineering.
3. HP-97 service documentation only where the shared implementation is established.
4. Contemporary HP engineering articles.
5. Independent microcode emulators and Nonpareil as cross-checks.
6. Owner manuals for user-visible acceptance testing.

If two sources disagree, record the conflict. Do not silently select the result that is easiest to implement.