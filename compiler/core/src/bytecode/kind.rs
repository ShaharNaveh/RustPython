use crate::byecode::Oparg;
use bitflags::bitflags;

pub trait OpargFamilyMember: Copy {
    fn try_from_u8(raw: u8) -> Result<Self, crate::marshal::MarshalError>;
}

/// Internal helper for [`oparg_enum!`].
///
/// Creates the following implementations for a given enum:
/// - `TryFrom<u8>`
/// - `TryFrom<OpargByte>`
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
        impl OpargFamilyMember for $name {
            fn try_from_u8(raw: u8) -> Result<Self, $crate::marshal::MarshalError> {
                Ok(match raw {
                    $($value => Self::$var,)*
                    _ => return Err($crate::marshal::MarshalError::InvalidBytecode)
                })
            }
        }

        /*
        impl From<$name> for $crate::bytecode::Oparg {
            fn from(oparg: $name) -> Self {
                Self::from(oparg as u8)
            }
        }
*/
        /*
        impl TryFrom<$crate::bytecode::OpargByte> for $name {
            type Error = $crate::marshal::MarshalError;

            fn try_from(oparg: $crate::bytecode::OpargByte) -> Result<Self, Self::Err> {
                Self::try_from(u8::from(oparg))
            }
        }

        impl TryFrom<$crate::bytecode::Oparg> for $name {
            type Error = $crate::marshal::MarshalError;

            fn try_from(oparg: $crate::bytecode::Oparg) -> Result<Self, Self::Err> {
                Self::try_from(u8::try_from(oparg).map_err(|_| Self::Error::InvalidBytecode)?)
            }
        }
        */

/*
        impl From<$name> for u8 {
            fn from(oparg: $name) -> Self {
                oparg as u8
            }
        }
*/
        /*
        impl From<$name> for u32 {
            fn from(oparg: $name) -> Self {
                u8::from(oparg) as u32
            }
        }
        */

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
    /// Values used in the oparg for `RealOpcode::Resume`.
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
    /// Flags used in the oparg for `RealOpcde::MakeFunction`.
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
    /// Intrinsic function for `RealOpcde::CallIntrinsic1`.
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[repr(u8)]
    pub enum IntrinsicFunction1Oparg {
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
    /// Intrinsic function for `RealOpcode::CallIntrinsic2`
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[repr(u8)]
    pub enum IntrinsicFunction2Oparg {
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum OpargFamily<T: Into<Oparg>> {
    Resume(ResumeOparg),
    BinaryOperator(BinaryOperatorOparg),
    IntrinsicFunction2(IntrinsicFunction2Oparg),
    IntrinsicFunction1(IntrinsicFunction1Oparg),
    None(T),
}
