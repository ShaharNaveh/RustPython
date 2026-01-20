use core::mem;

use num_enum::TryFromPrimitive;

use crate::marshal::MarshalError;

const fn new_invalid_bytecode<T>(_: T) -> MarshalError {
    MarshalError::InvalidBytecode
}

/// VM opcodes of CPython 3.14.
///
/// ## See also
/// - [CPython opcode IDs](https://github.com/python/cpython/blob/v3.14.2/Include/opcode_ids.h)
#[derive(Clone, Copy, Debug, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = new_invalid_bytecode))]
#[repr(u8)]
pub enum Opcode {
    Cache = 0,       // Placeholder
    BinarySlice = 1, // Placeholder
    BuildTemplate = 2,
    BinaryOpInplaceAddUnicode = 3, // Placeholder
    CallFunctionEx = 4,
    CheckEgMatch = 5,
    CheckExcMatch = 6,
    CleanupThrow = 7,
    DeleteSubscr = 8,
    EndFor = 9, // Placeholder
    EndSend = 10,
    ExitInitCheck = 11, // Placeholder
    FormatSimple = 12,
    FormatWithSpec = 13,
    GetAIter = 14,
    GetANext = 15,
    GetIter = 16,
    Reserved = 17,
    GetLen = 18,
    GetYieldFromIter = 19,
    InterpreterExit = 20, // Placeholder
    LoadBuildClass = 21,
    LoadLocals = 22, // Placeholder
    MakeFunction = 23,
    MatchKeys = 24,
    MatchMapping = 25,
    MatchSequence = 26,
    Nop = 27,
    NotTaken = 28, // Placeholder
    PopExcept = 29,
    PopIter = 30, // Placeholder
    PopTop = 31,
    PushExcInfo = 32,
    PushNull = 33,
    ReturnGenerator = 34, // Placeholder
    ReturnValue = 35,
    SetupAnnotations = 36,
    StoreSlice = 37, // Placeholder
    StoreSubscr = 38,
    ToBool = 39,
    UnaryInvert = 40,
    UnaryNegative = 41,
    UnaryNot = 42,
    WithExceptStart = 43,
    BinaryOp = 44,
    /// Build an Interpolation from value, expression string, and optional format_spec on stack.
    ///
    /// oparg encoding: (conversion << 2) | has_format_spec
    /// - has_format_spec (bit 0): if 1, format_spec is on stack
    /// - conversion (bits 2+): 0=None, 1=Str, 2=Repr, 3=Ascii
    ///
    /// Stack: [value, expression_str, format_spec?] -> [interpolation]
    BuildInterpolation = 45,
    BuildList = 46,
    BuildMap = 47,
    BuildSet = 48,
    BuildSlice = 49,
    BuildString = 50,
    BuildTuple = 51,
    Call = 52,
    CallIntrinsic1 = 53,
    CallIntrinsic2 = 54,
    CallKw = 55,
    CompareOp = 56,
    ContainsOp = 57,
    ConvertValue = 58,
    Copy = 59,
    CopyFreeVars = 60,
    DeleteAttr = 61,
    DeleteDeref = 62,
    DeleteFast = 63,
    DeleteGlobal = 64,
    DeleteName = 65,
    DictMerge = 66,
    DictUpdate = 67,
    EndAsyncFor = 68,
    ExtendedArg = 69,
    ForIter = 70,
    GetAwaitable = 71,
    ImportFrom = 72,
    ImportName = 73,
    IsOp = 74,
    JumpBackward = 75,
    JumpBackwardNoInterrupt = 76,
    JumpForward = 77,
    ListAppend = 78,
    ListExtend = 79,
    LoadAttr = 80,
    LoadCommonConstant = 81,
    LoadConst = 82,
    LoadDeref = 83,
    LoadFast = 84,
    LoadFastAndClear = 85,
    LoadFastBorrow = 86,
    LoadFastBorrowLoadFastBorrow = 87,
    LoadFastCheck = 88,
    LoadFastLoadFast = 89,
    LoadFromDictOrDeref = 90,
    LoadFromDictOrGlobals = 91,
    LoadGlobal = 92,
    LoadName = 93,
    LoadSmallInt = 94,
    LoadSpecial = 95,
    LoadSuperAttr = 96,
    MakeCell = 97,
    MapAdd = 98,
    MatchClass = 99,
    PopJumpIfFalse = 100,
    PopJumpIfNone = 101,
    PopJumpIfNotNone = 102,
    PopJumpIfTrue = 103,
    RaiseVarargs = 104,
    Reraise = 105,
    Send = 106,
    SetAdd = 107,
    SetFunctionAttribute = 108,
    SetUpdate = 109,
    StoreAttr = 110,
    StoreDeref = 111,
    StoreFast = 112,
    StoreFastLoadFast = 113,
    StoreFastStoreFast = 114,
    StoreGlobal = 115,
    StoreName = 116,
    Swap = 117,
    UnpackEx = 118,
    UnpackSequence = 119,
    YieldValue = 120,
    Resume = 128,
    BinaryOpAddFloat = 129,                     // Placeholder
    BinaryOpAddInt = 130,                       // Placeholder
    BinaryOpAddUnicode = 131,                   // Placeholder
    BinaryOpExtend = 132,                       // Placeholder
    BinaryOpMultiplyFloat = 133,                // Placeholder
    BinaryOpMultiplyInt = 134,                  // Placeholder
    BinarySubscrDict = 135,                     // Placeholder
    BinarySubscrGetitem = 136,                  // Placeholder
    BinarySubscrListInt = 137,                  // Placeholder
    BinarySubscrListSlice = 138,                // Placeholder
    BinarySubscrStrInt = 139,                   // Placeholder
    BinarySubscrTupleInt = 140,                 // Placeholder
    BinaryOpSubtractFloat = 141,                // Placeholder
    BinaryOpSubtractInt = 142,                  // Placeholder
    CallAllocAndEnterInit = 143,                // Placeholder
    CallBoundMethodExactArgs = 144,             // Placeholder
    CallBoundMethodGeneral = 145,               // Placeholder
    CallBuiltinClass = 146,                     // Placeholder
    CallBuiltinFast = 147,                      // Placeholder
    CallBuiltinFastWithKeywords = 148,          // Placeholder
    CallBuiltinO = 149,                         // Placeholder
    CallIsinstance = 150,                       // Placeholder
    CallKwBoundMethod = 151,                    // Placeholder
    CallKwNonPy = 152,                          // Placeholder
    CallKwPy = 153,                             // Placeholder
    CallLen = 154,                              // Placeholder
    CallListAppend = 155,                       // Placeholder
    CallMethodDescriptorFast = 156,             // Placeholder
    CallMethodDescriptorFastWithKeywords = 157, // Placeholder
    CallMethodDescriptorNoargs = 158,           // Placeholder
    CallMethodDescriptorO = 159,                // Placeholder
    CallNonPyGeneral = 160,                     // Placeholder
    CallPyExactArgs = 161,                      // Placeholder
    CallPyGeneral = 162,                        // Placeholder
    CallStr1 = 163,                             // Placeholder
    CallTuple1 = 164,                           // Placeholder
    CallType1 = 165,                            // Placeholder
    CompareOpFloat = 166,                       // Placeholder
    CompareOpInt = 167,                         // Placeholder
    CompareOpStr = 168,                         // Placeholder
    ContainsOpDict = 169,                       // Placeholder
    ContainsOpSet = 170,                        // Placeholder
    ForIterGen = 171,                           // Placeholder
    ForIterList = 172,                          // Placeholder
    ForIterRange = 173,                         // Placeholder
    ForIterTuple = 174,                         // Placeholder
    JumpBackwardJit = 175,                      // Placeholder
    JumpBackwardNoJit = 176,                    // Placeholder
    LoadAttrClass = 177,                        // Placeholder
    LoadAttrClassWithMetaclassCheck = 178,      // Placeholder
    LoadAttrGetattributeOverridden = 179,       // Placeholder
    LoadAttrInstanceValue = 180,                // Placeholder
    LoadAttrMethodLazyDict = 181,               // Placeholder
    LoadAttrMethodNoDict = 182,                 // Placeholder
    LoadAttrMethodWithValues = 183,             // Placeholder
    LoadAttrModule = 184,                       // Placeholder
    LoadAttrNondescriptorNoDict = 185,          // Placeholder
    LoadAttrNondescriptorWithValues = 186,      // Placeholder
    LoadAttrProperty = 187,                     // Placeholder
    LoadAttrSlot = 188,                         // Placeholder
    LoadAttrWithHint = 189,                     // Placeholder
    LoadConstImmortal = 190,                    // Placeholder
    LoadConstMortal = 191,                      // Placeholder
    LoadGlobalBuiltin = 192,                    // Placeholder
    LoadGlobalModule = 193,                     // Placeholder
    LoadSuperAttrAttr = 194,                    // Placeholder
    LoadSuperAttrMethod = 195,                  // Placeholder
    ResumeCheck = 196,                          // Placeholder
    SendGen = 197,                              // Placeholder
    StoreAttrInstanceValue = 198,               // Placeholder
    StoreAttrSlot = 199,                        // Placeholder
    StoreAttrWithHint = 200,                    // Placeholder
    StoreSubscrDict = 201,                      // Placeholder
    StoreSubscrListInt = 202,                   // Placeholder
    ToBoolAlwaysTrue = 203,                     // Placeholder
    ToBoolBool = 204,                           // Placeholder
    ToBoolInt = 205,                            // Placeholder
    ToBoolList = 206,                           // Placeholder
    ToBoolNone = 207,                           // Placeholder
    ToBoolStr = 208,                            // Placeholder
    UnpackSequenceList = 209,                   // Placeholder
    UnpackSequenceTuple = 210,                  // Placeholder
    UnpackSequenceTwoTuple = 211,               // Placeholder
    // RustPython-only opcodes (212-219)
    // These either don't exist in CPython 3.14 or are RustPython-specific.
    BeforeAsyncWith = 212,
    BeforeWith = 213,
    BinarySubscr = 214,
    Break = 215,
    Continue = 216,
    JumpIfNotExcMatch = 217,
    ReturnConst = 218,
    SetExcInfo = 219,
    // End of custom opcodes
    InstrumentedEndFor = 234,           // Placeholder
    InstrumentedPopIter = 235,          // Placeholder
    InstrumentedEndSend = 236,          // Placeholder
    InstrumentedForIter = 237,          // Placeholder
    InstrumentedInstruction = 238,      // Placeholder
    InstrumentedJumpForward = 239,      // Placeholder
    InstrumentedNotTaken = 240,         // Placeholder
    InstrumentedPopJumpIfTrue = 241,    // Placeholder
    InstrumentedPopJumpIfFalse = 242,   // Placeholder
    InstrumentedPopJumpIfNone = 243,    // Placeholder
    InstrumentedPopJumpIfNotNone = 244, // Placeholder
    InstrumentedResume = 245,           // Placeholder
    InstrumentedReturnValue = 246,      // Placeholder
    InstrumentedYieldValue = 247,       // Placeholder
    InstrumentedEndAsyncFor = 248,      // Placeholder
    InstrumentedLoadSuperAttr = 249,    // Placeholder
    InstrumentedCall = 250,             // Placeholder
    InstrumentedCallKw = 251,           // Placeholder
    InstrumentedCallFunctionEx = 252,   // Placeholder
    InstrumentedJumpBackward = 253,     // Placeholder
    InstrumentedLine = 254,             // Placeholder
    EnterExecutor = 255,                // Placeholder
}

