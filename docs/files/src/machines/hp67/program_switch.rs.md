# src/machines/hp67/program_switch.rs

## Purpose
Physical RUN/W-PRGM selector source and strict resolved CRC /PGMSW receiver.

## Why it exists
The CPU must receive the physical selector through the chip input rather than import an architectural PROGRAM Boolean. A mechanical selector cannot move on electronic reset.

## Relationships
Appends PgmSw to the existing net inventory, uses External stimulus ownership, feeds serial_crc_control and is retained by Hp67LiveMachine. UI still calls the same physical set_program_mode API.

## Responsibilities
Publish changed positions, preserve prior source identity/level on failure, reject missing/replaced/agreeing foreign sources and retain physical position for reset reconstruction.

## Implementation
HP-67 schematic keyboard/display sheet 1 and logic sheet 2 show the grounded PGM contact and /PGMSW connection to CRC SEL. Low means PROGRAM in the normalized bridge; High/RUN idle drive construction and exact sampling/bounce/voltage remain approximations. program_selected exposes physical position only for mechanical persistence; CPU reception uses the conductor.
