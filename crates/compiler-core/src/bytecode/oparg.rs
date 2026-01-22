use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use core::fmt;

use crate::bytecode::{
    CodeUnit,
    opcode::{AnyOpcode, Opcode, PseudoOpcode},
};

#[derive(Clone, Copy)]
pub enum AnyOparg {
    BinaryOperator(BinaryOperator),
    BuildSliceArgCount(BuildSliceArgCount),
    ComparisonOperator(ComparisonOperator),
    ConvertValue(ConvertValue),
    IntrinsicFunction1(IntrinsicFunction1),
    IntrinsicFunction2(IntrinsicFunction2),
    Invert(Invert),
    Label(Label),
    MakeFunctionFlags(MakeFunctionFlags),
    NameIdx(NameIdx),
    RaiseKind(RaiseKind),
    /// Untyped.
    Raw(u32),
    ResumeType(ResumeType),
    SpecialMethod(SpecialMethod),
    StoreFastLoadFast(StoreFastLoadFast),
    UnpackExArgs(UnpackExArgs),
}

struct OpargConstructor;

impl OpargConstructor {
    const INVERT_FN: fn(u32) -> AnyOparg = |v| Invert::try_from(v).map(Into::into);
    const LABEL_FN: fn(u32) -> AnyOparg = |v| Ok(Label::from(v).into());
    const NAME_IDX_FN: fn(u32) -> AnyOparg = |v| Ok(NameIdx::from(v).into());
    const RAW_FN: fn(u32) -> AnyOparg = |v| Ok(v.into());

    /// FIXME: Add docs
    const fn new(opcode: AnyOpcode) -> Option<impl Fn(u32) -> Result<AnyOparg, MarshalError>> {
        match opcode {
            AnyOpcode::Real(op) => Self::from_opcode(op),
            AnyOpcode::Pseudo(op) => Self::from_pseudo_opcode(op),
        }
    }

