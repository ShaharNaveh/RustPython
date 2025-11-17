//! Oparg definitions.

use crate::{AnyOparg, MarshalError, Oparg};
use std::ops::Deref;

/// Index inside [`CodeObject.constants`].
///
/// [CodeObject]: [crate::CodeObject]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct ConstIdxOparg(Oparg);

impl AnyOparg for ConstIdxOparg {}

impl Deref for ConstIdxOparg {
    type Target = Oparg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Oparg> for ConstIdxOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(Self::from(value))
    }
}

impl ConstIdxOparg {
    #[must_use]
    pub const fn new(value: Oparg) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct CountOparg(Oparg);

impl AnyOparg for CountOparg {}

impl Deref for CountOparg {
    type Target = Oparg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Oparg> for CountOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(Self::from(value))
    }
}

impl CountOparg {
    #[must_use]
    pub const fn new(value: Oparg) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct DeltaOparg(Oparg);

impl AnyOparg for DeltaOparg {}

impl Deref for DeltaOparg {
    type Target = Oparg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Oparg> for DeltaOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(Self::from(value))
    }
}

impl DeltaOparg {
    #[must_use]
    pub const fn new(value: Oparg) -> Self {
        Self(value)
    }
}

/// Index inside [`CodeObject.names`].
///
/// [CodeObject]: [crate::CodeObject]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct NameIdxOparg(Oparg);

impl AnyOparg for NameIdxOparg {}

impl Deref for NameIdxOparg {
    type Target = Oparg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Oparg> for NameIdxOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(Self::from(value))
    }
}

impl NameIdxOparg {
    #[must_use]
    pub const fn new(value: Oparg) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct VarNumOparg(Oparg);

impl AnyOparg for VarNumOparg {}

impl Deref for VarNumOparg {
    type Target = Oparg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Oparg> for VarNumOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(Self::from(value))
    }
}

impl VarNumOparg {
    #[must_use]
    pub const fn new(value: Oparg) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum BinOpOparg {
    /// `+`
    Add = 0,
    /// `&`
    And = 1,
    /// `//`
    FloorDivide = 2,
    /// `<<`
    Lshift = 3,
    /// `@`
    MatrixMultiply = 4,
    /// `*`
    Multiply = 5,
    /// `%`
    Remainder = 6,
    /// `|`
    Or = 7,
    /// `**`
    Power = 8,
    /// `>>`
    Rshift = 9,
    /// `-`
    Subtract = 10,
    /// `/`
    TrueDivide = 11,
    /// `^`
    Xor = 12,
    /// `+`
    InplaceAdd = 13,
    /// `&=`
    InplaceAnd = 14,
    /// `//=`
    InplaceFloorDivide = 15,
    /// `<<=`
    InplaceLshift = 16,
    /// `@=`
    InplaceMatrixMultiply = 17,
    /// `*=`
    InplaceMultiply = 18,
    /// `%=`
    InplaceRemainder = 19,
    /// `|=`
    InplaceOr = 20,
    /// `**=`
    InplacePower = 21,
    /// `>>=`
    InplaceRshift = 22,
    /// `-=`
    InplaceSubtract = 23,
    /// `/=`
    InplaceTrueDivide = 24,
    /// `^=`
    InplaceXor = 25,
}

impl AnyOparg for BinOpOparg {}

impl TryFrom<Oparg> for BinOpOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            0 => Self::Add,
            1 => Self::And,
            2 => Self::FloorDivide,
            3 => Self::Lshift,
            4 => Self::MatrixMultiply,
            5 => Self::Multiply,
            6 => Self::Remainder,
            7 => Self::Or,
            8 => Self::Power,
            9 => Self::Rshift,
            10 => Self::Subtract,
            11 => Self::TrueDivide,
            12 => Self::Xor,
            13 => Self::InplaceAdd,
            14 => Self::InplaceAnd,
            15 => Self::InplaceFloorDivide,
            16 => Self::InplaceLshift,
            17 => Self::InplaceMatrixMultiply,
            18 => Self::InplaceMultiply,
            19 => Self::InplaceRemainder,
            20 => Self::InplaceOr,
            21 => Self::InplacePower,
            22 => Self::InplaceRshift,
            23 => Self::InplaceSubtract,
            24 => Self::InplaceTrueDivide,
            25 => Self::InplaceXor,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

/// Specifies if a slice is built with either 2 or 3 arguments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum BuildSliceOparg {
    /// ```py
    /// x[5:10]
    /// ```
    Two = 2,
    /// ```py
    /// x[5:10:2]
    /// ```
    Three = 3,
}

