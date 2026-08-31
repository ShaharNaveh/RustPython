use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};
use core::{
    fmt,
    hash::{Hash, Hasher},
    mem,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr,
};

use malachite_bigint::BigInt;
use num_complex::Complex64;
use num_traits::Zero;

use rustpython_wtf8::{Wtf8, Wtf8Buf};

use crate::bytecode::{CodeObject, oparg};

pub trait Constant: Sized + Clone {
    type Name: AsRef<str>;

    /// Does [`self`] contains a NaN.
    fn contains_nan(&self) -> bool;

    /// Transforms the given Constant to a BorrowedConstant
    fn borrow_constant(&self) -> BorrowedConstant<'_, Self>;

    /// Whether or not python would return True/False for the given constant data.
    ///
    /// ```py
    /// bool(0) # False
    /// bool(1) # True
    /// bool([]) # False
    /// bool(...) # True
    /// ```
    fn truthiness(&self) -> bool;
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
#[derive(Clone, Debug)]
pub enum ConstantData {
    Tuple(Tuple),
    Integer(Integer),
    Float(f64),
    Complex(Complex),
    Boolean(bool),
    Str(Wtf8Buf),
    Bytes(Bytes),
    Code(Box<CodeObject>),
    Slice(Box<Slice>),
    Frozenset(Frozenset),
    None,
    Ellipsis,
}

impl PartialEq for ConstantData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Tuple(a), Self::Tuple(b)) => a == b,
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a.to_bits() == b.to_bits(),
            (Self::Complex(a), Self::Complex(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Str(a), Self::Str(b)) => a == b,
            (Self::Bytes(a), Self::Bytes(b)) => a == b,
            (Self::Code(a), Self::Code(b)) => ptr::eq(a.as_ref(), b.as_ref()),
            (Self::Slice(a), Self::Slice(b)) => a == b,
            (Self::Frozenset(a), Self::Frozenset(b)) => a == b,
            (Self::None, Self::None) => true,
            (Self::Ellipsis, Self::Ellipsis) => true,
            (_, _) => false,
        }
    }
}

impl Eq for ConstantData {}

impl Hash for ConstantData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);

        match self {
            Self::Boolean(v) => v.hash(state),
            Self::Bytes(bytes) => bytes.hash(state),
            Self::Code(code) => core::ptr::hash(code.as_ref(), state),
            Self::Complex(complex) => complex.hash(state),
            Self::Float(value) => value.to_bits().hash(state),
            Self::Frozenset(frozenset) => frozenset.hash(state),
            Self::Integer(int) => int.hash(state),
            Self::Slice(slice) => slice.hash(state),
            Self::Str(s) => s.hash(state),
            Self::Tuple(tup) => tup.hash(state),
            Self::Ellipsis | Self::None => {}
        }
    }
}