    /// FIXME: Add docs
    const fn from_opcode(opcode: Opcode) -> Option<impl Fn(u32) -> Result<AnyOparg, MarshalError>> {
        Some(match opcode {
            Opcode::Cache => return None,
            Opcode::BinarySlice => return None,
            Opcode::BuildTemplate => return None,
            Opcode::BinaryOpInplaceAddUnicode => return None,
            Opcode::CallFunctionEx => return None,
            Opcode::CheckEgMatch => return None,
            Opcode::CheckExcMatch => return None,
            Opcode::CleanupThrow => return None,
            Opcode::DeleteSubscr => return None,
            Opcode::EndFor => return None,
            Opcode::EndSend => return None,
            Opcode::ExitInitCheck => return None,
            Opcode::FormatSimple => return None,
            Opcode::FormatWithSpec => return None,
            Opcode::GetAIter => return None,
            Opcode::GetANext => return None,
            Opcode::GetIter => return None,
            Opcode::Reserved => return None,
            Opcode::GetLen => return None,
            Opcode::GetYieldFromIter => return None,
            Opcode::InterpreterExit => return None,
            Opcode::LoadBuildClass => return None,
            Opcode::LoadLocals => return None,
            Opcode::MakeFunction => return None,
            Opcode::MatchKeys => return None,
            Opcode::MatchMapping => return None,
            Opcode::MatchSequence => return None,
            Opcode::Nop => return None,
            Opcode::NotTaken => return None,
            Opcode::PopExcept => return None,
            Opcode::PopIter => return None,
            Opcode::PopTop => return None,
            Opcode::PushExcInfo => return None,
            Opcode::PushNull => return None,
            Opcode::ReturnGenerator => return None,
            Opcode::ReturnValue => return None,
            Opcode::SetupAnnotations => return None,
            Opcode::StoreSlice => return None,
            Opcode::StoreSubscr => return None,
            Opcode::ToBool => return None,
            Opcode::UnaryInvert => return None,
            Opcode::UnaryNegative => return None,
            Opcode::UnaryNot => return None,
            Opcode::WithExceptStart => return None,
            Opcode::BinaryOp => |v| BinaryOperator::try_from(v).map(Into::into),
            Opcode::BuildInterpolation => Self::RAW_FN,
            Opcode::BuildList => Self::RAW_FN,
            Opcode::BuildMap => Self::RAW_FN,
            Opcode::BuildSet => Self::RAW_FN,
            Opcode::BuildSlice => |v| BuildSliceArgCount::try_from(v).map(Into::into),
            Opcode::BuildTuple => Self::RAW_FN,
            Opcode::Call => Self::RAW_FN,
            Opcode::CallIntrinsic1 => |v| CallIntrinsic1::try_from(v).map(Into::into),
            Opcode::CallIntrinsic2 => |v| CallIntrinsic2::try_from(v).map(Into::into),
            Opcode::CallKw => Self::RAW_FN,
            Opcode::CompareOp => |v| ComparisonOperator::try_from(v).map(Into::into),
            Opcode::ContainsOp => Self::INVERT_FN,
            Opcode::ConvertValue => |v| ConvertValueOparg::try_from(v).map(Into::into),
            Opcode::Copy => Self::RAW_FN,
            Opcode::CopyFreeVars => Self::RAW_FN,
            Opcode::DeleteAttr => Self::NAME_IDX_FN,
            Opcode::DeleteDeref => Self::NAME_IDX_FN,
            Opcode::DeleteFast => Self::NAME_IDX_FN,
            Opcode::DeleteGlobal => Self::NAME_IDX_FN,
            Opcode::DeleteName => Self::NAME_IDX_FN,
            Opcode::DictMerge => Self::RAW_FN,
            Opcode::DictUpdate => Self::RAW_FN,
            Opcode::EndAsyncFor => return None,
            Opcode::ExtendedArg => return None,
            Opcode::ForIter => Self::LABEL_FN,
            Opcode::GetAwaitable => return None, // TODO: This should have an oparg
            Opcode::ImportFrom => Self::RAW_FN,
            Opcode::ImportName => Self::RAW_FN,
            Opcode::IsOp => Self::INVERT_FN,
            Opcode::JumpBackward => Self::LABEL_FN,
            Opcode::JumpBackwardNoInterrupt => Self::LABEL_FN,
            Opcode::JumpForward => Self::LABEL_FN,
            Opcode::ListAppend => Self::RAW_FN,
            Opcode::ListExtend => Self::RAW_FN,
            Opcode::LoadAttr => Self::NAME_IDX_FN,
            Opcode::LoadCommonConstant => Self::RAW_FN,
            Opcode::LoadConst => Self::RAW_FN,
            Opcode::LoadDeref => Self::NAME_IDX_FN,
            Opcode::LoadFast => Self::NAME_IDX_FN,
            Opcode::LoadFastAndClear => Self::NAME_IDX_FN,
            Opcode::LoadFastBorrow => Self::NAME_IDX_FN,
            Opcode::LoadFastBorrowLoadFastBorrow => Self::RAW_FN,
            Opcode::LoadFastCheck => Self::NAME_IDX_FN,
            Opcode::LoadFastLoadFast => Self::RAW_FN,
            Opcode::LoadFromDictOrDeref => Self::NAME_IDX_FN,
            Opcode::LoadFromDictOrGlobals => Self::NAME_IDX_FN,
            Opcode::LoadGlobal => Self::NAME_IDX_FN,
            Opcode::LoadName => Self::NAME_IDX_FN,
            Opcode::LoadSmallInt => Self::RAW_FN,
            Opcode::LoadSpecial => |v| SpecialMethod::try_from(v).map(Into::into),
            Opcode::LoadSuperAttr => Self::RAW_FN,
            Opcode::MakeCell => Self::NAME_IDX_FN,
            Opcode::MapAdd => Self::RAW_FN,
            Opcode::MatchClass => Self::RAW_FN,
            Opcode::PopJumpIfFalse => Self::LABEL_FN,
            Opcode::PopJumpIfNone => Self::LABEL_FN,
            Opcode::PopJumpIfNotNone => Self::LABEL_FN,
            Opcode::PopJumpIfTrue => Self::LABEL_FN,
            Opcode::RaiseVarargs => |v| RaiseKind::try_from(v).map(Into::into),
            Opcode::Reraise => Self::RAW_FN,
            Opcode::Send => Self::LABEL_FN,
            Opcode::SetAdd => Self::RAW_FN,
            Opcode::SetFunctionAttribute => |v| MakeFunctionFlags::try_from(v).map(Into::into),
            Opcode::SetUpdate => Self::RAW_FN,
            Opcode::StoreAttr => Self::NAME_IDX_FN,
            Opcode::StoreDeref => Self::NAME_IDX_FN,
            Opcode::StoreFast => Self::NAME_IDX_FN,
            Opcode::StoreFastLoadFast => |v| Ok(StoreFastLoadFast::from(v).into()),
            Opcode::StoreFastStoreFast => Self::RAW_FN,
            Opcode::StoreGlobal => Self::NAME_IDX_FN,
            Opcode::StoreName => Self::NAME_IDX_FN,
            Opcode::Swap => Self::RAW_FN,
            Opcode::UnpackEx => |v| Ok(UnpackExArgs::from(v).into()),
            Opcode::UnpackSequence => Self::RAW_FN,
            Opcode::YieldValue => Self::RAW_FN,
            Opcode::Resume => Self::RAW_FN,
            // Placeholders
            Opcode::BinaryOpAddFloat => return None,
            Opcode::BinaryOpAddInt => return None,
            Opcode::BinaryOpAddUnicode => return None,
            Opcode::BinaryOpExtend => return None,
            Opcode::BinaryOpMultiplyFloat => return None,
            Opcode::BinaryOpMultiplyInt => return None,
            Opcode::BinarySubscrDict => return None,
            Opcode::BinarySubscrGetitem => return None,
            Opcode::BinarySubscrListInt => return None,
            Opcode::BinarySubscrListSlice => return None,
            Opcode::BinarySubscrStrInt => return None,
            Opcode::BinarySubscrTupleInt => return None,
            Opcode::BinaryOpSubtractFloat => return None,
            Opcode::BinaryOpSubtractInt => return None,
            Opcode::CallAllocAndEnterInit => return None,
            Opcode::CallBoundMethodExactArgs => return None,
            Opcode::CallBoundMethodGeneral => return None,
            Opcode::CallBuiltinClass => return None,
            Opcode::CallBuiltinFast => return None,
            Opcode::CallBuiltinFastWithKeywords => return None,
            Opcode::CallBuiltinO => return None,
            Opcode::CallIsinstance => return None,
            Opcode::CallKwBoundMethod => return None,
            Opcode::CallKwNonPy => return None,
            Opcode::CallKwPy => return None,
            Opcode::CallLen => return None,
            Opcode::CallListAppend => return None,
            Opcode::CallMethodDescriptorFast => return None,
            Opcode::CallMethodDescriptorFastWithKeywords => return None,
            Opcode::CallMethodDescriptorNoargs => return None,
            Opcode::CallMethodDescriptorO => return None,
            Opcode::CallNonPyGeneral => return None,
            Opcode::CallPyExactArgs => return None,
            Opcode::CallPyGeneral => return None,
            Opcode::CallStr1 => return None,
            Opcode::CallTuple1 => return None,
            Opcode::CallType1 => return None,
            Opcode::CompareOpFloat => return None,
            Opcode::CompareOpInt => return None,
            Opcode::CompareOpStr => return None,
            Opcode::ContainsOpDict => return None,
            Opcode::ContainsOpSet => return None,
            Opcode::ForIterGen => return None,
            Opcode::ForIterList => return None,
            Opcode::ForIterRange => return None,
            Opcode::ForIterTuple => return None,
            Opcode::JumpBackwardJit => return None,
            Opcode::JumpBackwardNoJit => return None,
            Opcode::LoadAttrClass => return None,
            Opcode::LoadAttrClassWithMetaclassCheck => return None,
            Opcode::LoadAttrGetattributeOverridden => return None,
            Opcode::LoadAttrInstanceValue => return None,
            Opcode::LoadAttrMethodLazyDict => return None,
            Opcode::LoadAttrMethodNoDict => return None,
            Opcode::LoadAttrMethodWithValues => return None,
            Opcode::LoadAttrModule => return None,
            Opcode::LoadAttrNondescriptorNoDict => return None,
            Opcode::LoadAttrNondescriptorWithValues => return None,
            Opcode::LoadAttrProperty => return None,
            Opcode::LoadAttrSlot => return None,
            Opcode::LoadAttrWithHint => return None,
            Opcode::LoadConstImmortal => return None,
            Opcode::LoadConstMortal => return None,
            Opcode::LoadGlobalBuiltin => return None,
            Opcode::LoadGlobalModule => return None,
            Opcode::LoadSuperAttrAttr => return None,
            Opcode::LoadSuperAttrMethod => return None,
            Opcode::ResumeCheck => return None,
            Opcode::SendGen => return None,
            Opcode::StoreAttrInstanceValue => return None,
            Opcode::StoreAttrSlot => return None,
            Opcode::StoreAttrWithHint => return None,
            Opcode::StoreSubscrDict => return None,
            Opcode::StoreSubscrListInt => return None,
            Opcode::ToBoolAlwaysTrue => return None,
            Opcode::ToBoolBool => return None,
            Opcode::ToBoolInt => return None,
            Opcode::ToBoolList => return None,
            Opcode::ToBoolNone => return None,
            Opcode::ToBoolStr => return None,
            Opcode::UnpackSequenceList => return None,
            Opcode::UnpackSequenceTuple => return None,
            Opcode::UnpackSequenceTwoTuple => return None,
            Opcode::InstrumentedEndFor => return None,
            Opcode::InstrumentedPopIter => return None,
            Opcode::InstrumentedEndSend => return None,
            Opcode::InstrumentedForIter => return None,
            Opcode::InstrumentedInstruction => return None,
            Opcode::InstrumentedJumpForward => return None,
            Opcode::InstrumentedNotTaken => return None,
            Opcode::InstrumentedPopJumpIfTrue => return None,
            Opcode::InstrumentedPopJumpIfFalse => return None,
            Opcode::InstrumentedPopJumpIfNone => return None,
            Opcode::InstrumentedPopJumpIfNotNone => return None,
            Opcode::InstrumentedResume => return None,
            Opcode::InstrumentedReturnValue => return None,
            Opcode::InstrumentedYieldValue => return None,
            Opcode::InstrumentedEndAsyncFor => return None,
            Opcode::InstrumentedLoadSuperAttr => return None,
            Opcode::InstrumentedCall => return None,
            Opcode::InstrumentedCallKw => return None,
            Opcode::InstrumentedCallFunctionEx => return None,
            Opcode::InstrumentedJumpBackward => return None,
            Opcode::InstrumentedLine => return None,
            Opcode::EnterExecutor => return None,
        })
    }

