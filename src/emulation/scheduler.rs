//! Deterministic resolve -> snapshot -> evaluate -> commit electrical scheduler.
//!
//! Every device sees the same resolved input snapshot for a tick. Device output
//! drives are buffered and committed only after all devices have evaluated, so
//! Rust iteration order cannot accidentally become hardware propagation delay.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    Bias, Drive, DriveSink, DriverId, ElectricalDevice, LogicLevel, Net, NetReader, Tick, Trace,
    TraceSample,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerError<N> {
    Contention {
        tick: Tick,
        net: N,
        drivers: Vec<DriverId>,
    },
    DriverCollision {
        tick: Tick,
        net: N,
        driver: DriverId,
    },
}

#[derive(Debug, Clone)]
struct ResolvedSnapshot<N> {
    levels: BTreeMap<N, LogicLevel>,
}

impl<N> NetReader<N> for ResolvedSnapshot<N>
where
    N: Copy + Ord,
{
    fn read(&self, net: N) -> LogicLevel {
        *self
            .levels
            .get(&net)
            .expect("electrical device read a net not installed in the scheduler")
    }
}

struct DeviceDriveBuffer<N> {
    drives: BTreeMap<(N, DriverId), Drive>,
}

impl<N> Default for DeviceDriveBuffer<N> {
    fn default() -> Self {
        Self {
            drives: BTreeMap::new(),
        }
    }
}

impl<N> DriveSink<N> for DeviceDriveBuffer<N>
where
    N: Copy + Ord,
{
    fn drive(&mut self, net: N, driver: DriverId, drive: Drive) {
        // Within a single device evaluation, the final drive for one output pin
        // wins. Cross-device reuse of the same DriverId is rejected separately.
        self.drives.insert((net, driver), drive);
    }
}

/// Generic deterministic scheduler for calculator electrical devices.
pub struct ElectricalScheduler<N>
where
    N: Copy + Ord,
{
    tick: Tick,
    nets: BTreeMap<N, Net>,
    devices: Vec<Box<dyn ElectricalDevice<N>>>,
    trace_probes: BTreeSet<N>,
    trace: Trace<N>,
}

impl<N> Default for ElectricalScheduler<N>
where
    N: Copy + Ord,
{
    fn default() -> Self {
        Self {
            tick: Tick::ZERO,
            nets: BTreeMap::new(),
            devices: Vec::new(),
            trace_probes: BTreeSet::new(),
            trace: Trace::default(),
        }
    }
}

