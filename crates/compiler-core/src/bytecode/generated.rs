
use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opcode {
    Cache,
    BinarySlice,
    BuildTemplate,
    BinaryOpInplaceAddUnicode,
    CallFunctionEx,
    CheckEgMatch,
    CheckExcMatch,
    CleanupThrow,
    DeleteSubscr,
    EndFor,
    EndSend,
    ExitInitCheck,
    FormatSimple,
    FormatWithSpec,
    GetAIter,
    GetANext,
    GetIter,
    Reserved,
    GetLen,
    GetYieldFromIter,
    InterpreterExit,
    LoadBuildClass,
    LoadLocals,
    MakeFunction,
    MatchKeys,
    MatchMapping,
    MatchSequence,
    Nop,
    NotTaken,
    PopExcept,
    PopIter,
    PopTop,
    PushExcInfo,
    PushNull,
    ReturnGenerator,
    ReturnValue,
    SetupAnnotations,
    StoreSlice,
    StoreSubscr,
    ToBool,
    UnaryInvert,
    UnaryNegative,
    UnaryNot,
    WithExceptStart,
    BinaryOp,
    BuildInterpolation,
    BuildList,
    BuildMap,
    BuildSet,
    BuildSlice,
    BuildString,
    BuildTuple,
    Call,
    CallIntrinsic1,
    CallIntrinsic2,
    CallKw,
    CompareOp,
    ContainsOp,
    ConvertValue,
    Copy,
    CopyFreeVars,
    DeleteAttr,
    DeleteDeref,
    DeleteFast,
    DeleteGlobal,
    DeleteName,
    DictMerge,
    DictUpdate,
    EndAsyncFor,
    ExtendedArg,
    ForIter,
    GetAwaitable,
    ImportFrom,
    ImportName,
    IsOp,
    JumpBackward,
    JumpBackwardNoInterrupt,
    JumpForward,
    ListAppend,
    ListExtend,
    LoadAttr,
    LoadCommonConstant,
    LoadConst,
    LoadDeref,
    LoadFast,
    LoadFastAndClear,
    LoadFastBorrow,
    LoadFastBorrowLoadFastBorrow,
    LoadFastCheck,
    LoadFastLoadFast,
    LoadFromDictOrDeref,
    LoadFromDictOrGlobals,
    LoadGlobal,
    LoadName,
    LoadSmallInt,
    LoadSpecial,
    LoadSuperAttr,
    MakeCell,
    MapAdd,
    MatchClass,
    PopJumpIfFalse,
    PopJumpIfNone,
    PopJumpIfNotNone,
    PopJumpIfTrue,
    RaiseVarargs,
    Reraise,
    Send,
    SetAdd,
    SetFunctionAttribute,
    SetUpdate,
    StoreAttr,
    StoreDeref,
    StoreFast,
    StoreFastLoadFast,
    StoreFastStoreFast,
    StoreGlobal,
    StoreName,
    Swap,
    UnpackEx,
    UnpackSequence,
    YieldValue,
    Resume,
    BinaryOpAddFloat,
    BinaryOpAddInt,
    BinaryOpAddUnicode,
    BinaryOpExtend,
    BinaryOpMultiplyFloat,
    BinaryOpMultiplyInt,
    BinaryOpSubscrDict,
    BinaryOpSubscrGetitem,
    BinaryOpSubscrListInt,
    BinaryOpSubscrListSlice,
    BinaryOpSubscrStrInt,
    BinaryOpSubscrTupleInt,
    BinaryOpSubtractFloat,
    BinaryOpSubtractInt,
    CallAllocAndEnterInit,
    CallBoundMethodExactArgs,
    CallBoundMethodGeneral,
    CallBuiltinClass,
    CallBuiltinFast,
    CallBuiltinFastWithKeywords,
    CallBuiltinO,
    CallIsinstance,
    CallKwBoundMethod,
    CallKwNonPy,
    CallKwPy,
    CallLen,
    CallListAppend,
    CallMethodDescriptorFast,
    CallMethodDescriptorFastWithKeywords,
    CallMethodDescriptorNoargs,
    CallMethodDescriptorO,
    CallNonPyGeneral,
    CallPyExactArgs,
    CallPyGeneral,
    CallStr1,
    CallTuple1,
    CallType1,
    CompareOpFloat,
    CompareOpInt,
    CompareOpStr,
    ContainsOpDict,
    ContainsOpSet,
    ForIterGen,
    ForIterList,
    ForIterRange,
    ForIterTuple,
    JumpBackwardJit,
    JumpBackwardNoJit,
    LoadAttrClass,
    LoadAttrClassWithMetaclassCheck,
    LoadAttrGetattributeOverridden,
    LoadAttrInstanceValue,
    LoadAttrMethodLazyDict,
    LoadAttrMethodNoDict,
    LoadAttrMethodWithValues,
    LoadAttrModule,
    LoadAttrNondescriptorNoDict,
    LoadAttrNondescriptorWithValues,
    LoadAttrProperty,
    LoadAttrSlot,
    LoadAttrWithHint,
    LoadConstImmortal,
    LoadConstMortal,
    LoadGlobalBuiltin,
    LoadGlobalModule,
    LoadSuperAttrAttr,
    LoadSuperAttrMethod,
    ResumeCheck,
    SendGen,
    StoreAttrInstanceValue,
    StoreAttrSlot,
    StoreAttrWithHint,
    StoreSubscrDict,
    StoreSubscrListInt,
    ToBoolAlwaysTrue,
    ToBoolBool,
    ToBoolInt,
    ToBoolList,
    ToBoolNone,
    ToBoolStr,
    UnpackSequenceList,
    UnpackSequenceTuple,
    UnpackSequenceTwoTuple,
    InstrumentedEndFor,
    InstrumentedPopIter,
    InstrumentedEndSend,
    InstrumentedForIter,
    InstrumentedInstruction,
    InstrumentedJumpForward,
    InstrumentedNotTaken,
    InstrumentedPopJumpIfTrue,
    InstrumentedPopJumpIfFalse,
    InstrumentedPopJumpIfNone,
    InstrumentedPopJumpIfNotNone,
    InstrumentedResume,
    InstrumentedReturnValue,
    InstrumentedYieldValue,
    InstrumentedEndAsyncFor,
    InstrumentedLoadSuperAttr,
    InstrumentedCall,
    InstrumentedCallKw,
    InstrumentedCallFunctionEx,
    InstrumentedJumpBackward,
    InstrumentedLine,
    EnterExecutor,
}

