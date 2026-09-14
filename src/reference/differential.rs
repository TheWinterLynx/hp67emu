//! Differential harness between the semantic HP-67 oracle and a timed target.
//!
//! The target is intentionally expressed as a trait. The electrical ACT does
//! not exist yet, but once it can expose instruction-boundary architectural
//! state it can plug into this harness without coupling the reference model to
//! the electrical scheduler implementation.

use super::{
    hp67::{Hp67Reference, Hp67ReferenceError},
    rom::RomImage,
    snapshot::{ArchitecturalSnapshot, StateDiff},
};

/// Minimum contract required from a timed/electrical implementation.
pub trait InstructionBoundaryTarget {
    type Error;

    /// Monotonic number of completed Woodstock microinstructions.
    fn completed_instructions(&self) -> u64;

    /// Advance exactly one smallest timed/electrical scheduler tick.
    fn advance_tick(&mut self) -> Result<(), Self::Error>;

    /// Reconstruct CPU-visible architectural state at the current boundary.
    fn architectural_snapshot(&self) -> ArchitecturalSnapshot;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferentialStep {
    pub opcode: u16,
    pub target_ticks: u64,
    pub completed_instruction: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DifferentialError<E> {
    Reference(Hp67ReferenceError),
    Target(E),
    InitialDivergence(StateDiff),
    BoundaryTimeout {
        start_instruction: u64,
        max_ticks: u64,
    },
    SkippedBoundary {
        expected_instruction: u64,
        observed_instruction: u64,
    },
    Divergence {
        opcode: u16,
        completed_instruction: u64,
        diff: StateDiff,
    },
}

impl<E> From<Hp67ReferenceError> for DifferentialError<E> {
    fn from(value: Hp67ReferenceError) -> Self {
        Self::Reference(value)
    }
}

/// Verify that reference and target agree before advancing either machine.
pub fn assert_synchronized<T: InstructionBoundaryTarget>(
    reference: &Hp67Reference,
    target: &T,
) -> Result<(), DifferentialError<T::Error>> {
    let expected = ArchitecturalSnapshot::capture(&reference.core);
    let observed = target.architectural_snapshot();
    let diff = expected.diff(&observed);
    if diff.is_empty() {
        Ok(())
    } else {
        Err(DifferentialError::InitialDivergence(diff))
    }
}

/// Execute exactly one semantic HP-67 word and advance the timed target until
/// it reports exactly one matching instruction boundary.
pub fn step_and_compare<T: InstructionBoundaryTarget>(
    reference: &mut Hp67Reference,
    rom: &RomImage,
    target: &mut T,
    max_target_ticks: u64,
) -> Result<DifferentialStep, DifferentialError<T::Error>> {
    let start_instruction = target.completed_instructions();
    let expected_instruction = start_instruction + 1;

    let opcode = reference.step(rom)?;
    let expected = ArchitecturalSnapshot::capture(&reference.core);

    let mut target_ticks = 0;
    while target.completed_instructions() == start_instruction {
        if target_ticks >= max_target_ticks {
            return Err(DifferentialError::BoundaryTimeout {
                start_instruction,
                max_ticks: max_target_ticks,
            });
        }
        target.advance_tick().map_err(DifferentialError::Target)?;
        target_ticks += 1;
    }

    let observed_instruction = target.completed_instructions();
    if observed_instruction != expected_instruction {
        return Err(DifferentialError::SkippedBoundary {
            expected_instruction,
            observed_instruction,
        });
    }

    let observed = target.architectural_snapshot();
    let diff = expected.diff(&observed);
    if !diff.is_empty() {
        return Err(DifferentialError::Divergence {
            opcode,
            completed_instruction: observed_instruction,
            diff,
        });
    }

    Ok(DifferentialStep {
        opcode,
        target_ticks,
        completed_instruction: observed_instruction,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference::woodstock::ReferenceMachine;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct MockError;

    struct SemanticTimedTarget {
        machine: Hp67Reference,
        rom: RomImage,
        completed: u64,
        ticks_per_instruction: u64,
        ticks_until_boundary: u64,
    }

    impl SemanticTimedTarget {
        fn new(machine: Hp67Reference, rom: RomImage, ticks_per_instruction: u64) -> Self {
            Self {
                machine,
                rom,
                completed: 0,
                ticks_per_instruction,
                ticks_until_boundary: ticks_per_instruction,
            }
        }
    }

    impl InstructionBoundaryTarget for SemanticTimedTarget {
        type Error = MockError;

        fn completed_instructions(&self) -> u64 {
            self.completed
        }

        fn advance_tick(&mut self) -> Result<(), Self::Error> {
            self.ticks_until_boundary -= 1;
            if self.ticks_until_boundary == 0 {
                self.machine.step(&self.rom).map_err(|_| MockError)?;
                self.completed += 1;
                self.ticks_until_boundary = self.ticks_per_instruction;
            }
            Ok(())
        }

        fn architectural_snapshot(&self) -> ArchitecturalSnapshot {
            ArchitecturalSnapshot::capture(&self.machine.core)
        }
    }

    fn fixture_rom() -> RomImage {
        let mut rom = RomImage::new();
        rom.install_page(0, 0, &[0o0420, 0o0000, 0o0720])
            .expect("fixture ROM must install");
        rom
    }

    #[test]
    fn harness_waits_for_boundary_and_compares_architectural_state() {
        let rom = fixture_rom();
        let initial = Hp67Reference::default();
        let mut target = SemanticTimedTarget::new(initial.clone(), rom.clone(), 7);
        let mut reference = initial;

        assert_synchronized(&reference, &target).expect("initial state must match");
        let step = step_and_compare(&mut reference, &rom, &mut target, 16)
            .expect("identical semantic target must match");

        assert_eq!(step.opcode, 0o0420);
        assert_eq!(step.target_ticks, 7);
        assert_eq!(step.completed_instruction, 1);
    }

    #[test]
    fn harness_reports_precise_state_divergence() {
        let rom = fixture_rom();
        let initial = Hp67Reference::default();
        let mut target = SemanticTimedTarget::new(initial.clone(), rom.clone(), 1);
        let mut reference = initial;

        target.machine.core.cpu.a[0] = 9;
        let error = assert_synchronized(&reference, &target)
            .expect_err("different register state must fail synchronization");

        match error {
            DifferentialError::InitialDivergence(diff) => {
                assert!(diff.to_string().contains("A:"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn harness_times_out_if_target_never_reaches_a_boundary() {
        struct StuckTarget {
            state: ReferenceMachine,
        }

        impl InstructionBoundaryTarget for StuckTarget {
            type Error = MockError;

            fn completed_instructions(&self) -> u64 {
                0
            }

            fn advance_tick(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn architectural_snapshot(&self) -> ArchitecturalSnapshot {
                ArchitecturalSnapshot::capture(&self.state)
            }
        }

        let rom = fixture_rom();
        let mut reference = Hp67Reference::default();
        let mut target = StuckTarget {
            state: reference.core.clone(),
        };

        let error = step_and_compare(&mut reference, &rom, &mut target, 3)
            .expect_err("stuck target must time out");
        assert_eq!(
            error,
            DifferentialError::BoundaryTimeout {
                start_instruction: 0,
                max_ticks: 3,
            }
        );
    }
}
