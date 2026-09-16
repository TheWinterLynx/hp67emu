from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old not in text:
        raise SystemExit(f"anchor not found: {label}")
    return text.replace(old, new, 1)

# -----------------------------------------------------------------------------
# ACT: own the A/B -> b0..b7 word serializer.  This serializes directly from
# latched architectural nibbles and deliberately does not construct a byte.
# -----------------------------------------------------------------------------
act_path = Path("src/machines/hp67/act.rs")
act = act_path.read_text()
act = replace_once(
    act,
    "use super::isa::{ROM_ADDRESS_MASK, ROM_WORD_MASK};\n",
    "use crate::emulation::Drive;\n\nuse super::{\n    display::{display_role_for_scan_slot, Hp67DisplayRole},\n    isa::{wired_high_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK},\n    timing::display_data_serial_bit,\n};\n",
    "act imports",
)
anchor = "pub type ActRegister = [u8; ACT_WORD_DIGITS];\n"
insert = r'''pub type ActRegister = [u8; ACT_WORD_DIGITS];

/// Map one observed HP-67 display scan slot to the ACT A/B register digit that
/// supplies that slot's serial display data.
///
/// This mapping is structural and source-backed: exponent units/tens occupy
/// register digits 0/1, the shared signs occupy digit 2, mantissa digits 1..11
/// occupy digits 3..13, and scan slot 15 repeats exponent units.
pub const fn display_register_index_for_scan_slot(scan_slot: u8) -> Option<usize> {
    match display_role_for_scan_slot(scan_slot) {
        Some(Hp67DisplayRole::ExponentUnits | Hp67DisplayRole::ExponentUnitsDuplicate) => Some(0),
        Some(Hp67DisplayRole::ExponentTens) => Some(1),
        Some(Hp67DisplayRole::SharedSigns) => Some(2),
        Some(Hp67DisplayRole::MantissaDigit(digit)) => Some((digit + 2) as usize),
        None => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActDisplaySerialError {
    InvalidScanSlot(u8),
}

/// ACT-side source for the eight ROM0 display bits of one 56-bit machine word.
///
/// The endpoint snapshots only the two source nibbles for the selected scan
/// position. It never composes an eight-bit display value. b0..b3 are emitted
/// directly from A bit 0..3 and b4..b7 directly from B bit 0..3, using the
/// observed wired-high/release IS convention.
///
/// This is still a word-boundary bridge: A/B are architectural register arrays,
/// not the final intra-word shift-register/ALU implementation. Exact PHI launch
/// edges remain intentionally unspecified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActDisplayWordSerializer {
    register_index: usize,
    a_nibble: u8,
    b_nibble: u8,
}

impl ActDisplayWordSerializer {
    pub fn from_state(
        scan_slot: u8,
        state: &ActArchitecturalState,
    ) -> Result<Self, ActDisplaySerialError> {
        let register_index = display_register_index_for_scan_slot(scan_slot)
            .ok_or(ActDisplaySerialError::InvalidScanSlot(scan_slot))?;
        debug_assert!(register_index < ACT_WORD_DIGITS);
        Ok(Self {
            register_index,
            a_nibble: state.a[register_index] & 0x0f,
            b_nibble: state.b[register_index] & 0x0f,
        })
    }

    pub const fn register_index(&self) -> usize {
        self.register_index
    }

    /// Drive the ACT's display contribution for this word bit.
    pub const fn drive_for_bit(&self, word_bit: u8) -> Drive {
        let Some(serial_bit) = display_data_serial_bit(word_bit) else {
            return Drive::HighZ;
        };
        let bit = if serial_bit < 4 {
            ((self.a_nibble >> serial_bit) & 1) != 0
        } else {
            ((self.b_nibble >> (serial_bit - 4)) & 1) != 0
        };
        wired_high_drive(bit)
    }
}
'''
act = replace_once(act, anchor, insert, "ACT register alias")
# Add focused tests before the final test-module closing brace, anchored on an
# existing test near the bottom so this is deterministic.
test_anchor = "#[cfg(test)]\nmod tests {\n    use super::*;\n"
act = replace_once(
    act,
    test_anchor,
    test_anchor
    + r'''
    #[test]
    fn display_scan_slots_select_the_evidenced_act_digits() {
        assert_eq!(display_register_index_for_scan_slot(1), Some(0));
        assert_eq!(display_register_index_for_scan_slot(2), Some(1));
        assert_eq!(display_register_index_for_scan_slot(3), Some(2));
        assert_eq!(display_register_index_for_scan_slot(4), Some(13));
        assert_eq!(display_register_index_for_scan_slot(14), Some(3));
        assert_eq!(display_register_index_for_scan_slot(15), Some(0));
        assert_eq!(display_register_index_for_scan_slot(0), None);
        assert_eq!(display_register_index_for_scan_slot(16), None);
    }

    #[test]
    fn display_serializer_reads_a_then_b_without_composing_a_byte() {
        let mut state = ActArchitecturalState::default();
        state.a[13] = 0x0a;
        state.b[13] = 0x05;
        let serializer = ActDisplayWordSerializer::from_state(4, &state).unwrap();
        assert_eq!(serializer.register_index(), 13);
        let expected_high = [false, true, false, true, true, false, true, false];
        for (word_bit, high) in expected_high.into_iter().enumerate() {
            assert_eq!(
                serializer.drive_for_bit(word_bit as u8),
                wired_high_drive(high)
            );
        }
        assert_eq!(serializer.drive_for_bit(8), Drive::HighZ);
    }

    #[test]
    fn reset_act_nibbles_release_all_eight_display_bits() {
        let state = ActArchitecturalState::default();
        for scan_slot in 1..=15 {
            let serializer = ActDisplayWordSerializer::from_state(scan_slot, &state).unwrap();
            for word_bit in 0..8 {
                assert_eq!(serializer.drive_for_bit(word_bit), Drive::HighZ);
            }
        }
    }
''',
    "ACT test module",
)
act_path.write_text(act)

