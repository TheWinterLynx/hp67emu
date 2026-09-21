//! Dense HP-67 electrical fabric and staged scheduler primitive.
//!
//! The generic scheduler remains useful as a model-independent reference, but
//! its map-backed snapshot/pending machinery is deliberately not the production
//! path for a calculator whose net and device inventory is fixed. This module
//! provides the same resolve -> snapshot -> evaluate/stage -> commit contract
//! with compile-time HP-67 net/driver slots and no per-tick heap allocation.

use crate::emulation::{Bias, Drive, LogicLevel, Tick};

use super::wiring::{Hp67Driver, Hp67Net};

const MAX_PENDING_DRIVES: usize = Hp67Net::COUNT * Hp67Driver::COUNT;

#[derive(Debug, Clone)]
struct Hp67DenseNet {
    bias: Bias,
    drives: [Drive; Hp67Driver::COUNT],
    low_driver_count: u8,
    high_driver_count: u8,
}

impl Hp67DenseNet {
    const fn new(bias: Bias) -> Self {
        Self {
            bias,
            drives: [Drive::HighZ; Hp67Driver::COUNT],
            low_driver_count: 0,
            high_driver_count: 0,
        }
    }

    fn set_drive(&mut self, driver: Hp67Driver, drive: Drive) {
        let slot = &mut self.drives[driver.index()];
        let previous = *slot;
        if previous == drive {
            return;
        }

        match previous {
            Drive::HighZ => {}
            Drive::Low => self.low_driver_count -= 1,
            Drive::High => self.high_driver_count -= 1,
        }
        *slot = drive;
        match drive {
            Drive::HighZ => {}
            Drive::Low => self.low_driver_count += 1,
            Drive::High => self.high_driver_count += 1,
        }
    }

    const fn level(&self) -> LogicLevel {
        match (self.low_driver_count != 0, self.high_driver_count != 0) {
            (true, true) => LogicLevel::Contention,
            (true, false) => LogicLevel::Low,
            (false, true) => LogicLevel::High,
            (false, false) => match self.bias {
                Bias::Floating => LogicLevel::Floating,
                Bias::PullDown => LogicLevel::Low,
                Bias::PullUp => LogicLevel::High,
            },
        }
    }