    /// FIXME: Add docs
    const fn from_pseudo_opcode(
        opcode: PseudoOpcode,
    ) -> Option<impl Fn(u32) -> Result<AnyOparg, MarshalError>> {
        Some(match opcode {
            PseudoOpcode::AnnotationsPlaceholder => return None,
            PseudoOpcode::Jump => Self::LABEL_FN,
            PseudoOpcode::JumpIfFalse => Self::LABEL_FN,
            PseudoOpcode::JumpIfTrue => Self::LABEL_FN,
            PseudoOpcode::JumpNoInterrupt => Self::LABEL_FN,
            PseudoOpcode::LoadClosure => Self::NAME_IDX_FN,
            PseudoOpcode::PopBlock => return None,
            PseudoOpcode::SetupCleanup => return None,
            PseudoOpcode::SetupFinally => return None,
            PseudoOpcode::SetupWith => return None,
            // RustPython-specific pseudo instructions.
            PseudoOpcode::LoadAttrMethod => Self::NAME_IDX_FN,
            PseudoOpcode::LoadSuperMethod => Self::NAME_IDX_FN,
            PseudoOpcode::LoadZeroSuperAttr => Self::NAME_IDX_FN,
            PseudoOpcode::LoadZeroSuperMethod => Self::NAME_IDX_FN,
        })
    }
}

