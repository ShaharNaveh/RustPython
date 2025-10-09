use crate::bytecode::OpArgByte;
use bitflags::bitflags;

/// Values used in the oparg for `RealOpcode::Resume`.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum ResumeOpArg {
    // https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L61-L65
    AtFuncStart = 0,
    AfterYield = 1,
    AfterYieldFrom = 2,
    AfterAwait = 3,
}

/// CPython 3.11+ linetable location info codes
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PyCodeLocationInfoKind {
    // Short forms are 0 to 9
    Short0 = 0,
    Short1 = 1,
    Short2 = 2,
    Short3 = 3,
    Short4 = 4,
    Short5 = 5,
    Short6 = 6,
    Short7 = 7,
    Short8 = 8,
    Short9 = 9,
    // One line forms are 10 to 12
    OneLine0 = 10,
    OneLine1 = 11,
    OneLine2 = 12,
    NoColumns = 13,
    Long = 14,
    None = 15,
}

impl PyCodeLocationInfoKind {
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Short0),
            1 => Some(Self::Short1),
            2 => Some(Self::Short2),
            3 => Some(Self::Short3),
            4 => Some(Self::Short4),
            5 => Some(Self::Short5),
            6 => Some(Self::Short6),
            7 => Some(Self::Short7),
            8 => Some(Self::Short8),
            9 => Some(Self::Short9),
            10 => Some(Self::OneLine0),
            11 => Some(Self::OneLine1),
            12 => Some(Self::OneLine2),
            13 => Some(Self::NoColumns),
            14 => Some(Self::Long),
            15 => Some(Self::None),
            _ => Option::None,
        }
    }

    pub fn is_short(&self) -> bool {
        (*self as u8) <= 9
    }

    pub fn short_column_group(&self) -> Option<u8> {
        if self.is_short() {
            Some(*self as u8)
        } else {
            Option::None
        }
    }

    pub fn one_line_delta(&self) -> Option<i32> {
        match self {
            Self::OneLine0 => Some(0),
            Self::OneLine1 => Some(1),
            Self::OneLine2 => Some(2),
            _ => Option::None,
        }
    }
}

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L55-L59
bitflags! {
    /// Flags used in the oparg for `RealOpcde::MakeFunction`.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MakeFunctionFlags: u8 {
        const DEFAULTS = 0x01;
        const KW_DEFAULTS = 0x02;
        const ANNOTATIONS = 0x04;
        const CLOSURE = 0x08;
    }
}

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_opcode_utils.h#L67-L68
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ResumeOpArgMask: u8 {
        const LOCATION = 0x03;
        const DEPTH1 = 0x04;
    }
}

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_intrinsics.h#L8-L20
/// Intrinsic function for `RealOpcde::CallIntrinsic1`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IntrinsicFunction1 {
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

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/internal/pycore_intrinsics.h#L25-L31
/// Intrinsic function for `RealOpcode::CallIntrinsic2`
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IntrinsicFunction2 {
    Invalid = 0,
    PrepReraiseStar = 1,
    TypeVarWithBound = 2,
    TypeVarWithConstraint = 3,
    SetFunctionTypeParams = 4,
    /// Set default value for type parameter (PEP 695).
    SetTypeparamDefault = 5,
}

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/opcode.h#L10-L35
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum BinaryOperatorOpArg {
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

// https://github.com/python/cpython/blob/a15ae614deb58f78f9f4aa11ed18a0afc6a9df7d/Include/ceval.h#L127-L134
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
bitflags! {
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum OpArgKind {
    Resume(ResumeOpArg),
    IntrinsicFunction1(IntrinsicFunction1OpArg),
    IntrinsicFunction2(IntrinsicFunction2OpArg),
    BinaryOperator(BinaryOperatorOpArg),
    Raw(OpArgByte),
}

impl OpArgKind {
    #[must_use]
    pub const fn new(opcode: RealOpcode, oparg_byte: OpArgByte) -> crate::marshal::MarshalError {
        // TODO:
    }
}
