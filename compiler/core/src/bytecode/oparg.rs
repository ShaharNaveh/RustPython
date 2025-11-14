use crate::{CodeUnit, RealInstruction};

pub trait AnyOparg: Copy + TryFrom<Oparg> {}

/// an opcode argument that may be extended by a prior ExtendedArg
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct OpargByte(u8);

impl OpArgByte {
    pub const NULL: Self = Self::new(0);

    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for OpargByte {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for OpargByte {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for OpargByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Full 32-bit oparg, including any possible [`RealInstruction::ExtendedArg`] extension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct Oparg(u32);

impl AnyOparg for Oparg {}

impl std::ops::Deref for Oparg {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for Oparg {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl TryFrom<u32> for Oparg {
    type Error = crate::MarshalError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self::from(value))
    }
}

impl Oparg {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns how many CodeUnits an instruction with this oparg will be encoded as.
    #[inline]
    pub const fn instr_size(self) -> usize {
        (self.0 > 0xff) as usize + (self.0 > 0xff_ff) as usize + (self.0 > 0xff_ff_ff) as usize + 1
    }

    /// returns the arg split into any necessary ExtendedArg components (in big-endian order) and
    /// the arg for the real opcode itself
    #[inline(always)]
    pub fn split(self) -> (impl ExactSizeIterator<Item = OpargByte>, OpargByte) {
        let mut it = self
            .0
            .to_le_bytes()
            .map(OpargByte::new)
            .into_iter()
            .take(self.instr_size());
        let lo = it.next().unwrap();
        (it.rev(), lo)
    }
}

#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct OpargState {
    state: Oparg,
}

impl OpargState {
    #[inline(always)]
    pub fn get(&mut self, ins: CodeUnit) -> (Instruction, Oparg) {
        let arg = self.extend(ins.arg);
        if ins.op != Instruction::ExtendedArg {
            self.reset();
        }
        (ins.op, arg)
    }

    #[inline(always)]
    pub fn extend(&mut self, arg: OpargByte) -> OpArg {
        self.state = Oparg::from((self.state << 8) | u32::from(arg));
        self.state
    }

    #[inline(always)]
    pub const fn reset(&mut self) {
        self.state = Oparg::NULL;
    }
}