# -----------------------------------------------------------------------------
# Diagnostic snapshot: consume the ACT-owned mapping, but keep the whole-byte
# helper explicitly as a diagnostic/reference bridge only.
# -----------------------------------------------------------------------------
snap_path = Path("src/machines/hp67/display_snapshot.rs")
snap = snap_path.read_text()
snap = replace_once(
    snap,
    "    act::{ActRegister, ACT_WORD_DIGITS},\n",
    "    act::{display_register_index_for_scan_slot, ActRegister, ACT_WORD_DIGITS},\n",
    "snapshot ACT import",
)
start = snap.find("/// Map the observed HP-67 display scan order back to the ACT's 14-nibble register layout.")
end = snap.find("/// Compose the ROM0 display byte from one A/B register position.", start)
if start < 0 or end < 0:
    raise SystemExit("snapshot mapping block not found")
snap = snap[:start] + snap[end:]
snap_path.write_text(snap)

# -----------------------------------------------------------------------------
# ISA: remove the whole-byte ACT display driver.  Only generic wired-high drive,
# address serialization and ROM serialization remain here.
# -----------------------------------------------------------------------------
isa_path = Path("src/machines/hp67/isa.rs")
isa = isa_path.read_text()
isa = replace_once(
    isa,
    "use super::timing::{display_data_serial_bit, isa_window_for_bit, IsaWindow};\n",
    "use super::timing::{isa_window_for_bit, IsaWindow};\n",
    "ISA timing import",
)
start = isa.find("/// Drive contribution the ACT display path should make on IS for the eight-bit")
end = isa.find("/// Drive contribution the ACT should make on IS for a ROM address", start)
if start < 0 or end < 0:
    raise SystemExit("act_display_drive block not found")
isa = isa[:start] + isa[end:]
start = isa.find("    #[test]\n    fn eight_bit_display_code_is_emitted_lsb_first_on_b0_through_b7()")
end = isa.find("    #[test]\n    fn twelve_bit_address_is_emitted_lsb_first_on_b16_through_b27()", start)
if start < 0 or end < 0:
    raise SystemExit("act_display_drive test block not found")
isa = isa[:start] + isa[end:]
# Replace overlap test with the roles that still live in ISA.
old = r'''    #[test]
    fn display_address_and_rom_response_roles_do_not_overlap() {
        for bit in 0..56 {
            let display = act_display_drive(0xff, bit);
            let address = act_address_drive(0x0fff, bit);
            let rom = rom_word_drive(0x03ff, bit);
            let active = [display, address, rom]
                .into_iter()
                .filter(|drive| *drive == Drive::High)
                .count();
            assert!(active <= 1, "multiple active IS roles at b{bit}");
        }
    }
'''
new = r'''    #[test]
    fn address_and_rom_response_roles_do_not_overlap() {
        for bit in 0..56 {
            let address = act_address_drive(0x0fff, bit);
            let rom = rom_word_drive(0x03ff, bit);
            let active = [address, rom]
                .into_iter()
                .filter(|drive| *drive == Drive::High)
                .count();
            assert!(active <= 1, "multiple active IS fetch roles at b{bit}");
        }
    }
'''
isa = replace_once(isa, old, new, "ISA overlap test")
isa_path.write_text(isa)

