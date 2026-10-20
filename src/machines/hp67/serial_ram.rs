//! Autonomous aggregate RAM participant for the logical-level DATA bus.
//!
//! Commands come only from resolved IS/SYNC in the preceding word. Address
//! and write frames come only from resolved DATA. This is a separate composition
//! from the desktop's existing authority bridge, not physical RAM-chip mapping.

use super::{
    act::{ActRegister, ACT_WORD_DIGITS},
    data::data_register_serial_bit,
    machine::Hp67ElectricalBackplane,
    ram_command::{Hp67RamCommand as Command, Hp67RamCommandError, Hp67RamCommandReceiver},
    timing::{BITS_PER_WORD, ROM_WORD_FIRST_BIT, ROM_WORD_LAST_BIT},
    wiring::{Hp67Driver, Hp67Net},
};
use crate::emulation::{Drive, LogicLevel};

pub const HP67_SERIAL_RAM_WORDS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67SerialRamError {
    InvalidPhase {
        word_bit: u8,
        tick: u64,
    },
    InvalidLifecycle,
    UnexpectedBit {
        expected: u8,
        actual: u8,
    },
    BodyNotBound,
    IncompleteDataFrame,
    Command(Hp67RamCommandError),
    InvalidData {
        word_bit: u8,
        level: LogicLevel,
        active_drivers: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67SerialRamEvent {
    AddressLatched { address: u8 },
    WriteCommitted { address: u8 },
    BlockCleared { base: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Destination {
    Address,
    Register(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IncomingFrame {
    destination: Destination,
    word: ActRegister,
    received: u8,
}

/// Sixty-four logical registers with independent serial command/address/data
/// state. There is no ACT state, architectural RAM image or host payload input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hp67SerialRam {
    registers: [ActRegister; HP67_SERIAL_RAM_WORDS],
    address: u8,
    next_command: Command,
    executing: Command,
    active: bool,
    expected_bit: u8,
    body_bound: bool,
    command_receiver: Hp67RamCommandReceiver,
    incoming: Option<IncomingFrame>,
    read_body: Option<ActRegister>,
    read_tail: Option<[bool; 2]>,
}

impl Default for Hp67SerialRam {
    fn default() -> Self {
        Self {
            registers: [[0; ACT_WORD_DIGITS]; HP67_SERIAL_RAM_WORDS],
            address: 0,
            next_command: Command::Idle,
            executing: Command::Idle,
            active: false,
            expected_bit: 0,
            body_bound: false,
            command_receiver: Hp67RamCommandReceiver::default(),
            incoming: None,
            read_body: None,
            read_tail: None,
        }
    }
}

impl Hp67SerialRam {
    pub const fn address(&self) -> u8 {
        self.address
    }

    /// Read-only inspection, never the transport path used by ACT.
    pub fn register(&self, address: u8) -> Option<&ActRegister> {
        self.registers.get(usize::from(address))
    }

    /// Reset receive/control state and logical RAM using the existing cold
    /// logical-RAM convention, not a claim about physical power-up cell values.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn begin_cycle(&mut self, bus: &Hp67ElectricalBackplane) -> Result<(), Hp67SerialRamError> {
        if self.active {
            return Err(Hp67SerialRamError::InvalidLifecycle);
        }
        if bus.word_bit() != 0 || !bus.at_transport_launch_boundary() {
            return Err(Hp67SerialRamError::InvalidPhase {
                word_bit: bus.word_bit(),
                tick: bus.tick().get(),
            });
        }
        self.active = true;
        self.expected_bit = 0;
        self.body_bound = false;
        self.command_receiver = Hp67RamCommandReceiver::default();
        self.executing = self.next_command;
        self.next_command = Command::Idle;
        Ok(())
    }

    /// Bind after the previous frame's b0/b1 tail, before launching b2. This is
    /// a composition checkpoint, not newly established RAM silicon latch timing.
    pub fn bind_body(&mut self, bus: &Hp67ElectricalBackplane) -> Result<(), Hp67SerialRamError> {
        if !self.active || self.body_bound {
            return Err(Hp67SerialRamError::InvalidLifecycle);
        }
        if bus.word_bit() != 2 || !bus.at_transport_launch_boundary() || self.expected_bit != 2 {
            return Err(Hp67SerialRamError::InvalidPhase {
                word_bit: bus.word_bit(),
                tick: bus.tick().get(),
            });
        }
        if self.incoming.is_some() {
            return Err(Hp67SerialRamError::IncompleteDataFrame);
        }
        let selected = match self.executing {
            Command::Read(Some(low)) | Command::Write(Some(low)) => (self.address & 0xf0) | low,
            _ => self.address,
        };
        self.address = selected;
        self.read_body = match self.executing {
            Command::Read(_) => self.register(selected).copied(),
            _ => None,
        };
        let destination = match self.executing {
            Command::Address => Some(Destination::Address),
            Command::Write(_) if usize::from(selected) < HP67_SERIAL_RAM_WORDS => {
                Some(Destination::Register(selected))
            }
            _ => None,
        };
        self.incoming = destination.map(|destination| IncomingFrame {
            destination,
            word: [0; ACT_WORD_DIGITS],
            received: 0,
        });
        self.body_bound = true;
        Ok(())
    }

    fn check_coordinate(&self, bus: &Hp67ElectricalBackplane) -> Result<(), Hp67SerialRamError> {
        if !self.active {
            return Err(Hp67SerialRamError::InvalidLifecycle);
        }
        let bit = bus.word_bit();
        if bit != self.expected_bit {
            return Err(Hp67SerialRamError::UnexpectedBit {
                expected: self.expected_bit,
                actual: bit,
            });
        }
        if bit >= 2 && !self.body_bound {
            return Err(Hp67SerialRamError::BodyNotBound);
        }
        Ok(())
    }

    /// Immutable RAM contribution for the caller's joint serial-source commit.
    /// High/Low remain the explicit logical-level adapter, not output circuitry.
    pub fn propose_data_drive(
        &self,
        bus: &Hp67ElectricalBackplane,
    ) -> Result<Drive, Hp67SerialRamError> {
        self.check_coordinate(bus)?;
        let bit = bus.word_bit();
        if !bus.at_transport_launch_boundary() {
            return Err(Hp67SerialRamError::InvalidPhase {
                word_bit: bit,
                tick: bus.tick().get(),
            });
        }
        let logical = if bit < 2 {
            self.read_tail.map(|tail| tail[usize::from(bit)])
        } else {
            self.read_body
                .as_ref()
                .map(|word| data_register_serial_bit(word, bit - 2))
        };
        Ok(match logical {
            Some(true) => Drive::High,
            Some(false) => Drive::Low,
            None => Drive::HighZ,
        })
    }

    /// Validate all input samples first, then mutate the device. A failed IS,
    /// SYNC, DATA-owner or phase check cannot partially consume another input.
    pub fn sample_at_edge(
        &mut self,
        bus: &Hp67ElectricalBackplane,
    ) -> Result<Option<Hp67SerialRamEvent>, Hp67SerialRamError> {
        self.check_coordinate(bus)?;
        let bit = bus.word_bit();
        if !bus.at_transport_sample_edge() {
            return Err(Hp67SerialRamError::InvalidPhase {
                word_bit: bit,
                tick: bus.tick().get(),
            });
        }
        // Copy only the small receiver image, never the RAM array. Commit it
        // after DATA validation so an invalid input cannot consume the other.
        let mut command_receiver = self.command_receiver;
        if (ROM_WORD_FIRST_BIT..=ROM_WORD_LAST_BIT).contains(&bit) {
            command_receiver
                .sample_at_edge(bus)
                .map_err(Hp67SerialRamError::Command)?;
        }
        let data_sample = if self.incoming.is_some() {
            let level = bus.level(Hp67Net::Data);
            let active_drivers = bus.active_data_driver_count();
            if active_drivers != 1
                || bus.driver_drive(Hp67Net::Data, Hp67Driver::Act1820_2530) == Drive::HighZ
            {
                return Err(Hp67SerialRamError::InvalidData {
                    word_bit: bit,
                    level,
                    active_drivers,
                });
            }
            Some(match level {
                LogicLevel::High => true,
                LogicLevel::Low => false,
                _ => {
                    return Err(Hp67SerialRamError::InvalidData {
                        word_bit: bit,
                        level,
                        active_drivers,
                    })
                }
            })
        } else {
            None
        };
        // All fallible observation is above this line.
        self.command_receiver = command_receiver;
        let mut event = None;
        if let Some(value) = data_sample {
            let frame = self.incoming.as_mut().expect("validated incoming frame");
            let serial = frame.received;
            if value {
                frame.word[usize::from(serial / 4)] |= 1 << (serial % 4);
            }
            frame.received += 1;
            if frame.received == BITS_PER_WORD {
                let frame = self.incoming.take().expect("completed incoming frame");
                event = Some(match frame.destination {
                    Destination::Address => {
                        self.address = (frame.word[1] << 4) | frame.word[0];
                        Hp67SerialRamEvent::AddressLatched {
                            address: self.address,
                        }
                    }
                    Destination::Register(address) => {
                        self.registers[usize::from(address)] = frame.word;
                        Hp67SerialRamEvent::WriteCommitted { address }
                    }
                });
            }
        }
        if bit == 1 {
            self.read_tail = None;
        }
        if bit == BITS_PER_WORD - 1 {
            self.next_command = self
                .command_receiver
                .received_frame()
                .expect("all ten IS/SYNC bits validated")
                .command();
            self.read_tail = self.read_body.take().map(|word| {
                [
                    data_register_serial_bit(&word, 54),
                    data_register_serial_bit(&word, 55),
                ]
            });
            if self.executing == Command::Clear {
                let base = self.address & 0xf0;
                if usize::from(base) < HP67_SERIAL_RAM_WORDS {
                    self.registers[usize::from(base)..usize::from(base) + 16]
                        .fill([0; ACT_WORD_DIGITS]);
                    event = Some(Hp67SerialRamEvent::BlockCleared { base });
                }
            }
            self.active = false;
            self.expected_bit = 0;
        } else {
            self.expected_bit += 1;
        }
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{
        data::Hp67DataSerialSource,
        fetch::Hp67RomWordSource,
        hp67firmware::Hp67Firmware,
        isa::{act_address_drive, rom_word_drive},
        rom_devices::RomIsDrives,
    };

    struct Rig {
        bus: Hp67ElectricalBackplane,
        ram: Hp67SerialRam,
        act: Hp67DataSerialSource,
        collecting: Option<(ActRegister, u8)>,
        last_read: Option<ActRegister>,
        reads: usize,
        events: Vec<(u64, u8, Hp67SerialRamEvent)>,
        record_events: bool,
    }

    impl Rig {
        fn new() -> Self {
            Self {
                bus: Hp67ElectricalBackplane::default(),
                ram: Hp67SerialRam::default(),
                act: Hp67DataSerialSource::default(),
                collecting: None,
                last_read: None,
                reads: 0,
                events: Vec::new(),
                record_events: true,
            }
        }

        fn word(&mut self, fetched: u16, payload: Option<ActRegister>) {
            self.word_with(fetched, true, payload, |_, _, _| {});
        }

        fn word_with<F: FnMut(u8, &mut Hp67ElectricalBackplane, &mut Hp67SerialRam)>(
            &mut self,
            fetched: u16,
            instruction: bool,
            payload: Option<ActRegister>,
            mut inspect: F,
        ) {
            let start = self.bus.tick().get();
            self.ram.begin_cycle(&self.bus).unwrap();
            self.act.begin_word(payload);
            for bit in 0..BITS_PER_WORD {
                if bit == 2 {
                    self.ram.bind_body(&self.bus).unwrap();
                }
                let ram_drive = self.ram.propose_data_drive(&self.bus).unwrap();
                let act_drive = match self.act.logical_bit_for_word_bit(bit) {
                    Some(true) => Drive::High,
                    Some(false) => Drive::Low,
                    None => Drive::HighZ,
                };
                let sync = if (46..56).contains(&bit) && instruction {
                    Drive::High
                } else {
                    Drive::Low
                };
                self.bus.commit_serial_launch(
                    act_address_drive(0x123, bit),
                    RomIsDrives::Aggregate(rom_word_drive(fetched, bit)),
                    sync,
                    Some((act_drive, ram_drive)),
                );
                self.bus.advance_clock();
                self.bus.advance_clock();
                inspect(bit, &mut self.bus, &mut self.ram);
                if let Some(event) = self.ram.sample_at_edge(&self.bus).unwrap() {
                    if self.record_events {
                        self.events.push((self.bus.word_index(), bit, event));
                    }
                }
                if ram_drive != Drive::HighZ {
                    // ACT-side collection reads the actual net, not RAM storage.
                    assert_eq!(self.bus.active_data_driver_count(), 1);
                    assert_ne!(
                        self.bus
                            .driver_drive(Hp67Net::Data, Hp67Driver::StructuralRomResponder),
                        Drive::HighZ
                    );
                    let serial = if bit < 2 { bit + 54 } else { bit - 2 };
                    if serial == 0 {
                        assert!(self.collecting.is_none());
                        self.collecting = Some(([0; ACT_WORD_DIGITS], 0));
                    }
                    let frame = self.collecting.as_mut().expect("RAM return frame");
                    assert_eq!(serial, frame.1);
                    match self.bus.level(Hp67Net::Data) {
                        LogicLevel::High => frame.0[usize::from(serial / 4)] |= 1 << (serial % 4),
                        LogicLevel::Low => {}
                        level => panic!("invalid ACT receive level: {level:?}"),
                    }
                    frame.1 += 1;
                    if frame.1 == 56 {
                        self.last_read = Some(self.collecting.take().unwrap().0);
                        self.reads += 1;
                    }
                }
                self.bus.advance_clock();
                self.bus.advance_clock();
            }
            self.act.complete_word();
            assert_eq!(self.bus.tick().get() - start, 224);
        }
    }

    fn address_word(address: u8) -> ActRegister {
        let mut word = [9; ACT_WORD_DIGITS];
        word[0] = address & 0xf;
        word[1] = address >> 4;
        word
    }

    fn pattern(seed: u8) -> ActRegister {
        std::array::from_fn(|digit| seed.wrapping_add(digit as u8 * 3) & 0xf)
    }

    #[test]
    fn m15n_real_firmware_port_words_drive_address_write_and_read_without_host_ram() {
        let source = Hp67Firmware::default();
        let address = source.read_word_in_bank(0, 0xcec).unwrap();
        let write = source.read_word_in_bank(0, 0xcd2).unwrap();
        let read = source.read_word_in_bank(0, 0xced).unwrap();
        assert_eq!((address, write, read), (0x270, 0x2f0, 0x038));
        let value = pattern(7);
        let mut rig = Rig::new();
        rig.word(address, None);
        rig.word(write, Some(address_word(1)));
        assert_eq!(rig.ram.address(), 0); // Address frame still lacks its tail.
        rig.word(read, Some(value));
        assert_eq!(rig.ram.address(), 1);
        assert_eq!(rig.ram.register(1), Some(&[0; ACT_WORD_DIGITS]));
        rig.word_with(0, true, None, |bit, _, ram| {
            if bit <= 1 {
                assert_eq!(ram.register(1), Some(&[0; ACT_WORD_DIGITS]));
            }
        });
        assert_eq!(rig.ram.register(1), Some(&value));
        assert_eq!(rig.reads, 0); // Read also requires the following b0/b1.
        rig.word(0, None);
        assert_eq!(rig.last_read, Some(value));
        assert_eq!(rig.reads, 1);
        assert_eq!(
            rig.events,
            vec![
                (2, 1, Hp67SerialRamEvent::AddressLatched { address: 1 }),
                (3, 1, Hp67SerialRamEvent::WriteCommitted { address: 1 }),
            ]
        );
        assert_eq!(rig.bus.tick().get(), 5 * 224);
    }

    #[test]
    fn m15n_all_64_addresses_support_direct_and_low_selector_port_families() {
        for selected in [false, true] {
            for address in 0..64u8 {
                let mut rig = Rig::new();
                let value = pattern(address);
                let initial = if selected {
                    (address & 0xf0) | (address.wrapping_add(1) & 0xf)
                } else {
                    address
                };
                let low = u16::from(address & 0xf) << 6;
                let (write, read) = if selected {
                    (low | 0o50, low | 0o70)
                } else {
                    (0o1360, 0o0070)
                };
                rig.word(0o1160, None);
                rig.word(write, Some(address_word(initial)));
                rig.word(read, Some(value));
                rig.word(0, None);
                rig.word(0, None);
                assert_eq!(rig.ram.address(), address);
                assert_eq!(rig.last_read, Some(value));
                for other in 0..64u8 {
                    assert_eq!(
                        rig.ram.register(other),
                        Some(if other == address {
                            &value
                        } else {
                            &[0; ACT_WORD_DIGITS]
                        })
                    );
                }
            }
        }
    }

    #[test]
    fn m15n_uninstalled_addresses_release_data_and_implied_goto_payload_is_not_a_command() {
        for address in [0x40, 0x99, 0x9b, 0xff] {
            let mut rig = Rig::new();
            rig.word(0o1160, None);
            rig.word(0o1360, Some(address_word(address)));
            rig.word(0o0070, Some(pattern(5)));
            rig.word(0, None);
            rig.word(0, None);
            assert_eq!(rig.ram.address(), address);
            assert_eq!(rig.reads, 0);
            assert_eq!(
                rig.events,
                vec![(2, 1, Hp67SerialRamEvent::AddressLatched { address })]
            );
            assert!(rig.ram.register(address).is_none());
            assert!(rig
                .ram
                .registers
                .iter()
                .all(|word| *word == [0; ACT_WORD_DIGITS]));
        }
        for opcode in [0o1160, 0o1360, 0o0070, 0o1260, 0o0150, 0o0170] {
            let mut rig = Rig::new();
            rig.word_with(opcode, false, None, |_, _, _| {});
            rig.word(0, Some(pattern(4)));
            rig.word(0, None);
            assert_eq!(rig.ram.address(), 0);
            assert!(rig.events.is_empty());
            assert_eq!(rig.reads, 0);
        }
    }

    #[test]
    fn m15n_clear_uses_logical_block_semantics_and_preserves_neighbours() {
        for address in [0x0f, 0x1f, 0x2f, 0x3f] {
            let mut rig = Rig::new();
            // Initial test memory image only; runtime commands still enter IS.
            rig.ram.registers = std::array::from_fn(|index| pattern(index as u8));
            let before = rig.ram.registers;
            rig.word(0o1160, None);
            rig.word(0o1260, Some(address_word(address)));
            rig.word(0, None);
            for index in 0..64 {
                let expected = if (index as u8 & 0xf0) == (address & 0xf0) {
                    [0; ACT_WORD_DIGITS]
                } else {
                    before[index]
                };
                assert_eq!(rig.ram.registers[index], expected);
            }
            assert_eq!(
                rig.events.last(),
                Some(&(
                    2,
                    55,
                    Hp67SerialRamEvent::BlockCleared {
                        base: address & 0xf0
                    }
                ))
            );
            rig.ram.reset();
            assert_eq!(rig.ram, Hp67SerialRam::default());
        }
    }

    #[test]
    fn m15n_faults_are_atomic_across_instruction_and_data_samples() {
        let mut rig = Rig::new();
        rig.word(0o1160, None);
        rig.word(0o1360, Some(address_word(0x20)));
        rig.word_with(0o0070, true, Some(pattern(2)), |bit, bus, ram| {
            let before = ram.clone();
            for fault in 0..7 {
                let mut broken = bus.clone();
                match fault {
                    0 => {
                        broken.advance_clock();
                    }
                    1 => broken.drive(Hp67Net::Data, Hp67Driver::External, Drive::High),
                    2 => broken.drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::HighZ),
                    3 => {
                        let drive = broken.driver_drive(Hp67Net::Data, Hp67Driver::Act1820_2530);
                        broken.commit_data_level_bridge(Drive::HighZ, drive);
                    }
                    4 if bit >= 46 => {
                        broken.drive(Hp67Net::Isa, Hp67Driver::Act1820_2530, Drive::Low);
                        broken.drive(
                            Hp67Net::Isa,
                            Hp67Driver::StructuralRomResponder,
                            Drive::High,
                        );
                    }
                    5 if bit >= 46 => broken.drive(Hp67Net::Sync, Hp67Driver::External, Drive::Low),
                    6 if bit > 46 => {
                        broken.drive(Hp67Net::Sync, Hp67Driver::Act1820_2530, Drive::Low)
                    }
                    _ => continue,
                }
                assert!(
                    ram.sample_at_edge(&broken).is_err(),
                    "bit={bit} fault={fault}"
                );
                assert_eq!(*ram, before);
            }
            // Replaying an already consumed coordinate must also fail atomically.
            let mut replay = ram.clone();
            replay.sample_at_edge(bus).unwrap();
            let after = replay.clone();
            assert!(replay.sample_at_edge(bus).is_err());
            assert_eq!(replay, after);
        });
        rig.word(0, None);
        rig.word(0, None);
        assert_eq!(rig.last_read, Some(pattern(2)));
    }

    #[test]
    fn m15n_ram_stores_observed_data_instead_of_the_act_source_payload() {
        let mut rig = Rig::new();
        rig.word(0o1160, None);
        rig.word(0o1360, Some(address_word(2)));
        // ACT's original data payload is zero. Change its committed body bit
        // before sampling; RAM has no host-payload comparison/override path.
        rig.word_with(0o0070, true, Some([0; ACT_WORD_DIGITS]), |bit, bus, _| {
            if bit == 2 {
                bus.drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::High);
            }
        });
        rig.word(0, None);
        rig.word(0, None);
        let mut expected = [0; ACT_WORD_DIGITS];
        expected[0] = 1;
        assert_eq!(rig.ram.register(2), Some(&expected));
        assert_eq!(rig.last_read, Some(expected));
    }

    #[test]
    fn m15n_lifecycle_and_early_body_binding_fail_without_state_changes() {
        let mut rig = Rig::new();
        let initial = rig.ram.clone();
        assert_eq!(
            rig.ram.propose_data_drive(&rig.bus),
            Err(Hp67SerialRamError::InvalidLifecycle)
        );
        assert_eq!(rig.ram, initial);
        rig.word_with(0, true, None, |bit, bus, ram| {
            let before = ram.clone();
            assert_eq!(
                ram.begin_cycle(bus),
                Err(Hp67SerialRamError::InvalidLifecycle)
            );
            assert!(ram.bind_body(bus).is_err());
            assert_eq!(*ram, before);
            if bit >= 2 {
                assert!(ram.body_bound);
            }
        });
    }

    #[test]
    #[ignore = "release-only autonomous serial RAM benchmark; run explicitly"]
    fn m15n_autonomous_serial_ram_realtime_benchmark() {
        use std::{hint::black_box, time::Instant};
        assert!(!cfg!(debug_assertions), "run with --release");
        const PAIRS: usize = 5_000;
        const ROUNDS: usize = 7;
        let mut rig = Rig::new();
        rig.record_events = false;
        rig.word(0o1160, None);
        rig.word(0o1360, Some(address_word(0x20)));
        let mut times = Vec::with_capacity(ROUNDS);
        for _ in 0..ROUNDS {
            let start = Instant::now();
            for index in 0..PAIRS {
                rig.word(0o0070, Some(pattern(index as u8)));
                rig.word(0o1360, None);
            }
            black_box((&rig.bus, &rig.ram, rig.last_read));
            times.push(start.elapsed().as_secs_f64() * 1_000_000.0 / (2 * PAIRS) as f64);
        }
        times.sort_by(f64::total_cmp);
        let median = times[ROUNDS / 2];
        println!("\nM15N AUTONOMOUS SERIAL RAM / ELECTRICAL FIXTURE");
        println!(
            "median={median:.3} us/word; words/s={:.0}; vs hardware={:.2}x; best={:.3} worst={:.3}",
            1_000_000.0 / median,
            320.0 / median,
            times[0],
            times[ROUNDS - 1]
        );
        println!("Scope: autonomous logical 64-register RAM, real resolved IS/SYNC/DATA, joint source commit, 56 cells/224 PHI transitions. Includes test ACT receive checks; excludes desktop live/oracle and physical RAM-chip mapping/output stage. No correctness speed threshold.");
        assert_eq!(rig.bus.tick().get(), (2 + ROUNDS * PAIRS * 2) as u64 * 224);
        assert_eq!(rig.reads, ROUNDS * PAIRS - 1);
    }
}
