from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old not in text:
        raise SystemExit(f"anchor not found: {label}")
    return text.replace(old, new, 1)


# -----------------------------------------------------------------------------
# ACT: own the fifteen-word display phase.  The cathode driver must not tell the
# ACT which digit to emit; ACT generates RCD while ROM0 generates STR.
# -----------------------------------------------------------------------------
act_path = Path("src/machines/hp67/act.rs")
act = act_path.read_text()
act = replace_once(
    act,
    "    display::{display_role_for_scan_slot, Hp67DisplayRole},\n",
    "    display::{display_role_for_scan_slot, Hp67DisplayRole, HP67_DISPLAY_SCAN_SLOTS},\n",
    "ACT display imports",
)
start = act.find("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum ActDisplaySerialError")
end = act.find("#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]\npub enum ActInstructionState", start)
if start < 0 or end < 0:
    raise SystemExit("ACT display serializer block not found")
new_block = r'''#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActDisplaySerialError {
    InvalidScanSlot(u8),
}

/// Coarse ACT-owned display-scan event produced when one 56-bit display word
/// completes.
///
/// `rcd_after_word` records only the source-backed fact that ACT resets the
/// cathode driver at the end of the fifteen-slot sequence. It deliberately does
/// not claim the unresolved PHI-relative ordering of RCD versus the final STR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActDisplayScanEvent {
    pub scan_slot: u8,
    pub rcd_after_word: bool,
}

/// ACT-owned fifteen-word display phase.
///
/// Hardware evidence establishes the direction of control: ACT supplies one
/// display position per 56-bit word and generates RCD; ROM0 generates STR; the
/// 1820-1749 follows those signals.  Keeping this state on the ACT side prevents
/// the cathode driver from feeding a scan-slot choice backwards into the ACT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActDisplayScanSequencer {
    scan_slot: u8,
}

impl Default for ActDisplayScanSequencer {
    fn default() -> Self {
        Self { scan_slot: 1 }
    }
}

impl ActDisplayScanSequencer {
    pub const fn scan_slot(&self) -> u8 {
        self.scan_slot
    }

    /// Complete one displayed word and advance the ACT-owned phase.
    ///
    /// Slot 15 is the source-backed duplicate of slot 1. Completing it wraps the
    /// ACT phase to slot 1 and reports the coarse RCD boundary.
    pub fn complete_word(&mut self) -> ActDisplayScanEvent {
        let event = ActDisplayScanEvent {
            scan_slot: self.scan_slot,
            rcd_after_word: self.scan_slot == HP67_DISPLAY_SCAN_SLOTS,
        };
        self.scan_slot = if event.rcd_after_word {
            1
        } else {
            self.scan_slot + 1
        };
        event
    }
}

/// ACT-side source for the eight ROM0 display bits of one 56-bit machine word.
///
/// The endpoint snapshots only the two source nibbles for the ACT-selected scan
/// position. It never composes an eight-bit display value. b0..b3 are emitted
/// directly from A bit 0..3 and b4..b7 directly from B bit 0..3, using the
/// observed wired-high/release IS convention.
///
/// This is still a word-boundary register tap: A/B are architectural arrays,
/// not the final intra-word shift-register/ALU implementation. Exact PHI launch
/// edges remain intentionally unspecified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActDisplayWordSerializer {
    scan_slot: u8,
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
            scan_slot,
            register_index,
            a_nibble: state.a[register_index] & 0x0f,
            b_nibble: state.b[register_index] & 0x0f,
        })
    }

    pub const fn scan_slot(&self) -> u8 {
        self.scan_slot
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
act = act[:start] + new_block + act[end:]
act = replace_once(
    act,
    "        let serializer = ActDisplayWordSerializer::from_state(4, &state).unwrap();\n        assert_eq!(serializer.register_index(), 13);\n",
    "        let serializer = ActDisplayWordSerializer::from_state(4, &state).unwrap();\n        assert_eq!(serializer.scan_slot(), 4);\n        assert_eq!(serializer.register_index(), 13);\n",
    "serializer scan-slot test",
)
insert_anchor = "    #[test]\n    fn reset_act_nibbles_release_all_eight_display_bits() {\n"
scan_test = r'''    #[test]
    fn act_owns_fifteen_word_display_phase_and_rcd_boundary() {
        let mut scan = ActDisplayScanSequencer::default();
        for expected_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
            assert_eq!(scan.scan_slot(), expected_slot);
            let event = scan.complete_word();
            assert_eq!(event.scan_slot, expected_slot);
            assert_eq!(event.rcd_after_word, expected_slot == HP67_DISPLAY_SCAN_SLOTS);
        }
        assert_eq!(scan.scan_slot(), 1);
    }