# -----------------------------------------------------------------------------
# Shared word transport: one ACT serial endpoint owns display drive, address
# drive and returned-word sampling.  No display byte enters the runner.
# -----------------------------------------------------------------------------
fetch_path = Path("src/machines/hp67/fetch.rs")
fetch = fetch_path.read_text()
fetch = replace_once(
    fetch,
    "use super::{\n    display::{Rom0DisplayEndpoint, Rom0DisplayError},\n    isa::{act_address_drive, act_display_drive, rom_word_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK},\n",
    "use super::{\n    act::{ActArchitecturalState, ActDisplaySerialError, ActDisplayWordSerializer},\n    display::{Rom0DisplayEndpoint, Rom0DisplayError},\n    isa::{act_address_drive, rom_word_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK},\n",
    "fetch imports",
)
fetch = replace_once(
    fetch,
    "pub enum StructuralWordError {\n    Fetch(SerialFetchError),\n    Display(Rom0DisplayError),\n}\n",
    "pub enum StructuralWordError {\n    Fetch(SerialFetchError),\n    Display(Rom0DisplayError),\n    ActDisplay(ActDisplaySerialError),\n}\n",
    "StructuralWordError",
)
fetch = replace_once(
    fetch,
    "impl From<Rom0DisplayError> for StructuralWordError {\n    fn from(error: Rom0DisplayError) -> Self {\n        Self::Display(error)\n    }\n}\n",
    "impl From<Rom0DisplayError> for StructuralWordError {\n    fn from(error: Rom0DisplayError) -> Self {\n        Self::Display(error)\n    }\n}\n\nimpl From<ActDisplaySerialError> for StructuralWordError {\n    fn from(error: ActDisplaySerialError) -> Self {\n        Self::ActDisplay(error)\n    }\n}\n",
    "ActDisplay error conversion",
)
# Replace the ACT endpoint block wholesale.
start = fetch.find("/// ACT-side state for one 56-bit ROM fetch cycle.")
end = fetch.find("/// ROM-side state for one 56-bit serial fetch cycle.", start)
if start < 0 or end < 0:
    raise SystemExit("ACT endpoint block not found")
endpoint = r'''/// ACT-side state for one structural 56-bit machine word.
///
/// One endpoint owns every currently modeled ACT role on IS: optional display
/// serialization at b0..b7, ROM address serialization at b16..b27, and ROM-word
/// reception at b46..b55. This avoids representing the physical ACT as separate
/// display and fetch drivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialEndpoint {
    address: u16,
    display: Option<ActDisplayWordSerializer>,
    received_word: u16,
    received_mask: u16,
}

impl ActSerialEndpoint {
    pub const fn new(address: u16) -> Self {
        Self {
            address: address & ROM_ADDRESS_MASK,
            display: None,
            received_word: 0,
            received_mask: 0,
        }
    }

    /// Start a fetch-only word, explicitly releasing the display window.
    pub fn begin_fetch_cycle(&mut self, address: u16) {
        self.address = address & ROM_ADDRESS_MASK;
        self.display = None;
        self.received_word = 0;
        self.received_mask = 0;
    }

    /// Start a combined display/fetch word from current ACT architectural state.
    ///
    /// The serializer snapshots only the selected A/B nibbles and emits their
    /// individual bits later as b0..b7 are visited. No eight-bit display value is
    /// assembled at this boundary.
    pub fn begin_display_fetch_cycle(
        &mut self,
        address: u16,
        scan_slot: u8,
        state: &ActArchitecturalState,
    ) -> Result<(), ActDisplaySerialError> {
        self.address = address & ROM_ADDRESS_MASK;
        self.display = Some(ActDisplayWordSerializer::from_state(scan_slot, state)?);
        self.received_word = 0;
        self.received_mask = 0;
        Ok(())
    }

    pub const fn address(&self) -> u16 {
        self.address
    }

    /// ACT contribution to IS for the current bit cell.
    pub const fn drive_for_bit(&self, word_bit: u8) -> Drive {
        if display_data_serial_bit(word_bit).is_some() {
            return match self.display {
                Some(serializer) => serializer.drive_for_bit(word_bit),
                None => Drive::HighZ,
            };
        }
        act_address_drive(self.address, word_bit)
    }

    /// Sample a resolved IS/ISA level during the ROM return window.
    pub fn sample_for_bit(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
    ) -> Result<(), SerialFetchError> {
        let IsaWindow::RomWord { serial_bit } = isa_window_for_bit(word_bit) else {
            return Ok(());
        };

        if sample_isa(level, word_bit)? {
            self.received_word |= 1u16 << serial_bit;
        } else {
            self.received_word &= !(1u16 << serial_bit);
        }
        self.received_mask |= 1u16 << serial_bit;
        Ok(())
    }

    /// Return the complete reconstructed 10-bit word after b55 has been sampled.
    pub const fn fetched_word(&self) -> Result<u16, SerialFetchError> {
        if self.received_mask != COMPLETE_WORD_MASK {
            return Err(SerialFetchError::IncompleteRomWord {
                received_mask: self.received_mask,
            });
        }
        Ok(self.received_word & ROM_WORD_MASK)
    }
}

'''
fetch = fetch[:start] + endpoint + fetch[end:]
# Replace internal runner and public runner section.
start = fetch.find("fn run_structural_word_transport<S: Hp67RomWordSource>(")
end = fetch.find("#[cfg(test)]\nmod tests {", start)
if start < 0 or end < 0:
    raise SystemExit("transport runner section not found")