    fn drive(&self, driver: Hp67Driver) -> Drive {
        self.drives[driver.index()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hp67ElectricalError {
    Contention {
        tick: Tick,
        net: Hp67Net,
        drivers: Vec<Hp67Driver>,
    },
}

/// Immutable resolved input image shared by every device evaluation in one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67ElectricalSnapshot {
    tick: Tick,
    levels: [LogicLevel; Hp67Net::COUNT],
}

impl Hp67ElectricalSnapshot {
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    pub const fn level(&self, net: Hp67Net) -> LogicLevel {
        self.levels[net.index()]
    }
}

/// Fixed-topology HP-67 electrical state and staged drive commit buffer.
///
/// Normal ticks allocate nothing. Pending output changes are indexed directly
/// by net and driver, and a fixed dirty-key array makes commit cost proportional
/// to outputs that actually changed rather than all possible combinations.
#[derive(Debug, Clone)]
pub struct Hp67ElectricalFabric {
    tick: Tick,
    nets: [Hp67DenseNet; Hp67Net::COUNT],
    pending: [[Option<Drive>; Hp67Driver::COUNT]; Hp67Net::COUNT],
    dirty_keys: [(u8, u8); MAX_PENDING_DRIVES],
    dirty_len: usize,
}

impl Default for Hp67ElectricalFabric {
    fn default() -> Self {
        let mut nets = std::array::from_fn(|index| {
            let net = Hp67Net::ALL[index];
            let bias = if net == Hp67Net::Isa {
                Bias::PullDown
            } else {
                Bias::Floating
            };
            Hp67DenseNet::new(bias)
        });

        // Direct HP-67 captures show both ACT clock outputs high between their
        // alternating low-going pulses.
        nets[Hp67Net::Phi1.index()].set_drive(Hp67Driver::Act1820_2530, Drive::High);
        nets[Hp67Net::Phi2.index()].set_drive(Hp67Driver::Act1820_2530, Drive::High);

        Self {
            tick: Tick::ZERO,
            nets,
            pending: [[None; Hp67Driver::COUNT]; Hp67Net::COUNT],
            dirty_keys: [(0, 0); MAX_PENDING_DRIVES],
            dirty_len: 0,
        }
    }
}

impl Hp67ElectricalFabric {
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    pub fn level(&self, net: Hp67Net) -> LogicLevel {
        self.nets[net.index()].level()
    }

    pub fn drive(&self, net: Hp67Net, driver: Hp67Driver) -> Drive {
        self.nets[net.index()].drive(driver)
    }

    /// Immediate committed drive used by the current M14A structural bridge.
    ///
    /// M14B device evaluation should prefer stage_drive plus commit_staged so
    /// every device reads one immutable snapshot.
    pub fn set_drive_immediate(&mut self, net: Hp67Net, driver: Hp67Driver, drive: Drive) {
        self.nets[net.index()].set_drive(driver, drive);
    }

    /// Advance canonical electrical time without staged outputs.
    ///
    /// This exists for the current explicit PHI bridge, where the edge itself is
    /// already the committed event. Device-scheduled M14B work uses
    /// commit_staged instead.
    pub fn advance_tick(&mut self) -> Tick {
        self.tick = Tick::new(self.tick.get().wrapping_add(1));
        self.tick
    }

    /// Resolve one immutable input image before any device publishes next-tick outputs.
    pub fn snapshot(&self) -> Result<Hp67ElectricalSnapshot, Hp67ElectricalError> {
        self.ensure_no_contention(self.tick)?;
        Ok(Hp67ElectricalSnapshot {
            tick: self.tick,
            levels: std::array::from_fn(|index| self.nets[index].level()),
        })
    }

    /// Stage one device output without changing any currently resolved input.
    ///
    /// Repeated writes by the same physical driver to the same net in one
    /// evaluation interval use final-write-wins semantics.
    pub fn stage_drive(&mut self, net: Hp67Net, driver: Hp67Driver, drive: Drive) {
        let net_index = net.index();
        let driver_index = driver.index();
        if self.pending[net_index][driver_index].is_none() {
            debug_assert!(self.dirty_len < MAX_PENDING_DRIVES);
            self.dirty_keys[self.dirty_len] = (net_index as u8, driver_index as u8);
            self.dirty_len += 1;
        }
        self.pending[net_index][driver_index] = Some(drive);
    }

    /// Commit every staged device output atomically and advance one scheduler tick.
    pub fn commit_staged(&mut self) -> Result<Tick, Hp67ElectricalError> {
        for dirty_index in 0..self.dirty_len {
            let (net_index, driver_index) = self.dirty_keys[dirty_index];
            let net_index = usize::from(net_index);
            let driver_index = usize::from(driver_index);
            let drive = self.pending[net_index][driver_index]
                .take()
                .expect("dirty HP-67 electrical slot must contain a staged drive");
            let driver = Hp67Driver::ALL[driver_index];
            self.nets[net_index].set_drive(driver, drive);
        }
        self.dirty_len = 0;
        let tick = self.advance_tick();
        self.ensure_no_contention(tick)?;
        Ok(tick)
    }

    fn ensure_no_contention(&self, tick: Tick) -> Result<(), Hp67ElectricalError> {
        for net in Hp67Net::ALL {
            let dense = &self.nets[net.index()];
            if dense.level() == LogicLevel::Contention {
                let drivers = Hp67Driver::ALL
                    .into_iter()
                    .filter(|driver| dense.drive(*driver) != Drive::HighZ)
                    .collect();
                return Err(Hp67ElectricalError::Contention { tick, net, drivers });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidenced_idle_levels_are_encoded_in_dense_fabric() {
        let fabric = Hp67ElectricalFabric::default();
        for net in Hp67Net::ALL {
            let expected = match net {
                Hp67Net::Phi1 | Hp67Net::Phi2 => LogicLevel::High,
                Hp67Net::Isa => LogicLevel::Low,
                _ => LogicLevel::Floating,
            };
            assert_eq!(fabric.level(net), expected, "unexpected idle level for {net:?}");
        }
    }

    #[test]
    fn staged_outputs_are_invisible_until_atomic_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        let snapshot = fabric.snapshot().unwrap();
        assert_eq!(snapshot.tick(), Tick::ZERO);
        assert_eq!(snapshot.level(Hp67Net::Data), LogicLevel::Floating);

        fabric.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::High);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Floating);
        assert_eq!(snapshot.level(Hp67Net::Data), LogicLevel::Floating);

        assert_eq!(fabric.commit_staged(), Ok(Tick::new(1)));
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);
        assert_eq!(snapshot.level(Hp67Net::Data), LogicLevel::Floating);
    }

    #[test]
    fn repeated_staging_by_one_driver_is_final_write_wins_without_duplicate_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::High);
        fabric.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::Low);
        fabric.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::HighZ);
        assert_eq!(fabric.commit_staged(), Ok(Tick::new(1)));
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Floating);
    }

    #[test]
    fn simultaneous_opposite_drives_report_contention_after_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::High);
        fabric.stage_drive(
            Hp67Net::Data,
            Hp67Driver::StructuralRomResponder,
            Drive::Low,
        );

        assert_eq!(
            fabric.commit_staged(),
            Err(Hp67ElectricalError::Contention {
                tick: Tick::new(1),
                net: Hp67Net::Data,
                drivers: vec![
                    Hp67Driver::Act1820_2530,
                    Hp67Driver::StructuralRomResponder,
                ],
            })
        );
    }

    #[test]
    fn dense_resolution_matches_generic_net_for_all_two_driver_states() {
        use crate::emulation::{DriverId, Net};

        const REF_ACT: DriverId = DriverId::new("reference-act");
        const REF_ROM: DriverId = DriverId::new("reference-rom");
        let drives = [Drive::HighZ, Drive::Low, Drive::High];

        for (net, bias) in [
            (Hp67Net::Data, Bias::Floating),
            (Hp67Net::Isa, Bias::PullDown),
        ] {
            for act_drive in drives {
                for rom_drive in drives {
                    let mut reference = Net::new(bias);
                    reference.set_drive(REF_ACT, act_drive);
                    reference.set_drive(REF_ROM, rom_drive);

                    let mut dense = Hp67ElectricalFabric::default();
                    dense.set_drive_immediate(net, Hp67Driver::Act1820_2530, act_drive);
                    dense.set_drive_immediate(
                        net,
                        Hp67Driver::StructuralRomResponder,
                        rom_drive,
                    );

                    assert_eq!(
                        dense.level(net),
                        reference.level(),
                        "dense resolver drift for {net:?}: ACT={act_drive:?}, ROM={rom_drive:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn dense_driver_slots_update_resolved_level_in_constant_index_space() {
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.set_drive_immediate(Hp67Net::Isa, Hp67Driver::Act1820_2530, Drive::High);
        assert_eq!(fabric.level(Hp67Net::Isa), LogicLevel::High);
        fabric.set_drive_immediate(Hp67Net::Isa, Hp67Driver::Act1820_2530, Drive::HighZ);
        assert_eq!(fabric.level(Hp67Net::Isa), LogicLevel::Low);
    }
}