'''
act = replace_once(act, insert_anchor, scan_test + insert_anchor, "ACT display-scan test anchor")
act_path.write_text(act)


# -----------------------------------------------------------------------------
# Shared word transport: embed the ACT-owned scan sequencer in the ACT endpoint.
# The public combined runner no longer accepts a cathode-selected scan slot.
# -----------------------------------------------------------------------------
fetch_path = Path("src/machines/hp67/fetch.rs")
fetch = fetch_path.read_text()
fetch = replace_once(
    fetch,
    "    act::{ActArchitecturalState, ActDisplaySerialError, ActDisplayWordSerializer},\n",
    "    act::{\n        ActArchitecturalState, ActDisplayScanEvent, ActDisplayScanSequencer, ActDisplaySerialError,\n        ActDisplayWordSerializer,\n    },\n",
    "fetch ACT imports",
)
fetch = replace_once(
    fetch,
    "pub struct StructuralWordResult {\n    pub fetched_word: u16,\n    pub display_byte: u8,\n}\n",
    "pub struct StructuralWordResult {\n    pub fetched_word: u16,\n    pub display_byte: u8,\n    pub display_scan_slot: u8,\n    pub rcd_after_word: bool,\n}\n",
    "StructuralWordResult fields",
)
fetch = replace_once(
    fetch,
    "pub struct ActSerialEndpoint {\n    address: u16,\n    display: Option<ActDisplayWordSerializer>,\n    received_word: u16,\n    received_mask: u16,\n}\n",
    "pub struct ActSerialEndpoint {\n    address: u16,\n    display_scan: ActDisplayScanSequencer,\n    display: Option<ActDisplayWordSerializer>,\n    received_word: u16,\n    received_mask: u16,\n}\n",
    "ActSerialEndpoint fields",
)
fetch = replace_once(
    fetch,
    "        Self {\n            address: address & ROM_ADDRESS_MASK,\n            display: None,\n            received_word: 0,\n            received_mask: 0,\n        }\n",
    "        Self {\n            address: address & ROM_ADDRESS_MASK,\n            display_scan: ActDisplayScanSequencer::default(),\n            display: None,\n            received_word: 0,\n            received_mask: 0,\n        }\n",
    "ActSerialEndpoint constructor",
)
old_begin = r'''    /// Start a combined display/fetch word from current ACT architectural state.
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
'''
new_begin = r'''    /// Start a combined display/fetch word from current ACT architectural state.
    ///
    /// The ACT's own display sequencer chooses the slot. The serializer snapshots
    /// only that slot's A/B nibbles and emits their individual bits later as
    /// b0..b7 are visited. No cathode state and no eight-bit display value enters
    /// this boundary.
    pub fn begin_display_fetch_cycle(
        &mut self,
        address: u16,
        state: &ActArchitecturalState,
    ) -> Result<(), ActDisplaySerialError> {
        self.address = address & ROM_ADDRESS_MASK;
        let scan_slot = self.display_scan.scan_slot();
        self.display = Some(ActDisplayWordSerializer::from_state(scan_slot, state)?);
        self.received_word = 0;
        self.received_mask = 0;
        Ok(())
    }

    pub const fn address(&self) -> u16 {
        self.address
    }

    pub const fn display_scan_slot(&self) -> u8 {
        self.display_scan.scan_slot()
    }

    /// Finish a successfully transported display word and advance the ACT-owned
    /// scan phase. The returned RCD flag is coarse word-boundary information;
    /// no PHI-relative edge ordering is implied.
    pub fn complete_display_word(&mut self) -> ActDisplayScanEvent {
        let event = self.display_scan.complete_word();
        if let Some(serializer) = self.display.take() {
            debug_assert_eq!(serializer.scan_slot(), event.scan_slot);
        } else {
            debug_assert!(false, "display word completed without an active serializer");
        }
        event
    }
