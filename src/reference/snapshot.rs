//! Instruction-boundary snapshot and diff utilities for reference validation.

use core::fmt;

use super::woodstock::{ArchitecturalState, ReferenceMachine, Register, STATUS_BITS, WORD_DIGITS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitecturalSnapshot {
    pub cpu: ArchitecturalState,
}

impl ArchitecturalSnapshot {
    pub fn capture(machine: &ReferenceMachine) -> Self {
        Self {
            cpu: machine.cpu.clone(),
        }
    }

    pub fn diff(&self, other: &Self) -> StateDiff {
        let mut lines = Vec::new();

        diff_register(&mut lines, "A", self.cpu.a, other.cpu.a);
        diff_register(&mut lines, "B", self.cpu.b, other.cpu.b);
        diff_register(&mut lines, "C", self.cpu.c, other.cpu.c);
        diff_register(&mut lines, "Y", self.cpu.y, other.cpu.y);
        diff_register(&mut lines, "Z", self.cpu.z, other.cpu.z);
        diff_register(&mut lines, "T", self.cpu.t, other.cpu.t);
        diff_register(&mut lines, "M1", self.cpu.m1, other.cpu.m1);
        diff_register(&mut lines, "M2", self.cpu.m2, other.cpu.m2);

        diff_value(&mut lines, "F", self.cpu.f, other.cpu.f);
        diff_value(&mut lines, "P", self.cpu.p, other.cpu.p);
        diff_value(
            &mut lines,
            "P_CHANGE",
            self.cpu.p_change,
            other.cpu.p_change,
        );
        diff_value(&mut lines, "DECIMAL", self.cpu.decimal, other.cpu.decimal);
        diff_value(&mut lines, "CARRY", self.cpu.carry, other.cpu.carry);
        diff_value(
            &mut lines,
            "PREV_CARRY",
            self.cpu.previous_carry,
            other.cpu.previous_carry,
        );
        diff_value(
            &mut lines,
            "PC",
            OctalPc(self.cpu.pc),
            OctalPc(other.cpu.pc),
        );
        diff_value(&mut lines, "BANK", self.cpu.bank, other.cpu.bank);
        diff_value(
            &mut lines,
            "DELAYED_ROM",
            self.cpu.delayed_rom,
            other.cpu.delayed_rom,
        );
        diff_value(
            &mut lines,
            "STACK",
            self.cpu.return_stack,
            other.cpu.return_stack,
        );
        diff_value(
            &mut lines,
            "SP",
            self.cpu.stack_pointer,
            other.cpu.stack_pointer,
        );
        diff_value(
            &mut lines,
            "INST_STATE",
            self.cpu.instruction_state,
            other.cpu.instruction_state,
        );
        diff_value(
            &mut lines,
            "KEY_BUFFER",
            self.cpu.key_buffer,
            other.cpu.key_buffer,
        );
        diff_value(
            &mut lines,
            "DISPLAY_ENABLE",
            self.cpu.display_enable,
            other.cpu.display_enable,
        );
        diff_value(
            &mut lines,
            "DISPLAY_14",
            self.cpu.display_14_digit,
            other.cpu.display_14_digit,
        );
        diff_value(
            &mut lines,
            "RAM_ADDR",
            HexByte(self.cpu.ram_address),
            HexByte(other.cpu.ram_address),
        );

        let status_changes: Vec<String> = (0..STATUS_BITS)
            .filter(|&index| self.cpu.status[index] != other.cpu.status[index])
            .map(|index| {
                format!(
                    "S{index}: {:?} -> {:?}",
                    self.cpu.status[index], other.cpu.status[index]
                )
            })
            .collect();
        if !status_changes.is_empty() {
            lines.push(format!("STATUS: {}", status_changes.join(", ")));
        }

        StateDiff { lines }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDiff {
    lines: Vec<String>,
}

impl StateDiff {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

impl fmt::Display for StateDiff {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.lines.is_empty() {
            return formatter.write_str("no architectural differences");
        }
        for (index, line) in self.lines.iter().enumerate() {
            if index != 0 {
                formatter.write_str("\n")?;
            }
            formatter.write_str(line)?;
        }
        Ok(())
    }
}

fn diff_register(lines: &mut Vec<String>, name: &str, left: Register, right: Register) {
    if left != right {
        lines.push(format!(
            "{name}: {} -> {}",
            RegisterText(left),
            RegisterText(right)
        ));
    }
}

fn diff_value<T>(lines: &mut Vec<String>, name: &str, left: T, right: T)
where
    T: PartialEq + fmt::Debug,
{
    if left != right {
        lines.push(format!("{name}: {left:?} -> {right:?}"));
    }
}

struct RegisterText(Register);

impl fmt::Display for RegisterText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for index in (0..WORD_DIGITS).rev() {
            write!(formatter, "{:x}", self.0[index])?;
        }
        Ok(())
    }
}

#[derive(PartialEq)]
struct OctalPc(u16);

impl fmt::Debug for OctalPc {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:05o}", self.0)
    }
}

#[derive(PartialEq)]
struct HexByte(u8);

impl fmt::Debug for HexByte {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "0x{:02x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_snapshots_have_no_diff() {
        let machine = ReferenceMachine::default();
        let left = ArchitecturalSnapshot::capture(&machine);
        let right = ArchitecturalSnapshot::capture(&machine);
        let diff = left.diff(&right);

        assert!(diff.is_empty());
        assert_eq!(diff.to_string(), "no architectural differences");
    }

    #[test]
    fn diff_names_changed_registers_control_state_and_status_bits() {
        let machine = ReferenceMachine::default();
        let left = ArchitecturalSnapshot::capture(&machine);

        let mut changed = machine;
        changed.cpu.a[0] = 0x0a;
        changed.cpu.pc = 0o1234;
        changed.cpu.carry = true;
        changed.cpu.status[7] = true;
        changed.cpu.ram_address = 0x2f;
        let right = ArchitecturalSnapshot::capture(&changed);

        let text = left.diff(&right).to_string();
        assert!(text.contains("A:"));
        assert!(text.contains("PC: 00000 -> 01234"));
        assert!(text.contains("CARRY: false -> true"));
        assert!(text.contains("STATUS: S7: false -> true"));
        assert!(text.contains("RAM_ADDR: 0x00 -> 0x2f"));
    }
}