impl Opcode {
    /// Whether opcode ID have 'HAS_ARG_FLAG' set.
    #[must_use]
    pub const fn has_arg(self) -> bool {
        matches!(
            self,
            Self::BinaryOp
                | Self::BuildInterpolation
                | Self::BuildList
                | Self::BuildMap
                | Self::BuildSet
                | Self::BuildSlice
                | Self::BuildString
                | Self::BuildTuple
                | Self::Call
                | Self::CallIntrinsic1
                | Self::CallIntrinsic2
                | Self::CallKw
                | Self::CompareOp
                | Self::ContainsOp
                | Self::ConvertValue
                | Self::Copy
                | Self::CopyFreeVars
                | Self::DeleteAttr
                | Self::DeleteDeref
                | Self::DeleteFast
                | Self::DeleteGlobal
                | Self::DeleteName
                | Self::DictMerge
                | Self::DictUpdate
                | Self::EndAsyncFor
                | Self::ExtendedArg
                | Self::ForIter
                | Self::GetAwaitable
                | Self::ImportFrom
                | Self::ImportName
                | Self::IsOp
                | Self::JumpBackward
                | Self::JumpBackwardNoInterrupt
                | Self::JumpForward
                | Self::ListAppend
                | Self::ListExtend
                | Self::LoadAttr
                | Self::LoadCommonConstant
                | Self::LoadConst
                | Self::LoadDeref
                | Self::LoadFast
                | Self::LoadFastAndClear
                | Self::LoadFastBorrow
                | Self::LoadFastBorrowLoadFastBorrow
                | Self::LoadFastCheck
                | Self::LoadFastLoadFast
                | Self::LoadFromDictOrDeref
                | Self::LoadFromDictOrGlobals
                | Self::LoadGlobal
                | Self::LoadName
                | Self::LoadSmallInt
                | Self::LoadSpecial
                | Self::LoadSuperAttr
                | Self::MakeCell
                | Self::MapAdd
                | Self::MatchClass
                | Self::PopJumpIfFalse
                | Self::PopJumpIfNone
                | Self::PopJumpIfNotNone
                | Self::PopJumpIfTrue
                | Self::RaiseVarargs
                | Self::Reraise
                | Self::Send
                | Self::SetAdd
                | Self::SetFunctionAttribute
                | Self::SetUpdate
                | Self::StoreAttr
                | Self::StoreDeref
                | Self::StoreFast
                | Self::StoreFastLoadFast
                | Self::StoreFastStoreFast
                | Self::StoreGlobal
                | Self::StoreName
                | Self::Swap
                | Self::UnpackEx
                | Self::UnpackSequence
                | Self::YieldValue
                | Self::Resume
                | Self::CallAllocAndEnterInit
                | Self::CallBoundMethodExactArgs
                | Self::CallBoundMethodGeneral
                | Self::CallBuiltinClass
                | Self::CallBuiltinFast
                | Self::CallBuiltinFastWithKeywords
                | Self::CallBuiltinO
                | Self::CallIsinstance
                | Self::CallKwBoundMethod
                | Self::CallKwNonPy
                | Self::CallKwPy
                | Self::CallListAppend
                | Self::CallMethodDescriptorFast
                | Self::CallMethodDescriptorFastWithKeywords
                | Self::CallMethodDescriptorNoargs
                | Self::CallMethodDescriptorO
                | Self::CallNonPyGeneral
                | Self::CallPyExactArgs
                | Self::CallPyGeneral
                | Self::CallStr1
                | Self::CallTuple1
                | Self::CallType1
                | Self::CompareOpFloat
                | Self::CompareOpInt
                | Self::CompareOpStr
                | Self::ContainsOpDict
                | Self::ContainsOpSet
                | Self::ForIterGen
                | Self::ForIterList
                | Self::ForIterRange
                | Self::ForIterTuple
                | Self::JumpBackwardJit
                | Self::JumpBackwardNoJit
                | Self::LoadAttrClass
                | Self::LoadAttrClassWithMetaclassCheck
                | Self::LoadAttrGetattributeOverridden
                | Self::LoadAttrInstanceValue
                | Self::LoadAttrMethodLazyDict
                | Self::LoadAttrMethodNoDict
                | Self::LoadAttrMethodWithValues
                | Self::LoadAttrModule
                | Self::LoadAttrNondescriptorNoDict
                | Self::LoadAttrNondescriptorWithValues
                | Self::LoadAttrProperty
                | Self::LoadAttrSlot
                | Self::LoadAttrWithHint
                | Self::LoadConstImmortal
                | Self::LoadConstMortal
                | Self::LoadGlobalBuiltin
                | Self::LoadGlobalModule
                | Self::LoadSuperAttrAttr
                | Self::LoadSuperAttrMethod
                | Self::SendGen
                | Self::StoreAttrWithHint
                | Self::UnpackSequenceList
                | Self::UnpackSequenceTuple
                | Self::UnpackSequenceTwoTuple
                | Self::InstrumentedForIter
                | Self::InstrumentedJumpForward
                | Self::InstrumentedPopJumpIfTrue
                | Self::InstrumentedPopJumpIfFalse
                | Self::InstrumentedPopJumpIfNone
                | Self::InstrumentedPopJumpIfNotNone
                | Self::InstrumentedResume
                | Self::InstrumentedYieldValue
                | Self::InstrumentedEndAsyncFor
                | Self::InstrumentedLoadSuperAttr
                | Self::InstrumentedCall
                | Self::InstrumentedCallKw
                | Self::InstrumentedJumpBackward
                | Self::EnterExecutor
        )
    }