'''
fetch = replace_once(fetch, old_begin, new_begin, "combined ACT begin-cycle block")
old_runner = r'''/// Run one structural 56-bit word with ACT display traffic and instruction fetch
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
new_runner = r'''/// Run one structural 56-bit word with ACT display traffic and instruction fetch
/// sharing the same resolved HP-67 IS net and backplane timing coordinate.
///
/// The caller supplies ACT architectural state only. `ActSerialEndpoint` owns
/// the fifteen-word display phase, chooses the source A/B digit, serializes it on
/// b0..b7, and reports the coarse RCD boundary after slot 15. The cathode driver
/// is downstream state and cannot choose what the ACT emits.
///
/// This establishes source-backed word/bit ownership only. It does not claim
/// final intra-word ACT ALU/register timing, PHI-relative launch/sample edges,
/// ROM0 sampling edges, or ordering inside the observed final STR/RCD overlap.
pub fn run_structural_display_fetch_cycle<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    act_state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    source: &S,
) -> Result<StructuralWordResult, StructuralWordError> {
    act.begin_display_fetch_cycle(address, act_state)?;
    let fetched_word =
        run_structural_word_transport(backplane, act, rom, source, Some(&mut *rom0))?;
    let display_byte = rom0.display_byte()?;
    let display_event = act.complete_display_word();
    Ok(StructuralWordResult {
        fetched_word,
        display_byte,
        display_scan_slot: display_event.scan_slot,
        rcd_after_word: display_event.rcd_after_word,
    })
}
'''
fetch = replace_once(fetch, old_runner, new_runner, "combined runner API")
old_test = r'''        let mut act_state = ActArchitecturalState::default();
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
        .expect("display and fetch transport must share one structural word");

        assert_eq!(result.fetched_word, 0x04c);
        assert_eq!(result.display_byte, 0x30);
        assert_eq!(rom.received_address(), Ok(0x07b));
        assert_eq!(rom0.decoded_anodes(4), Ok(Hp67SegmentMask::DP));
'''
new_test = r'''        let mut act_state = ActArchitecturalState::default();
        // ACT starts at scan slot 1 -> register digit 0. A=0, B=3 must
        // serialize 0,0,0,0,1,1,0,0 on b0..b7 without cathode input.
        act_state.a[0] = 0x00;
        act_state.b[0] = 0x03;
        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            0x07b,
            &act_state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("display and fetch transport must share one structural word");

        assert_eq!(result.fetched_word, 0x04c);
        assert_eq!(result.display_byte, 0x30);
        assert_eq!(result.display_scan_slot, 1);
        assert!(!result.rcd_after_word);
        assert_eq!(act.display_scan_slot(), 2);
        assert_eq!(rom.received_address(), Ok(0x07b));
        assert_eq!(rom0.decoded_anodes(1), Ok(Hp67SegmentMask::DP));
'''
fetch = replace_once(fetch, old_test, new_test, "combined transport test")
insert_anchor = "    #[test]\n    fn fetched_word_enters_execution_on_the_following_machine_cycle() {\n"
sequence_test = r'''    #[test]
    fn act_not_cathode_owns_fifteen_slot_sequence_and_rcd_boundary() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();
        let mut act_state = ActArchitecturalState::default();
        act_state.a[0] = 0x05;

        let mut first_code = None;
        for expected_slot in 1..=15 {
            let result = run_structural_display_fetch_cycle(
                &mut backplane,
                0x07b,
                &act_state,
                &mut act,
                &mut rom,
                &mut rom0,
                &source,
            )
            .expect("ACT-owned display scan must transport");
            assert_eq!(result.display_scan_slot, expected_slot);
            assert_eq!(result.rcd_after_word, expected_slot == 15);
            if expected_slot == 1 {
                first_code = Some(result.display_byte);
            }
            if expected_slot == 15 {
                assert_eq!(result.display_byte, first_code.unwrap());
            }
        }
        assert_eq!(act.display_scan_slot(), 1);

        let next = run_structural_display_fetch_cycle(
            &mut backplane,
            0x07b,
            &act_state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("scan must continue from slot 1 after RCD boundary");
        assert_eq!(next.display_scan_slot, 1);
        assert!(!next.rcd_after_word);
    }

'''
fetch = replace_once(fetch, insert_anchor, sequence_test + insert_anchor, "fetch sequence-test anchor")
fetch_path.write_text(fetch)