/// FIXME: Add docs
#[derive(Clone, Copy)]
pub struct AnyOpargBuilder {
    opcode: AnyOpcode,
    oparg: Option<u32>,
}

impl AnyOpargBuilder {
    /// FIXME: Add docs
    pub const fn new(opcode: AnyOpcode) -> Self {
        Self { opcode, None }
    }

    /// FIXME: Add docs
    pub const fn oparg(mut self, oparg: u32) -> Self {
        self.oparg = Some(oparg);
        self
    }

    /// FIXME: Add docs
    pub fn build(self) -> Result<Option<AnyOparg>, MarshalError> {
        Ok(match OpargConstructor::new(self.opcode) {
            Some(f) => match self.oparg {
                Some(v) => f(v),
                None => Err(MarshalError::InvalidBytecode),
            },
            None => None,
        })
    }
}

impl AnyOparg {
    /// FIXME: Add docs
    pub const fn builder(opcode: AnyOpcode) -> AnyOpargBuilder {
        AnyOpargBuilder::new(opcode)
    }
}

#[derive(Clone, Copy)]
pub struct StoreFastLoadFast {
    store_idx: NameIdx,
    load_idx: NameIdx,
}

impl From<u32> for StoreFastLoadFast {
    fn from(value: u32) -> Self {
        todo!()
    }
}