    /// Whether opcode ID have 'HAS_CONST_FLAG' set.
    #[must_use]
    pub const fn has_const(self) -> bool {
        matches!(
            self,
            Self::LoadConst | Self::LoadConstImmortal | Self::LoadConstMortal
        )
    }

    /// Whether opcode ID have 'HAS_PURE_FLAG' set.
    #[must_use]
    pub const fn has_exc(self) -> bool {
        matches!(
            self,
            Self::EndSend
                | Self::Nop
                | Self::NotTaken
                | Self::PopIter
                | Self::PopTop
                | Self::PushNull
                | Self::UnaryNot
                | Self::Copy
                | Self::LoadFast
                | Self::LoadFastBorrow
                | Self::Swap
        )
    }

    /// Whether opcode ID have 'HAS_JUMP_FLAG' set.
    #[must_use]
    pub const fn has_jump(self) -> bool {
        matches!(
            self,
            Self::EndAsyncFor
                | Self::ForIter
                | Self::JumpBackward
                | Self::JumpBackwardNoInterrupt
                | Self::JumpForward
                | Self::PopJumpIfFalse
                | Self::PopJumpIfNone
                | Self::PopJumpIfNotNone
                | Self::PopJumpIfTrue
                | Self::Send
                | Self::ForIterList
                | Self::ForIterRange
                | Self::ForIterTuple
                | Self::JumpBackwardJit
                | Self::JumpBackwardNoJit
                | Self::InstrumentedForIter
                | Self::InstrumentedEndAsyncFor
        )
    }

    /// Whether opcode ID have 'HAS_LOCAL_FLAG' set.
    #[must_use]
    pub const fn has_local(self) -> bool {
        matches!(
            self,
            Self::BinaryOpInplaceAddUnicode
                | Self::DeleteFast
                | Self::LoadDeref
                | Self::LoadFast
                | Self::LoadFastAndClear
                | Self::LoadFastBorrow
                | Self::LoadFastBorrowLoadFastBorrow
                | Self::LoadFastCheck
                | Self::LoadFastLoadFast
                | Self::StoreFast
                | Self::StoreFastLoadFast
                | Self::StoreFastStoreFast
        )
    }