# -----------------------------------------------------------------------------
# UI live machine: consume the ACT-selected slot, then let ROM0 STR advance the
# cathode and ACT RCD reset it at the coarse slot-15 boundary.
# -----------------------------------------------------------------------------
ui_path = Path("src/hp67.rs")
ui = ui_path.read_text()
ui = replace_once(
    ui,
    "/// display/fetch word transport used by the structural smoke test. The remaining\n/// temporary bridge is A/B -> b0..b7 inside `display_byte_from_act_registers()`.\n",
    "/// display/fetch word transport used by the structural smoke test. ACT owns\n/// the display phase and emits b0..b7; ROM0/1820-1749 are downstream consumers.\n/// The remaining fidelity boundary is the word-boundary A/B register tap inside\n/// `ActDisplayWordSerializer`, not a caller-composed display value or slot.\n",
    "UI live-machine comment",
)
old_transport = r'''        let address = self.machine.pc();
        let scan_slot = self.cathode.scan_slot();
        let result = run_structural_display_fetch_cycle(
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
        self.pipeline.complete_cycle(result.fetched_word);

        // Before firmware executes its first display-control instruction, do not
        // use the architectural `display_enable` default as an electrical inhibit.
        // A real HP-67 visibly emits during this interval, while the semantic
        // model's reset value is not hardware evidence.  The emitted CONTENT is
        // still entirely derived from ACT A/B -> IS -> ROM0 for this word.
        if !self.display_control_seen || self.machine.act.state.display_enable {
            let anodes =
                decode_rom0_display_byte(scan_slot, result.display_byte).map_err(|error| {
                    format!("live cycle {cycle} ROM0 decode failed at slot {scan_slot}: {error:?}")
                })?;
            self.display.capture_scan_slot(scan_slot, anodes)?;
        } else {
            self.display.clear();
        }

        self.cathode.str_falling_edge();
        Ok(())
'''
new_transport = r'''        let address = self.machine.pc();
        if self.cathode.scan_slot() != self.act_serial.display_scan_slot() {
            return Err(format!(
                "live cycle {cycle} display phase divergence before word: ACT slot {}, cathode slot {}",
                self.act_serial.display_scan_slot(),
                self.cathode.scan_slot()
            ));
        }
        let result = run_structural_display_fetch_cycle(
            &mut self.backplane,
            address,
            &self.machine.act.state,
            &mut self.act_serial,
            &mut self.fetch_rom,
            &mut self.display_rom0,
            &self.source,
        )
        .map_err(|error| format!("live cycle {cycle} shared word failed: {error:?}"))?;
        self.pipeline.complete_cycle(result.fetched_word);
        let scan_slot = result.display_scan_slot;

        if self.cathode.scan_slot() != scan_slot {
            return Err(format!(
                "live cycle {cycle} ROM0/ACT slot mismatch: ACT emitted slot {scan_slot}, cathode is at {}",
                self.cathode.scan_slot()
            ));
        }

        // Before firmware executes its first display-control instruction, do not
        // use the architectural `display_enable` default as an electrical inhibit.
        // A real HP-67 visibly emits during this interval, while the semantic
        // model's reset value is not hardware evidence. The emitted CONTENT is
        // still entirely derived from ACT A/B -> IS -> ROM0 for this word.
        if !self.display_control_seen || self.machine.act.state.display_enable {
            let anodes =
                decode_rom0_display_byte(scan_slot, result.display_byte).map_err(|error| {
                    format!("live cycle {cycle} ROM0 decode failed at slot {scan_slot}: {error:?}")
                })?;
            self.display.capture_scan_slot(scan_slot, anodes)?;
        } else {
            self.display.clear();
        }

        // ROM0 owns STR, so it advances the downstream cathode position. ACT
        // owns RCD; the source says it overlaps the final STR, but exact PHI
        // ordering is still unknown. At this coarse boundary both events have
        // completed before the next 56-bit word begins.
        self.cathode.str_falling_edge();
        if result.rcd_after_word {
            self.cathode.rcd_falling_edge();
        }
        if self.cathode.scan_slot() != self.act_serial.display_scan_slot() {
            return Err(format!(
                "live cycle {cycle} display phase divergence after STR/RCD: ACT slot {}, cathode slot {}",
                self.act_serial.display_scan_slot(),
                self.cathode.scan_slot()
            ));
        }
        Ok(())
'''
ui = replace_once(ui, old_transport, new_transport, "UI structural display transport")
ui_path.write_text(ui)


# -----------------------------------------------------------------------------
# Power-on smoke: derive slot from ACT result; cathode can only follow STR/RCD.
# The final display gate traverses the next complete ACT-owned 15-slot cycle
# from whatever phase real boot has reached, rather than forcibly resetting it.
# -----------------------------------------------------------------------------
smoke_path = Path("src/bin/hp67_poweron_smoke.rs")
smoke = smoke_path.read_text()
start = smoke.find("fn verify_boot_idle_display<S: Hp67RomWordSource>(")
end = smoke.find("fn parse_arguments()", start)
if start < 0 or end < 0:
    raise SystemExit("smoke idle-display verifier not found")
