//! Oparg definitions.

use crate::{MarshalError, Oparg};

macro_rules! any_oparg_struct {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ( $inner:ty );
    ) => {
        $(#[$meta])*
        $vis struct $name($inner);

        impl $name {
            #[must_use]
            pub const fn new(value: $inner) -> Self {
                Self(value)
            }
        }

        impl ::core::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self::new(value)
            }
        }

        impl $crate::AnyOparg for $name {
            fn try_from_oparg(value: $inner) -> Result<Self, $crate::MarshalError> {
                Ok(Self::from(value))
            }

            fn as_oparg(self) -> $inner {
                *self
            }
        }
    };
}

macro_rules! any_oparg_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $value,
            )*
        }

        impl From<$name> for $crate::Oparg {
            fn from(value: $name) -> Self {
                Self::new(value as u32)
            }
        }

        impl $crate::AnyOparg for $name {
            fn try_from_oparg(value: $crate::Oparg) -> Result<Self, $crate::MarshalError> {
                Ok(match *value {
                    $(
                        $value => Self::$variant,
                    )*
                    _ => return Err($crate::MarshalError::InvalidBytecode)
                })
            }

            fn as_oparg(self) -> $crate::Oparg {
                $crate::Oparg::from(self)
            }
        }
    };
}

any_oparg_struct!(
    /// Index inside [`CodeObject.constants`].
    ///
    /// [CodeObject]: [crate::CodeObject]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct ConstIdxOparg(Oparg);
);

any_oparg_struct!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct CountOparg(Oparg);
);

any_oparg_struct!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct DeltaOparg(Oparg);
);

any_oparg_struct!(
    /// Index inside [`CodeObject.names`].
    ///
    /// [CodeObject]: [crate::CodeObject]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct NameIdxOparg(Oparg);
);

any_oparg_struct!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct VarNumOparg(Oparg);
);

any_oparg_enum!(
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
);

any_oparg_enum!(
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
);

any_oparg_enum!(
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
);

any_oparg_enum!(
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
);

any_oparg_struct!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct CompareOparg(Oparg);
);

impl CompareOparg {
    #[must_use]
    pub fn cmp_op(self) -> Result<CmpOp, MarshalError> {
        CmpOp::try_from(self)
    }

    /// Indicated if the comparison result should be coerced to bool.
    #[must_use]
    pub const fn coerce_bool(self) -> bool {
        (oparg.as_u32() & 16) != 0
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
        Self::try_from(Oparg::new(value.as_u32() >> 5))
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

any_oparg_enum!(
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
);

any_oparg_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum InvertOparg {
        No = 0,
        Yes = 1,
    }
);

any_oparg_enum!(
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
);

any_oparg_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ResumeOparg {
        AtFuncStart = 0,
        AfterYield = 1,
        AfterYieldFrom = 2,
        AfterAwait = 3,
    }
);

any_oparg_enum!(
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
);

any_oparg_enum!(
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
);