runners = r'''fn run_structural_word_transport<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    source: &S,
    mut rom0: Option<&mut Rom0DisplayEndpoint>,
) -> Result<u16, StructuralWordError> {
    rom.begin_cycle();
    if let Some(endpoint) = rom0.as_deref_mut() {
        endpoint.begin_word();
    }

    for expected_bit in 0..BITS_PER_WORD {
        debug_assert_eq!(backplane.word_bit(), expected_bit);

        backplane.drive(Hp67Net::Isa, ACT_IS_DRIVER, act.drive_for_bit(expected_bit));
        backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, rom.drive_for_bit(expected_bit));

        if let Some(endpoint) = rom0.as_deref_mut() {
            if display_data_serial_bit(expected_bit).is_some() {
                endpoint.sample_for_bit(expected_bit, backplane.level(Hp67Net::Isa))?;
            }
        }

        if matches!(
            isa_window_for_bit(expected_bit),
            IsaWindow::RomAddress { .. }
        ) {
            rom.sample_for_bit(expected_bit, backplane.level(Hp67Net::Isa), source)?;
            backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, rom.drive_for_bit(expected_bit));
        }

        if matches!(isa_window_for_bit(expected_bit), IsaWindow::RomWord { .. }) {
            act.sample_for_bit(expected_bit, backplane.level(Hp67Net::Isa))?;
        }

        if backplane.level(Hp67Net::Isa) == LogicLevel::Contention {
            return Err(SerialFetchError::IsaContention {
                word_bit: expected_bit,
            }
            .into());
        }

        for _ in 0..4 {
            backplane.advance_clock();
        }
    }

    Ok(act.fetched_word()?)
}

/// Run one structural 56-bit ACT↔ROM fetch through the resolved HP-67 IS net.
///
/// This is a bit-cell harness, not the final edge-accurate device scheduler. It
/// deliberately advances the current four-subphase timing scaffold once per bit
/// cell while keeping every address/response bit on the resolved electrical net.
pub fn run_structural_fetch_cycle<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    source: &S,
) -> Result<u16, SerialFetchError> {
    act.begin_fetch_cycle(address);
    match run_structural_word_transport(backplane, act, rom, source, None) {
        Ok(word) => Ok(word),
        Err(StructuralWordError::Fetch(error)) => Err(error),
        Err(StructuralWordError::Display(_)) | Err(StructuralWordError::ActDisplay(_)) => {
            unreachable!("fetch-only structural cycle cannot produce a display error")
        }
    }
}

/// Run one structural 56-bit word with ACT display traffic and instruction fetch
/// sharing the same resolved HP-67 IS net and backplane timing coordinate.
///
/// The caller supplies ACT architectural state and the current cathode scan slot;
/// `ActSerialEndpoint` selects the appropriate A/B nibbles and serializes their
/// bits directly as b0..b7. No precomposed display byte crosses this API.
///
/// This establishes simultaneous coarse word-level transport only. It does not
/// claim final intra-word ACT register/ALU timing, PHI-relative launch/sample
/// edges, ROM0 sampling edges, or STR/RCD propagation timing.
pub fn run_structural_display_fetch_cycle<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    scan_slot: u8,
    act_state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    source: &S,
) -> Result<StructuralWordResult, StructuralWordError> {
    act.begin_display_fetch_cycle(address, scan_slot, act_state)?;
    let fetched_word =
        run_structural_word_transport(backplane, act, rom, source, Some(&mut *rom0))?;
    Ok(StructuralWordResult {
        fetched_word,
        display_byte: rom0.display_byte()?,
    })
}

'''
fetch = fetch[:start] + runners + fetch[end:]
# Update tests names and calls.
fetch = fetch.replace("ActFetchEndpoint::new", "ActSerialEndpoint::new")
old_call = r'''        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            0x07b,
            0x30,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
'''
new_call = r'''        let mut act_state = ActArchitecturalState::default();
        // Slot 4 is mantissa digit 11 -> ACT register digit 13.  A=0, B=3
        // must therefore serialize 0,0,0,0,1,1,0,0 on b0..b7.
        act_state.a[13] = 0x00;
        act_state.b[13] = 0x03;
        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            0x07b,
            4,
            &act_state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
'''
fetch = replace_once(fetch, old_call, new_call, "combined transport test")
fetch_path.write_text(fetch)

