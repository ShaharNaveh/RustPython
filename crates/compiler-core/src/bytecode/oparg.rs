use crate::{CodeUnit, MarshalError, RealInstruction};
use std::{
    fmt,
    marker::PhantomData,
    ops::{ Deref },
};

pub trait AnyOparg: Copy {
    fn try_from_oparg(value: Oparg) -> Result<Self, MarshalError>;

    fn as_oparg(self) -> Oparg;

    fn as_u32(self) -> u32 {
        self.as_oparg().as_u32()
    }
}

/// Zero sized struct for holding a possible Oparg type.
#[derive(Copy, Clone)]
pub struct OpargType<T: AnyOparg>(PhantomData<T>);

impl<T: AnyOparg> OpargType<T> {
    pub const MARKER: Self = Self(PhantomData);

    #[inline(always)]
    pub fn get(self, oparg: Oparg) -> Result<T, MarshalError> {
        T::try_from_oparg(oparg)
    }

    /// # Safety
    /// T::try_from(oparg) must succeed.
    #[inline(always)]
    pub unsafe fn get_unchecked(self, oparg: Oparg) -> T {
        // SAFETY: requirements forwarded from caller
        unsafe { self.get(oparg).unwrap_unchecked() }
    }
}

impl<T: AnyOparg> PartialEq for OpargType<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T: AnyOparg> Eq for OpargType<T> {}

impl<T: AnyOparg> fmt::Debug for OpargType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OpargType<{}>", std::any::type_name::<T>())
    }
}

/// an opcode argument that may be extended by a prior ExtendedArg
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct OpargByte(u8);

impl OpargByte {
    pub const NULL: Self = Self::new(0);

    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

impl Deref for OpargByte {
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct Oparg(u32);

impl AnyOparg for Oparg {
    fn try_from_oparg(value: Self) -> Result<Self, MarshalError> {
        Ok(value)
    }

    fn to_oparg(self) -> Self {
        self
    }
}

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

impl From<Oparg> for u32 {
    fn from(value: Oparg) -> Self {
        value.0
    }
}

impl Oparg {
    pub const NULL: Self = Self::new(0);

    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    // Const hack; Use `u32::from(oparg)` when it can be used in a const context.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
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
    pub fn get(&mut self, ins: CodeUnit) -> (RealInstruction, Oparg) {
        let arg = self.extend(ins.arg);
        if !matches!(ins.op, RealInstruction::ExtendedArg(_)) {
            self.reset();
        }
        (ins.op, arg)
    }

    #[inline(always)]
    pub fn extend(&mut self, arg: OpargByte) -> Oparg {
        let
        self.state = Oparg::new((self.state.as_u32() << 8) | u32::from(*arg));
        self.state
    }

    #[inline(always)]
    pub const fn reset(&mut self) {
        self.state = Oparg::NULL;
    }
}