new_verify = r'''fn verify_boot_idle_display<S: Hp67RomWordSource>(
    machine: &Hp67ArchitecturalMachine,
    backplane: &mut Hp67ElectricalBackplane,
    act_serial: &mut ActSerialEndpoint,
    fetch_rom: &mut RomFetchEndpoint,
    display_rom0: &mut Rom0DisplayEndpoint,
    display_cathode: &mut CathodeDriver1820_1749,
    source: &S,
) -> Result<(), String> {
    // Freeze the already-reached architectural A/B state and observe one whole
    // ACT-owned display phase. We deliberately do not reset the cathode or tell
    // ACT which slot to emit: their phases must already agree after real boot.
    let address = machine.pc();
    let mut codes = [0u8; HP67_DISPLAY_SCAN_SLOTS as usize];
    let mut segments = [0u8; HP67_DISPLAY_SCAN_SLOTS as usize];

    for _ in 0..HP67_DISPLAY_SCAN_SLOTS {
        let expected_slot = act_serial.display_scan_slot();
        if display_cathode.scan_slot() != expected_slot {
            return Err(format!(
                "boot display phase mismatch before word: ACT slot {expected_slot}, cathode slot {}",
                display_cathode.scan_slot()
            ));
        }

        let result = run_structural_display_fetch_cycle(
            backplane,
            address,
            &machine.act.state,
            act_serial,
            fetch_rom,
            display_rom0,
            source,
        )
        .map_err(|error| {
            format!("boot display shared word failed at ACT slot {expected_slot}: {error:?}")
        })?;

        if result.display_scan_slot != expected_slot {
            return Err(format!(
                "boot display ACT phase changed unexpectedly: got slot {}, expected {expected_slot}",
                result.display_scan_slot
            ));
        }
        let decoded = decode_rom0_display_byte(result.display_scan_slot, result.display_byte)
            .map_err(|error| format!("boot display ROM0 decode failed: {error:?}"))?;
        let index = usize::from(result.display_scan_slot - 1);
        codes[index] = result.display_byte;
        segments[index] = decoded.bits();

        display_cathode.str_falling_edge();
        if result.rcd_after_word {
            display_cathode.rcd_falling_edge();
        }
        if display_cathode.scan_slot() != act_serial.display_scan_slot() {
            return Err(format!(
                "boot display phase mismatch after STR/RCD: ACT slot {}, cathode slot {}",
                act_serial.display_scan_slot(),
                display_cathode.scan_slot()
            ));
        }
    }

    if codes != EXPECTED_BOOT_DISPLAY_CODES {
        return Err(format!(
            "boot display code mismatch: got [{}], expected [{}]",
            format_byte_sequence(&codes),
            format_byte_sequence(&EXPECTED_BOOT_DISPLAY_CODES)
        ));
    }

    if segments != EXPECTED_BOOT_DISPLAY_SEGMENTS {
        return Err(format!(
            "boot display segment mismatch: got [{}], expected [{}]",
            format_byte_sequence(&segments),
            format_byte_sequence(&EXPECTED_BOOT_DISPLAY_SEGMENTS)
        ));
    }

    println!(
        "BOOT DISPLAY PASS: ACT-owned 15-word display phase -> A/B bits on shared IS b0..b7 -> ROM0 STR -> 1820-1749 cathode scan, with ACT RCD at the coarse slot-15 boundary, produces the source-backed power-on 0.00 pattern."
    );
    println!("BOOT DISPLAY CODES: {}", format_byte_sequence(&codes));
    Ok(())
}

'''
smoke = smoke[:start] + new_verify + smoke[end:]
old_fetch = r'''    let requested_bank = machine.prepare_hp67_fetch();
    source.select_bank(requested_bank);
    let address = machine.pc();
    let scan_slot = display_cathode.scan_slot();
    let result = run_structural_display_fetch_cycle(
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

    let fetched = result.fetched_word;
    display_cathode.str_falling_edge();
    pipeline.complete_cycle(fetched);
'''
new_fetch = r'''    let requested_bank = machine.prepare_hp67_fetch();
    source.select_bank(requested_bank);
    let address = machine.pc();
    if display_cathode.scan_slot() != act_serial.display_scan_slot() {
        return Err(format!(
            "cycle {cycle} display phase mismatch before word: ACT slot {}, cathode slot {}",
            act_serial.display_scan_slot(),
            display_cathode.scan_slot()
        ));
    }
    let result = run_structural_display_fetch_cycle(
        backplane,
        address,
        &machine.act.state,
        act_serial,
        fetch_rom,
        display_rom0,
        source,
    )
    .map_err(|error| format!("cycle {cycle} shared display/fetch word failed: {error:?}"))?;

    let fetched = result.fetched_word;
    if display_cathode.scan_slot() != result.display_scan_slot {
        return Err(format!(
            "cycle {cycle} ACT emitted display slot {} while cathode is at {}",
            result.display_scan_slot,
            display_cathode.scan_slot()
        ));
    }
    display_cathode.str_falling_edge();
    if result.rcd_after_word {
        display_cathode.rcd_falling_edge();
    }
    if display_cathode.scan_slot() != act_serial.display_scan_slot() {
        return Err(format!(
            "cycle {cycle} display phase mismatch after STR/RCD: ACT slot {}, cathode slot {}",
            act_serial.display_scan_slot(),
            display_cathode.scan_slot()
        ));
    }
    pipeline.complete_cycle(fetched);
'''
smoke = replace_once(smoke, old_fetch, new_fetch, "smoke fetch display phase")
smoke = replace_once(
    smoke,
    '        "word path: ACT A/B bits b0..b7 + address b16..b27 -> resolved IS -> ROM b46..b55 -> ACT"\n',
    '        "word path: ACT-owned display phase + A/B bits b0..b7 + address b16..b27 -> resolved IS -> ROM b46..b55 -> ACT; ROM0 STR / ACT RCD drive cathode phase"\n',
    "smoke path banner",
)
smoke_path.write_text(smoke)


