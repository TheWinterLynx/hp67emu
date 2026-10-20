//! Dense HP-67 electrical fabric and staged scheduler primitive.
//!
//! The generic scheduler remains useful as a model-independent reference, but
//! its map-backed snapshot/pending machinery is deliberately not the production
//! path for a calculator whose net and device inventory is fixed. This module
//! provides the same resolve -> snapshot -> evaluate/stage -> commit contract
//! with compile-time HP-67 net/driver slots and no per-tick heap allocation.

use crate::emulation::{Bias, Drive, LogicLevel, Tick};

use super::{
    rom_devices::{RomIsDrives, HP67_ROM_PROFILES},
    wiring::{Hp67Driver, Hp67Net},
};

const MAX_PENDING_DRIVES: usize = Hp67Net::COUNT * Hp67Driver::COUNT;

#[derive(Debug, Clone, Copy)]
struct Hp67PendingDrive {
    net: Hp67Net,
    driver: Hp67Driver,
    drive: Drive,
}

const EMPTY_PENDING_DRIVE: Hp67PendingDrive = Hp67PendingDrive {
    net: Hp67Net::Phi1,
    driver: Hp67Driver::Act1820_2530,
    drive: Drive::HighZ,
};

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

    fn set_drive(&mut self, driver: Hp67Driver, drive: Drive) -> bool {
        let slot = &mut self.drives[driver.index()];
        let previous = *slot;
        if previous == drive {
            return false;
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
        true
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
    DriverAlreadyOwned {
        driver: Hp67Driver,
    },
}

/// Opaque one-time ownership token for one HP-67 electrical output identity.
///
/// A fabric issues at most one token for each Hp67Driver. Device composition
/// keeps the token and reuses it across every later evaluation, so ownership is
/// validated once rather than on every electrical edge.
#[derive(Debug, PartialEq, Eq)]
pub struct Hp67DriverOwner {
    driver: Hp67Driver,
}

impl Hp67DriverOwner {
    pub const fn driver(&self) -> Hp67Driver {
        self.driver
    }
}

/// Immutable resolved input image shared by every device evaluation in one tick.
#[derive(Debug, Clone, Copy)]
pub struct Hp67ElectricalSnapshot<'a> {
    tick: Tick,
    levels: &'a [LogicLevel; Hp67Net::COUNT],
}

impl<'a> Hp67ElectricalSnapshot<'a> {
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    pub const fn level(&self, net: Hp67Net) -> LogicLevel {
        self.levels[net.index()]
    }
}

/// Mutable staging half of one HP-67 device-evaluation interval.
///
/// It deliberately has no access to committed nets, so publishing outputs cannot
/// change what any device samples during the same evaluation interval.
pub struct Hp67ElectricalStager<'a> {
    pending_slot_markers: &'a mut [[u8; Hp67Driver::COUNT]; Hp67Net::COUNT],
    pending_changes: &'a mut [Hp67PendingDrive; MAX_PENDING_DRIVES],
    pending_len: &'a mut usize,
}

impl<'a> Hp67ElectricalStager<'a> {
    pub fn stage_drive(&mut self, owner: &Hp67DriverOwner, net: Hp67Net, drive: Drive) {
        let driver = owner.driver;
        let net_index = net.index();
        let driver_index = driver.index();
        let marker = &mut self.pending_slot_markers[net_index][driver_index];

        if *marker == 0 {
            debug_assert!(*self.pending_len < MAX_PENDING_DRIVES);
            let pending_index = *self.pending_len;
            self.pending_changes[pending_index] = Hp67PendingDrive { net, driver, drive };
            *marker = (pending_index + 1) as u8;
            *self.pending_len += 1;
        } else {
            let pending_index = usize::from(*marker - 1);
            debug_assert!(pending_index < *self.pending_len);
            self.pending_changes[pending_index].drive = drive;
        }
    }
}

