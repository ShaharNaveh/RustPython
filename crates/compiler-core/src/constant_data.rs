use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};
use core::{
    fmt, hash, mem,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use malachite_bigint::BigInt;
use num_complex::Complex64;
use num_traits::Zero;

use rustpython_wtf8::{Wtf8, Wtf8Buf};

use crate::bytecode::{CodeObject, oparg};

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
    Tuple {
        elements: Vec<Self>,
    },
    Integer {
        value: BigInt,
    },
    Float {
        value: f64,
    },
    Complex {
        value: Complex64,
    },
    Boolean {
        value: bool,
    },
    Str {
        value: Wtf8Buf,
    },
    Bytes {
        value: Vec<u8>,
    },
    Code {
        code: Box<CodeObject>,
    },
    /// Constant slice(start, stop, step)
    Slice {
        elements: Box<[Self; 3]>,
    },
    Frozenset {
        elements: Vec<Self>,
    },
    None,
    Ellipsis,
}

impl ConstantData {
    /// Whether or not python would return True/False for the given constant data.
    ///
    /// ```py
    /// bool(0) # False
    /// bool(1) # True
    /// bool([]) # False
    /// bool(...) # True
    /// ```
    #[must_use]
    pub fn truthiness(&self) -> bool {
        match self {
            Self::Tuple { elements } | Self::Frozenset { elements } => !elements.is_empty(),
            Self::Integer { value } => !value.is_zero(),
            Self::Float { value } => *value != 0.0,
            Self::Complex { value } => value.re != 0.0 || value.im != 0.0,
            Self::Boolean { value } => *value,
            Self::Str { value } => !value.is_empty(),
            Self::Bytes { value } => !value.is_empty(),
            Self::Code { .. } | Self::Slice { .. } | Self::Ellipsis => true,
            Self::None => false,
        }
    }
}

impl PartialEq for ConstantData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer { value: a }, Self::Integer { value: b }) => a == b,
            (Self::Float { value: a }, Self::Float { value: b }) => a.to_bits() == b.to_bits(),
            (Self::Complex { value: a }, Self::Complex { value: b }) => {
                a.re.to_bits() == b.re.to_bits() && a.im.to_bits() == b.im.to_bits()
            }
            (Self::Boolean { value: a }, Self::Boolean { value: b }) => a == b,
            (Self::Str { value: a }, Self::Str { value: b }) => a == b,
            (Self::Bytes { value: a }, Self::Bytes { value: b }) => a == b,
            (Self::Code { code: a }, Self::Code { code: b }) => {
                core::ptr::eq(a.as_ref(), b.as_ref())
            }
            (Self::Tuple { elements: a }, Self::Tuple { elements: b }) => a == b,
            (Self::Slice { elements: a }, Self::Slice { elements: b }) => a == b,
            (Self::Frozenset { elements: a }, Self::Frozenset { elements: b }) => a == b,
            (Self::None, Self::None) => true,
            (Self::Ellipsis, Self::Ellipsis) => true,
            _ => false,
        }
    }
}

impl Eq for ConstantData {}