# -----------------------------------------------------------------------------
# Public exports.
# -----------------------------------------------------------------------------
mod_path = Path("src/machines/hp67/mod.rs")
mod = mod_path.read_text()
mod = replace_once(
    mod,
    "    ActArchitecturalCore, ActArchitecturalState, ActError, ActExecution, ActInstructionState,\n    ActOperation, ActRamImage, ActRegister, PowerOnActCore, PowerOnActError, PowerOnExecution,\n",
    "    display_register_index_for_scan_slot, ActArchitecturalCore, ActArchitecturalState,\n    ActDisplaySerialError, ActDisplayWordSerializer, ActError, ActExecution, ActInstructionState,\n    ActOperation, ActRamImage, ActRegister, PowerOnActCore, PowerOnActError, PowerOnExecution,\n",
    "mod ACT exports",
)
mod = replace_once(
    mod,
    "pub use display_snapshot::{\n    display_byte_from_act_registers, display_register_index_for_scan_slot,\n    structural_display_scan_from_act_registers, StructuralDisplaySlot,\n};\n",
    "pub use display_snapshot::{\n    display_byte_from_act_registers, structural_display_scan_from_act_registers, StructuralDisplaySlot,\n};\n",
    "mod snapshot exports",
)
mod = replace_once(
    mod,
    "    run_structural_display_fetch_cycle, run_structural_fetch_cycle, ActFetchEndpoint,\n",
    "    run_structural_display_fetch_cycle, run_structural_fetch_cycle, ActSerialEndpoint,\n",
    "mod fetch exports",
)
mod = replace_once(
    mod,
    "pub use isa::{\n    act_address_drive, act_display_drive, rom_word_drive, wired_high_drive, ROM_ADDRESS_MASK,\n    ROM_WORD_MASK,\n};\n",
    "pub use isa::{act_address_drive, rom_word_drive, wired_high_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK};\n",
    "mod ISA exports",
)
mod_path.write_text(mod)

# -----------------------------------------------------------------------------
# Live UI machine: no display-byte composition in production.
# -----------------------------------------------------------------------------
hp_path = Path("src/hp67.rs")
hp = hp_path.read_text()
hp = replace_once(
    hp,
    "        decode_rom0_display_byte, display_byte_from_act_registers,\n        run_structural_display_fetch_cycle, ActFetchEndpoint, ActOperation, CathodeDriver1820_1749,\n",
    "        decode_rom0_display_byte, run_structural_display_fetch_cycle, ActOperation,\n        ActSerialEndpoint, CathodeDriver1820_1749,\n",
    "hp67 imports",
)
hp = hp.replace("fetch_act: ActFetchEndpoint,", "act_serial: ActSerialEndpoint,")
hp = hp.replace("fetch_act: ActFetchEndpoint::new(0),", "act_serial: ActSerialEndpoint::new(0),")
hp = hp.replace("self.fetch_act = ActFetchEndpoint::new(0);", "self.act_serial = ActSerialEndpoint::new(0);")
old = r'''        let display_byte = display_byte_from_act_registers(
            scan_slot,
            &self.machine.act.state.a,
            &self.machine.act.state.b,
        )
        .map_err(|error| {
            format!("live cycle {cycle} display byte composition failed: {error:?}")
        })?;
        let result = run_structural_display_fetch_cycle(
            &mut self.backplane,
            address,
            display_byte,
            &mut self.fetch_act,
            &mut self.fetch_rom,
            &mut self.display_rom0,
            &self.source,
        )
        .map_err(|error| format!("live cycle {cycle} shared word failed: {error:?}"))?;
        if result.display_byte != display_byte {
            return Err(format!(
                "live cycle {cycle} ROM0 reconstructed display byte 0x{:02x}, expected 0x{display_byte:02x}",
                result.display_byte
            ));
        }
'''
new = r'''        let result = run_structural_display_fetch_cycle(
            &mut self.backplane,
            address,
            scan_slot,
            &self.machine.act.state,
            &mut self.act_serial,
            &mut self.fetch_rom,
            &mut self.display_rom0,
            &self.source,
        )
        .map_err(|error| format!("live cycle {cycle} shared word failed: {error:?}"))?;
'''
hp = replace_once(hp, old, new, "hp67 transport")
# The startup test must now inspect the ACT serializer itself, not the diagnostic
# byte-composition helper.
old = r'''    use hp67emu::machines::hp67::HP67_DISPLAY_SCAN_SLOTS;
'''
new = r'''    use hp67emu::{
        emulation::Drive,
        machines::hp67::{ActDisplayWordSerializer, HP67_DISPLAY_SCAN_SLOTS},
    };
'''
hp = replace_once(hp, old, new, "hp67 test imports")
old = r'''    fn reset_act_registers_naturally_encode_zero_display_bytes() {
        let machine = Hp67ArchitecturalMachine::default();
        assert!(!machine.act.state.display_enable);
        for scan_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
            let code = display_byte_from_act_registers(
                scan_slot,
                &machine.act.state.a,
                &machine.act.state.b,
            )
            .unwrap();
            assert_eq!(code, 0x00);
        }
    }
'''
new = r'''    fn reset_act_registers_naturally_release_all_display_serial_bits() {
        let machine = Hp67ArchitecturalMachine::default();
        assert!(!machine.act.state.display_enable);
        for scan_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
            let serializer =
                ActDisplayWordSerializer::from_state(scan_slot, &machine.act.state).unwrap();
            for word_bit in 0..8 {
                assert_eq!(serializer.drive_for_bit(word_bit), Drive::HighZ);
            }
        }
    }
'''
hp = replace_once(hp, old, new, "hp67 reset serializer test")
hp_path.write_text(hp)