/// Fixed-topology HP-67 electrical state and staged drive commit buffer.
///
/// Normal ticks allocate nothing. A fixed slot-marker matrix deduplicates
/// `(net, driver)` publication while a fixed typed pending list carries the
/// final staged drive directly into commit, so commit work stays proportional
/// to outputs that were staged rather than all possible combinations.
#[derive(Debug, Clone)]
pub struct Hp67ElectricalFabric {
    tick: Tick,
    nets: [Hp67DenseNet; Hp67Net::COUNT],
    resolved_levels: [LogicLevel; Hp67Net::COUNT],
    pending_slot_markers: [[u8; Hp67Driver::COUNT]; Hp67Net::COUNT],
    pending_changes: [Hp67PendingDrive; MAX_PENDING_DRIVES],
    pending_len: usize,
    owned_drivers: [bool; Hp67Driver::COUNT],
    contention_net_count: u8,
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

        let resolved_levels = std::array::from_fn(|index| nets[index].level());

        Self {
            tick: Tick::ZERO,
            nets,
            resolved_levels,
            pending_slot_markers: [[0; Hp67Driver::COUNT]; Hp67Net::COUNT],
            pending_changes: [EMPTY_PENDING_DRIVE; MAX_PENDING_DRIVES],
            pending_len: 0,
            owned_drivers: [false; Hp67Driver::COUNT],
            contention_net_count: 0,
        }
    }
}

impl Hp67ElectricalFabric {
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    pub const fn level(&self, net: Hp67Net) -> LogicLevel {
        self.resolved_levels[net.index()]
    }

    pub fn drive(&self, net: Hp67Net, driver: Hp67Driver) -> Drive {
        self.nets[net.index()].drive(driver)
    }

    /// Publish the current structural serial sources as one observable image.
    /// Optional DATA leaves that net untouched when the caller has no DATA role.
    /// External/other chip contributions survive. This commits no PHI/time edge.
    #[cfg(test)]
    pub(crate) fn commit_serial_launch(
        &mut self,
        act_is: Drive,
        rom_is: Drive,
        sync: Drive,
        data: Option<(Drive, Drive)>,
    ) {
        self.commit_serial_launch_with_roms(act_is, RomIsDrives::Aggregate(rom_is), sync, data);
    }

    pub(crate) fn commit_serial_launch_with_roms(
        &mut self,
        act_is: Drive,
        roms: RomIsDrives,
        sync: Drive,
        data: Option<(Drive, Drive)>,
    ) {
        self.commit_serial_launch_with_data_owner(
            act_is,
            roms,
            sync,
            data,
            Hp67Driver::StructuralRomResponder,
        );
    }

    pub(crate) fn commit_serial_launch_with_data_owner(
        &mut self,
        act_is: Drive,
        roms: RomIsDrives,
        sync: Drive,
        data: Option<(Drive, Drive)>,
        data_owner: Hp67Driver,
    ) {
        let isa = &mut self.nets[Hp67Net::Isa.index()];
        let mut isa_changed = isa.set_drive(Hp67Driver::Act1820_2530, act_is);
        match roms {
            RomIsDrives::Aggregate(drive) => {
                isa_changed |= isa.set_drive(Hp67Driver::StructuralRomResponder, drive);
            }
            RomIsDrives::Chips(drives) => {
                isa_changed |= isa.set_drive(Hp67Driver::StructuralRomResponder, Drive::HighZ);
                for (profile, drive) in HP67_ROM_PROFILES.into_iter().zip(drives) {
                    isa_changed |= isa.set_drive(profile.driver, drive);
                }
            }
        }
        let sync_changed =
            self.nets[Hp67Net::Sync.index()].set_drive(Hp67Driver::Act1820_2530, sync);
        let data_changed = if let Some((act, ram)) = data {
            let data_net = &mut self.nets[Hp67Net::Data.index()];
            data_net.set_drive(Hp67Driver::Act1820_2530, act) | data_net.set_drive(data_owner, ram)
        } else {
            false
        };
        // No receiver/snapshot can observe the partially updated contribution
        // set. Resolve only after all source proposals have been installed.
        for (net, changed) in [
            (Hp67Net::Isa, isa_changed),
            (Hp67Net::Sync, sync_changed),
            (Hp67Net::Data, data_changed),
        ] {
            if changed {
                self.refresh_resolved_level(net.index());
            }
        }
    }