impl<N> ElectricalScheduler<N>
where
    N: Copy + Ord,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&self) -> Tick {
        self.tick
    }

    pub fn install_net(&mut self, id: N, bias: Bias) {
        self.nets.insert(id, Net::new(bias));
    }

    pub fn add_device<D>(&mut self, device: D)
    where
        D: ElectricalDevice<N> + 'static,
    {
        self.devices.push(Box::new(device));
    }

    pub fn level(&self, id: N) -> LogicLevel {
        self.nets
            .get(&id)
            .expect("requested net is not installed in the scheduler")
            .level()
    }

    /// Record this resolved net after every successfully committed tick.
    pub fn add_trace_probe(&mut self, net: N) {
        assert!(
            self.nets.contains_key(&net),
            "trace probe net is not installed in the scheduler"
        );
        self.trace_probes.insert(net);
    }

    pub fn remove_trace_probe(&mut self, net: N) {
        self.trace_probes.remove(&net);
    }

    pub fn trace(&self) -> &Trace<N> {
        &self.trace
    }

    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }

    /// Apply a host/external drive such as a switch contact or temporary test
    /// stimulus. Device drives should normally be produced through `evaluate`.
    pub fn set_external_drive(&mut self, net: N, driver: DriverId, drive: Drive) {
        self.nets
            .get_mut(&net)
            .expect("requested net is not installed in the scheduler")
            .set_drive(driver, drive);
    }

    /// Advance one deterministic electrical tick.
    pub fn step(&mut self) -> Result<Tick, SchedulerError<N>> {
        self.ensure_no_contention(self.tick)?;
        let snapshot = self.snapshot();
        let next_tick = Tick::new(self.tick.get() + 1);
        let mut pending: BTreeMap<(N, DriverId), Drive> = BTreeMap::new();

        for device in &mut self.devices {
            let mut local = DeviceDriveBuffer::default();
            device.evaluate(next_tick, &snapshot, &mut local);
            for (key @ (net, driver), drive) in local.drives {
                if pending.insert(key, drive).is_some() {
                    return Err(SchedulerError::DriverCollision {
                        tick: next_tick,
                        net,
                        driver,
                    });
                }
            }
        }

        for ((net, driver), drive) in pending {
            self.nets
                .get_mut(&net)
                .expect("electrical device drove a net not installed in the scheduler")
                .set_drive(driver, drive);
        }

        self.tick = next_tick;
        self.ensure_no_contention(next_tick)?;
        self.capture_trace(next_tick);
        Ok(next_tick)
    }

    fn snapshot(&self) -> ResolvedSnapshot<N> {
        ResolvedSnapshot {
            levels: self
                .nets
                .iter()
                .map(|(id, net)| (*id, net.level()))
                .collect(),
        }
    }

    fn ensure_no_contention(&self, tick: Tick) -> Result<(), SchedulerError<N>> {
        for (id, net) in &self.nets {
            if net.level() == LogicLevel::Contention {
                return Err(SchedulerError::Contention {
                    tick,
                    net: *id,
                    drivers: net.active_drives().map(|(driver, _)| driver).collect(),
                });
            }
        }
        Ok(())
    }

    fn capture_trace(&mut self, tick: Tick) {
        for net in self.trace_probes.iter().copied() {
            self.trace.push(TraceSample {
                tick,
                net,
                level: self
                    .nets
                    .get(&net)
                    .expect("trace probe net disappeared from scheduler")
                    .level(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum TestNet {
        Input,
        Intermediate,
        Output,
    }

    const EXTERNAL: DriverId = DriverId::new("external");
    const FIRST: DriverId = DriverId::new("first");
    const SECOND: DriverId = DriverId::new("second");
    const A: DriverId = DriverId::new("A");
    const B: DriverId = DriverId::new("B");

    struct FollowHigh {
        name: &'static str,
        input: TestNet,
        output: TestNet,
        driver: DriverId,
    }

    impl ElectricalDevice<TestNet> for FollowHigh {
        fn device_name(&self) -> &'static str {
            self.name
        }

        fn evaluate(
            &mut self,
            _tick: Tick,
            inputs: &dyn NetReader<TestNet>,
            outputs: &mut dyn DriveSink<TestNet>,
        ) {
            outputs.drive(
                self.output,
                self.driver,
                if inputs.read(self.input) == LogicLevel::High {
                    Drive::High
                } else {
                    Drive::HighZ
                },
            );
        }
    }

    struct ConstantDrive {
        name: &'static str,
        net: TestNet,
        driver: DriverId,
        drive: Drive,
    }

    impl ElectricalDevice<TestNet> for ConstantDrive {
        fn device_name(&self) -> &'static str {
            self.name
        }

        fn evaluate(
            &mut self,
            _tick: Tick,
            _inputs: &dyn NetReader<TestNet>,
            outputs: &mut dyn DriveSink<TestNet>,
        ) {
            outputs.drive(self.net, self.driver, self.drive);
        }
    }

    fn propagation_scheduler(reverse_devices: bool) -> ElectricalScheduler<TestNet> {
        let mut scheduler = ElectricalScheduler::new();
        for net in [TestNet::Input, TestNet::Intermediate, TestNet::Output] {
            scheduler.install_net(net, Bias::Floating);
        }
        scheduler.set_external_drive(TestNet::Input, EXTERNAL, Drive::High);

        let first = FollowHigh {
            name: "input-to-intermediate",
            input: TestNet::Input,
            output: TestNet::Intermediate,
            driver: FIRST,
        };
        let second = FollowHigh {
            name: "intermediate-to-output",
            input: TestNet::Intermediate,
            output: TestNet::Output,
            driver: SECOND,
        };
        if reverse_devices {
            scheduler.add_device(second);
            scheduler.add_device(first);
        } else {
            scheduler.add_device(first);
            scheduler.add_device(second);
        }
        scheduler
    }

    #[test]
    fn outputs_are_committed_only_after_every_device_samples_the_same_snapshot() {
        let mut scheduler = propagation_scheduler(false);

        scheduler.step().expect("first tick must resolve");
        assert_eq!(scheduler.level(TestNet::Intermediate), LogicLevel::High);
        assert_eq!(scheduler.level(TestNet::Output), LogicLevel::Floating);

        scheduler.step().expect("second tick must resolve");
        assert_eq!(scheduler.level(TestNet::Output), LogicLevel::High);
    }

    #[test]
    fn device_iteration_order_cannot_change_propagation_result() {
        let mut forward = propagation_scheduler(false);
        let mut reverse = propagation_scheduler(true);

        for _ in 0..4 {
            assert_eq!(forward.step(), reverse.step());
            for net in [TestNet::Input, TestNet::Intermediate, TestNet::Output] {
                assert_eq!(forward.level(net), reverse.level(net));
            }
        }
    }

    #[test]
    fn trace_probes_are_sampled_after_commit_in_stable_net_order() {
        let mut scheduler = propagation_scheduler(false);
        scheduler.add_trace_probe(TestNet::Output);
        scheduler.add_trace_probe(TestNet::Input);

        scheduler.step().expect("tick must resolve");
        assert_eq!(
            scheduler.trace().samples(),
            &[
                TraceSample {
                    tick: Tick::new(1),
                    net: TestNet::Input,
                    level: LogicLevel::High,
                },
                TraceSample {
                    tick: Tick::new(1),
                    net: TestNet::Output,
                    level: LogicLevel::Floating,
                },
            ]
        );
    }

    #[test]
    fn contention_fails_with_tick_net_and_stable_driver_names() {
        let mut scheduler = ElectricalScheduler::new();
        scheduler.install_net(TestNet::Output, Bias::Floating);
        scheduler.add_device(ConstantDrive {
            name: "high-source",
            net: TestNet::Output,
            driver: B,
            drive: Drive::High,
        });
        scheduler.add_device(ConstantDrive {
            name: "low-source",
            net: TestNet::Output,
            driver: A,
            drive: Drive::Low,
        });

        assert_eq!(
            scheduler.step(),
            Err(SchedulerError::Contention {
                tick: Tick::new(1),
                net: TestNet::Output,
                drivers: vec![A, B],
            })
        );
        assert!(scheduler.trace().samples().is_empty());
    }

    #[test]
    fn duplicate_driver_identity_across_devices_is_rejected_before_commit() {
        let mut scheduler = ElectricalScheduler::new();
        scheduler.install_net(TestNet::Output, Bias::Floating);
        scheduler.add_device(ConstantDrive {
            name: "source-one",
            net: TestNet::Output,
            driver: A,
            drive: Drive::High,
        });
        scheduler.add_device(ConstantDrive {
            name: "source-two",
            net: TestNet::Output,
            driver: A,
            drive: Drive::High,
        });

        assert_eq!(
            scheduler.step(),
            Err(SchedulerError::DriverCollision {
                tick: Tick::new(1),
                net: TestNet::Output,
                driver: A,
            })
        );
        assert_eq!(scheduler.level(TestNet::Output), LogicLevel::Floating);
        assert_eq!(scheduler.tick(), Tick::ZERO);
    }
}
