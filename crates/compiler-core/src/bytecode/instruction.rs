use core::{fmt, marker::PhantomData, mem};

use crate::{
    bytecode::{
        BorrowedConstant, Constant, InstrDisplayContext,
        oparg::{AnyOparg, OpArg, OpArgByte, RaiseKind},
        oparg::{
            BinaryOperator, BuildSliceArgCount, ComparisonOperator, ConvertValueOparg,
            IntrinsicFunction1, IntrinsicFunction2, Invert, Label, MakeFunctionFlags, NameIdx,
            OpArg, OpArgByte, OpArgType, RaiseKind, SpecialMethod, UnpackExArgs,
        },
        opcode::{AnyOpcode, Opcode, PseudoOpcode},
    },
    marshal::MarshalError,
};

/// Single bytecode instruction that is executed by the VM.
#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    opcode: Opcode,
    oparg: Option<AnyOparg>,
}

/// Instructions used by the compiler. They are not executed by the VM.
#[derive(Clone, Copy, Debug)]
pub struct PseudoInstruction {
    opcode: PseudoOpcode,
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

macro_rules! inst_either {
    (fn $name:ident ( &self $(, $arg:ident : $arg_ty:ty )* ) -> $ret:ty ) => {
        fn $name(&self $(, $arg : $arg_ty )* ) -> $ret {
            match self {
                Self::Real(op) => op.$name($($arg),*),
                Self::Pseudo(op) => op.$name($($arg),*),
            }
        }
    };
}

impl InstructionMetadata for AnyInstruction {
    inst_either!(fn label_arg(&self) -> Option<Arg<Label>>);

    inst_either!(fn is_unconditional_jump(&self) -> bool);

    inst_either!(fn is_scope_exit(&self) -> bool);

    inst_either!(fn stack_effect(&self, arg: OpArg) -> i32);

    inst_either!(fn fmt_dis(
        &self,
        arg: OpArg,
        f: &mut fmt::Formatter<'_>,
        ctx: &impl InstrDisplayContext,
        expand_code_objects: bool,
        pad: usize,
        level: usize
    ) -> fmt::Result);
}

impl AnyInstruction {
    /// Gets the inner value of [`Self::Real`].
    pub const fn real(self) -> Option<Instruction> {
        match self {
            Self::Real(ins) => Some(ins),
            _ => None,
        }
    }

    /// Gets the inner value of [`Self::Pseudo`].
    pub const fn pseudo(self) -> Option<PseudoInstruction> {
        match self {
            Self::Pseudo(ins) => Some(ins),
            _ => None,
        }
    }

    /// Same as [`Self::real`] but panics if wasn't called on [`Self::Real`].
    ///
    /// # Panics
    ///
    /// If was called on something else other than [`Self::Real`].
    pub const fn expect_real(self) -> Instruction {
        self.real()
            .expect("Expected Instruction::Real, found Instruction::Pseudo")
    }

    /// Same as [`Self::pseudo`] but panics if wasn't called on [`Self::Pseudo`].
    ///
    /// # Panics
    ///
    /// If was called on something else other than [`Self::Pseudo`].
    pub const fn expect_pseudo(self) -> PseudoInstruction {
        self.pseudo()
            .expect("Expected Instruction::Pseudo, found Instruction::Real")
    }
}