impl hash::Hash for ConstantData {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);

        match self {
            Self::Integer { value } => value.hash(state),
            Self::Float { value } => value.to_bits().hash(state),
            Self::Complex { value } => {
                value.re.to_bits().hash(state);
                value.im.to_bits().hash(state);
            }
            Self::Boolean { value } => value.hash(state),
            Self::Str { value } => value.hash(state),
            Self::Bytes { value } => value.hash(state),
            Self::Code { code } => core::ptr::hash(code.as_ref(), state),
            Self::Tuple { elements } => elements.hash(state),
            Self::Slice { elements } => elements.hash(state),
            Self::Frozenset { elements } => elements.hash(state),
            Self::None => {}
            Self::Ellipsis => {}
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
    Slice { elements: &'a [C; 3] },
    Frozenset { elements: &'a [C] },
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
            BorrowedConstant::Slice { elements } => {
                write!(f, "slice(")?;
                elements[0].borrow_constant().fmt_display(f)?;
                write!(f, ", ")?;
                elements[1].borrow_constant().fmt_display(f)?;
                write!(f, ", ")?;
                elements[2].borrow_constant().fmt_display(f)?;
                write!(f, ")")
            }
            BorrowedConstant::Frozenset { elements } => {
                write!(f, "frozenset({{")?;
                let mut first = true;
                for c in *elements {
                    if first {
                        first = false
                    } else {
                        write!(f, ", ")?;
                    }
                    c.borrow_constant().fmt_display(f)?;
                }
                write!(f, "}})")
            }
            BorrowedConstant::None => write!(f, "None"),
            BorrowedConstant::Ellipsis => write!(f, "..."),
        }
    }

    #[must_use]
    pub fn to_owned(self) -> ConstantData {
        match self {
            BorrowedConstant::Integer { value } => ConstantData::Integer {
                value: value.clone(),
            },
            BorrowedConstant::Float { value } => ConstantData::Float { value },
            BorrowedConstant::Complex { value } => ConstantData::Complex { value },
            BorrowedConstant::Boolean { value } => ConstantData::Boolean { value },
            BorrowedConstant::Str { value } => ConstantData::Str {
                value: value.to_owned(),
            },
            BorrowedConstant::Bytes { value } => ConstantData::Bytes {
                value: value.to_owned(),
            },
            BorrowedConstant::Code { code } => ConstantData::Code {
                code: Box::new(code.map_clone_bag(&BasicBag)),
            },
            BorrowedConstant::Tuple { elements } => ConstantData::Tuple {
                elements: elements
                    .iter()
                    .map(|c| c.borrow_constant().to_owned())
                    .collect(),
            },
            BorrowedConstant::Slice { elements } => ConstantData::Slice {
                elements: Box::new(elements.each_ref().map(|c| c.borrow_constant().to_owned())),
            },
            BorrowedConstant::Frozenset { elements } => ConstantData::Frozenset {
                elements: elements
                    .iter()
                    .map(|c| c.borrow_constant().to_owned())
                    .collect(),
            },
            BorrowedConstant::None => ConstantData::None,
            BorrowedConstant::Ellipsis => ConstantData::Ellipsis,
        }
    }
}

pub trait Constant: Sized + Clone {
    type Name: AsRef<str>;

    /// Transforms the given Constant to a BorrowedConstant
    fn borrow_constant(&self) -> BorrowedConstant<'_, Self>;
}

impl Constant for ConstantData {
    type Name = String;

    fn borrow_constant(&self) -> BorrowedConstant<'_, Self> {
        match self {
            Self::Integer { value } => BorrowedConstant::Integer { value },
            Self::Float { value } => BorrowedConstant::Float { value: *value },
            Self::Complex { value } => BorrowedConstant::Complex { value: *value },
            Self::Boolean { value } => BorrowedConstant::Boolean { value: *value },
            Self::Str { value } => BorrowedConstant::Str { value },
            Self::Bytes { value } => BorrowedConstant::Bytes { value },
            Self::Code { code } => BorrowedConstant::Code { code },
            Self::Tuple { elements } => BorrowedConstant::Tuple { elements },
            Self::Slice { elements } => BorrowedConstant::Slice { elements },
            Self::Frozenset { elements } => BorrowedConstant::Frozenset { elements },
            Self::None => BorrowedConstant::None,
            Self::Ellipsis => BorrowedConstant::Ellipsis,
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

#[derive(Clone)]
pub struct Constants<C: Constant>(Box<[C]>);

impl<C: Constant> Deref for Constants<C> {
    type Target = [C];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: Constant> DerefMut for Constants<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<C: Constant> Index<oparg::ConstIdx> for Constants<C> {
    type Output = C;

    fn index(&self, consti: oparg::ConstIdx) -> &Self::Output {
        &self.0[consti.as_usize()]
    }
}

impl<C: Constant> IndexMut<oparg::ConstIdx> for Constants<C> {
    fn index_mut(&mut self, consti: oparg::ConstIdx) -> &mut Self::Output {
        &mut self.0[consti.as_usize()]
    }
}

impl<C: Constant> FromIterator<C> for Constants<C> {
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

// TODO: Newtype "CodeObject.varnames". Make sure only `oparg:VarNum` can be used as index
impl<T> Index<oparg::VarNum> for [T] {
    type Output = T;

    fn index(&self, var_num: oparg::VarNum) -> &Self::Output {
        &self[var_num.as_usize()]
    }
}

// TODO: Newtype "CodeObject.varnames". Make sure only `oparg:VarNum` can be used as index
impl<T> IndexMut<oparg::VarNum> for [T] {
    fn index_mut(&mut self, var_num: oparg::VarNum) -> &mut Self::Output {
        &mut self[var_num.as_usize()]
    }
}