# -----------------------------------------------------------------------------
# Power-on smoke: same shared word, now driven by the ACT serial endpoint.
# -----------------------------------------------------------------------------
smoke_path = Path("src/bin/hp67_poweron_smoke.rs")
smoke = smoke_path.read_text()
smoke = replace_once(
    smoke,
    "        decode_rom0_display_byte, display_byte_from_act_registers,\n        run_structural_display_fetch_cycle, ActError, ActFetchEndpoint, CathodeDriver1820_1749,\n",
    "        decode_rom0_display_byte, run_structural_display_fetch_cycle, ActError,\n        ActSerialEndpoint, CathodeDriver1820_1749,\n",
    "smoke imports",
)
smoke = smoke.replace("fetch_act: &mut ActFetchEndpoint", "act_serial: &mut ActSerialEndpoint")
smoke = smoke.replace("let mut fetch_act = ActFetchEndpoint::new(0);", "let mut act_serial = ActSerialEndpoint::new(0);")
smoke = smoke.replace("&mut fetch_act", "&mut act_serial")
# verify_boot_idle_display body: remove diagnostic composition and feed state+slot.
old = r'''        let code = display_byte_from_act_registers(
            expected_slot,
            &machine.act.state.a,
            &machine.act.state.b,
        )
        .map_err(|error| format!("boot display byte composition failed: {error:?}"))?;

        let result = run_structural_display_fetch_cycle(
            backplane,
            address,
            code,
            fetch_act,
            fetch_rom,
            display_rom0,
            source,
        )
        .map_err(|error| {
            format!("boot display shared word failed at scan slot {expected_slot}: {error:?}")
        })?;

        if result.display_byte != code {
            return Err(format!(
                "boot display transport mismatch at slot {expected_slot}: got 0x{:02X}, expected 0x{code:02X}",
                result.display_byte
            ));
        }
'''
new = r'''        let result = run_structural_display_fetch_cycle(
            backplane,
            address,
            expected_slot,
            &machine.act.state,
            act_serial,
            fetch_rom,
            display_rom0,
            source,
        )
        .map_err(|error| {
            format!("boot display shared word failed at scan slot {expected_slot}: {error:?}")
        })?;
'''
smoke = replace_once(smoke, old, new, "smoke idle display transport")
# fetch_cycle body.
old = r'''    let display_byte =
        display_byte_from_act_registers(scan_slot, &machine.act.state.a, &machine.act.state.b)
            .map_err(|error| {
                format!(
            "cycle {cycle} display byte composition failed at scan slot {scan_slot}: {error:?}"
        )
            })?;

    let result = run_structural_display_fetch_cycle(
        backplane,
        address,
        display_byte,
        fetch_act,
        fetch_rom,
        display_rom0,
        source,
    )
    .map_err(|error| format!("cycle {cycle} shared display/fetch word failed: {error:?}"))?;

    if result.display_byte != display_byte {
        return Err(format!(
            "cycle {cycle} display transport mismatch at scan slot {scan_slot}: got 0x{:02X}, expected 0x{display_byte:02X}",
            result.display_byte
        ));
    }
'''
new = r'''    let result = run_structural_display_fetch_cycle(
        backplane,
        address,
        scan_slot,
        &machine.act.state,
        act_serial,
        fetch_rom,
        display_rom0,
        source,
    )
    .map_err(|error| format!("cycle {cycle} shared display/fetch word failed: {error:?}"))?;
'''
smoke = replace_once(smoke, old, new, "smoke fetch transport")
smoke = smoke.replace(
    "word path: ACT display b0..b7 + address b16..b27 -> resolved IS -> ROM b46..b55 -> ACT",
    "word path: ACT A/B bits b0..b7 + address b16..b27 -> resolved IS -> ROM b46..b55 -> ACT",
)
smoke = smoke.replace(
    "BOOT DISPLAY PASS: real firmware A/B -> shared resolved IS word cycle (display b0..b7 + fetch b16..b27/b46..b55) -> ROM0 decode -> 15-slot cathode scan produces the source-backed power-on 0.00 pattern.",
    "BOOT DISPLAY PASS: real firmware ACT A/B bits -> shared resolved IS word cycle (b0..b7 + fetch b16..b27/b46..b55) -> ROM0 decode -> 15-slot cathode scan produces the source-backed power-on 0.00 pattern.",
)
smoke_path.write_text(smoke)