/// Opcode argument that may be extended by a prior [`Opcode::ExtendedArg`].
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct OpArgByte(pub u8);

impl OpArgByte {
    pub const NULL: Self = Self(0);
}

impl From<u8> for OpArgByte {
    fn from(raw: u8) -> Self {
        Self(raw)
    }
}

impl fmt::Debug for OpArgByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Full 32-bit op_arg, including any possible [`Opcode::ExtendedArg`] extension.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct OpArg(pub u32);

impl OpArg {
    pub const NULL: Self = Self(0);

    /// Returns how many CodeUnits a instruction with this op_arg will be encoded as
    #[inline]
    pub const fn instr_size(self) -> usize {
        (self.0 > 0xff) as usize + (self.0 > 0xff_ff) as usize + (self.0 > 0xff_ff_ff) as usize + 1
    }

    /// returns the arg split into any necessary ExtendedArg components (in big-endian order) and
    /// the arg for the real opcode itself
    #[inline(always)]
    pub fn split(self) -> (impl ExactSizeIterator<Item = OpArgByte>, OpArgByte) {
        let mut it = self
            .0
            .to_le_bytes()
            .map(OpArgByte)
            .into_iter()
            .take(self.instr_size());
        let lo = it.next().unwrap();
        (it.rev(), lo)
    }
}

impl From<u32> for OpArg {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

#[derive(Default, Copy, Clone)]
#[repr(transparent)]
pub struct OpArgState {
    state: u32,
}

impl OpArgState {
    #[inline(always)]
    pub fn get(&mut self, ins: CodeUnit) -> (Opcode, OpArg) {
        let arg = self.extend(ins.arg);
        if !matches!(ins.op, Opcode::ExtendedArg) {
            self.reset();
        }
        (ins.op, arg)
    }