impl AnyOparg for BuildSliceOparg {}

impl TryFrom<Oparg> for BuildSliceOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            2 => Self::Two,
            3 => Self::Three,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CallIntrinsic1Oparg {
    /// Not valid.
    Invalid = 0,
    /// Prints the argument to standard out. Used in the REPL.
    Print = 1,
    /// Performs `import *` for the named module.
    ImportStar = 2,
    /// Extracts the return value from a `StopIteration` exception.
    StopIterationError = 3,
    /// Wraps an async generator value.
    AsyncGenWrap = 4,
    /// Performs the unary `+` operation.
    UnaryPositive = 5,
    /// Converts a list to a tuple.
    ListToTuple = 6,
    /// Creates a [`typing.TypeVar`].
    ///
    /// [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar,
    TypeVar = 7,
    /// Crates a [`typing.ParamSpec`].
    ///
    /// [typing.ParamSpec]: https://docs.python.org/3.13/library/typing.html#typing.ParamSpec
    ParamSpec = 8,
    /// Crates a [`typing.TypeVarTuple`]
    ///
    /// [typing.TypeVarTuple]: https://docs.python.org/3.13/library/typing.html#typing.TypeVarTuple
    TypeVarTuple = 9,
    /// Generic subscript for [`PEP695`].
    ///
    /// [PEP695]: https://peps.python.org/pep-0695/
    SubscriptGeneric = 10,
    /// Creates a [`typing.TypeAliasType`].
    ///
    /// Used in the [`type`] statement. The argument is a tuple of the type aliass name, type parameters, and value.
    ///
    /// [type]: https://docs.python.org/3.13/reference/simple_stmts.html#type
    /// [typing.TypeAliasType]: https://docs.python.org/3.13/library/typing.html#typing.TypeAliasType
    TypeAlias = 11,
}

impl AnyOparg for CallIntrinsic1Oparg {}

impl TryFrom<Oparg> for CallIntrinsic1Oparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            0 => Self::Invalid,
            1 => Self::Print,
            2 => Self::ImportStar,
            3 => Self::StopIterationError,
            4 => Self::AsyncGenWrap,
            5 => Self::UnaryPositive,
            6 => Self::ListToTuple,
            7 => Self::TypeVar,
            8 => Self::ParamSpec,
            9 => Self::TypeVarTuple,
            10 => Self::SubscriptGeneric,
            11 => Self::TypeAlias,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CallIntrinsic2Oparg {
    /// Not valid.
    Invalid = 0,
    /// Calculates the [`ExceptionGroup`] to raise from a `try-except*`.
    ///
    /// [ExceptionGroup]: https://docs.python.org/3.13/library/exceptions.html#ExceptionGroup
    PrepReraiseStar = 1,
    /// Creates a [`typing.TypeVar`] with a bound.
    ///
    /// [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar
    TypeVarWithBound = 2,
    /// Creates a [`typing.TypeVar`] with constraints.
    ///
    /// [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar
    TypeVarWithConstraint = 3,
    /// Sets the `__type_params__` attribute of a function.
    SetFunctionTypeParams = 4,
}

impl AnyOparg for CallIntrinsic2Oparg {}

impl TryFrom<Oparg> for CallIntrinsic2Oparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            0 => Self::Invalid,
            1 => Self::PrepReraiseStar,
            2 => Self::TypeVarWithBound,
            3 => Self::TypeVarWithConstraint,
            4 => Self::SetFunctionTypeParams,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct CompareOparg(Oparg);

impl AnyOparg for CompareOparg {}

impl Deref for CompareOparg {
    type Target = Oparg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Oparg> for CompareOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(Self::from(value))
    }
}

impl CompareOparg {
    #[must_use]
    pub fn cmp_op(self) -> Result<CmpOp, MarshalError> {
        CmpOp::try_from(self)
    }

    /// Indicated if the comparison result should be coerced to bool.
    #[must_use]
    pub const fn coerce_bool(self) -> bool {
        (*self & 16) != 0
    }