# -----------------------------------------------------------------------------
# Public exports.
# -----------------------------------------------------------------------------
mod_path = Path("src/machines/hp67/mod.rs")
mod = mod_path.read_text()
mod = replace_once(
    mod,
    "    display_register_index_for_scan_slot, ActArchitecturalCore, ActArchitecturalState,\n    ActDisplaySerialError, ActDisplayWordSerializer, ActError, ActExecution, ActInstructionState,\n",
    "    display_register_index_for_scan_slot, ActArchitecturalCore, ActArchitecturalState,\n    ActDisplayScanEvent, ActDisplayScanSequencer, ActDisplaySerialError, ActDisplayWordSerializer,\n    ActError, ActExecution, ActInstructionState,\n",
    "ACT public exports",
)
mod_path.write_text(mod)


# -----------------------------------------------------------------------------
# Companion docs and evidence note.
# -----------------------------------------------------------------------------
Path("docs/files/src/machines/hp67/act.rs.md").write_text(r'''# `src/machines/hp67/act.rs`

## Purpose

Implements the independent instruction-boundary Woodstock ACT core plus the ACT-owned structural display phase and bit serializer used while the final pin/timing-accurate 1820-2530 is being built.

## Why it exists

The architectural core is required for complete firmware execution, but physical device ownership must already be correct. HP-67 evidence says the ACT supplies one display position per 56-bit word and generates RCD, while ROM0 generates STR and the 1820-1749 cathode driver follows those signals. The cathode driver therefore must never choose which A/B digit the ACT emits.

## Relationships

`fetch.rs` embeds `ActDisplayScanSequencer` and `ActDisplayWordSerializer` inside the single `ActSerialEndpoint`. `display.rs` supplies the evidenced fifteen-slot role order and models the downstream ROM0/cathode devices. `display_snapshot.rs` retains whole-byte composition only as a diagnostic/reference bridge. The semantic reference model remains test-only.

## Responsibilities

Maintain architecturally visible ACT state and Woodstock instruction-boundary semantics. For structural display transport, own the fifteen-word scan phase, select the appropriate A/B register digit, emit b0..b3 from A and b4..b7 from B using wired-high/release IS behavior, repeat exponent units at slot 15, and report the coarse RCD boundary when slot 15 completes.

## Implementation

`ActDisplayScanSequencer` starts at structural slot 1, advances once after each successful display word and wraps after slot 15 while reporting `rcd_after_word`. `ActDisplayWordSerializer` receives only the ACT-selected slot and snapshots that slot's A/B nibbles; callers cannot inject cathode phase into it. This fixes device ownership without inventing PHI timing. A/B are still instruction-boundary arrays rather than the final serial shift-register/ALU state, and the exact RCD edge relative to the final ROM0 STR remains intentionally unresolved.
''')

