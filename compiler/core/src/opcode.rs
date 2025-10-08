pub use crate::opcodes::{PseudoOpcode, RealOpcode};

macro_rules! gen_has_attr_fn {
    ($attr:ident) => {
        #[inline]
        pub const fn $attr(&self) -> bool {
            match self {
                Real(val) => val.$attr(),
                Pseudo(val) => val.$attr(),
            }
        }
    };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Opcode {
    Real(RealOpcode),
    Pseudo(PseudoOpcode),
}

impl Opcode {
    #[inline]
    pub const fn real(self) -> Option<RealOpcode> {
        match self {
            Real(val) => Some(val),
            _ => None,
        }
    }

    #[inline]
    pub const fn pseudo(self) -> Option<RealOpcode> {
        match self {
            Pseudo(val) => Some(val),
            _ => None,
        }
    }

    #[inline]
    pub const fn is_block_push(&self) -> bool {
        // https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L19-L22
        matches!(
            self.pseudo(),
            Some(PseudoOpcode::SetupFinally | PseudoOpcode::SetupWith | PseudoOpcode::SetupCleanup)
        )
    }

    #[inline]
    pub const fn has_target(&self) -> bool {
        // https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L24-L25
        self.has_jump() || self.is_block_push()
    }

    #[inline]
    /// Opcodes that must be last in the basicblock.
    pub const fn is_terminator(&self) -> bool {
        // https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L27-L29
        self.has_jump() || self.is_scope_exit()
    }

    #[inline]
    /// Opcodes which are not emitted in codegen stage, only by the assembler.
    pub const fn is_assembler(&self) -> bool {
        // https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L31-L35
        matches!(
            self.real(),
            Some(
                RealOpcode::JumpForward
                    | RealOpcode::JumpBackward
                    | RealOpcode::JumpBackwardNoInterrupt
            )
        )
    }

    #[inline]
    pub const fn is_backwards_jump(&self) -> bool {
        // https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L37-L39
        matches!(
            self.real(),
            Some(RealOpcode::JumpBackward | RealOpcode::JumpBackwardNoInterrupt)
        )
    }

    #[inline]
    pub const fn is_unconditional_jump(&self) -> bool {
        // https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L41-L46
        matches!(
            self.pseudo(),
            Some(PseudoOpcode::Jump | PseudoOpcode::JumpNoInterrupt)
        ) || matches!(
            self.real(),
            Some(
                RealOpcode::JumpForward
                    | RealOpcode::JumpBackward
                    | RealOpcode::JumpBackwardNoInterrupt
            )
        )
    }

    #[inline]
    pub const fn is_scope_exit(&self) -> bool {
        // https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L48-L52
        matches!(
            self.real(),
            Some(
                RealOpcode::ReturnValue
                    | RealOpcode::ReturnConst
                    | RealOpcode::RaiseVarargs
                    | RealOpcode::Reraise
            )
        )
    }

    gen_has_attr_fn!(has_arg);
    gen_has_attr_fn!(has_const);
    gen_has_attr_fn!(has_exc);
    gen_has_attr_fn!(has_free);
    gen_has_attr_fn!(has_jump);
    gen_has_attr_fn!(has_local);
}

macro_rules! impl_try_from {
    ($struct_name:ident, $($t:ty),+ $(,)?) => {
        $(
            impl TryFrom<$t> for $struct_name {
                type Error = ();

                fn try_from(raw: $t) -> Result<Self, Self::Error> {
                    RealOpcode::try_from(raw)
                        .map(Opcode::Real)
                        .or_else(|_| PseudoOpcode::try_from(raw).map(Opcode::Pseudo))
                }
            }
        )+
    };
}

impl_try_from!(
    Opcode, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
