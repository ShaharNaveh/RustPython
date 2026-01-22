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

    fn new(opcode: AnyOpcode) -> Option<impl Fn(u32) -> Result<AnyOparg, MarshalError>> {
        match opcode {
            AnyOpcode::Real(op) => Self::from_opcode(op),
            AnyOpcode::Pseudo(op) => Self::from_pseudo_opcode(op),
        }
    }

    fn from_opcode(opcode: Opcode) -> Option<impl Fn(u32) -> Result<AnyOparg, MarshalError>> {
        todo!()
    }

    fn from_pseudo_opcode(
        opcode: PseudoOpcode,
    ) -> Option<impl Fn(u32) -> Result<AnyOparg, MarshalError>> {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub struct AnyOpargBuilder {
    opcode: AnyOpcode,
    oparg: Option<u32>,
}

impl AnyOpargBuilder {
    pub const fn new(opcode: AnyOpcode) -> Self {
        Self { opcode, None }
    }

    pub const fn oparg(mut self, oparg: u32) -> Self {
        self.oparg = Some(oparg);
        self
    }

    pub fn build(self) -> Result<Option<AnyOparg>, MarshalError> {
        match opcode {
            AnyOpcode::Real(op) => self.build_opcode(op),
            AnyOpcode::Pseudo(op) => self.build_pseudo_opcode(op),
        }
    }

    fn oparg_constructor<F>(self) -> Option<F>
    where
        F: Fn(u32) -> Result<Option<AnyOparg>, MarshalError>,
    {
    }
}

impl AnyOparg {
    pub fn from_any_opcode(
        self,
        opcode: AnyOpcode,
        value: u32,
    ) -> Result<Option<Self>, MarshalError> {
        match opcode {
            AnyOpcode::Real(op) => self.from_opcode(op, value),
            AnyOpcode::Pseudo(op) => self.from_pseudo_opcode(op, value),
        }
    }

    pub fn from_opcode(
        self,
        opcode: Opcode,
        value: Option<u32>,
    ) -> Result<Option<Self>, MarshalError> {
        let oparg = match opcode {
            Opcode::Cache => None,
            Opcode::BinarySlice => None,
            Opcode::BuildTemplate => None,
            Opcode::BinaryOpInplaceAddUnicode => None,
            Opcode::CallFunctionEx => None,
            Opcode::CheckEgMatch => None,
            Opcode::CheckExcMatch => None,
            Opcode::CleanupThrow => None,
            Opcode::DeleteSubscr => None,
            Opcode::EndFor => None,
            Opcode::EndSend => None,
            Opcode::ExitInitCheck => None,
            Opcode::FormatSimple => None,
            Opcode::FormatWithSpec => None,
            Opcode::GetAIter => None,
            Opcode::GetANext => None,
            Opcode::GetIter => None,
            Opcode::Reserved => None,
            Opcode::GetLen => None,
            Opcode::GetYieldFromIter => None,
            Opcode::InterpreterExit => None,
            Opcode::LoadBuildClass => None,
            Opcode::LoadLocals => None,
            Opcode::MakeFunction => None,
            Opcode::MatchKeys => None,
            Opcode::MatchMapping => None,
            Opcode::MatchSequence => None,
            Opcode::Nop => None,
            Opcode::NotTaken => None,
            Opcode::PopExcept => None,
            Opcode::PopIter => None,
            Opcode::PopTop => None,
            Opcode::PushExcInfo => None,
            Opcode::PushNull => None,
            Opcode::ReturnGenerator => None,
            Opcode::ReturnValue => None,
            Opcode::SetupAnnotations => None,
            Opcode::StoreSlice => None,
            Opcode::StoreSubscr => None,
            Opcode::ToBool => None,
            Opcode::UnaryInvert => None,
            Opcode::UnaryNegative => None,
            Opcode::UnaryNot => None,
            Opcode::WithExceptStart => None,
            Opcode::BinaryOp => Some(BinaryOperator::try_from(value)?),
            Opcode::BuildInterpolation => Some(value),
            Opcode::BuildList => Some(value),
            Opcode::BuildMap => Some(value),
            Opcode::BuildSet => Some(value),
            Opcode::BuildSlice => Some(BuildSliceArgCount::try_from(value)?),
            Opcode::BuildTuple => Some(value),
            Opcode::Call => Some(value),
            Opcode::CallIntrinsic1 => Some(CallIntrinsic1::try_from(value)?),
            Opcode::CallIntrinsic2 => Some(CallIntrinsic2::try_from(value)?),
            Opcode::CallKw => Some(value),
            Opcode::CompareOp => Some(ComparisonOperator::try_from(value)?),
            Opcode::ContainsOp => Some(Invert::try_from(value)?),
            Opcode::ConvertValue => Some(ConvertValueOparg::try_from(value)?),
            Opcode::Copy => Some(value),
            Opcode::CopyFreeVars => Some(value),
            Opcode::DeleteAttr => Some(NameIdx::from(value)),
            Opcode::DeleteDeref => Some(NameIdx::from(value)),
            Opcode::DeleteFast => Some(NameIdx::from(value)),
            Opcode::DeleteGlobal => Some(NameIdx::from(value)),
            Opcode::DeleteName => Some(NameIdx::from(value)),
            Opcode::DictMerge => Some(value),
            Opcode::DictUpdate => Some(value),
            Opcode::EndAsyncFor => None,
            Opcode::ExtendedArg => None,
            Opcode::ForIter => Some(Label::from(value)),
            Opcode::GetAwaitable => None, // TODO: This should have an oparg
            Opcode::ImportFrom => Some(value),
            Opcode::ImportName => Some(value),
            Opcode::IsOp => Some(Invert::try_from(value)?),
            Opcode::JumpBackward => Some(Label::from(value)),
            Opcode::JumpBackwardNoInterrupt => Some(Label::from(value)),
            Opcode::JumpForward => Some(Label::from(value)),
            Opcode::ListAppend => Some(value),
            Opcode::ListExtend => Some(value),
            Opcode::LoadAttr => Some(NameIdx::from(value)),
            Opcode::LoadCommonConstant => Some(value),
            Opcode::LoadConst => Some(value),
            Opcode::LoadDeref => Some(NameIdx::from(value)),
            Opcode::LoadFast => Some(NameIdx::from(value)),
            Opcode::LoadFastAndClear => Some(NameIdx::from(value)),
            Opcode::LoadFastBorrow => Some(NameIdx::from(value)),
            Opcode::LoadFastBorrowLoadFastBorrow => Some(value),
            Opcode::LoadFastCheck => Some(NameIdx::from(value)),
            Opcode::LoadFastLoadFast => Some(value),
            Opcode::LoadFromDictOrDeref => Some(NameIdx::from(value)),
            Opcode::LoadFromDictOrGlobals => Some(NameIdx::from(value)),
            Opcode::LoadGlobal => Some(NameIdx::from(value)),
            Opcode::LoadName => Some(NameIdx::from(value)),
            Opcode::LoadSmallInt => Some(value),
            Opcode::LoadSpecial => Some(SpecialMethod::try_from(value)?),
            Opcode::LoadSuperAttr => Some(value),
            Opcode::MakeCell => Some(NameIdx::from(value)),
            Opcode::MapAdd => Some(value),
            Opcode::MatchClass => Some(value),
            Opcode::PopJumpIfFalse => Some(Label::from(value)),
            Opcode::PopJumpIfNone => Some(Label::from(value)),
            Opcode::PopJumpIfNotNone => Some(Label::from(value)),
            Opcode::PopJumpIfTrue => Some(Label::from(value)),
            Opcode::RaiseVarargs => Some(RaiseKind::try_from(value)?),
            Opcode::Reraise => Some(value),
            Opcode::Send => Some(Label::from(value)),
            Opcode::SetAdd => Some(value),
            Opcode::SetFunctionAttribute => Some(MakeFunctionFlags::try_from(value)?),
            Opcode::SetUpdate => Some(value),
            Opcode::StoreAttr => Some(NameIdx::from(value)),
            Opcode::StoreDeref => Some(NameIdx::from(value)),
            Opcode::StoreFast => Some(NameIdx::from(value)),
            Opcode::StoreFastLoadFast => Some(StoreFastLoadFast::from(value)),
            Opcode::StoreFastStoreFast => Some(value),
            Opcode::StoreGlobal => Some(NameIdx::from(value)),
            Opcode::StoreName => Some(NameIdx::from(value)),
            Opcode::Swap => Some(value),
            Opcode::UnpackEx => Some(UnpackExArgs::from(value)),
            Opcode::UnpackSequence => Some(value),
            Opcode::YieldValue => Some(value),
            Opcode::Resume => Some(value),
            // Placeholders
            Opcode::BinaryOpAddFloat => None,
            Opcode::BinaryOpAddInt => None,
            Opcode::BinaryOpAddUnicode => None,
            Opcode::BinaryOpExtend => None,
            Opcode::BinaryOpMultiplyFloat => None,
            Opcode::BinaryOpMultiplyInt => None,
            Opcode::BinarySubscrDict => None,
            Opcode::BinarySubscrGetitem => None,
            Opcode::BinarySubscrListInt => None,
            Opcode::BinarySubscrListSlice => None,
            Opcode::BinarySubscrStrInt => None,
            Opcode::BinarySubscrTupleInt => None,
            Opcode::BinaryOpSubtractFloat => None,
            Opcode::BinaryOpSubtractInt => None,
            Opcode::CallAllocAndEnterInit => None,
            Opcode::CallBoundMethodExactArgs => None,
            Opcode::CallBoundMethodGeneral => None,
            Opcode::CallBuiltinClass => None,
            Opcode::CallBuiltinFast => None,
            Opcode::CallBuiltinFastWithKeywords => None,
            Opcode::CallBuiltinO => None,
            Opcode::CallIsinstance => None,
            Opcode::CallKwBoundMethod => None,
            Opcode::CallKwNonPy => None,
            Opcode::CallKwPy => None,
            Opcode::CallLen => None,
            Opcode::CallListAppend => None,
            Opcode::CallMethodDescriptorFast => None,
            Opcode::CallMethodDescriptorFastWithKeywords => None,
            Opcode::CallMethodDescriptorNoargs => None,
            Opcode::CallMethodDescriptorO => None,
            Opcode::CallNonPyGeneral => None,
            Opcode::CallPyExactArgs => None,
            Opcode::CallPyGeneral => None,
            Opcode::CallStr1 => None,
            Opcode::CallTuple1 => None,
            Opcode::CallType1 => None,
            Opcode::CompareOpFloat => None,
            Opcode::CompareOpInt => None,
            Opcode::CompareOpStr => None,
            Opcode::ContainsOpDict => None,
            Opcode::ContainsOpSet => None,
            Opcode::ForIterGen => None,
            Opcode::ForIterList => None,
            Opcode::ForIterRange => None,
            Opcode::ForIterTuple => None,
            Opcode::JumpBackwardJit => None,
            Opcode::JumpBackwardNoJit => None,
            Opcode::LoadAttrClass => None,
            Opcode::LoadAttrClassWithMetaclassCheck => None,
            Opcode::LoadAttrGetattributeOverridden => None,
            Opcode::LoadAttrInstanceValue => None,
            Opcode::LoadAttrMethodLazyDict => None,
            Opcode::LoadAttrMethodNoDict => None,
            Opcode::LoadAttrMethodWithValues => None,
            Opcode::LoadAttrModule => None,
            Opcode::LoadAttrNondescriptorNoDict => None,
            Opcode::LoadAttrNondescriptorWithValues => None,
            Opcode::LoadAttrProperty => None,
            Opcode::LoadAttrSlot => None,
            Opcode::LoadAttrWithHint => None,
            Opcode::LoadConstImmortal => None,
            Opcode::LoadConstMortal => None,
            Opcode::LoadGlobalBuiltin => None,
            Opcode::LoadGlobalModule => None,
            Opcode::LoadSuperAttrAttr => None,
            Opcode::LoadSuperAttrMethod => None,
            Opcode::ResumeCheck => None,
            Opcode::SendGen => None,
            Opcode::StoreAttrInstanceValue => None,
            Opcode::StoreAttrSlot => None,
            Opcode::StoreAttrWithHint => None,
            Opcode::StoreSubscrDict => None,
            Opcode::StoreSubscrListInt => None,
            Opcode::ToBoolAlwaysTrue => None,
            Opcode::ToBoolBool => None,
            Opcode::ToBoolInt => None,
            Opcode::ToBoolList => None,
            Opcode::ToBoolNone => None,
            Opcode::ToBoolStr => None,
            Opcode::UnpackSequenceList => None,
            Opcode::UnpackSequenceTuple => None,
            Opcode::UnpackSequenceTwoTuple => None,
            Opcode::InstrumentedEndFor => None,
            Opcode::InstrumentedPopIter => None,
            Opcode::InstrumentedEndSend => None,
            Opcode::InstrumentedForIter => None,
            Opcode::InstrumentedInstruction => None,
            Opcode::InstrumentedJumpForward => None,
            Opcode::InstrumentedNotTaken => None,
            Opcode::InstrumentedPopJumpIfTrue => None,
            Opcode::InstrumentedPopJumpIfFalse => None,
            Opcode::InstrumentedPopJumpIfNone => None,
            Opcode::InstrumentedPopJumpIfNotNone => None,
            Opcode::InstrumentedResume => None,
            Opcode::InstrumentedReturnValue => None,
            Opcode::InstrumentedYieldValue => None,
            Opcode::InstrumentedEndAsyncFor => None,
            Opcode::InstrumentedLoadSuperAttr => None,
            Opcode::InstrumentedCall => None,
            Opcode::InstrumentedCallKw => None,
            Opcode::InstrumentedCallFunctionEx => None,
            Opcode::InstrumentedJumpBackward => None,
            Opcode::InstrumentedLine => None,
            Opcode::EnterExecutor => None,
        };

        Ok(oparg.map(Into::into))
    }

    pub fn from_pseudo_opcode(
        self,
        opcode: PseudoOpcode,
        value: u32,
    ) -> Result<Option<Self>, MarshalError> {
        None // TODO
    }

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

/// Opcode argument that may be extended by a prior ExtendedArg.
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

/// Full 32-bit op_arg, including any possible ExtendedArg extension.
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
    pub fn get(&mut self, ins: CodeUnit) -> (Instruction, OpArg) {
        let arg = self.extend(ins.arg);
        if !matches!(ins.op, Instruction::ExtendedArg) {
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

/// Oparg values for [`Instruction::ConvertValue`].
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

/// The possible Binary operators.
///
/// # Examples
///
/// ```rust
/// use rustpython_compiler_core::bytecode::{Arg, BinaryOperator, Instruction};
/// let (op, _) = Arg::new(BinaryOperator::Add);
/// let instruction = Instruction::BinaryOp { op };
/// ```
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