    /// Whether opcode ID have 'HAS_NAME_FLAG' set.
    #[must_use]
    pub const fn has_name(self) -> bool {
        matches!(
            self,
            Self::DeleteAttr
                | Self::DeleteGlobal
                | Self::DeleteName
                | Self::ImportFrom
                | Self::ImportName
                | Self::LoadAttr
                | Self::LoadFromDictOrGlobals
                | Self::LoadGlobal
                | Self::LoadName
                | Self::LoadSuperAttr
                | Self::StoreAttr
                | Self::StoreGlobal
                | Self::StoreName
                | Self::LoadAttrGetattributeOverridden
                | Self::LoadAttrWithHint
                | Self::LoadSuperAttrAttr
                | Self::LoadSuperAttrMethod
                | Self::StoreAttrWithHint
                | Self::InstrumentedLoadSuperAttr
        )
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Cache => "CACHE",
            Self::BinarySlice => "BINARY_SLICE",
            Self::BuildTemplate => "BUILD_TEMPLATE",
            Self::BinaryOpInplaceAddUnicode => "BINARY_OP_INPLACE_ADD_UNICODE",
            Self::CallFunctionEx => "CALL_FUNCTION_EX",
            Self::CheckEgMatch => "CHECK_EG_MATCH",
            Self::CheckExcMatch => "CHECK_EXC_MATCH",
            Self::CleanupThrow => "CLEANUP_THROW",
            Self::DeleteSubscr => "DELETE_SUBSCR",
            Self::EndFor => "END_FOR",
            Self::EndSend => "END_SEND",
            Self::ExitInitCheck => "EXIT_INIT_CHECK",
            Self::FormatSimple => "FORMAT_SIMPLE",
            Self::FormatWithSpec => "FORMAT_WITH_SPEC",
            Self::GetAIter => "GET_AITER",
            Self::GetANext => "GET_ANEXT",
            Self::GetIter => "GET_ITER",
            Self::Reserved => "RESERVED",
            Self::GetLen => "GET_LEN",
            Self::GetYieldFromIter => "GET_YIELD_FROM_ITER",
            Self::InterpreterExit => "INTERPRETER_EXIT",
            Self::LoadBuildClass => "LOAD_BUILD_CLASS",
            Self::LoadLocals => "LOAD_LOCALS",
            Self::MakeFunction => "MAKE_FUNCTION",
            Self::MatchKeys => "MATCH_KEYS",
            Self::MatchMapping => "MATCH_MAPPING",
            Self::MatchSequence => "MATCH_SEQUENCE",
            Self::Nop => "NOP",
            Self::NotTaken => "NOT_TAKEN",
            Self::PopExcept => "POP_EXCEPT",
            Self::PopIter => "POP_ITER",
            Self::PopTop => "POP_TOP",
            Self::PushExcInfo => "PUSH_EXC_INFO",
            Self::PushNull => "PUSH_NULL",
            Self::ReturnGenerator => "RETURN_GENERATOR",
            Self::ReturnValue => "RETURN_VALUE",
            Self::SetupAnnotations => "SETUP_ANNOTATIONS",
            Self::StoreSlice => "STORE_SLICE",
            Self::StoreSubscr => "STORE_SUBSCR",
            Self::ToBool => "TO_BOOL",
            Self::UnaryInvert => "UNARY_INVERT",
            Self::UnaryNegative => "UNARY_NEGATIVE",
            Self::UnaryNot => "UNARY_NOT",
            Self::WithExceptStart => "WITH_EXCEPT_START",
            Self::BinaryOp => "BINARY_OP",
            Self::BuildInterpolation => "BUILD_INTERPOLATION",
            Self::BuildList => "BUILD_LIST",
            Self::BuildMap => "BUILD_MAP",
            Self::BuildSet => "BUILD_SET",
            Self::BuildSlice => "BUILD_SLICE",
            Self::BuildString => "BUILD_STRING",
            Self::BuildTuple => "BUILD_TUPLE",
            Self::Call => "CALL",
            Self::CallIntrinsic1 => "CALL_INTRINSIC_1",
            Self::CallIntrinsic2 => "CALL_INTRINSIC_2",
            Self::CallKw => "CALL_KW",
            Self::CompareOp => "COMPARE_OP",
            Self::ContainsOp => "CONTAINS_OP",
            Self::ConvertValue => "CONVERT_VALUE",
            Self::Copy => "COPY",
            Self::CopyFreeVars => "COPY_FREE_VARS",
            Self::DeleteAttr => "DELETE_ATTR",
            Self::DeleteDeref => "DELETE_DEREF",
            Self::DeleteFast => "DELETE_FAST",
            Self::DeleteGlobal => "DELETE_GLOBAL",
            Self::DeleteName => "DELETE_NAME",
            Self::DictMerge => "DICT_MERGE",
            Self::DictUpdate => "DICT_UPDATE",
            Self::EndAsyncFor => "END_ASYNC_FOR",
            Self::ExtendedArg => "EXTENDED_ARG",
            Self::ForIter => "FOR_ITER",
            Self::GetAwaitable => "GET_AWAITABLE",
            Self::ImportFrom => "IMPORT_FROM",
            Self::ImportName => "IMPORT_NAME",
            Self::IsOp => "IS_OP",
            Self::JumpBackward => "JUMP_BACKWARD",
            Self::JumpBackwardNoInterrupt => "JUMP_BACKWARD_NO_INTERRUPT",
            Self::JumpForward => "JUMP_FORWARD",
            Self::ListAppend => "LIST_APPEND",
            Self::ListExtend => "LIST_EXTEND",
            Self::LoadAttr => "LOAD_ATTR",
            Self::LoadCommonConstant => "LOAD_COMMON_CONSTANT",
            Self::LoadConst => "LOAD_CONST",
            Self::LoadDeref => "LOAD_DEREF",
            Self::LoadFast => "LOAD_FAST",
            Self::LoadFastAndClear => "LOAD_FAST_AND_CLEAR",
            Self::LoadFastBorrow => "LOAD_FAST_BORROW",
            Self::LoadFastBorrowLoadFastBorrow => "LOAD_FAST_BORROW_LOAD_FAST_BORROW",
            Self::LoadFastCheck => "LOAD_FAST_CHECK",
            Self::LoadFastLoadFast => "LOAD_FAST_LOAD_FAST",
            Self::LoadFromDictOrDeref => "LOAD_FROM_DICT_OR_DEREF",
            Self::LoadFromDictOrGlobals => "LOAD_FROM_DICT_OR_GLOBALS",
            Self::LoadGlobal => "LOAD_GLOBAL",
            Self::LoadName => "LOAD_NAME",
            Self::LoadSmallInt => "LOAD_SMALL_INT",
            Self::LoadSpecial => "LOAD_SPECIAL",
            Self::LoadSuperAttr => "LOAD_SUPER_ATTR",
            Self::MakeCell => "MAKE_CELL",
            Self::MapAdd => "MAP_ADD",
            Self::MatchClass => "MATCH_CLASS",
            Self::PopJumpIfFalse => "POP_JUMP_IF_FALSE",
            Self::PopJumpIfNone => "POP_JUMP_IF_NONE",
            Self::PopJumpIfNotNone => "POP_JUMP_IF_NOT_NONE",
            Self::PopJumpIfTrue => "POP_JUMP_IF_TRUE",
            Self::RaiseVarargs => "RAISE_VARARGS",
            Self::Reraise => "RERAISE",
            Self::Send => "SEND",
            Self::SetAdd => "SET_ADD",
            Self::SetFunctionAttribute => "SET_FUNCTION_ATTRIBUTE",
            Self::SetUpdate => "SET_UPDATE",
            Self::StoreAttr => "STORE_ATTR",
            Self::StoreDeref => "STORE_DEREF",
            Self::StoreFast => "STORE_FAST",
            Self::StoreFastLoadFast => "STORE_FAST_LOAD_FAST",
            Self::StoreFastStoreFast => "STORE_FAST_STORE_FAST",
            Self::StoreGlobal => "STORE_GLOBAL",
            Self::StoreName => "STORE_NAME",
            Self::Swap => "SWAP",
            Self::UnpackEx => "UNPACK_EX",
            Self::UnpackSequence => "UNPACK_SEQUENCE",
            Self::YieldValue => "YIELD_VALUE",
            Self::Resume => "RESUME",
            Self::BinaryOpAddFloat => "BINARY_OP_ADD_FLOAT",
            Self::BinaryOpAddInt => "BINARY_OP_ADD_INT",
            Self::BinaryOpAddUnicode => "BINARY_OP_ADD_UNICODE",
            Self::BinaryOpExtend => "BINARY_OP_EXTEND",
            Self::BinaryOpMultiplyFloat => "BINARY_OP_MULTIPLY_FLOAT",
            Self::BinaryOpMultiplyInt => "BINARY_OP_MULTIPLY_INT",
            Self::BinaryOpSubscrDict => "BINARY_OP_SUBSCR_DICT",
            Self::BinaryOpSubscrGetitem => "BINARY_OP_SUBSCR_GETITEM",
            Self::BinaryOpSubscrListInt => "BINARY_OP_SUBSCR_LIST_INT",
            Self::BinaryOpSubscrListSlice => "BINARY_OP_SUBSCR_LIST_SLICE",
            Self::BinaryOpSubscrStrInt => "BINARY_OP_SUBSCR_STR_INT",
            Self::BinaryOpSubscrTupleInt => "BINARY_OP_SUBSCR_TUPLE_INT",
            Self::BinaryOpSubtractFloat => "BINARY_OP_SUBTRACT_FLOAT",
            Self::BinaryOpSubtractInt => "BINARY_OP_SUBTRACT_INT",
            Self::CallAllocAndEnterInit => "CALL_ALLOC_AND_ENTER_INIT",
            Self::CallBoundMethodExactArgs => "CALL_BOUND_METHOD_EXACT_ARGS",
            Self::CallBoundMethodGeneral => "CALL_BOUND_METHOD_GENERAL",
            Self::CallBuiltinClass => "CALL_BUILTIN_CLASS",
            Self::CallBuiltinFast => "CALL_BUILTIN_FAST",
            Self::CallBuiltinFastWithKeywords => "CALL_BUILTIN_FAST_WITH_KEYWORDS",
            Self::CallBuiltinO => "CALL_BUILTIN_O",
            Self::CallIsinstance => "CALL_ISINSTANCE",
            Self::CallKwBoundMethod => "CALL_KW_BOUND_METHOD",
            Self::CallKwNonPy => "CALL_KW_NON_PY",
            Self::CallKwPy => "CALL_KW_PY",
            Self::CallLen => "CALL_LEN",
            Self::CallListAppend => "CALL_LIST_APPEND",
            Self::CallMethodDescriptorFast => "CALL_METHOD_DESCRIPTOR_FAST",
            Self::CallMethodDescriptorFastWithKeywords => {
                "CALL_METHOD_DESCRIPTOR_FAST_WITH_KEYWORDS"
            }
            Self::CallMethodDescriptorNoargs => "CALL_METHOD_DESCRIPTOR_NOARGS",
            Self::CallMethodDescriptorO => "CALL_METHOD_DESCRIPTOR_O",
            Self::CallNonPyGeneral => "CALL_NON_PY_GENERAL",
            Self::CallPyExactArgs => "CALL_PY_EXACT_ARGS",
            Self::CallPyGeneral => "CALL_PY_GENERAL",
            Self::CallStr1 => "CALL_STR_1",
            Self::CallTuple1 => "CALL_TUPLE_1",
            Self::CallType1 => "CALL_TYPE_1",
            Self::CompareOpFloat => "COMPARE_OP_FLOAT",
            Self::CompareOpInt => "COMPARE_OP_INT",
            Self::CompareOpStr => "COMPARE_OP_STR",
            Self::ContainsOpDict => "CONTAINS_OP_DICT",
            Self::ContainsOpSet => "CONTAINS_OP_SET",
            Self::ForIterGen => "FOR_ITER_GEN",
            Self::ForIterList => "FOR_ITER_LIST",
            Self::ForIterRange => "FOR_ITER_RANGE",
            Self::ForIterTuple => "FOR_ITER_TUPLE",
            Self::JumpBackwardJit => "JUMP_BACKWARD_JIT",
            Self::JumpBackwardNoJit => "JUMP_BACKWARD_NO_JIT",
            Self::LoadAttrClass => "LOAD_ATTR_CLASS",
            Self::LoadAttrClassWithMetaclassCheck => "LOAD_ATTR_CLASS_WITH_METACLASS_CHECK",
            Self::LoadAttrGetattributeOverridden => "LOAD_ATTR_GETATTRIBUTE_OVERRIDDEN",
            Self::LoadAttrInstanceValue => "LOAD_ATTR_INSTANCE_VALUE",
            Self::LoadAttrMethodLazyDict => "LOAD_ATTR_METHOD_LAZY_DICT",
            Self::LoadAttrMethodNoDict => "LOAD_ATTR_METHOD_NO_DICT",
            Self::LoadAttrMethodWithValues => "LOAD_ATTR_METHOD_WITH_VALUES",
            Self::LoadAttrModule => "LOAD_ATTR_MODULE",
            Self::LoadAttrNondescriptorNoDict => "LOAD_ATTR_NONDESCRIPTOR_NO_DICT",
            Self::LoadAttrNondescriptorWithValues => "LOAD_ATTR_NONDESCRIPTOR_WITH_VALUES",
            Self::LoadAttrProperty => "LOAD_ATTR_PROPERTY",
            Self::LoadAttrSlot => "LOAD_ATTR_SLOT",
            Self::LoadAttrWithHint => "LOAD_ATTR_WITH_HINT",
            Self::LoadConstImmortal => "LOAD_CONST_IMMORTAL",
            Self::LoadConstMortal => "LOAD_CONST_MORTAL",
            Self::LoadGlobalBuiltin => "LOAD_GLOBAL_BUILTIN",
            Self::LoadGlobalModule => "LOAD_GLOBAL_MODULE",
            Self::LoadSuperAttrAttr => "LOAD_SUPER_ATTR_ATTR",
            Self::LoadSuperAttrMethod => "LOAD_SUPER_ATTR_METHOD",
            Self::ResumeCheck => "RESUME_CHECK",
            Self::SendGen => "SEND_GEN",
            Self::StoreAttrInstanceValue => "STORE_ATTR_INSTANCE_VALUE",
            Self::StoreAttrSlot => "STORE_ATTR_SLOT",
            Self::StoreAttrWithHint => "STORE_ATTR_WITH_HINT",
            Self::StoreSubscrDict => "STORE_SUBSCR_DICT",
            Self::StoreSubscrListInt => "STORE_SUBSCR_LIST_INT",
            Self::ToBoolAlwaysTrue => "TO_BOOL_ALWAYS_TRUE",
            Self::ToBoolBool => "TO_BOOL_BOOL",
            Self::ToBoolInt => "TO_BOOL_INT",
            Self::ToBoolList => "TO_BOOL_LIST",
            Self::ToBoolNone => "TO_BOOL_NONE",
            Self::ToBoolStr => "TO_BOOL_STR",
            Self::UnpackSequenceList => "UNPACK_SEQUENCE_LIST",
            Self::UnpackSequenceTuple => "UNPACK_SEQUENCE_TUPLE",
            Self::UnpackSequenceTwoTuple => "UNPACK_SEQUENCE_TWO_TUPLE",
            Self::InstrumentedEndFor => "INSTRUMENTED_END_FOR",
            Self::InstrumentedPopIter => "INSTRUMENTED_POP_ITER",
            Self::InstrumentedEndSend => "INSTRUMENTED_END_SEND",
            Self::InstrumentedForIter => "INSTRUMENTED_FOR_ITER",
            Self::InstrumentedInstruction => "INSTRUMENTED_INSTRUCTION",
            Self::InstrumentedJumpForward => "INSTRUMENTED_JUMP_FORWARD",
            Self::InstrumentedNotTaken => "INSTRUMENTED_NOT_TAKEN",
            Self::InstrumentedPopJumpIfTrue => "INSTRUMENTED_POP_JUMP_IF_TRUE",
            Self::InstrumentedPopJumpIfFalse => "INSTRUMENTED_POP_JUMP_IF_FALSE",
            Self::InstrumentedPopJumpIfNone => "INSTRUMENTED_POP_JUMP_IF_NONE",
            Self::InstrumentedPopJumpIfNotNone => "INSTRUMENTED_POP_JUMP_IF_NOT_NONE",
            Self::InstrumentedResume => "INSTRUMENTED_RESUME",
            Self::InstrumentedReturnValue => "INSTRUMENTED_RETURN_VALUE",
            Self::InstrumentedYieldValue => "INSTRUMENTED_YIELD_VALUE",
            Self::InstrumentedEndAsyncFor => "INSTRUMENTED_END_ASYNC_FOR",
            Self::InstrumentedLoadSuperAttr => "INSTRUMENTED_LOAD_SUPER_ATTR",
            Self::InstrumentedCall => "INSTRUMENTED_CALL",
            Self::InstrumentedCallKw => "INSTRUMENTED_CALL_KW",
            Self::InstrumentedCallFunctionEx => "INSTRUMENTED_CALL_FUNCTION_EX",
            Self::InstrumentedJumpBackward => "INSTRUMENTED_JUMP_BACKWARD",
            Self::InstrumentedLine => "INSTRUMENTED_LINE",
            Self::EnterExecutor => "ENTER_EXECUTOR",
        };
        write!(f, "{name}")
    }
}