Path("docs/files/src/machines/hp67/fetch.rs.md").write_text(r'''# `src/machines/hp67/fetch.rs`

## Purpose

Provides the structural HP-67 shared-word transport and a single ACT serial endpoint for ACT-owned display output, ROM-address output and ROM-word input on the resolved IS/ISA electrical net.

## Why it exists

Direct HP-67 evidence places display traffic at b0..b7, ACT ROM address at b16..b27 and selected-ROM response at b46..b55 in the same 56-bit word. The same hardware evidence establishes control direction: ACT generates the display sequence/RCD, ROM0 generates STR and the cathode driver follows. A maximum-fidelity transport must preserve both the shared bus and that ownership.

## Relationships

Uses `ActDisplayScanSequencer` and `ActDisplayWordSerializer` from `act.rs`, evidence-backed windows from `timing.rs`, wired-high/address/ROM helpers from `isa.rs`, `Rom0DisplayEndpoint` from `display.rs`, and `Hp67ElectricalBackplane` for the weak-low resolved IS net. Firmware remains external behind `Hp67RomWordSource`.

## Responsibilities

`ActSerialEndpoint` owns every currently modeled ACT role on IS and the fifteen-word display phase: A/B display-bit drive at b0..b7, address drive at b16..b27 and returned-word sampling at b46..b55. `RomFetchEndpoint` reconstructs the address and emits the selected ROM word. ROM0 independently reconstructs display bits. No caller may supply a cathode-selected display slot or prebuilt display byte.

## Implementation

`run_structural_display_fetch_cycle()` now receives only ACT architectural state; `ActSerialEndpoint` chooses its current display slot internally. On successful completion it returns ROM0's reconstructed byte together with `display_scan_slot` and a coarse `rcd_after_word` event. Slot 15 reuses register digit 0 and reports RCD before the next word begins. Downstream code applies ROM0 STR to the cathode driver and applies ACT RCD at that same coarse completion boundary.

The remaining fidelity boundary is explicit: A/B source nibbles are still sampled from instruction-boundary architectural arrays. Exact intra-word ACT ALU/register mutation, PHI launch/sample edges and ordering inside the observed final STR/RCD overlap are not claimed.
''')

Path("docs/files/src/hp67.rs.md").write_text(r'''# `src/hp67.rs`

## Purpose

Owns the remaining UI mechanical controls and a UI-local HP-67 structural machine whose LED state is derived from real firmware, ACT-owned serial display output, ROM0 and the downstream cathode driver.

## Why it exists

The frontend must expose hardware-derived behavior without formatting numbers, constructing display bytes or deciding display scan phase. Startup zeroes and final `0.00` must emerge from machine state and the physical direction of control.

## Relationships

`Hp67LiveMachine` uses `Hp67ArchitecturalMachine`, `ActSerialEndpoint`, shared display/fetch transport, `Rom0DisplayEndpoint` and `CathodeDriver1820_1749`. ACT owns the fifteen-word display phase; ROM0 supplies the coarse STR event; the cathode driver follows STR and ACT's coarse RCD boundary. Firmware remains external through `RomCorpus`.

## Responsibilities

Load exactly 5120 normalized external firmware words; keep the display unclaimed during the pre-SYNC reset interval; derive visible segments only from ACT bits received by ROM0; enforce that ACT and cathode phases remain synchronized; detect real firmware display-control instructions; and never synthesize a display value or feed cathode phase backwards into ACT.

## Implementation

`transport_fetch_word()` no longer reads `cathode.scan_slot()` to choose ACT data. The combined runner returns the slot that ACT itself emitted. The UI verifies that the downstream cathode is on that slot, decodes ROM0's received byte, applies ROM0's structural STR advance, then applies ACT's structural RCD at the slot-15 boundary. Because the source only establishes overlap rather than exact PHI-relative ordering, both final-slot events are treated as complete before the next 56-bit word with no invented subphase edge.
''')

