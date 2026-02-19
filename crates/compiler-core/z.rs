
    Cache = 0,
    BinarySlice = 1,
    BuildTemplate = 2,
    BinaryOpInplaceAddUnicode = 3,
    CallFunctionEx = 4,
    CheckEgMatch = 5,
    CheckExcMatch = 6,
    CleanupThrow = 7,
    DeleteSubscr = 8,
    EndFor = 9,
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
    LoadLocals = 22,
    MakeFunction = 23,
    MatchKeys = 24,
    MatchMapping = 25,
    MatchSequence = 26,
    Nop = 27,
    NotTaken = 28,
    PopExcept = 29,
    PopIter = 30,
    PopTop = 31,
    PushExcInfo = 32,
    PushNull = 33,
    ReturnGenerator = 34,
    ReturnValue = 35,
    SetupAnnotations = 36,
    StoreSlice = 37,
    StoreSubscr = 38,
    ToBool = 39,
    UnaryInvert = 40,
    UnaryNegative = 41,
    UnaryNot = 42,
    WithExceptStart = 43,
    BinaryOp {
        op: Arg<BinaryOperator>,
    } = 44,
    BuildInterpolation {
        oparg: Arg<u32>,
    } = 45,
    BuildList {
        size: Arg<u32>,
    } = 46,
    BuildMap {
        size: Arg<u32>,
    } = 47,
    BuildSet {
        size: Arg<u32>,
    } = 48,
    BuildSlice {
        argc: Arg<BuildSliceArgCount>,
    } = 49,
    BuildString {
        size: Arg<u32>,
    } = 50,
    BuildTuple {
        size: Arg<u32>,
    } = 51,
    Call {
        nargs: Arg<u32>,
    } = 52,
    CallIntrinsic1 {
        func: Arg<IntrinsicFunction1>,
    } = 53,
    CallIntrinsic2 {
        func: Arg<IntrinsicFunction2>,
    } = 54,
    CallKw {
        nargs: Arg<u32>,
    } = 55,
    CompareOp {
        op: Arg<ComparisonOperator>,
    } = 56,
    ContainsOp(Arg<Invert>) = 57,
    ConvertValue {
        oparg: Arg<ConvertValueOparg>,
    } = 58,
    Copy {
        index: Arg<u32>,
    } = 59,
    CopyFreeVars {
        count: Arg<u32>,
    } = 60,
    DeleteAttr {
        idx: Arg<NameIdx>,
    } = 61,
    DeleteDeref(Arg<NameIdx>) = 62,
    DeleteFast(Arg<NameIdx>) = 63,
    DeleteGlobal(Arg<NameIdx>) = 64,
    DeleteName(Arg<NameIdx>) = 65,
    DictMerge {
        index: Arg<u32>,
    } = 66,
    DictUpdate {
        index: Arg<u32>,
    } = 67,
    EndAsyncFor = 68,
    ExtendedArg = 69,
    ForIter {
        target: Arg<Label>,
    } = 70,
    GetAwaitable {
        arg: Arg<u32>,
    } = 71,
    ImportFrom {
        idx: Arg<NameIdx>,
    } = 72,
    ImportName {
        idx: Arg<NameIdx>,
    } = 73,
    IsOp(Arg<Invert>) = 74,
    JumpBackward {
        target: Arg<Label>,
    } = 75,
    JumpBackwardNoInterrupt {
        target: Arg<Label>,
    } = 76, // Placeholder
    JumpForward {
        target: Arg<Label>,
    } = 77,
    ListAppend {
        i: Arg<u32>,
    } = 78,
    ListExtend {
        i: Arg<u32>,
    } = 79,
    LoadAttr {
        idx: Arg<LoadAttr>,
    } = 80,
    LoadCommonConstant {
        idx: Arg<CommonConstant>,
    } = 81,
    LoadConst {
        idx: Arg<u32>,
    } = 82,
    LoadDeref(Arg<NameIdx>) = 83,
    LoadFast(Arg<NameIdx>) = 84,
    LoadFastAndClear(Arg<NameIdx>) = 85,
    LoadFastBorrow(Arg<NameIdx>) = 86,
    LoadFastBorrowLoadFastBorrow {
        arg: Arg<u32>,
    } = 87,
    LoadFastCheck(Arg<NameIdx>) = 88,
    LoadFastLoadFast {
        arg: Arg<u32>,
    } = 89,
    LoadFromDictOrDeref(Arg<NameIdx>) = 90,
    LoadFromDictOrGlobals(Arg<NameIdx>) = 91,
    LoadGlobal(Arg<NameIdx>) = 92,
    LoadName(Arg<NameIdx>) = 93,
    LoadSmallInt {
        idx: Arg<u32>,
    } = 94,
    LoadSpecial {
        method: Arg<SpecialMethod>,
    } = 95,
    LoadSuperAttr {
        arg: Arg<LoadSuperAttr>,
    } = 96,
    MakeCell(Arg<NameIdx>) = 97,
    MapAdd {
        i: Arg<u32>,
    } = 98,
    MatchClass(Arg<u32>) = 99,
    PopJumpIfFalse {
        target: Arg<Label>,
    } = 100,
    PopJumpIfNone {
        target: Arg<Label>,
    } = 101,
    PopJumpIfNotNone {
        target: Arg<Label>,
    } = 102,
    PopJumpIfTrue {
        target: Arg<Label>,
    } = 103,
    RaiseVarargs {
        kind: Arg<RaiseKind>,
    } = 104,
    Reraise {
        depth: Arg<u32>,
    } = 105,
    Send {
        target: Arg<Label>,
    } = 106,
    SetAdd {
        i: Arg<u32>,
    } = 107,
    SetFunctionAttribute {
        attr: Arg<MakeFunctionFlags>,
    } = 108,
    SetUpdate {
        i: Arg<u32>,
    } = 109,
    StoreAttr {
        idx: Arg<NameIdx>,
    } = 110,
    StoreDeref(Arg<NameIdx>) = 111,
    StoreFast(Arg<NameIdx>) = 112,
    StoreFastLoadFast {
        var_nums: Arg<StoreFastLoadFast>,
    } = 113,
    StoreFastStoreFast {
        arg: Arg<u32>,
    } = 114,
    StoreGlobal(Arg<NameIdx>) = 115,
    StoreName(Arg<NameIdx>) = 116,
    Swap {
        index: Arg<u32>,
    } = 117,
    UnpackEx {
        args: Arg<UnpackExArgs>,
    } = 118,
    UnpackSequence {
        size: Arg<u32>,
    } = 119,
    YieldValue {
        arg: Arg<u32>,
    } = 120,
    Resume {
        arg: Arg<u32>,
    } = 128,
    BinaryOpAddFloat = 129,                     // Placeholder
    BinaryOpAddInt = 130,                       // Placeholder
    BinaryOpAddUnicode = 131,                   // Placeholder
    BinaryOpExtend = 132,                       // Placeholder
    BinaryOpMultiplyFloat = 133,                // Placeholder
    BinaryOpMultiplyInt = 134,                  // Placeholder
    BinaryOpSubscrDict = 135,                   // Placeholder
    BinaryOpSubscrGetitem = 136,                // Placeholder
    BinaryOpSubscrListInt = 137,                // Placeholder
    BinaryOpSubscrListSlice = 138,              // Placeholder
    BinaryOpSubscrStrInt = 139,                 // Placeholder
    BinaryOpSubscrTupleInt = 140,               // Placeholder
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
    InstrumentedEndFor = 234,      // Placeholder
    InstrumentedPopIter = 235,     // Placeholder
    InstrumentedEndSend = 236,     // Placeholder
    InstrumentedForIter = 237,     // Placeholder
    InstrumentedInstruction = 238, // Placeholder
    InstrumentedJumpForward = 239, // Placeholder
    InstrumentedNotTaken = 240, // Placeholder
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
    AnnotationsPlaceholder = 256,
    Jump { target: Arg<Label> } = 257,
    JumpIfFalse { target: Arg<Label> } = 258,
    JumpIfTrue { target: Arg<Label> } = 259,
    JumpNoInterrupt { target: Arg<Label> } = 260,
    LoadClosure(Arg<NameIdx>) = 261,
    PopBlock = 262,
    SetupCleanup { target: Arg<Label> } = 263,
    SetupFinally { target: Arg<Label> } = 264,
    SetupWith { target: Arg<Label> } = 265,
    StoreFastMaybeNull(Arg<NameIdx>) = 266,