    /// Commit the two structural DATA source contributions as one observable
    /// image. This is a logical-level bridge, not a physical output-stage model.
    pub fn commit_data_level_bridge(&mut self, act: Drive, ram: Drive) {
        let index = Hp67Net::Data.index();
        self.nets[index].set_drive(Hp67Driver::Act1820_2530, act);
        self.nets[index].set_drive(Hp67Driver::StructuralRomResponder, ram);
        self.refresh_resolved_level(index);
    }

    /// Number of active logical contributors, independent of resolved level.
    pub fn active_driver_count(&self, net: Hp67Net) -> u8 {
        let net = &self.nets[net.index()];
        net.low_driver_count + net.high_driver_count
    }

    /// Observable DATA contributions, including additional physical/stimulus slots.
    pub fn active_data_driver_count(&self) -> u8 {
        let data = &self.nets[Hp67Net::Data.index()];
        data.low_driver_count + data.high_driver_count
    }

    /// IS permits active-high/release contributions from at most one driver.
    ///
    /// This checks the source-backed ownership policy, independently of logic
    /// resolution: two High contributions still resolve High. It does not infer
    /// output-enable timing or check roles that are currently releasing the bus.
    pub fn isa_drives_valid(&self) -> bool {
        let isa = &self.nets[Hp67Net::Isa.index()];
        isa.low_driver_count == 0 && isa.high_driver_count <= 1
    }

    /// Diagnostic identities of all active IS contributors, in dense-slot order.
    /// Called only after an ownership/polarity violation, never on a valid cell.
    pub fn active_isa_driver_mask(&self) -> u16 {
        Hp67Driver::ALL.into_iter().fold(0, |mask, driver| {
            if self.drive(Hp67Net::Isa, driver) == Drive::HighZ {
                mask
            } else {
                mask | (1u16 << driver.index())
            }
        })
    }

    /// Claim one physical output identity once during machine composition.
    ///
    /// The returned token is intentionally non-Clone and non-Copy. Repeated
    /// runtime evaluations reuse the same token without rechecking ownership.
    pub fn claim_driver_owner(
        &mut self,
        driver: Hp67Driver,
    ) -> Result<Hp67DriverOwner, Hp67ElectricalError> {
        let owned = &mut self.owned_drivers[driver.index()];
        if *owned {
            return Err(Hp67ElectricalError::DriverAlreadyOwned { driver });
        }
        *owned = true;
        Ok(Hp67DriverOwner { driver })
    }