# -----------------------------------------------------------------------------
# Documentation: record the fidelity gain and the remaining intra-word boundary.
# -----------------------------------------------------------------------------
Path("docs/files/src/machines/hp67/act.rs.md").write_text('''# `src/machines/hp67/act.rs`

## Purpose

Implements the independent instruction-boundary Woodstock ACT core for HP-67 bring-up and now owns the ACT-side word serializer that emits ROM0 display bits directly from A/B nibbles.

## Why it exists

The architectural core is required for complete firmware execution while the final pin/timing-accurate 1820-2530 is still being built. The display path must nevertheless respect the real ACT chip boundary: production transport must not precompose `(B << 4) | A` outside the ACT and hand a finished byte to the bus.

## Relationships

`fetch.rs` uses `ActDisplayWordSerializer` through a single `ActSerialEndpoint` that owns the currently modeled ACT roles on IS. `display.rs` supplies the source-backed fifteen-slot role order. `display_snapshot.rs` retains a whole-byte composition helper only as an instruction-boundary diagnostic/reference bridge. Production UI and power-on smoke no longer use that helper. The semantic reference model remains test-only.

## Responsibilities

Maintain all architecturally visible ACT state needed by HP-67 firmware and implement Woodstock instruction-boundary behavior. For display transport, map the current physical scan slot to its ACT A/B register digit and emit b0..b3 directly from A bit 0..3 and b4..b7 directly from B bit 0..3 using the wired-high/release IS convention. Reject invalid scan slots rather than inventing data.

## Implementation

`ActDisplayWordSerializer::from_state()` snapshots only the selected A and B nibbles at the structural word boundary. `drive_for_bit()` never constructs an eight-bit display code: it selects the corresponding source bit only when b0..b7 is visited. This removes the previous whole-byte production bridge while deliberately preserving a clear remaining boundary: A/B are still instruction-boundary arrays, not the final serial shift-register/ALU state evolving inside the 56-bit word. Exact PHI launch edges and true intra-word ACT mutation remain future work.
''')

Path("docs/files/src/machines/hp67/fetch.rs.md").write_text('''# `src/machines/hp67/fetch.rs`

## Purpose

Provides the structural HP-67 shared-word transport and a single ACT serial endpoint for display output, ROM-address output and ROM-word input on the resolved IS/ISA electrical net.

## Why it exists

Direct HP-67 evidence places ACT/ROM0 display traffic at b0..b7, ACT ROM address at b16..b27 and selected-ROM response at b46..b55 within the same 56-bit word. A maximum-fidelity transport must preserve both the shared physical bus and the physical ACT boundary; it must not accept a prebuilt display byte from the UI or smoke harness.

## Relationships

Uses `ActDisplayWordSerializer` from `act.rs`, evidence-backed windows from `timing.rs`, wired-high/address/ROM helpers from `isa.rs`, `Rom0DisplayEndpoint` from `display.rs`, and `Hp67ElectricalBackplane` for the weak-low resolved IS net. Firmware remains behind `Hp67RomWordSource` and is not embedded in this layer.

## Responsibilities

`ActSerialEndpoint` owns every currently modeled ACT role on IS: direct A/B display-bit drive at b0..b7, address drive at b16..b27 and returned-word sampling at b46..b55. `RomFetchEndpoint` reconstructs the twelve address bits and emits the selected ten-bit ROM word. ROM0 independently reconstructs the eight display bits from the same resolved line. Floating/contentious samples remain hard failures.

## Implementation

`run_structural_display_fetch_cycle()` now receives the current ACT architectural state and scan slot, not a display byte. It asks `ActSerialEndpoint` to begin a combined word; the endpoint snapshots only the selected A/B nibbles and later emits each source bit when its real b0..b7 coordinate arrives. The API therefore has no `(B << 4) | A` display payload. The returned `StructuralWordResult.display_byte` is reconstructed only on the ROM0 side and exists for observation/testing, not as ACT input.

The remaining fidelity boundary is explicit: the serializer snapshots architectural nibbles at the word boundary, while a real ACT shifts and modifies serial register state inside the word. Exact intra-word ALU/register timing, PHI launch/sample edges, ROM0 edge timing and STR/RCD overlap ordering are not claimed here.
''')