    #[inline(always)]
    pub fn extend(&mut self, arg: OpArgByte) -> OpArg {
        self.state = (self.state << 8) | u32::from(arg.0);
        OpArg(self.state)
    }

    #[inline(always)]
    pub const fn reset(&mut self) {
        self.state = 0
    }
}

/// Oparg values for [`Opcode::ConvertValue`].
///
/// ## See also
///
/// - [CPython FVC_* flags](https://github.com/python/cpython/blob/8183fa5e3f78ca6ab862de7fb8b14f3d929421e0/Include/ceval.h#L129-L132)
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
pub enum ConvertValue {
    /// No conversion.
    ///
    /// ```python
    /// f"{x}"
    /// f"{x:4}"
    /// ```
    // Ruff `ConversionFlag::None` is `-1i8`, when its converted to `u8` its value is `u8::MAX`.
    #[num_enum(alternatives = [255])]
    None = 0,
    /// Converts by calling `str(<value>)`.
    ///
    /// ```python
    /// f"{x!s}"
    /// f"{x!s:2}"
    /// ```
    Str = 1,
    /// Converts by calling `repr(<value>)`.
    ///
    /// ```python
    /// f"{x!r}"
    /// f"{x!r:2}"
    /// ```
    Repr = 2,
    /// Converts by calling `ascii(<value>)`.
    ///
    /// ```python
    /// f"{x!a}"
    /// f"{x!a:2}"
    /// ```
    Ascii = 3,
}

impl fmt::Display for ConvertValueOparg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = match self {
            Self::Str => "1 (str)",
            Self::Repr => "2 (repr)",
            Self::Ascii => "3 (ascii)",
            // We should never reach this. `FVC_NONE` are being handled by `Opcode::FormatSimple`
            Self::None => "",
        };

        write!(f, "{out}")
    }
}