    #[must_use]
    pub const fn new(value: Oparg) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CmpOp {
    /// `<`
    Lt = 2,
    /// `<=`
    Le = 10,
    /// `==`
    Eq = 8,
    /// `!=`
    Ne = 7,
    /// `>`
    Gt = 4,
    /// `>=`
    Ge = 12,
}

impl TryFrom<CompareOparg> for CmpOp {
    type Error = MarshalError;

    fn try_from(value: CompareOparg) -> Result<Self, Self::Error> {
        let cmp_bit = u32::from(*value) >> 5;
        Self::try_from(Oparg::from(cmp_bit)).map_err(|_| Self::Error::InvalidBytecode)
    }
}

impl TryFrom<Oparg> for CmpOp {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            2 => Self::Lt,
            10 => Self::Le,
            8 => Self::Eq,
            7 => Self::Ne,
            4 => Self::Gt,
            12 => Self::Ge,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

/// Used for implementing formatted string literals (f-strings).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ConvertValueOparg {
    /// No conversion.
    None = 0,
    /// Converts by calling `str(...)`.
    Str = 1,
    /// Converts by calling `repr(...)`.
    Repr = 2,
    /// Converts by calling `ascii(...)`.
    Ascii = 3,
}

impl AnyOparg for ConvertValueOparg {}

impl TryFrom<Oparg> for ConvertValueOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            0 => Self::None,
            1 => Self::Str,
            2 => Self::Repr,
            3 => Self::Ascii,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum InvertOparg {
    No = 0,
    Yes = 1,
}

impl AnyOparg for InvertOparg {}

impl TryFrom<Oparg> for InvertOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            0 => Self::No,
            1 => Self::Yes,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

/// Raises an exception using one of the 3 forms of the `raise` statement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RaiseVarArgsOparg {
    /// Re-Raise previous exception.
    ///
    /// ```py
    /// raise
    /// ```
    Reraise = 0,
    /// Raise exception instance or type at `STACK[-1]`.
    ///
    /// ```py
    /// raise STACK[-1]
    /// ```
    Raise = 1,
    /// Raise exception instance or type at `STACK[-2]` with `__cause__` set to `STACK[-1]`.
    ///
    /// ```py
    /// raise STACK[-2] from STACK[-1]
    /// ```
    RaiseCause = 2,
}

impl AnyOparg for RaiseVarArgsOparg {}

impl TryFrom<Oparg> for RaiseVarArgsOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            0 => Self::Reraise,
            1 => Self::Raise,
            2 => Self::RaiseCause,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ResumeOparg {
    AtFuncStart = 0,
    AfterYield = 1,
    AfterYieldFrom = 2,
    AfterAwait = 3,
}

impl AnyOparg for ResumeOparg {}

impl TryFrom<Oparg> for ResumeOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            0 => Self::AtFuncStart,
            1 => Self::AfterYield,
            2 => Self::AfterYieldFrom,
            3 => Self::AfterAwait,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

/// Determines which attribute to set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SetFunctionAttributeOparg {
    /// A tuple of default values for positional-only and positional-or-keyword parameters in positional order.
    Defaults = 1,
    /// A dictionary of keyword-only parameters' default values.
    KwDefaults = 2,
    /// A tuple of strings containing parameters' annotations.
    Annotations = 4,
    /// A tuple containing cells for free variables, making a closure.
    Closure = 8,
}

impl AnyOparg for SetFunctionAttributeOparg {}

impl TryFrom<Oparg> for SetFunctionAttributeOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            1 => Self::Defaults,
            2 => Self::KwDefaults,
            4 => Self::Annotations,
            8 => Self::Closure,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}

/// Indicates where the instruction occurs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum WhereOparg {
    /// Nowhere
    NoWhere = 0,
    /// After a call to `__aenter__`.
    AfterAEnter = 1,
    /// After a call to `__aexit__`.
    AfterAExit = 2,
}

impl AnyOparg for WhereOparg {}

impl TryFrom<Oparg> for WhereOparg {
    type Error = MarshalError;

    fn try_from(value: Oparg) -> Result<Self, Self::Error> {
        Ok(match *value {
            0 => Self::NoWhere,
            1 => Self::AfterAEnter,
            2 => Self::AfterAExit,
            _ => return Err(MarshalError::InvalidBytecode),
        })
    }
}
