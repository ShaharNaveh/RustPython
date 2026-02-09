use crate::bytecode::{
    oparg::AnyOparg,
    opcode::{Opcode, PseudoOpcode},
};

/// A valid instruction, used by the compiler and executed by the VM.
#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    /// [`Opcode`] of the instruction.
    opcode: Opcode,
    /// Oparg of the instruction (if the opcode has an oparg).
    oparg: Option<AnyOparg>,
}

/// A valid pseudo instruction, used only by the compiler and never executed by the VM.
#[derive(Clone, Copy, Debug)]
pub struct PseudoInstruction {
    /// [`PseudoOpcode`] of the instruction.
    opcode: PseudoOpcode,
    /// Oparg of the instruction (if the opcode has an oparg).
    oparg: Option<AnyOparg>,
}

#[derive(Clone, Copy, Debug)]
pub enum AnyInstruction {
    Real(Instruction),
    Pseudo(PseudoInstruction),
}

impl From<Instruction> for AnyInstruction {
    fn from(value: Instruction) -> Self {
        Self::Real(value)
    }
}

impl From<PseudoInstruction> for AnyInstruction {
    fn from(value: PseudoInstruction) -> Self {
        Self::Pseudo(value)
    }
}