    /// Immediate committed drive used by the current M14A structural bridge.
    ///
    /// M14B device evaluation should prefer stage_drive plus commit_staged so
    /// every device reads one immutable snapshot.
    pub fn set_drive_immediate(&mut self, net: Hp67Net, driver: Hp67Driver, drive: Drive) {
        self.apply_drive(net.index(), driver, drive);
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

    /// Borrow the already-resolved inputs and the independent output staging
    /// buffer for one device-evaluation interval without copying all net levels.
    ///
    /// The returned snapshot references only committed resolved levels. The
    /// stager references only pending outputs, so Rust enforces that staged
    /// writes cannot mutate what any device samples in this interval.
    pub fn begin_evaluation(
        &mut self,
    ) -> Result<(Hp67ElectricalSnapshot<'_>, Hp67ElectricalStager<'_>), Hp67ElectricalError> {
        if self.contention_net_count != 0 {
            return Err(self.contention_error(self.tick));
        }

        let tick = self.tick;
        let snapshot = Hp67ElectricalSnapshot {
            tick,
            levels: &self.resolved_levels,
        };
        let stager = Hp67ElectricalStager {
            pending_slot_markers: &mut self.pending_slot_markers,
            pending_changes: &mut self.pending_changes,
            pending_len: &mut self.pending_len,
        };
        Ok((snapshot, stager))
    }

    /// Convenience staging entry point for transitional callers that do not
    /// need to hold a zero-copy resolved snapshot.
    pub fn stage_drive(&mut self, owner: &Hp67DriverOwner, net: Hp67Net, drive: Drive) {
        let mut stager = Hp67ElectricalStager {
            pending_slot_markers: &mut self.pending_slot_markers,
            pending_changes: &mut self.pending_changes,
            pending_len: &mut self.pending_len,
        };
        stager.stage_drive(owner, net, drive);
    }

    /// Commit every staged device output atomically without advancing PHI/time.
    ///
    /// Committing electrical state and advancing the timing coordinate are
    /// deliberately separate operations. The current PHI scaffold performs one
    /// commit followed by one explicit tick advance, while future zero-time
    /// settling/propagation events may commit without implying an extra PHI edge.
    pub fn commit_staged(&mut self) -> Result<(), Hp67ElectricalError> {
        for pending_index in 0..self.pending_len {
            let pending = self.pending_changes[pending_index];
            let marker =
                &mut self.pending_slot_markers[pending.net.index()][pending.driver.index()];
            debug_assert_eq!(*marker, (pending_index + 1) as u8);
            *marker = 0;
            self.apply_drive(pending.net.index(), pending.driver, pending.drive);
        }
        self.pending_len = 0;
        if self.contention_net_count != 0 {
            return Err(self.contention_error(self.tick));
        }
        Ok(())
    }

    fn apply_drive(&mut self, net_index: usize, driver: Hp67Driver, drive: Drive) {
        self.nets[net_index].set_drive(driver, drive);
        self.refresh_resolved_level(net_index);
    }

    fn refresh_resolved_level(&mut self, net_index: usize) {
        let previous_level = self.resolved_levels[net_index];
        let next_level = self.nets[net_index].level();
        self.resolved_levels[net_index] = next_level;

        match (
            previous_level == LogicLevel::Contention,
            next_level == LogicLevel::Contention,
        ) {
            (false, true) => self.contention_net_count += 1,
            (true, false) => self.contention_net_count -= 1,
            _ => {}
        }
    }

    fn contention_error(&self, tick: Tick) -> Hp67ElectricalError {
        let net = Hp67Net::ALL
            .into_iter()
            .find(|net| self.resolved_levels[net.index()] == LogicLevel::Contention)
            .expect("cached HP-67 contention count must identify a contentious net");
        let dense = &self.nets[net.index()];
        let drivers = Hp67Driver::ALL
            .into_iter()
            .filter(|driver| dense.drive(*driver) != Drive::HighZ)
            .collect();
        Hp67ElectricalError::Contention { tick, net, drivers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m15r_ram_launch_preserves_foreign_rom_data_contribution() {
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.set_drive_immediate(
            Hp67Net::Data,
            Hp67Driver::StructuralRomResponder,
            Drive::High,
        );
        fabric.commit_serial_launch_with_data_owner(
            Drive::HighZ,
            RomIsDrives::Aggregate(Drive::HighZ),
            Drive::Low,
            Some((Drive::HighZ, Drive::Low)),
            Hp67Driver::RamAggregate,
        );
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Contention);
        assert_eq!(fabric.active_data_driver_count(), 2);
        assert_eq!(
            fabric.drive(Hp67Net::Data, Hp67Driver::StructuralRomResponder),
            Drive::High
        );
        fabric.set_drive_immediate(
            Hp67Net::Data,
            Hp67Driver::StructuralRomResponder,
            Drive::HighZ,
        );
        fabric.commit_serial_launch_with_data_owner(
            Drive::HighZ,
            RomIsDrives::Aggregate(Drive::HighZ),
            Drive::Low,
            Some((Drive::HighZ, Drive::HighZ)),
            Hp67Driver::RamAggregate,
        );
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Floating);
        assert_eq!(fabric.tick(), Tick::ZERO);
    }

    #[test]
    fn m15i_serial_launch_matches_generic_resolution_and_contention_cache() {
        use crate::emulation::{DriverId, Net};
        let drives = [Drive::HighZ, Drive::Low, Drive::High];
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.set_drive_immediate(Hp67Net::Str, Hp67Driver::External, Drive::High);
        // Every combination of the five serial source contributions, with an
        // independent third contributor on each net. Reuse fabric across commits
        // to exercise handoffs and changes into/out of cached contention.
        for external in drives {
            for net in [Hp67Net::Isa, Hp67Net::Sync, Hp67Net::Data] {
                fabric.set_drive_immediate(net, Hp67Driver::External, external);
            }
            for code in 0..243usize {
                let mut digits = code;
                let image: [Drive; 5] = std::array::from_fn(|_| {
                    let drive = drives[digits % 3];
                    digits /= 3;
                    drive
                });
                let tick = fabric.tick();
                fabric.commit_serial_launch(
                    image[0],
                    image[1],
                    image[2],
                    Some((image[3], image[4])),
                );
                assert_eq!(fabric.tick(), tick);
                for (net, bias, act, rom) in [
                    (Hp67Net::Isa, Bias::PullDown, image[0], image[1]),
                    (Hp67Net::Sync, Bias::Floating, image[2], Drive::HighZ),
                    (Hp67Net::Data, Bias::Floating, image[3], image[4]),
                ] {
                    let mut reference = Net::new(bias);
                    reference.set_drive(DriverId::new("act"), act);
                    reference.set_drive(DriverId::new("rom"), rom);
                    reference.set_drive(DriverId::new("external"), external);
                    assert_eq!(
                        fabric.level(net),
                        reference.level(),
                        "code={code} net={net:?}"
                    );
                    assert_eq!(
                        fabric.active_driver_count(net),
                        [act, rom, external]
                            .into_iter()
                            .filter(|d| *d != Drive::HighZ)
                            .count() as u8
                    );
                    assert_eq!(fabric.drive(net, Hp67Driver::External), external);
                }
                let expected_contention = Hp67Net::ALL
                    .into_iter()
                    .filter(|net| fabric.level(*net) == LogicLevel::Contention)
                    .count();
                assert_eq!(fabric.contention_net_count as usize, expected_contention);
                assert_eq!(fabric.begin_evaluation().is_err(), expected_contention != 0);
                assert_eq!(fabric.level(Hp67Net::Str), LogicLevel::High);
            }
        }
    }

    #[test]
    fn m15i_serial_handoff_and_absent_data_role_preserve_other_contributions() {
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.commit_serial_launch(
            Drive::High,
            Drive::HighZ,
            Drive::High,
            Some((Drive::Low, Drive::HighZ)),
        );
        fabric.commit_serial_launch(
            Drive::HighZ,
            Drive::High,
            Drive::Low,
            Some((Drive::HighZ, Drive::High)),
        );
        assert!(fabric.isa_drives_valid());
        assert_eq!(fabric.active_driver_count(Hp67Net::Isa), 1);
        assert_eq!(fabric.active_data_driver_count(), 1);
        assert_eq!(fabric.level(Hp67Net::Isa), LogicLevel::High);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);
        assert_eq!(fabric.contention_net_count, 0);
        fabric.commit_serial_launch(Drive::HighZ, Drive::HighZ, Drive::Low, None);
        assert_eq!(fabric.level(Hp67Net::Isa), LogicLevel::Low);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);
        assert_eq!(
            fabric.drive(Hp67Net::Data, Hp67Driver::StructuralRomResponder),
            Drive::High
        );
        fabric.commit_serial_launch(
            Drive::HighZ,
            Drive::HighZ,
            Drive::Low,
            Some((Drive::HighZ, Drive::HighZ)),
        );
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Floating);
        assert_eq!(fabric.tick().get(), 0);
    }

    #[test]
    fn m15e_data_bridge_handoff_publishes_one_resolved_image_without_advancing_time() {
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.commit_data_level_bridge(Drive::High, Drive::HighZ);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);
        fabric.commit_data_level_bridge(Drive::HighZ, Drive::Low);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Low);
        assert_eq!(
            fabric.drive(Hp67Net::Data, Hp67Driver::Act1820_2530),
            Drive::HighZ
        );
        assert_eq!(
            fabric.drive(Hp67Net::Data, Hp67Driver::StructuralRomResponder),
            Drive::Low
        );
        assert_eq!(fabric.active_data_driver_count(), 1);
        assert!(fabric.begin_evaluation().is_ok());
        assert_eq!(fabric.tick(), Tick::ZERO);
        fabric.commit_data_level_bridge(Drive::HighZ, Drive::HighZ);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Floating);
        assert_eq!(fabric.active_data_driver_count(), 0);
    }

    #[test]
    fn m15b_is_policy_detects_same_level_overlap_and_active_low() {
        let mut fabric = Hp67ElectricalFabric::default();
        assert!(fabric.isa_drives_valid());
        assert_eq!(fabric.active_isa_driver_mask(), 0);
        let act = Hp67Driver::Act1820_2530;
        let rom = Hp67Driver::StructuralRomResponder;
        fabric.set_drive_immediate(Hp67Net::Isa, act, Drive::High);
        assert!(fabric.isa_drives_valid());
        fabric.set_drive_immediate(Hp67Net::Isa, rom, Drive::High);
        assert_eq!(fabric.level(Hp67Net::Isa), LogicLevel::High);
        assert!(!fabric.isa_drives_valid());
        assert_eq!(
            fabric.active_isa_driver_mask(),
            (1u16 << act.index()) | (1u16 << rom.index())
        );
        fabric.set_drive_immediate(Hp67Net::Isa, act, Drive::HighZ);
        assert!(fabric.isa_drives_valid());
        fabric.set_drive_immediate(Hp67Net::Isa, rom, Drive::Low);
        assert_eq!(fabric.level(Hp67Net::Isa), LogicLevel::Low);
        assert!(!fabric.isa_drives_valid());
        fabric.set_drive_immediate(Hp67Net::Isa, rom, Drive::HighZ);
        assert!(fabric.isa_drives_valid());
    }

    #[test]
    fn m15b_staged_is_handoff_checks_final_committed_contributions() {
        for reversed in [false, true] {
            let mut fabric = Hp67ElectricalFabric::default();
            let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
            let rom = fabric
                .claim_driver_owner(Hp67Driver::StructuralRomResponder)
                .unwrap();
            fabric.set_drive_immediate(Hp67Net::Isa, act.driver(), Drive::High);
            let (snapshot, mut stager) = fabric.begin_evaluation().unwrap();
            assert_eq!(snapshot.level(Hp67Net::Isa), LogicLevel::High);
            if reversed {
                stager.stage_drive(&rom, Hp67Net::Isa, Drive::High);
                stager.stage_drive(&act, Hp67Net::Isa, Drive::HighZ);
            } else {
                stager.stage_drive(&act, Hp67Net::Isa, Drive::HighZ);
                stager.stage_drive(&rom, Hp67Net::Isa, Drive::High);
            }
            assert_eq!(snapshot.level(Hp67Net::Isa), LogicLevel::High);
            fabric.commit_staged().unwrap();
            assert!(fabric.isa_drives_valid());
            assert_eq!(
                fabric.active_isa_driver_mask(),
                1u16 << rom.driver().index()
            );
            assert_eq!(fabric.tick(), Tick::ZERO);
        }
    }

    #[test]
    fn evidenced_idle_levels_are_encoded_in_dense_fabric() {
        let fabric = Hp67ElectricalFabric::default();
        for net in Hp67Net::ALL {
            let expected = match net {
                Hp67Net::Phi1 | Hp67Net::Phi2 => LogicLevel::High,
                Hp67Net::Isa => LogicLevel::Low,
                _ => LogicLevel::Floating,
            };
            assert_eq!(
                fabric.level(net),
                expected,
                "unexpected idle level for {net:?}"
            );
        }
    }

    #[test]
    fn staged_outputs_are_invisible_until_atomic_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
        {
            let (snapshot, mut stager) = fabric.begin_evaluation().unwrap();
            assert_eq!(snapshot.tick(), Tick::ZERO);
            assert_eq!(snapshot.level(Hp67Net::Data), LogicLevel::Floating);

            stager.stage_drive(&act, Hp67Net::Data, Drive::High);
            assert_eq!(snapshot.level(Hp67Net::Data), LogicLevel::Floating);
        }

        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Floating);
        assert_eq!(fabric.commit_staged(), Ok(()));
        assert_eq!(fabric.tick(), Tick::ZERO);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);
    }

    #[test]
    fn commit_does_not_advance_timing_coordinate() {
        let mut fabric = Hp67ElectricalFabric::default();
        let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();

        fabric.stage_drive(&act, Hp67Net::Data, Drive::High);
        assert_eq!(fabric.commit_staged(), Ok(()));
        assert_eq!(fabric.tick(), Tick::ZERO);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);

        assert_eq!(fabric.advance_tick(), Tick::new(1));
        assert_eq!(fabric.tick(), Tick::new(1));
    }

    #[test]
    fn repeated_staging_by_one_driver_is_final_write_wins_without_duplicate_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
        fabric.stage_drive(&act, Hp67Net::Data, Drive::High);
        fabric.stage_drive(&act, Hp67Net::Data, Drive::Low);
        fabric.stage_drive(&act, Hp67Net::Data, Drive::HighZ);
        assert_eq!(fabric.commit_staged(), Ok(()));
        assert_eq!(fabric.tick(), Tick::ZERO);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Floating);
    }

    #[test]
    fn staged_slot_marker_is_reusable_after_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();

        fabric.stage_drive(&act, Hp67Net::Data, Drive::High);
        assert_eq!(fabric.commit_staged(), Ok(()));
        assert_eq!(fabric.tick(), Tick::ZERO);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);

        fabric.stage_drive(&act, Hp67Net::Data, Drive::Low);
        assert_eq!(fabric.commit_staged(), Ok(()));
        assert_eq!(fabric.tick(), Tick::ZERO);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Low);
    }

    #[test]
    fn driver_ownership_is_claimed_once_and_reused_across_evaluations() {
        let mut fabric = Hp67ElectricalFabric::default();
        let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
        assert_eq!(act.driver(), Hp67Driver::Act1820_2530);
        assert_eq!(
            fabric.claim_driver_owner(Hp67Driver::Act1820_2530),
            Err(Hp67ElectricalError::DriverAlreadyOwned {
                driver: Hp67Driver::Act1820_2530,
            })
        );

        fabric.stage_drive(&act, Hp67Net::Data, Drive::High);
        assert_eq!(fabric.commit_staged(), Ok(()));
        assert_eq!(fabric.tick(), Tick::ZERO);
        fabric.stage_drive(&act, Hp67Net::Data, Drive::Low);
        assert_eq!(fabric.commit_staged(), Ok(()));
        assert_eq!(fabric.tick(), Tick::ZERO);
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Low);
    }

    #[test]
    fn simultaneous_opposite_drives_report_contention_after_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
        let rom = fabric
            .claim_driver_owner(Hp67Driver::StructuralRomResponder)
            .unwrap();
        fabric.stage_drive(&act, Hp67Net::Data, Drive::High);
        fabric.stage_drive(&rom, Hp67Net::Data, Drive::Low);

        assert_eq!(
            fabric.commit_staged(),
            Err(Hp67ElectricalError::Contention {
                tick: Tick::ZERO,
                net: Hp67Net::Data,
                drivers: vec![Hp67Driver::Act1820_2530, Hp67Driver::StructuralRomResponder,],
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
                    dense.set_drive_immediate(net, Hp67Driver::StructuralRomResponder, rom_drive);

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
