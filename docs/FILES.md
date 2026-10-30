# Per-file documentation index

The documentation regression uses these companion files as the source-level ownership map.

## Desktop/UI binary

- [src/main.rs](files/src/main.rs.md)
- [src/app.rs](files/src/app.rs.md)
- [src/hp67.rs](files/src/hp67.rs.md)
- [src/hp67/m12_tests.rs](files/src/hp67/m12_tests.rs.md)
- [src/panel.rs](files/src/panel.rs.md)
- [src/program_library.rs](files/src/program_library.rs.md)
- [src/ui/mod.rs](files/src/ui/mod.rs.md)
- [src/ui/classic_display.rs](files/src/ui/classic_display.rs.md)
- [src/ui/program_card.rs](files/src/ui/program_card.rs.md)
- [src/ui/sliders.rs](files/src/ui/sliders.rs.md)
- [src/ui/top_keys.rs](files/src/ui/top_keys.rs.md)

## Reusable library

- [src/lib.rs](files/src/lib.rs.md)
- [src/emulation/mod.rs](files/src/emulation/mod.rs.md)
- [src/emulation/clock.rs](files/src/emulation/clock.rs.md)
- [src/emulation/net.rs](files/src/emulation/net.rs.md)
- [src/emulation/device.rs](files/src/emulation/device.rs.md)
- [src/emulation/trace.rs](files/src/emulation/trace.rs.md)
- [src/machines/mod.rs](files/src/machines/mod.rs.md)
- [src/machines/hp67/act_serial_boundary.rs](files/src/machines/hp67/act_serial_boundary.rs.md)
- [src/machines/hp67/act_serial_crc_data.rs](files/src/machines/hp67/act_serial_crc_data.rs.md)
- [src/machines/hp67/act_serial_crc_control.rs](files/src/machines/hp67/act_serial_crc_control.rs.md)
- [src/machines/hp67/act_serial_ram.rs](files/src/machines/hp67/act_serial_ram.rs.md)
- [src/machines/hp67/act_serial_flow.rs](files/src/machines/hp67/act_serial_flow.rs.md)
- [src/machines/hp67/mod.rs](files/src/machines/hp67/mod.rs.md)
- [src/machines/hp67/card_transport.rs](files/src/machines/hp67/card_transport.rs.md)
- [src/machines/hp67/wiring.rs](files/src/machines/hp67/wiring.rs.md)
- [src/machines/hp67/machine.rs](files/src/machines/hp67/machine.rs.md)
- [src/machines/hp67/display_snapshot.rs](files/src/machines/hp67/display_snapshot.rs.md)

- [src/hp67/authority_benchmark_tests.rs](files/src/hp67/authority_benchmark_tests.rs.md)

## Regression tests

- [tests/documentation_contract.rs](files/tests/documentation_contract.rs.md)
- [tests/architecture_contract.rs](files/tests/architecture_contract.rs.md)

## Development tools

- [tools/gate_m16g.ps1](files/tools/gate_m16g.ps1.md)

- [tools/convert_hp67_card.py](files/tools/convert_hp67_card.py.md)
- [tools/extract_hp67_card_artwork.py](files/tools/extract_hp67_card_artwork.py.md)
- [tools/generate_hp67_diagnostic_pacs.py](files/tools/generate_hp67_diagnostic_pacs.py.md)

## Media format reference

- [HP-67 program media formats](HP67_CARD_FORMATS.md)

- [src/machines/hp67/display_exposure.rs](files/src/machines/hp67/display_exposure.rs.md)

## M16D keyboard connectivity

- [src/machines/hp67/keyboard_matrix.rs](files/src/machines/hp67/keyboard_matrix.rs.md)
- [src/machines/hp67/keyboard_matrix_tests.rs](files/src/machines/hp67/keyboard_matrix_tests.rs.md)

## M16E SYNC source integrity

- [src/machines/hp67/sync.rs](files/src/machines/hp67/sync.rs.md)
- [src/machines/hp67/sync_tests.rs](files/src/machines/hp67/sync_tests.rs.md)
- [src/hp67/sync_integrity_tests.rs](files/src/hp67/sync_integrity_tests.rs.md)

- [src/machines/hp67/sync_cache_tests.rs](files/src/machines/hp67/sync_cache_tests.rs.md)

## M16F IS window source identity

- [src/machines/hp67/is_owner_tests.rs](files/src/machines/hp67/is_owner_tests.rs.md)
- [src/hp67/is_owner_tests.rs](files/src/hp67/is_owner_tests.rs.md)

## M16G resolved MOTOR

- [src/machines/hp67/motor.rs](files/src/machines/hp67/motor.rs.md)
- [src/machines/hp67/motor_tests.rs](files/src/machines/hp67/motor_tests.rs.md)
- [src/hp67/motor_tests.rs](files/src/hp67/motor_tests.rs.md)

## M16H resolved card input

- [src/machines/hp67/card_contact.rs](files/src/machines/hp67/card_contact.rs.md)
- [src/machines/hp67/card_contact_tests.rs](files/src/machines/hp67/card_contact_tests.rs.md)
- [src/hp67/card_contact_tests.rs](files/src/hp67/card_contact_tests.rs.md)
- [tools/gate_m16h.ps1](files/tools/gate_m16h.ps1.md)

## M16I head and write-permission inputs

- [src/machines/hp67/reader_contacts.rs](files/src/machines/hp67/reader_contacts.rs.md)
- [src/machines/hp67/reader_contacts_tests.rs](files/src/machines/hp67/reader_contacts_tests.rs.md)
- [src/hp67/reader_contacts_tests.rs](files/src/hp67/reader_contacts_tests.rs.md)
- [tools/gate_m16i.ps1](files/tools/gate_m16i.ps1.md)

## M16J physical RUN/W-PRGM input

- [src/machines/hp67/program_switch.rs](files/src/machines/hp67/program_switch.rs.md)
- [src/machines/hp67/program_switch_tests.rs](files/src/machines/hp67/program_switch_tests.rs.md)
- [src/hp67/program_switch_tests.rs](files/src/hp67/program_switch_tests.rs.md)
- [tools/gate_m16j.ps1](files/tools/gate_m16j.ps1.md)


## M16K peripheral transactions

- `src/machines/hp67/crc_transaction_tests.rs`: prior-copy parity and fault/retry/kernel comparison; companion under docs/files.
- `tools/gate_m16k.ps1`: full owner gate and additional CRC kernel; companion under docs/files.


## M16L pin integrity

- src/machines/hp67/pin_integrity.rs and pin_integrity_tests.rs: offline production source/coverage audit and regressions.
- src/integrity_cli_tests.rs: native file/exit-status regressions.
- tools/gate_m16l.ps1: owner nine-stage gate, exact tested SHA.
Each has a docs/files companion. See PIN_CAPTURE_v1.6.md.