# Update live UI companion.
Path("docs/files/src/hp67.rs.md").write_text('''# `src/hp67.rs`

## Purpose
Owns the remaining UI mechanical controls and a UI-local HP-67 structural machine whose LED state is derived from machine state, real firmware and ACT-owned serial display output.

## Why it exists
The frontend must expose hardware-derived behavior without formatting numbers or constructing display bytes. Startup zeroes and the final `0.00` must both emerge from ACT state, shared IS transport, ROM0 decoding and cathode selection.

## Relationships
`Hp67LiveMachine` uses the reusable `Hp67ArchitecturalMachine`, `ActSerialEndpoint`, shared display/fetch word transport, ROM0 decoder and 1820-1749 structural cathode driver. Firmware remains external through `RomCorpus`. `app.rs` advances the machine with elapsed wall-clock time and `panel.rs` consumes only raw segment masks.

## Responsibilities
Load exactly 5120 normalized external firmware words; keep the display unclaimed during the pre-SYNC reset interval; after firmware fetch begins, update each physical slot only from ACT-owned bit serialization across the shared word; detect the firmware's real display-off/toggle instructions; and never synthesize a startup or idle display value.

## Implementation
`transport_fetch_word()` passes the current scan slot and ACT state into `run_structural_display_fetch_cycle()`. It no longer calls `display_byte_from_act_registers()` and never sees an expected display byte before transport. `ActSerialEndpoint` emits the selected A/B bits on b0..b7; ROM0 reconstructs the resulting byte from the resolved IS net and the UI decodes only that received value. The architectural reset A=B=0 therefore naturally releases all eight wired-high display bits, which ROM0 observes as zero under the weak-low bias. After the first explicit firmware display-control opcode, `display_enable` governs emission normally.
''')

# Smoke companion.
smoke_doc = Path("docs/files/src/bin/hp67_poweron_smoke.rs.md")
if smoke_doc.exists():
    smoke_doc.write_text('''# `src/bin/hp67_poweron_smoke.rs`

## Purpose
Runs external HP-67 firmware through the structural shared-word path and proves the documented power-on sequence reaches the no-key idle loop with the source-backed `0.00` display state.

## Why it exists
A long real-microcode smoke catches integration errors that isolated opcode and bus tests cannot. Display verification must use the same ACT-owned serial path as runtime rather than precompose A/B into a display byte.

## Relationships
Uses `Hp67ArchitecturalMachine`, `ActSerialEndpoint`, `RomFetchEndpoint`, `Rom0DisplayEndpoint`, `CathodeDriver1820_1749` and `FetchPipelineLatch`. Firmware is loaded from the normalized external corpus. The same resolved IS word carries ACT display bits b0..b7, ACT address b16..b27 and ROM response b46..b55.

## Responsibilities
Verify startup fetch anchors, execute real firmware to the documented idle landmarks, preserve the one-word pipeline, exercise the physical delayed-ROM path and validate the final fifteen-slot ROM0/cathode scan. Expected idle codes are assertions on the observed ROM0 result, never inputs to ACT transport.

## Implementation
Every runtime fetch calls `run_structural_display_fetch_cycle()` with the current scan slot and ACT state. The ACT serial endpoint chooses and emits the individual A/B source bits; the smoke harness does not call `display_byte_from_act_registers()`. At idle, the coarse cathode counter is reset only to make the fifteen-slot verification deterministic; each verification word still crosses the same shared backplane and ROM0 receiver. Exact PHI/RCD/STR edges and intra-word ACT register evolution remain explicitly outside this checkpoint.
''')
