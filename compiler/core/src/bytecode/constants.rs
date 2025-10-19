use super::CodeObject;
use malachite_bigint::BigInt;
use num_complex::Complex64;
use rustpython_wtf8::{Wtf8, Wtf8Buf};
use std::{fmt, hash, mem};

pub trait Constant: Sized {
    type Name: AsRef<str>;

    /// Transforms the given Constant to a BorrowedConstant
    fn borrow_constant(&self) -> BorrowedConstant<'_, Self>;
}

impl Constant for ConstantData {
    type Name = String;

    fn borrow_constant(&self) -> BorrowedConstant<'_, Self> {
        use BorrowedConstant::*;

        match self {
            Self::Integer { value } => Integer { value },
            Self::Float { value } => Float { value: *value },
            Self::Complex { value } => Complex { value: *value },
            Self::Boolean { value } => Boolean { value: *value },
            Self::Str { value } => Str { value },
            Self::Bytes { value } => Bytes { value },
            Self::Code { code } => Code { code },
            Self::Tuple { elements } => Tuple { elements },
            Self::None => None,
            Self::Ellipsis => Ellipsis,
        }
    }
}

/// A Constant Bag
pub trait ConstantBag: Sized + Copy {
    type Constant: Constant;

    fn make_constant<C: Constant>(&self, constant: BorrowedConstant<'_, C>) -> Self::Constant;

    fn make_int(&self, value: BigInt) -> Self::Constant;

    fn make_tuple(&self, elements: impl Iterator<Item = Self::Constant>) -> Self::Constant;

    fn make_code(&self, code: CodeObject<Self::Constant>) -> Self::Constant;

    fn make_name(&self, name: &str) -> <Self::Constant as Constant>::Name;
}

pub trait AsBag {
    type Bag: ConstantBag;

    #[allow(clippy::wrong_self_convention)]
    fn as_bag(self) -> Self::Bag;
}

impl<Bag: ConstantBag> AsBag for Bag {
    type Bag = Self;

    fn as_bag(self) -> Self {
        self
    }
}

#[derive(Clone, Copy)]
pub struct BasicBag;

impl ConstantBag for BasicBag {
    type Constant = ConstantData;

    fn make_constant<C: Constant>(&self, constant: BorrowedConstant<'_, C>) -> Self::Constant {
        constant.to_owned()
    }

    fn make_int(&self, value: BigInt) -> Self::Constant {
        ConstantData::Integer { value }
    }

    fn make_tuple(&self, elements: impl Iterator<Item = Self::Constant>) -> Self::Constant {
        ConstantData::Tuple {
            elements: elements.collect(),
        }
    }

    fn make_code(&self, code: CodeObject<Self::Constant>) -> Self::Constant {
        ConstantData::Code {
            code: Box::new(code),
        }
    }

    fn make_name(&self, name: &str) -> <Self::Constant as Constant>::Name {
        name.to_owned()
    }
}

/// A Constant (which usually encapsulates data within it)
///
/// # Examples
/// ```
/// use rustpython_compiler_core::bytecode::ConstantData;
/// let a = ConstantData::Float {value: 120f64};
/// let b = ConstantData::Boolean {value: false};
/// assert_ne!(a, b);
/// ```
#[derive(Debug, Clone)]
pub enum ConstantData {
    Tuple { elements: Vec<ConstantData> },
    Integer { value: BigInt },
    Float { value: f64 },
    Complex { value: Complex64 },
    Boolean { value: bool },
    Str { value: Wtf8Buf },
    Bytes { value: Vec<u8> },
    Code { code: Box<CodeObject> },
    None,
    Ellipsis,
}

impl PartialEq for ConstantData {
    fn eq(&self, other: &Self) -> bool {
        use ConstantData::*;

        match (self, other) {
            (Integer { value: a }, Integer { value: b }) => a == b,
            // we want to compare floats *by actual value* - if we have the *exact same* float
            // already in a constant cache, we want to use that
            (Float { value: a }, Float { value: b }) => a.to_bits() == b.to_bits(),
            (Complex { value: a }, Complex { value: b }) => {
                a.re.to_bits() == b.re.to_bits() && a.im.to_bits() == b.im.to_bits()
            }
            (Boolean { value: a }, Boolean { value: b }) => a == b,
            (Str { value: a }, Str { value: b }) => a == b,
            (Bytes { value: a }, Bytes { value: b }) => a == b,
            (Code { code: a }, Code { code: b }) => std::ptr::eq(a.as_ref(), b.as_ref()),
            (Tuple { elements: a }, Tuple { elements: b }) => a == b,
            (None, None) => true,
            (Ellipsis, Ellipsis) => true,
            _ => false,
        }
    }
}

impl Eq for ConstantData {}

impl hash::Hash for ConstantData {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        use ConstantData::*;

