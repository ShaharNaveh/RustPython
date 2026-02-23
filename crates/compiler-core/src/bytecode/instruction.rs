use crate::bytecode::generated::{Instruction, Opcode, PseudoInstruction, PseudoOpcode};

impl Instruction {
    const fn is_unconditional_jump(self) -> bool {
        matches!(
            self.opcode(),
            Opcode::JumpForward | Opcode::JumpBackward | Opcode::JumpBackwardNoInterrupt
        )
    }

    const fn is_scope_exit(self) -> bool {
        matches!(
            self.opcode(),
            Opcode::ReturnValue | Opcode::RaiseVarargs | Opcode::Reraise
        )
    }
}

impl PseudoInstruction {
    const fn is_scope_exit(&self) -> bool {
        false
    }

    const fn is_unconditional_jump(&self) -> bool {
        matches!(
            self.opcode(),
            PseudoOpcode::Jump | PseudoOpcode::JumpNoInterrupt
        )
    }

    /// Returns true if this is a block push pseudo instruction
    /// (SETUP_FINALLY, SETUP_CLEANUP, or SETUP_WITH).
    pub const fn is_block_push(&self) -> bool {
        matches!(
            self.opcode(),
            PseudoOpcode::SetupCleanup | PseudoOpcode::SetupFinally | PseudoOpcode::SetupWith
        )
    }
}

#[derive(Clone, Copy)]
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
    ($vis:vis $(const)? fn $name:ident ( self $(, $arg:ident : $arg_ty:ty )* ) -> $ret:ty ) => {
      $vis $(const)? fn $name(self $(, $arg : $arg_ty )* ) -> $ret {
            match self {
                Self::Real(instr) => instr.$name($($arg),*),
                Self::Pseudo(instr) => instr.$name($($arg),*),
            }
        }
    };
}

impl AnyInstruction {
    inst_either!(pub const fn label_oparg(self) -> Option<crate::bytecode::oparg::Label>);

    inst_either!(pub const fn is_unconditional_jump(self) -> bool);

    inst_either!(pub const fn is_scope_exit(self) -> bool);

    inst_either!(pub fn stack_effect(self) -> i32);

    inst_either!(pub fn stack_effect_info(self) -> StackEffect);

    inst_either!(pub fn fmt_dis(
        self,
        f: &mut fmt::Formatter<'_>,
        ctx: &impl InstrDisplayContext,
        expand_code_objects: bool,
        pad: usize,
        level: usize
    ) -> core::fmt::Result);
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

    /// Returns true if this is a block push pseudo instruction
    /// (SETUP_FINALLY, SETUP_CLEANUP, or SETUP_WITH).
    pub const fn is_block_push(self) -> bool {
        matches!(self, Self::Pseudo(instr) if instr.is_block_push())
    }

    /// Returns true if this is a POP_BLOCK pseudo instruction.
    pub const fn is_pop_block(self) -> bool {
        matches!(self, Self::Pseudo(PseudoInstruction::PopBlock))
    }
}

/// What effect the instruction has on the stack.
#[derive(Clone, Copy)]
pub struct StackEffect {
    /// How many items the instruction is pushing on the stack.
    pushed: u32,
    /// How many items the instruction is popping from the stack.
    popped: u32,
}

impl StackEffect {
    /// Creates a new [`Self`].
    pub const fn new(pushed: u32, popped: u32) -> Self {
        Self { pushed, popped }
    }

    /// Get the calculated stack effect as [`i32`].
    pub const fn effect(self) -> i32 {
        (self.pushed() as i32) - (self.popped() as i32)
    }

    /// Get the pushed count.
    pub const fn pushed(self) -> u32 {
        self.pushed
    }

    /// Get the popped count.
    pub const fn popped(self) -> u32 {
        self.popped
    }
}

impl From<StackEffect> for i32 {
    fn from(stack_effect: StackEffect) -> Self {
        stack_effect.effect()
    }
}
