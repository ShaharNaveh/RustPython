//! Implement python as a virtual machine with bytecode. This module
//! implements bytecode structure.

mod code;
mod constants;
mod instruction;
mod instructions;
mod oparg;

use std::mem;

pub use code::{CodeFlags, CodeObject};
pub use constants::{AsBag, Bag, BorrowedConstant, Constant, ConstantBag};
pub use instruction::Instruction;
pub use instructions::{PseudoInstruction, RealInstruction};
pub use oparg::{Oparg, OpargByte, OpargType};

#[derive(Default, Copy, Clone)]
#[repr(transparent)]
pub struct OpArgState {
    state: u32,
}

impl OpArgState {
    #[inline(always)]
    pub fn get(&mut self, ins: CodeUnit) -> (Instruction, Oparg) {
        let arg = self.extend(ins.arg);
        if ins.op != Instruction::ExtendedArg {
            self.reset();
        }
        (ins.op, arg)
    }

    #[inline(always)]
    pub fn extend(&mut self, arg: OpargByte) -> Oparg {
        self.state = (self.state << 8) | u32::from(arg);
        Oparg(self.state)
    }

    #[inline(always)]
    pub const fn reset(&mut self) {
        self.state = 0
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CodeUnit {
    pub op: RealInstruction,
    pub arg: OpargByte,
}

const _: () = assert!(mem::size_of::<CodeUnit>() == 2);

impl CodeUnit {
    pub const fn new(op: Instruction, arg: OpargByte) -> Self {
        Self { op, arg }
    }
}