/// A borrowed [`Constant`].
pub enum BorrowedConstant<'a, C: Constant> {
    Integer(&'a Integer),
    Float(f64),
    Complex(Complex),
    Boolean(bool),
    Str(&'a Wtf8),
    Bytes(&'a BytesInner),
    Code(&'a CodeObject<C>),
    Tuple(&'a Tuple<C>),
    Slice(&'a Slice<C>),
    Frozenset(&'a Frozenset<C>),
    None,
    Ellipsis,
}

impl<C: Constant> Copy for BorrowedConstant<'_, C> {}

impl<C: Constant> Clone for BorrowedConstant<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: Constant + fmt::Debug> BorrowedConstant<'_, C> {
    pub fn fmt_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowedConstant::Integer(value) => write!(f, "{value}"),
            BorrowedConstant::Float(value) => write!(f, "{value}"),
            BorrowedConstant::Complex(value) => write!(f, "{value}"),
            BorrowedConstant::Boolean(value) => {
                write!(f, "{}", if *value { "True" } else { "False" })
            }
            BorrowedConstant::Str(value) => write!(f, "{value:?}"),
            BorrowedConstant::Bytes(value) => write!(f, r#"b"{}""#, value.escape_ascii()),
            BorrowedConstant::Code(code) => write!(f, "{code:?}"),
            BorrowedConstant::Tuple(elements) => {
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
            BorrowedConstant::Slice(slice) => {
                write!(f, "slice(")?;
                slice.start.borrow_constant().fmt_display(f)?;
                write!(f, ", ")?;
                slice.stop.borrow_constant().fmt_display(f)?;
                write!(f, ", ")?;
                slice.step.borrow_constant().fmt_display(f)?;
                write!(f, ")")
            }
            BorrowedConstant::Frozenset(elements) => {
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
            BorrowedConstant::Integer(value) => ConstantData::Integer(value.clone()),
            BorrowedConstant::Float(value) => ConstantData::Float(value),
            BorrowedConstant::Complex(value) => ConstantData::Complex(value),
            BorrowedConstant::Boolean(value) => ConstantData::Boolean(value),
            BorrowedConstant::Str(value) => ConstantData::Str(value.to_owned()),
            BorrowedConstant::Bytes(value) => ConstantData::Bytes(value.to_owned().into()),
            BorrowedConstant::Code(code) => {
                ConstantData::Code(code.map_clone_bag(&BasicBag).into())
            }
            BorrowedConstant::Tuple(elements) => ConstantData::Tuple(
                elements
                    .iter()
                    .map(|c| c.borrow_constant().to_owned())
                    .collect(),
            ),
            BorrowedConstant::Slice(slice) => ConstantData::Slice(Box::new(Slice {
                start: slice.start.borrow_constant().to_owned(),
                stop: slice.stop.borrow_constant().to_owned(),
                step: slice.step.borrow_constant().to_owned(),
            })),
            BorrowedConstant::Frozenset(elements) => ConstantData::Frozenset(
                elements
                    .0
                    .iter()
                    .map(|c| c.borrow_constant().to_owned())
                    .collect(),
            ),
            BorrowedConstant::None => ConstantData::None,
            BorrowedConstant::Ellipsis => ConstantData::Ellipsis,
        }
    }
}

impl Constant for ConstantData {
    type Name = String;

    fn borrow_constant(&self) -> BorrowedConstant<'_, Self> {
        match self {
            Self::Integer(value) => BorrowedConstant::Integer(value),
            Self::Float(value) => BorrowedConstant::Float(*value),
            Self::Complex(value) => BorrowedConstant::Complex(*value),
            Self::Boolean(value) => BorrowedConstant::Boolean(*value),
            Self::Str(value) => BorrowedConstant::Str(value),
            Self::Bytes(value) => BorrowedConstant::Bytes(value),
            Self::Code(code) => BorrowedConstant::Code(code),
            Self::Tuple(elements) => BorrowedConstant::Tuple(elements),
            Self::Slice(slice) => BorrowedConstant::Slice(slice),
            Self::Frozenset(elements) => BorrowedConstant::Frozenset(elements),
            Self::None => BorrowedConstant::None,
            Self::Ellipsis => BorrowedConstant::Ellipsis,
        }
    }

    fn contains_nan(&self) -> bool {
        todo!()
    }

    fn truthiness(&self) -> bool {
        match self {
            Self::Tuple(value) => value.truthiness(),
            Self::Frozenset(value) => value.truthiness(),
            Self::Integer(value) => value.truthiness(),
            Self::Float(value) => *value != 0.0,
            Self::Complex(value) => value.truthiness(),
            Self::Boolean(value) => *value,
            Self::Str(value) => value.is_empty(),
            Self::Bytes(value) => value.truthiness(),
            Self::Code(_) => true,
            Self::Slice(value) => value.truthiness(),
            Self::Ellipsis => true,
            Self::None => false,
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
        ConstantData::Integer(value.into())
    }

    fn make_tuple(&self, elements: impl Iterator<Item = Self::Constant>) -> Self::Constant {
        ConstantData::Tuple(elements.collect())
    }

    fn make_code(&self, code: CodeObject<Self::Constant>) -> Self::Constant {
        ConstantData::Code(code.into())
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

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Complex(Complex64);

impl From<Complex64> for Complex {
    fn from(value: Complex64) -> Self {
        Self(value)
    }
}

impl Constant for Complex {
    type Name = String;

    fn contains_nan(&self) -> bool {
        todo!()
    }

    fn borrow_constant(&self) -> BorrowedConstant<'_, Self> {
        todo!()
    }

    fn truthiness(&self) -> bool {
        self.re != 0.0 || self.im != 0.0
    }
}

impl Deref for Complex {
    type Target = Complex64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Complex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Eq for Complex {}

impl Hash for Complex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.re.to_bits().hash(state);
        self.im.to_bits().hash(state);
    }
}

pub type BytesInner = [u8];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Bytes(Box<BytesInner>);

impl Constant for Bytes {
    type Name = String;

    fn contains_nan(&self) -> bool {
        todo!()
    }

    fn borrow_constant(&self) -> BorrowedConstant<'_, Self> {
        todo!()
    }

    fn truthiness(&self) -> bool {
        !self.is_empty()
    }
}

impl Deref for Bytes {
    type Target = BytesInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value.into())
    }
}

/// Constant
/// ```py
/// slice(start, stop, step)
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Slice<C = ConstantData> {
    pub start: C,
    pub stop: C,
    pub step: C,
}

impl<C> Slice<C> {
    #[must_use]
    pub const fn new(start: C, stop: C, step: C) -> Self {
        Self { start, stop, step }
    }
}

impl Constant for Slice {
    type Name = String;

    fn contains_nan(&self) -> bool {
        self.start.contains_nan() || self.stop.contains_nan() || self.stop.contains_nan()
    }

    fn borrow_constant(&self) -> BorrowedConstant<'_, Self> {
        todo!()
    }

    fn truthiness(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Tuple<C = ConstantData>(Vec<C>);

impl<T> IntoIterator for Tuple<T> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Tuple<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T> FromIterator<T> for Tuple<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<T> From<Vec<T>> for Tuple<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl Constant for Tuple {
    type Name = String;

    fn contains_nan(&self) -> bool {
        self.iter().any(Constant::contains_nan)
    }

    fn borrow_constant(&self) -> BorrowedConstant<'_, Self> {
        todo!()
    }

    fn truthiness(&self) -> bool {
        !self.is_empty()
    }
}

impl<C> Deref for Tuple<C> {
    type Target = Vec<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for Tuple<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Integer(BigInt);

impl Constant for Integer {
    type Name = String;

    fn contains_nan(&self) -> bool {
        todo!()
    }

    fn borrow_constant(&self) -> BorrowedConstant<'_, Self> {
        todo!()
    }

    fn truthiness(&self) -> bool {
        !self.is_zero()
    }
}

impl From<BigInt> for Integer {
    fn from(value: BigInt) -> Self {
        Self(value)
    }
}

impl From<&Integer> for i32 {
    fn from(value: &Integer) -> Self {
        value.into()
    }
}

impl Deref for Integer {
    type Target = BigInt;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Frozenset<C = ConstantData>(Vec<C>);

impl<T> FromIterator<T> for Frozenset<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<T> From<Vec<T>> for Frozenset<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> IntoIterator for Frozenset<T> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Frozenset<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Constant for Frozenset {
    type Name = String;

    fn contains_nan(&self) -> bool {
        self.iter().any(Constant::contains_nan)
    }

    fn borrow_constant(&self) -> BorrowedConstant<'_, Self> {
        todo!()
    }

    fn truthiness(&self) -> bool {
        !self.is_empty()
    }
}

impl<C> Deref for Frozenset<C> {
    type Target = Vec<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for Frozenset<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
