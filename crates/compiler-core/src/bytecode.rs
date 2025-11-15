//! Implement python as a virtual machine with bytecode. This module
//! implements bytecode structure.

mod code;
mod constant;
mod instructions;
mod oparg;
mod oparg_types;

pub use code::{CodeFlags, CodeObject, CodeUnit, CodeUnits};
pub use constant::{AsBag, BorrowedConstant, Constant, ConstantBag, ConstantData};
pub use instructions::{PseudoInstruction, RealInstruction};
pub use oparg::{AnyOparg, Oparg, OpargByte, OpargState, OpargType};
pub use oparg_types::*;

/*
use itertools::Itertools;
use malachite_bigint::BigInt;
use num_complex::Complex64;
use rustpython_wtf8::{Wtf8, Wtf8Buf};
use std::{collections::BTreeSet, fmt, hash, marker::PhantomData, mem, ops::Deref};
*/
