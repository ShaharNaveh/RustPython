use crate::{byecode::Oparg, marshal::MarshalError};
use bitflags::bitflags;
use std::{fmt, marker::PhantomData, ops::Deref};

pub trait OpargType: Copy + Into<Oparg> + TryFrom<Oparg> {}

/// Internal helper for [`oparg_enum!`].
///
/// Creates the following implementations for a given enum:
/// - `TryFrom<Oparg>`
/// - `Into<Oparg>`
/// - [`OpargType`](crate::bytecode::OpargType)
///
/// Should not be used directly outside of macro expansion.
///
/// # Safety
///
/// The generated conversion performs strict range checking and
/// returns [`MarshalError::InvalidBytecode`](crate::marshal::MarshalError::InvalidBytecode)
/// for any unmapped operand value.
macro_rules! oparg_enum_impl {
    (enum $name:ident { $($(#[$var_attr:meta])* $var:ident = $value:literal,)* }) => {
        impl TryFrom<Oparg> for $name {
            type Error = $crate::marshal::MarshalError;

            fn try_from(oparg: Oparg) -> Result<Self, Self::Error> {
                let raw = u8::try_from(oparg).map_err(|_| Self::Error::InvalidBytecode)?;
                Ok(match raw {
                    $($value => Self::$var,)*
                    _ => return Err(Self::Error::InvalidBytecode)
                })
            }
        }

        impl From<$name> for Oparg {
            fn from(oparg_type: $name) -> Self {
                Self::from(oparg_type as u8)
            }
        }

        impl OpargType for $name {}
    };
}

/// Defines an enum representing a valid set of opcode argument (`oparg`) values,
/// and automatically implements conversions from raw bytecode operands.
///
/// This macro generates both the enum declaration **and** the necessary
/// `TryFrom` implementations for safely converting from:
/// - `u8` (a raw bytecode argument byte)
/// - [`Oparg`](crate::bytecode::Oparg) (a full 32-bit operand wrapper)
///
/// # Examples
///
/// ```rust
/// use crate::bytecode::Oparg;
///
/// oparg_enum! {
///     /// Example operation argument variants.
///     #[derive(Copy, Clone, Debug, Eq, PartialEq)]
///		#[repr(u8)]
///     pub enum MyOparg {
///         Foo = 0,
///         Bar = 1,
///     }
/// }
///
/// assert_eq!(MyOparg::try_from(0u8), Ok(MyOparg::Add));
/// assert_eq!(MyOparg::try_from(Oparg(1)), Ok(MyOparg::Sub));
/// assert!(MyOparg::try_from(255u8).is_err());
/// ```
macro_rules! oparg_enum {
    ($(#[$attr:meta])* $vis:vis enum $name:ident { $($(#[$var_attr:meta])* $var:ident = $value:literal,)* }) => {
        $(#[$attr])*
        $vis enum $name {
            $($(#[$var_attr])* $var = $value,)*
        }

        oparg_enum_impl!(enum $name {
            $($(#[$var_attr])* $var = $value,)*
        });
    };
}

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L61-L65
oparg_enum!(
    /// Values used in the oparg for `RealInstruction::Resume`.
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[repr(u8)]
    pub enum ResumeOparg {
        AtFuncStart = 0,
        AfterYield = 1,
        AfterYieldFrom = 2,
        AfterAwait = 3,
    }
);

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L55-L59
bitflags! {
    /// Flags used in the oparg for `RealInstruction::MakeFunction`.
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    pub struct MakeFunctionFlags: u8 {
        const DEFAULTS = 0x01;
        const KW_DEFAULTS = 0x02;
        const ANNOTATIONS = 0x04;
        const CLOSURE = 0x08;
    }
}

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L67-L68
bitflags! {
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    pub struct ResumeOpargMask: u8 {
        const LOCATION = 0x03;
        const DEPTH1 = 0x04;
    }
}

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_intrinsics.h#L8-L20
oparg_enum!(
    /// Intrinsic function for `RealInstruction::CallIntrinsic1`.
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[repr(u8)]
    pub enum CallIntrinsic1Oparg {
        Invalid = 0,
        Print = 1,
        /// Import * operation.
        ImportStar = 2,
        StopIterationError = 3,
        AsyncGenWrap = 4,
        UnaryPositive = 5,
        /// Convert List to Tuple.
        ListToTuple = 6,
        /// Type parameter related.
        TypeVar = 7,
        ParamSpec = 8,
        TypeVarTuple = 9,
        /// Generic subscript for PEP 695.
        SubscriptGeneric = 10,
        TypeAlias = 11,
    }
);

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_intrinsics.h#L25-L31
oparg_enum!(
    /// Intrinsic function for `RealInstruction::CallIntrinsic2`
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[repr(u8)]
    pub enum CallIntrinsic2Oparg {
        Invalid = 0,
        PrepReraiseStar = 1,
        TypeVarWithBound = 2,
        TypeVarWithConstraint = 3,
        SetFunctionTypeParams = 4,
        /// Set default value for type parameter (PEP 695).
        SetTypeparamDefault = 5,
    }
);

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/opcode.h#L10-L35
oparg_enum!(
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[repr(u8)]
    pub enum BinaryOperatorOparg {
        Add = 0,
        And = 1,
        FloorDivide = 2,
        Lshift = 3,
        MatrixMultiply = 4,
        Multiply = 5,
        Remainder = 6,
        Or = 7,
        Power = 8,
        Rshift = 9,
        Subtract = 10,
        TrueDivide = 11,
        Xor = 12,
        InplaceAdd = 13,
        InplaceAnd = 14,
        InplaceFloorDivide = 15,
        InplaceLshift = 16,
        InplaceMatrixMultiply = 17,
        InplaceMultiply = 18,
        InplaceRemainder = 19,
        InplaceOr = 20,
        InplacePower = 21,
        InplaceRshift = 22,
        InplaceSubtract = 23,
        InplaceTrueDivide = 24,
        InplaceXor = 25,
    }
);

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/ceval.h#L127-L134
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
bitflags! {
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    pub struct FormatValueConversion: u8 {
        // const MASK = 0x03;
        /// No conversion
        const NONE = 0x00;
        /// Converts by calling `str(<value>)`.
        const STR = 0x01;
        /// Converts by calling `repr(<value>)`.
        const REPR = 0x02;
        /// Converts by calling `ascii(<value>)`.
        const ASCII = 0x03;
    }
}

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/ceval.h#L127-L134
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
bitflags! {
    pub struct FormatValueSpec: u8 {
        const MASK = 0x04;
        const HAVE_SPEC = 0x04;
    }
}

oparg_enum!(
    /// The kind of Raise that occurred.
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[repr(u8)]
    pub enum RaiseVarArgsOparg {
        Reraise = 0,
        Raise = 1,
        RaiseCause = 2,
    }
);

/// an opcode argument that may be extended by a prior ExtendedArg
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct OpargByte(pub u8);

impl OpargByte {
    pub const NULL: Self = Self(0);
}

impl From<OpargByte> for u8 {
    fn from(oparg_byte: OpargByte) -> Self {
        oparg_byte.0
    }
}

impl From<OpargByte> for u32 {
    fn from(oparg_byte: OpargByte) -> Self {
        u32::from(oparg_byte.0)
    }
}

impl fmt::Debug for OpargByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// a full 32-bit op_arg, including any possible ExtendedArg extension
#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub struct Oparg(pub u32);

impl OpargType for Oparg {}

impl Oparg {
    pub const NULL: Self = Self(0);

    /// Returns how many CodeUnits a instruction with this op_arg will be encoded as
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
            .map(OpargByte)
            .into_iter()
            .take(self.instr_size());
        let lo = it.next().unwrap();
        (it.rev(), lo)
    }
}

impl Deref for Oparg {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for Oparg {
    fn from(raw: u8) -> Self {
        Self::from(u32::from(raw))
    }
}

impl From<u32> for Oparg {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl TryFrom<Oparg> for u8 {
    type Error = std::num::TryFromIntError;

    fn try_from(oparg: Oparg) -> Result<Self, Self::Error> {
        u8::try_from(oparg.0)
    }
}

#[derive(Copy, Clone)]
pub struct Arg<T: OpargType>(PhantomData<T>);

impl<T: OpargType> Arg<T> {
    #[inline]
    pub const fn marker() -> Self {
        Self(PhantomData)
    }

    #[inline]
    pub fn new(arg: T) -> (Self, Oparg) {
        (Self(PhantomData), Oparg::from(arg))
    }

    #[inline]
    pub fn new_single(arg: T) -> (Self, OpargByte)
    where
        T: Into<u8>,
    {
        (Self(PhantomData), OpargByte(arg.into()))
    }

    #[inline(always)]
    pub fn get(self, arg: Oparg) -> T {
        self.try_get(arg).unwrap()
    }

    #[inline(always)]
    pub fn try_get(self, arg: Oparg) -> Result<T, MarshalError> {
        T::try_from(arg).map_err(|_| MarshalError::InvalidBytecode)
    }

    /// # Safety
    /// T::from_op_arg(self) must succeed
    #[inline(always)]
    pub unsafe fn get_unchecked(self, arg: Oparg) -> T {
        // SAFETY: requirements forwarded from caller
        unsafe { T::try_from(arg).unwrap_unchecked() }
    }
}

impl<T: OpargType> PartialEq for Arg<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T: OpargType> Eq for Arg<T> {}

impl<T: OpargType> fmt::Debug for Arg<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Arg<{}>", std::any::type_name::<T>())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct DeltaOparg(pub u32);

impl TryFrom<Oparg> for DeltaOparg {
    type Error = MarshalError;

    fn try_from(oparg: Oparg) -> Result<Self, Self::Error> {
        Ok(Self(oparg.0))
    }
}

impl From<DeltaOparg> for Oparg {
    fn from(delta: DeltaOparg) -> Self {
        Self(delta.0)
    }
}

impl Deref for DeltaOparg {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for DeltaOparg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl OpargType for DeltaOparg {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct NameIdxOparg(pub u32);

impl TryFrom<Oparg> for NameIdxOparg {
    type Error = MarshalError;

    fn try_from(oparg: Oparg) -> Result<Self, Self::Error> {
        Ok(Self(oparg.0))
    }
}

impl From<NameIdxOparg> for Oparg {
    fn from(namei: NameIdxOparg) -> Self {
        Self(namei.0)
    }
}

impl Deref for NameIdxOparg {
    type Target = u32;

    fn deref(self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for NameIdxOparg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl OpargType for NameIdxOparg {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct ConstIdxOparg(pub u32);

impl TryFrom<Oparg> for ConstIdxOparg {
    type Error = MarshalError;

    fn try_from(oparg: Oparg) -> Result<Self, Self::Error> {
        Ok(Self(oparg.0))
    }
}

impl From<ConstIdxOparg> for Oparg {
    fn from(consti: ConstIdxOparg) -> Self {
        Self(consti.0)
    }
}

impl Deref for ConstIdxOparg {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for ConstIdxOparg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl OpargType for ConstIdxOparg {}

#[derive(Copy, Clone)]
pub struct UnpackExOparg(Oparg);

impl UnpackExOparg {
    #[must_use]
    pub fn before(&self) -> u8 {
        self.to_le_bytes()[0]
    }

    #[must_use]
    pub fn after(&self) -> u8 {
        self.to_le_bytes()[1]
    }
}

impl Deref for UnpackExOparg {
    type Target = Oparg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Oparg> for UnpackExOparg {
    type Error = MarshalError;

    #[inline(always)]
    fn try_from(oparg: Oparg) -> Result<Self, Self::Error> {
        Ok(Self(oparg))
    }
}

impl From<UnpackExOparg> for Oparg {
    #[inline(always)]
    fn from(unpack_ex_oparg: UnpackExOparg) -> Self {
        unpack_ex_oparg.0
    }
}

impl OpargType for UnpackExOparg {}

impl fmt::Display for UnpackExOparg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "before: {}, after: {}", self.before(), self.after())
    }
}