impl TryFrom<u8> for Opcode {
    type Error = crate::marshal::MarshalError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Cache,
            1 => Self::BinarySlice,
            2 => Self::BuildTemplate,
            3 => Self::BinaryOpInplaceAddUnicode,
            4 => Self::CallFunctionEx,
            5 => Self::CheckEgMatch,
            6 => Self::CheckExcMatch,
            7 => Self::CleanupThrow,
            8 => Self::DeleteSubscr,
            9 => Self::EndFor,
            10 => Self::EndSend,
            11 => Self::ExitInitCheck,
            12 => Self::FormatSimple,
            13 => Self::FormatWithSpec,
            14 => Self::GetAIter,
            15 => Self::GetANext,
            16 => Self::GetIter,
            17 => Self::Reserved,
            18 => Self::GetLen,
            19 => Self::GetYieldFromIter,
            20 => Self::InterpreterExit,
            21 => Self::LoadBuildClass,
            22 => Self::LoadLocals,
            23 => Self::MakeFunction,
            24 => Self::MatchKeys,
            25 => Self::MatchMapping,
            26 => Self::MatchSequence,
            27 => Self::Nop,
            28 => Self::NotTaken,
            29 => Self::PopExcept,
            30 => Self::PopIter,
            31 => Self::PopTop,
            32 => Self::PushExcInfo,
            33 => Self::PushNull,
            34 => Self::ReturnGenerator,
            35 => Self::ReturnValue,
            36 => Self::SetupAnnotations,
            37 => Self::StoreSlice,
            38 => Self::StoreSubscr,
            39 => Self::ToBool,
            40 => Self::UnaryInvert,
            41 => Self::UnaryNegative,
            42 => Self::UnaryNot,
            43 => Self::WithExceptStart,
            44 => Self::BinaryOp,
            45 => Self::BuildInterpolation,
            46 => Self::BuildList,
            47 => Self::BuildMap,
            48 => Self::BuildSet,
            49 => Self::BuildSlice,
            50 => Self::BuildString,
            51 => Self::BuildTuple,
            52 => Self::Call,
            53 => Self::CallIntrinsic1,
            54 => Self::CallIntrinsic2,
            55 => Self::CallKw,
            56 => Self::CompareOp,
            57 => Self::ContainsOp,
            58 => Self::ConvertValue,
            59 => Self::Copy,
            60 => Self::CopyFreeVars,
            61 => Self::DeleteAttr,
            62 => Self::DeleteDeref,
            63 => Self::DeleteFast,
            64 => Self::DeleteGlobal,
            65 => Self::DeleteName,
            66 => Self::DictMerge,
            67 => Self::DictUpdate,
            68 => Self::EndAsyncFor,
            69 => Self::ExtendedArg,
            70 => Self::ForIter,
            71 => Self::GetAwaitable,
            72 => Self::ImportFrom,
            73 => Self::ImportName,
            74 => Self::IsOp,
            75 => Self::JumpBackward,
            76 => Self::JumpBackwardNoInterrupt,
            77 => Self::JumpForward,
            78 => Self::ListAppend,
            79 => Self::ListExtend,
            80 => Self::LoadAttr,
            81 => Self::LoadCommonConstant,
            82 => Self::LoadConst,
            83 => Self::LoadDeref,
            84 => Self::LoadFast,
            85 => Self::LoadFastAndClear,
            86 => Self::LoadFastBorrow,
            87 => Self::LoadFastBorrowLoadFastBorrow,
            88 => Self::LoadFastCheck,
            89 => Self::LoadFastLoadFast,
            90 => Self::LoadFromDictOrDeref,
            91 => Self::LoadFromDictOrGlobals,
            92 => Self::LoadGlobal,
            93 => Self::LoadName,
            94 => Self::LoadSmallInt,
            95 => Self::LoadSpecial,
            96 => Self::LoadSuperAttr,
            97 => Self::MakeCell,
            98 => Self::MapAdd,
            99 => Self::MatchClass,
            100 => Self::PopJumpIfFalse,
            101 => Self::PopJumpIfNone,
            102 => Self::PopJumpIfNotNone,
            103 => Self::PopJumpIfTrue,
            104 => Self::RaiseVarargs,
            105 => Self::Reraise,
            106 => Self::Send,
            107 => Self::SetAdd,
            108 => Self::SetFunctionAttribute,
            109 => Self::SetUpdate,
            110 => Self::StoreAttr,
            111 => Self::StoreDeref,
            112 => Self::StoreFast,
            113 => Self::StoreFastLoadFast,
            114 => Self::StoreFastStoreFast,
            115 => Self::StoreGlobal,
            116 => Self::StoreName,
            117 => Self::Swap,
            118 => Self::UnpackEx,
            119 => Self::UnpackSequence,
            120 => Self::YieldValue,
            128 => Self::Resume,
            129 => Self::BinaryOpAddFloat,
            130 => Self::BinaryOpAddInt,
            131 => Self::BinaryOpAddUnicode,
            132 => Self::BinaryOpExtend,
            133 => Self::BinaryOpMultiplyFloat,
            134 => Self::BinaryOpMultiplyInt,
            135 => Self::BinaryOpSubscrDict,
            136 => Self::BinaryOpSubscrGetitem,
            137 => Self::BinaryOpSubscrListInt,
            138 => Self::BinaryOpSubscrListSlice,
            139 => Self::BinaryOpSubscrStrInt,
            140 => Self::BinaryOpSubscrTupleInt,
            141 => Self::BinaryOpSubtractFloat,
            142 => Self::BinaryOpSubtractInt,
            143 => Self::CallAllocAndEnterInit,
            144 => Self::CallBoundMethodExactArgs,
            145 => Self::CallBoundMethodGeneral,
            146 => Self::CallBuiltinClass,
            147 => Self::CallBuiltinFast,
            148 => Self::CallBuiltinFastWithKeywords,
            149 => Self::CallBuiltinO,
            150 => Self::CallIsinstance,
            151 => Self::CallKwBoundMethod,
            152 => Self::CallKwNonPy,
            153 => Self::CallKwPy,
            154 => Self::CallLen,
            155 => Self::CallListAppend,
            156 => Self::CallMethodDescriptorFast,
            157 => Self::CallMethodDescriptorFastWithKeywords,
            158 => Self::CallMethodDescriptorNoargs,
            159 => Self::CallMethodDescriptorO,
            160 => Self::CallNonPyGeneral,
            161 => Self::CallPyExactArgs,
            162 => Self::CallPyGeneral,
            163 => Self::CallStr1,
            164 => Self::CallTuple1,
            165 => Self::CallType1,
            166 => Self::CompareOpFloat,
            167 => Self::CompareOpInt,
            168 => Self::CompareOpStr,
            169 => Self::ContainsOpDict,
            170 => Self::ContainsOpSet,
            171 => Self::ForIterGen,
            172 => Self::ForIterList,
            173 => Self::ForIterRange,
            174 => Self::ForIterTuple,
            175 => Self::JumpBackwardJit,
            176 => Self::JumpBackwardNoJit,
            177 => Self::LoadAttrClass,
            178 => Self::LoadAttrClassWithMetaclassCheck,
            179 => Self::LoadAttrGetattributeOverridden,
            180 => Self::LoadAttrInstanceValue,
            181 => Self::LoadAttrMethodLazyDict,
            182 => Self::LoadAttrMethodNoDict,
            183 => Self::LoadAttrMethodWithValues,
            184 => Self::LoadAttrModule,
            185 => Self::LoadAttrNondescriptorNoDict,
            186 => Self::LoadAttrNondescriptorWithValues,
            187 => Self::LoadAttrProperty,
            188 => Self::LoadAttrSlot,
            189 => Self::LoadAttrWithHint,
            190 => Self::LoadConstImmortal,
            191 => Self::LoadConstMortal,
            192 => Self::LoadGlobalBuiltin,
            193 => Self::LoadGlobalModule,
            194 => Self::LoadSuperAttrAttr,
            195 => Self::LoadSuperAttrMethod,
            196 => Self::ResumeCheck,
            197 => Self::SendGen,
            198 => Self::StoreAttrInstanceValue,
            199 => Self::StoreAttrSlot,
            200 => Self::StoreAttrWithHint,
            201 => Self::StoreSubscrDict,
            202 => Self::StoreSubscrListInt,
            203 => Self::ToBoolAlwaysTrue,
            204 => Self::ToBoolBool,
            205 => Self::ToBoolInt,
            206 => Self::ToBoolList,
            207 => Self::ToBoolNone,
            208 => Self::ToBoolStr,
            209 => Self::UnpackSequenceList,
            210 => Self::UnpackSequenceTuple,
            211 => Self::UnpackSequenceTwoTuple,
            234 => Self::InstrumentedEndFor,
            235 => Self::InstrumentedPopIter,
            236 => Self::InstrumentedEndSend,
            237 => Self::InstrumentedForIter,
            238 => Self::InstrumentedInstruction,
            239 => Self::InstrumentedJumpForward,
            240 => Self::InstrumentedNotTaken,
            241 => Self::InstrumentedPopJumpIfTrue,
            242 => Self::InstrumentedPopJumpIfFalse,
            243 => Self::InstrumentedPopJumpIfNone,
            244 => Self::InstrumentedPopJumpIfNotNone,
            245 => Self::InstrumentedResume,
            246 => Self::InstrumentedReturnValue,
            247 => Self::InstrumentedYieldValue,
            248 => Self::InstrumentedEndAsyncFor,
            249 => Self::InstrumentedLoadSuperAttr,
            250 => Self::InstrumentedCall,
            251 => Self::InstrumentedCallKw,
            252 => Self::InstrumentedCallFunctionEx,
            253 => Self::InstrumentedJumpBackward,
            254 => Self::InstrumentedLine,
            255 => Self::EnterExecutor,
            _ => return Err(Self::Error::InvalidBytecode),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PseudoOpcode {
    AnnotationsPlaceholder,
    Jump,
    JumpIfFalse,
    JumpIfTrue,
    JumpNoInterrupt,
    LoadClosure,
    PopBlock,
    SetupCleanup,
    SetupFinally,
    SetupWith,
    StoreFastMaybeNull,
}

impl PseudoOpcode {
    /// Whether opcode ID have 'HAS_ARG_FLAG' set.
    #[must_use]
    pub const fn has_arg(self) -> bool {
        matches!(
            self,
            Self::Jump
                | Self::JumpIfFalse
                | Self::JumpIfTrue
                | Self::JumpNoInterrupt
                | Self::LoadClosure
                | Self::StoreFastMaybeNull
        )
    }