        mem::discriminant(self).hash(state);
        match self {
            Integer { value } => value.hash(state),
            Float { value } => value.to_bits().hash(state),
            Complex { value } => {
                value.re.to_bits().hash(state);
                value.im.to_bits().hash(state);
            }
            Boolean { value } => value.hash(state),
            Str { value } => value.hash(state),
            Bytes { value } => value.hash(state),
            Code { code } => std::ptr::hash(code.as_ref(), state),
            Tuple { elements } => elements.hash(state),
            None => {}
            Ellipsis => {}
        }
    }
}

/// A borrowed Constant
pub enum BorrowedConstant<'a, C: Constant> {
    Integer { value: &'a BigInt },
    Float { value: f64 },
    Complex { value: Complex64 },
    Boolean { value: bool },
    Str { value: &'a Wtf8 },
    Bytes { value: &'a [u8] },
    Code { code: &'a CodeObject<C> },
    Tuple { elements: &'a [C] },
    None,
    Ellipsis,
}

impl<C: Constant> Copy for BorrowedConstant<'_, C> {}

impl<C: Constant> Clone for BorrowedConstant<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: Constant> BorrowedConstant<'_, C> {
    pub fn fmt_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowedConstant::Integer { value } => write!(f, "{value}"),
            BorrowedConstant::Float { value } => write!(f, "{value}"),
            BorrowedConstant::Complex { value } => write!(f, "{value}"),
            BorrowedConstant::Boolean { value } => {
                write!(f, "{}", if *value { "True" } else { "False" })
            }
            BorrowedConstant::Str { value } => write!(f, "{value:?}"),
            BorrowedConstant::Bytes { value } => write!(f, r#"b"{}""#, value.escape_ascii()),
            BorrowedConstant::Code { code } => write!(f, "{code:?}"),
            BorrowedConstant::Tuple { elements } => {
                write!(f, "(")?;
                let mut first = true;
                for c in *elements {
                    if first {
                        first = false
                    } else {
                        write!(f, ", ")?;
                    }
                    c.borrow_constant().fmt_display(f)?;
                }
                write!(f, ")")
            }
            BorrowedConstant::None => write!(f, "None"),
            BorrowedConstant::Ellipsis => write!(f, "..."),
        }
    }

    pub fn to_owned(self) -> ConstantData {
        use ConstantData::*;

        match self {
            BorrowedConstant::Integer { value } => Integer {
                value: value.clone(),
            },
            BorrowedConstant::Float { value } => Float { value },
            BorrowedConstant::Complex { value } => Complex { value },
            BorrowedConstant::Boolean { value } => Boolean { value },
            BorrowedConstant::Str { value } => Str {
                value: value.to_owned(),
            },
            BorrowedConstant::Bytes { value } => Bytes {
                value: value.to_owned(),
            },
            BorrowedConstant::Code { code } => Code {
                code: Box::new(code.map_clone_bag(&BasicBag)),
            },
            BorrowedConstant::Tuple { elements } => Tuple {
                elements: elements
                    .iter()
                    .map(|c| c.borrow_constant().to_owned())
                    .collect(),
            },
            BorrowedConstant::None => None,
            BorrowedConstant::Ellipsis => Ellipsis,
        }
    }
}