Path("docs/research/HP67_DISPLAY_SCAN_OWNERSHIP.md").write_text(r'''# HP-67 display scan ownership

Research snapshot: 2026-09-16.

## Scope

This note records the control-direction facts used to remove cathode-driven display selection from the production HP-67 path. It does not assign unresolved PHI-relative edges.

## Evidence

The HP-67 logic-analyser material in *Notes on HP's Classic Calculators* (`https://literature.hpcalc.org/community/classic-notes.pdf`, HP-67 display section around pages 75-77) identifies ROM0 (`1818-0268`) as the source of STR and the ACT as the source of RCD. RCD resets the `1820-1749` cathode driver to the first position; STR advances the cathode scan. The same capture documents fifteen display-data/STR slots, with slot 15 carrying the same exponent-units data as slot 1 and RCD overlapping the final STR.

HP-origin Woodstock design literature and the closely related HP-97 service description corroborate the direction: the ACT sends information for one display position during each 56-bit word, subsequent words provide subsequent positions, ROM0 decodes the serial display information, and the cathode driver is downstream scan state. These sources support ownership and sequence, not an exact internal ACT shift-register edge model.

## Production consequence

The production structural path must be directional:

```text
ACT display phase + A/B -> IS b0..b7 -> ROM0 -> STR -> 1820-1749
          |
          +-------------------------------- RCD -> 1820-1749
```

The cathode driver is therefore forbidden from supplying a `scan_slot` argument to the ACT. `ActSerialEndpoint` owns a fifteen-word phase. Each successful combined word returns the slot actually emitted; after slot 15 the ACT phase wraps to slot 1 and reports a coarse RCD boundary. ROM0 reconstruction supplies the display byte, and downstream code advances the cathode via STR.

## Deliberately unresolved timing

The source says RCD overlaps the final STR, but the project does not yet have sufficient evidence to assign their exact PHI-relative transitions, pulse widths or propagation order. The structural model therefore treats both events as completed at the end of the slot-15 word and asserts only that ACT and cathode phases agree before the next word begins.

Likewise, the ACT display serializer still taps instruction-boundary A/B nibbles. This ownership fix is a prerequisite for a true intra-word serial ACT; it is not a claim that ALU/register writeback has already been reproduced PHI by PHI.
''')

# Update the research timing note's implementation mapping and open-boundary text.
timing_doc_path = Path("docs/research/HP67_ISA_TIMING.md")
timing_doc = timing_doc_path.read_text()
timing_doc = replace_once(
    timing_doc,
    "`src/machines/hp67/fetch.rs` represents that boundary explicitly: the ACT endpoint sends a 12-bit address through the resolved IS net one bit at a time; the ROM endpoint reconstructs it from those electrical levels, latches a caller-supplied 10-bit ROM word, and returns that word through the same resolved net one bit at a time. A pipeline latch promotes a word fetched in cycle N to the executing slot when cycle N+1 begins.\n",
    "`src/machines/hp67/fetch.rs` represents that boundary explicitly: the ACT endpoint owns its display phase, sends display data and a 12-bit address through the resolved IS net one bit at a time, and reconstructs the returned ROM word from electrical levels. The ROM endpoint reconstructs the address, latches a caller-supplied 10-bit word and returns it through the same resolved net. A pipeline latch promotes a word fetched in cycle N to the executing slot when cycle N+1 begins.\n",
    "timing research pipeline paragraph",
)
timing_doc = replace_once(
    timing_doc,
    "- `src/machines/hp67/fetch.rs`\n  - ACT address shift-out and ROM-word shift-in;\n  - ROM address reconstruction and response serialization;\n  - one-cycle fetch/execution pipeline latch;\n  - hard failure on floating/contentious samples or absent fixture words.\n",
    "- `src/machines/hp67/act.rs`\n  - ACT-owned fifteen-word display phase and A/B bit serializer;\n  - coarse RCD boundary after slot 15 without invented PHI placement.\n- `src/machines/hp67/fetch.rs`\n  - ACT display/address shift-out and ROM-word shift-in through one endpoint;\n  - ROM address reconstruction and response serialization;\n  - returned ACT display slot/RCD event rather than caller-supplied cathode phase;\n  - one-cycle fetch/execution pipeline latch;\n  - hard failure on floating/contentious samples or absent fixture words.\n",
    "timing research implementation map",
)
timing_doc_path.write_text(timing_doc)

# Guard against accidentally leaving production call sites that still pass a
# cathode scan slot into the combined runner. Diagnostic snapshot helpers are
# intentionally excluded from this check.
for path in [Path("src/hp67.rs"), Path("src/bin/hp67_poweron_smoke.rs")]:
    text = path.read_text()
    if "run_structural_display_fetch_cycle(\n" not in text:
        raise SystemExit(f"combined runner missing from {path}")