    /// Whether opcode ID have 'HAS_CONST_FLAG' set.
    #[must_use]
    pub const fn has_const(self) -> bool {
        false
    }

    /// Whether opcode ID have 'HAS_PURE_FLAG' set.
    #[must_use]
    pub const fn has_exc(self) -> bool {
        matches!(
            self,
            Self::AnnotationsPlaceholder
                | Self::LoadClosure
                | Self::PopBlock
                | Self::SetupCleanup
                | Self::SetupFinally
                | Self::SetupWith
        )
    }

    /// Whether opcode ID have 'HAS_JUMP_FLAG' set.
    #[must_use]
    pub const fn has_jump(self) -> bool {
        matches!(
            self,
            Self::Jump | Self::JumpIfFalse | Self::JumpIfTrue | Self::JumpNoInterrupt
        )
    }

    /// Whether opcode ID have 'HAS_LOCAL_FLAG' set.
    #[must_use]
    pub const fn has_local(self) -> bool {
        matches!(self, Self::LoadClosure | Self::StoreFastMaybeNull)
    }

    /// Whether opcode ID have 'HAS_NAME_FLAG' set.
    #[must_use]
    pub const fn has_name(self) -> bool {
        false
    }
}

impl fmt::Display for PseudoOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::AnnotationsPlaceholder => "ANNOTATIONS_PLACEHOLDER",
            Self::Jump => "JUMP",
            Self::JumpIfFalse => "JUMP_IF_FALSE",
            Self::JumpIfTrue => "JUMP_IF_TRUE",
            Self::JumpNoInterrupt => "JUMP_NO_INTERRUPT",
            Self::LoadClosure => "LOAD_CLOSURE",
            Self::PopBlock => "POP_BLOCK",
            Self::SetupCleanup => "SETUP_CLEANUP",
            Self::SetupFinally => "SETUP_FINALLY",
            Self::SetupWith => "SETUP_WITH",
            Self::StoreFastMaybeNull => "STORE_FAST_MAYBE_NULL",
        };
        write!(f, "{name}")
    }
}

impl TryFrom<u16> for PseudoOpcode {
    type Error = crate::marshal::MarshalError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            256 => Self::AnnotationsPlaceholder,
            257 => Self::Jump,
            258 => Self::JumpIfFalse,
            259 => Self::JumpIfTrue,
            260 => Self::JumpNoInterrupt,
            261 => Self::LoadClosure,
            262 => Self::PopBlock,
            263 => Self::SetupCleanup,
            264 => Self::SetupFinally,
            265 => Self::SetupWith,
            266 => Self::StoreFastMaybeNull,
            _ => return Err(Self::Error::InvalidBytecode),
        })
    }
}