const _: () = assert!(mem::size_of::<Opcode>() == 1);

/// Opcodes that are used by the compiler. They are not executed by the VM.
#[derive(Clone, Copy, Debug, TryFromPrimitive)]
#[num_enum(error_type(name = MarshalError, constructor = new_invalid_bytecode))]
#[repr(u16)]
pub enum PseudoOpcode {
    AnnotationsPlaceholder = 256, // Placeholder
    Jump = 257,
    JumpIfFalse = 258,
    JumpIfTrue = 259,
    JumpNoInterrupt = 260, // Placeholder
    LoadClosure = 261,
    PopBlock = 262,
    SetupCleanup = 263,       // Placeholder
    SetupFinally = 264,       // Placeholder
    SetupWith = 265,          // Placeholder
    StoreFastMaybeNull = 266, // Placeholder
    // RustPython-only opcodes (267-270)
    // These either don't exist in CPython 3.14 or are RustPython-specific.
    LoadAttrMethod = 267,
    // "Zero" variants are for 0-arg super() calls (has_class=false).
    // Non-"Zero" variants are for 2-arg super(cls, self) calls (has_class=true).
    /// 2-arg super(cls, self).method() - has_class=true, load_method=true
    LoadSuperMethod = 268,
    LoadZeroSuperAttr = 269,
    LoadZeroSuperMethod = 270,
    // End of custom opcodes
}

const _: () = assert!(mem::size_of::<PseudoOpcode>() == 2);

#[derive(Clone, Copy, Debug)]
pub enum AnyOpcode {
    Real(Opcode),
    Pseudo(PseudoOpcode),
}

impl From<Opcode> for AnyOpcode {
    fn from(value: Opcode) -> Self {
        Self::Real(value)
    }
}

impl From<PseudoOpcode> for AnyOpcode {
    fn from(value: PseudoOpcode) -> Self {
        Self::Pseudo(value)
    }
}

impl TryFrom<u8> for AnyOpcode {
    type Error = MarshalError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Opcode::try_from(value)?.into())
    }
}

impl TryFrom<u16> for AnyOpcode {
    type Error = MarshalError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match PseudoOpcode::try_from(value) {
            Ok(op) => Ok(op.into()),
            Err(_) => {
                Opcode::try_from(u8::try_from(value).map_err(|_| Self::Error::InvalidBytecode)?)
                    .map(Into::into)
            }
        }
    }
}