/// Resume type for the RESUME instruction
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
#[repr(u8)]
pub enum ResumeType {
    AtFuncStart = 0,
    AfterYield = 1,
    AfterYieldFrom = 2,
    AfterAwait = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct NameIdx(u32);

impl From<u32> for NameIdx {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<NameIdx> for u32 {
    fn from(value: NameIdx) -> Self {
        value.0
    }
}

impl fmt::Display for NameIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Label(u32);

impl From<u32> for Label {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Label> for u32 {
    fn from(value: Label) -> Self {
        value.0
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The kind of Raise that occurred.
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
#[repr(u8)]
pub enum RaiseKind {
    /// Bare `raise` statement with no arguments.
    /// Gets the current exception from VM state (topmost_exception).
    /// Maps to RAISE_VARARGS with oparg=0.
    BareRaise = 0,
    /// `raise exc` - exception is on the stack.
    /// Maps to RAISE_VARARGS with oparg=1.
    Raise = 1,
    /// `raise exc from cause` - exception and cause are on the stack.
    /// Maps to RAISE_VARARGS with oparg=2.
    RaiseCause = 2,
    /// Reraise exception from the stack top.
    /// Used in exception handler cleanup blocks (finally, except).
    /// Gets exception from stack, not from VM state.
    /// Maps to the RERAISE opcode.
    ReraiseFromStack = 3,
}

<<<<<<< HEAD
op_arg_enum!(
    /// Intrinsic function for CALL_INTRINSIC_1
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum IntrinsicFunction1 {
        // Invalid = 0,
        Print = 1,
        /// Import * operation
        ImportStar = 2,
        /// Convert StopIteration to RuntimeError in async context
        StopIterationError = 3,
        AsyncGenWrap = 4,
        UnaryPositive = 5,
        /// Convert list to tuple
        ListToTuple = 6,
        /// Type parameter related
        TypeVar = 7,
        ParamSpec = 8,
        TypeVarTuple = 9,
        /// Generic subscript for PEP 695
        SubscriptGeneric = 10,
        TypeAlias = 11,
    }
);
||||||| parent of 6598518de (Save)
op_arg_enum!(
    /// Intrinsic function for CALL_INTRINSIC_1
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum IntrinsicFunction1 {
        // Invalid = 0,
        Print = 1,
        /// Import * operation
        ImportStar = 2,
        // StopIterationError = 3,
        // AsyncGenWrap = 4,
        UnaryPositive = 5,
        /// Convert list to tuple
        ListToTuple = 6,
        /// Type parameter related
        TypeVar = 7,
        ParamSpec = 8,
        TypeVarTuple = 9,
        /// Generic subscript for PEP 695
        SubscriptGeneric = 10,
        TypeAlias = 11,
    }
);
=======
/// Intrinsic function for CALL_INTRINSIC_1
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
#[repr(u8)]
pub enum IntrinsicFunction1 {
    // Invalid = 0,
    Print = 1,
    /// Import * operation
    ImportStar = 2,
    // StopIterationError = 3,
    // AsyncGenWrap = 4,
    UnaryPositive = 5,
    /// Convert list to tuple
    ListToTuple = 6,
    /// Type parameter related
    TypeVar = 7,
    ParamSpec = 8,
    TypeVarTuple = 9,
    /// Generic subscript for PEP 695
    SubscriptGeneric = 10,
    TypeAlias = 11,
}
>>>>>>> 6598518de (Save)

/// Intrinsic function for CALL_INTRINSIC_2
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
#[repr(u8)]
pub enum IntrinsicFunction2 {
    PrepReraiseStar = 1,
    TypeVarWithBound = 2,
    TypeVarWithConstraint = 3,
    SetFunctionTypeParams = 4,
    /// Set default value for type parameter (PEP 695)
    SetTypeparamDefault = 5,
}

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct MakeFunctionFlags: u8 {
        const CLOSURE = 0x01;
        const ANNOTATIONS = 0x02;
        const KW_ONLY_DEFAULTS = 0x04;
        const DEFAULTS = 0x08;
        const TYPE_PARAMS = 0x10;
        /// PEP 649: __annotate__ function closure (instead of __annotations__ dict)
        const ANNOTATE = 0x20;
    }
}

impl TryFrom<u32> for MakeFunctionFlags {
    type Error = MarshalError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_bits(u8::try_from(value).map_err(|_| Self::Error::InvalidBytecode)?)
    }
}

impl From<MakeFunctionFlags> for u32 {
    fn from(value: MakeFunctionFlags) -> Self {
        value.bits().into()
    }
}

/// The possible comparison operators.
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
#[repr(u8)]
pub enum ComparisonOperator {
    // be intentional with bits so that we can do eval_ord with just a bitwise and
    // bits: | Equal | Greater | Less |
    Less = 0b001,
    Greater = 0b010,
    NotEqual = 0b011,
    Equal = 0b100,
    LessOrEqual = 0b101,
    GreaterOrEqual = 0b110,
}

/// The possible binary operators.
///
/// See also:
/// - [_PyEval_BinaryOps](https://github.com/python/cpython/blob/8183fa5e3f78ca6ab862de7fb8b14f3d929421e0/Python/ceval.c#L316-L343)
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
#[repr(u8)]
pub enum BinaryOperator {
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
    /// `+=`
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
    /// `[]` subscript
    Subscr = 26,
}

impl BinaryOperator {
    /// Get the "inplace" version of the operator.
    /// This has no effect if `self` is already an "inplace" operator.
    ///
    /// # Example
    /// ```rust
    /// use rustpython_compiler_core::bytecode::BinaryOperator;
    ///
    /// assert_eq!(BinaryOperator::Power.as_inplace(), BinaryOperator::InplacePower);
    ///
    /// assert_eq!(BinaryOperator::InplaceSubtract.as_inplace(), BinaryOperator::InplaceSubtract);
    /// ```
    #[must_use]
    pub const fn as_inplace(self) -> Self {
        match self {
            Self::Add => Self::InplaceAdd,
            Self::And => Self::InplaceAnd,
            Self::FloorDivide => Self::InplaceFloorDivide,
            Self::Lshift => Self::InplaceLshift,
            Self::MatrixMultiply => Self::InplaceMatrixMultiply,
            Self::Multiply => Self::InplaceMultiply,
            Self::Remainder => Self::InplaceRemainder,
            Self::Or => Self::InplaceOr,
            Self::Power => Self::InplacePower,
            Self::Rshift => Self::InplaceRshift,
            Self::Subtract => Self::InplaceSubtract,
            Self::TrueDivide => Self::InplaceTrueDivide,
            Self::Xor => Self::InplaceXor,
            _ => self,
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            Self::Add => "+",
            Self::And => "&",
            Self::FloorDivide => "//",
            Self::Lshift => "<<",
            Self::MatrixMultiply => "@",
            Self::Multiply => "*",
            Self::Remainder => "%",
            Self::Or => "|",
            Self::Power => "**",
            Self::Rshift => ">>",
            Self::Subtract => "-",
            Self::TrueDivide => "/",
            Self::Xor => "^",
            Self::InplaceAdd => "+=",
            Self::InplaceAnd => "&=",
            Self::InplaceFloorDivide => "//=",
            Self::InplaceLshift => "<<=",
            Self::InplaceMatrixMultiply => "@=",
            Self::InplaceMultiply => "*=",
            Self::InplaceRemainder => "%=",
            Self::InplaceOr => "|=",
            Self::InplacePower => "**=",
            Self::InplaceRshift => ">>=",
            Self::InplaceSubtract => "-=",
            Self::InplaceTrueDivide => "/=",
            Self::InplaceXor => "^=",
            Self::Subscr => "[]",
        };
        write!(f, "{op}")
    }
}

/// Whether or not to invert the operation.
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
#[repr(u8)]
pub enum Invert {
    /// ```py
    /// foo is bar
    /// x in lst
    /// ```
    No = 0,
    /// ```py
    /// foo is not bar
    /// x not in lst
    /// ```
    Yes = 1,
}

/// Special method for LOAD_SPECIAL opcode (context managers).
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
#[repr(u8)]
pub enum SpecialMethod {
    /// `__enter__` for sync context manager
    Enter = 0,
    /// `__exit__` for sync context manager
    Exit = 1,
    /// `__aenter__` for async context manager
    AEnter = 2,
    /// `__aexit__` for async context manager
    AExit = 3,
}

impl fmt::Display for SpecialMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method_name = match self {
            Self::Enter => "__enter__",
            Self::Exit => "__exit__",
            Self::AEnter => "__aenter__",
            Self::AExit => "__aexit__",
        };
        write!(f, "{method_name}")
    }
}

/// Specifies if a slice is built with either 2 or 3 arguments.
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = MarshalError::new_invalid_bytecode))]
pub enum BuildSliceArgCount {
    /// ```py
    /// x[5:10]
    /// ```
    Two = 2,
    /// ```py
    /// x[5:10:2]
    /// ```
    Three = 3,
}

#[derive(Copy, Clone)]
pub struct UnpackExArgs {
    pub before: u8,
    pub after: u8,
}

impl From<u32> for UnpackExArgs {
    fn from(value: u32) -> Self {
        let [before, after, ..] = value.to_le_bytes();
        Self { before, after }
    }
}

impl From<UnpackExArgs> for u32 {
    fn from(value: UnpackExArgs) -> Self {
        Self::from_le_bytes([value.before, value.after, 0, 0])
    }
}

impl fmt::Display for UnpackExArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "before: {}, after: {}", self.before, self.after)
    }
}
