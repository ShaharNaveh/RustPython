use core::{fmt, marker::PhantomData, mem};

use crate::{
    bytecode::{
        BorrowedConstant, Constant, InstrDisplayContext,
        oparg::{AnyOparg, OpArg, OpArgByte, RaiseKind},
        opcode::{AnyOpcode, Opcode, PseudoOpcode},
    },
    marshal::MarshalError,
};

/// Single bytecode instruction that is executed by the VM.
#[derive(Clone, Copy)]
pub struct Instruction {
    opcode: Opcode,
    oparg: Option<AnyOparg>,
}
impl Instruction {
    fn stack_effect(&self) -> i32 {
        // Assume that we have a valid oparg for this instruction.
        let oparg = self.oparg.map_or(0, |op| {
            i32::try_from(u32::from(op)).expect("oparg does not fit in an i32")
        });

        match self.opcode {
            Self::Nop => 0,
            Self::NotTaken => 0,
            Self::ImportName => -1,
            Self::ImportFrom => 1,
            Self::LoadFast => 1,
            Self::LoadFastBorrow => 1,
            Self::LoadFastAndClear => 1,
            Self::LoadName => 1,
            Self::LoadGlobal => 1,
            Self::LoadDeref => 1,
            Self::StoreFast => -1,
            Self::StoreName => -1,
            Self::StoreGlobal => -1,
            Self::StoreDeref => -1,
            Self::StoreFastLoadFast => 0, // pop 1, push 1
            Self::DeleteFast => 0,
            Self::DeleteName => 0,
            Self::DeleteGlobal => 0,
            Self::DeleteDeref => 0,
            Self::LoadFromDictOrDeref => 1,
            Self::StoreSubscr => -3,
            Self::DeleteSubscr => -2,
            Self::LoadAttr => 0,
            Self::StoreAttr => -2,
            Self::DeleteAttr => -1,
            Self::LoadCommonConstant => 1,
            Self::LoadConst => 1,
            Self::LoadSmallInt => 1,
            Self::LoadSpecial => 0,
            Self::Reserved => 0,
            Self::BinaryOp => -1,
            Self::CompareOp => -1,
            Self::Copy => 1,
            Self::PopTop => -1,
            Self::Swap => 0,
            Self::ToBool => 0,
            Self::GetIter => 0,
            Self::GetLen => 1,
            Self::CallIntrinsic1 => 0,  // Takes 1, pushes 1
            Self::CallIntrinsic2 => -1, // Takes 2, pushes 1
            Self::PopJumpIfTrue => -1,
            Self::PopJumpIfFalse => -1,
            Self::MakeFunction => {
                // CPython 3.14 style: MakeFunction only pops code object
                -1 + 1 // pop code, push function
            }
            Self::SetFunctionAttribute => {
                // pops attribute value and function, pushes function back
                -2 + 1
            }
            // Call: pops nargs + self_or_null + callable, pushes result
            Self::Call => -oparg - 2 + 1,
            // CallKw: pops kw_names_tuple + nargs + self_or_null + callable, pushes result
            Self::CallKw => -1 - oparg - 2 + 1,
            // CallFunctionEx: always pops kwargs_or_null + args_tuple + self_or_null + callable, pushes result
            Self::CallFunctionEx => -4 + 1,
            Self::CheckEgMatch => 0, // pops 2 (exc, type), pushes 2 (rest, match)
            Self::ConvertValue => 0,
            Self::FormatSimple => 0,
            Self::FormatWithSpec => -1,
            Self::ForIter => 1, // push next value
            Self::IsOp => -1,
            Self::ContainsOp => -1,
            Self::ReturnValue => -1,
            Self::Resume => 0,
            Self::YieldValue => 0,
            // SEND: (receiver, val) -> (receiver, retval) - no change, both paths keep same depth
            Self::Send => 0,
            // END_SEND: (receiver, value) -> (value)
            Self::EndSend => -1,
            // CLEANUP_THROW: (sub_iter, last_sent_val, exc) -> (None, value) = 3 pop, 2 push = -1
            Self::CleanupThrow => -1,
            Self::PushExcInfo => 1,   // [exc] -> [prev_exc, exc]
            Self::CheckExcMatch => 0, // [exc, type] -> [exc, bool] (pops type, pushes bool)
            Self::Reraise => 0,       // Exception raised, stack effect doesn't matter
            Self::SetupAnnotations => 0,
            Self::WithExceptStart => 1, // push __exit__ result
            Self::RaiseVarargs => {
                // Stack effects for different raise kinds:
                // - Reraise (0): gets from VM state, no stack pop
                // - Raise (1): pops 1 exception
                // - RaiseCause (2): pops 2 (exception + cause)
                // - ReraiseFromStack (3): pops 1 exception from stack
                match self
                    .oparg
                    .expect("Opcode::RaiseVarargs expects to have an oparg")
                    .expect_raise_kind()
                {
                    RaiseKind::BareRaise => 0,
                    RaiseKind::Raise => -1,
                    RaiseKind::RaiseCause => -2,
                    RaiseKind::ReraiseFromStack => -1,
                }
            }
            Self::BuildString { size } => -oparg + 1,
            Self::BuildTuple { size, .. } => -oparg + 1,
            Self::BuildList { size, .. } => -oparg + 1,
            Self::BuildSet { size, .. } => -oparg + 1,
            Self::BuildMap { size } => -(oparg * 2) + 1,
            Self::DictUpdate => -1,
            Self::DictMerge => -1,
            Self::BuildSlice => {
                // push 1
                // pops either 2/3
                // Default to Two (2 args) if arg is invalid
                1 - oparg
            }
            Self::ListAppend => -1,
            Self::ListExtend => -1,
            Self::SetAdd => -1,
            Self::SetUpdate => -1,
            Self::MapAdd => -2,
            Self::LoadBuildClass => 1,
            Self::UnpackSequence => -1 + oparg,
            Self::UnpackEx { args } => {
                let UnpackExArgs { before, after } = self
                    .oparg
                    .expect("Opcode::UnpackEx expects to have an oparg")
                    .expect_unpack_ex_args();
                -1 + before as i32 + 1 + after as i32
            }
            Self::PopExcept => -1,
            Self::PopIter => -1,
            Self::GetAwaitable => 0,
            Self::GetAIter => 0,
            Self::GetANext => 1,
            Self::EndAsyncFor => -2,  // pops (awaitable, exc) from stack
            Self::MatchMapping => 1,  // Push bool result
            Self::MatchSequence => 1, // Push bool result
            Self::MatchKeys => 1, // Pop 2 (subject, keys), push 3 (subject, keys_or_none, values_or_none)
            Self::MatchClass => -2,
            Self::ExtendedArg => 0,
            Self::UnaryInvert => 0,
            Self::UnaryNegative => 0,
            Self::UnaryNot => 0,
            Self::GetYieldFromIter => 0,
            Self::PushNull => 1, // Push NULL for call protocol
            // LoadSuperAttr: pop [super, class, self], push [attr] or [method, self_or_null]
            // stack_effect depends on load_method flag (bit 0 of oparg)
            Self::LoadSuperAttr { arg: idx } => {
                let (_, load_method, _) = decode_load_super_attr_arg(idx.get(arg));
                if load_method { -3 + 2 } else { -3 + 1 }
            }
            // Pseudo instructions (calculated before conversion)
            Self::Cache => 0,
            Self::BinarySlice => -2, // (container, start, stop -- res)
            Self::BinaryOpInplaceAddUnicode => 0,
            Self::EndFor => -1,        // pop next value at end of loop iteration
            Self::ExitInitCheck => -1, // (should_be_none -- )
            Self::InterpreterExit => 0,
            Self::LoadLocals => 1,      // ( -- locals)
            Self::ReturnGenerator => 1, // pushes None for POP_TOP to consume
            Self::StoreSlice => -4,     // (v, container, start, stop -- )
            Self::CopyFreeVars => 0,
            Self::EnterExecutor => 0,
            Self::JumpBackwardNoInterrupt => 0,
            Self::JumpBackward => 0,
            Self::JumpForward => 0,
            Self::LoadFastCheck => 0,
            Self::LoadFastLoadFast => 2,
            Self::LoadFastBorrowLoadFastBorrow => 2,
            Self::LoadFromDictOrGlobals => 0,
            Self::MakeCell => 0,
            Self::StoreFastStoreFast => 0,
            Self::PopJumpIfNone => -1,    // (value -- )
            Self::PopJumpIfNotNone => -1, // (value -- )
            Self::BinaryOpAddFloat => 0,
            Self::BinaryOpAddInt => 0,
            Self::BinaryOpAddUnicode => 0,
            Self::BinaryOpExtend => 0,
            Self::BinaryOpMultiplyFloat => 0,
            Self::BinaryOpMultiplyInt => 0,
            Self::BinaryOpSubtractFloat => 0,
            Self::BinaryOpSubtractInt => 0,
            Self::BinaryOpSubscrDict => 0,
            Self::BinaryOpSubscrGetitem => 0,
            Self::BinaryOpSubscrListInt => 0,
            Self::BinaryOpSubscrListSlice => 0,
            Self::BinaryOpSubscrStrInt => 0,
            Self::BinaryOpSubscrTupleInt => 0,
            Self::CallAllocAndEnterInit => 0,
            Self::CallBoundMethodExactArgs => 0,
            Self::CallBoundMethodGeneral => 0,
            Self::CallBuiltinClass => 0,
            Self::CallBuiltinFast => 0,
            Self::CallBuiltinFastWithKeywords => 0,
            Self::CallBuiltinO => 0,
            Self::CallIsinstance => 0,
            Self::CallKwBoundMethod => 0,
            Self::CallKwNonPy => 0,
            Self::CallKwPy => 0,
            Self::CallLen => 0,
            Self::CallListAppend => 0,
            Self::CallMethodDescriptorFast => 0,
            Self::CallMethodDescriptorFastWithKeywords => 0,
            Self::CallMethodDescriptorNoargs => 0,
            Self::CallMethodDescriptorO => 0,
            Self::CallNonPyGeneral => 0,
            Self::CallPyExactArgs => 0,
            Self::CallPyGeneral => 0,
            Self::CallStr1 => 0,
            Self::CallTuple1 => 0,
            Self::CallType1 => 0,
            Self::CompareOpFloat => 0,
            Self::CompareOpInt => 0,
            Self::CompareOpStr => 0,
            Self::ContainsOpDict => 0,
            Self::ContainsOpSet => 0,
            Self::ForIterGen => 0,
            Self::ForIterList => 0,
            Self::ForIterRange => 0,
            Self::ForIterTuple => 0,
            Self::JumpBackwardJit => 0,
            Self::JumpBackwardNoJit => 0,
            Self::LoadAttrClass => 0,
            Self::LoadAttrClassWithMetaclassCheck => 0,
            Self::LoadAttrGetattributeOverridden => 0,
            Self::LoadAttrInstanceValue => 0,
            Self::LoadAttrMethodLazyDict => 0,
            Self::LoadAttrMethodNoDict => 0,
            Self::LoadAttrMethodWithValues => 0,
            Self::LoadAttrModule => 0,
            Self::LoadAttrNondescriptorNoDict => 0,
            Self::LoadAttrNondescriptorWithValues => 0,
            Self::LoadAttrProperty => 0,
            Self::LoadAttrSlot => 0,
            Self::LoadAttrWithHint => 0,
            Self::LoadConstImmortal => 0,
            Self::LoadConstMortal => 0,
            Self::LoadGlobalBuiltin => 0,
            Self::LoadGlobalModule => 0,
            Self::LoadSuperAttrAttr => 0,
            Self::LoadSuperAttrMethod => 0,
            Self::ResumeCheck => 0,
            Self::SendGen => 0,
            Self::StoreAttrInstanceValue => 0,
            Self::StoreAttrSlot => 0,
            Self::StoreAttrWithHint => 0,
            Self::StoreSubscrDict => 0,
            Self::StoreSubscrListInt => 0,
            Self::ToBoolAlwaysTrue => 0,
            Self::ToBoolBool => 0,
            Self::ToBoolInt => 0,
            Self::ToBoolList => 0,
            Self::ToBoolNone => 0,
            Self::ToBoolStr => 0,
            Self::UnpackSequenceList => 0,
            Self::UnpackSequenceTuple => 0,
            Self::UnpackSequenceTwoTuple => 0,
            Self::InstrumentedEndFor => 0,
            Self::InstrumentedPopIter => -1,
            Self::InstrumentedEndSend => 0,
            Self::InstrumentedForIter => 0,
            Self::InstrumentedInstruction => 0,
            Self::InstrumentedJumpForward => 0,
            Self::InstrumentedNotTaken => 0,
            Self::InstrumentedJumpBackward => 0,
            Self::InstrumentedPopJumpIfTrue => 0,
            Self::InstrumentedPopJumpIfFalse => 0,
            Self::InstrumentedPopJumpIfNone => 0,
            Self::InstrumentedPopJumpIfNotNone => 0,
            Self::InstrumentedResume => 0,
            Self::InstrumentedReturnValue => 0,
            Self::InstrumentedYieldValue => 0,
            Self::InstrumentedEndAsyncFor => -2,
            Self::InstrumentedLoadSuperAttr => 0,
            Self::InstrumentedCall => 0,
            Self::InstrumentedCallKw => 0,
            Self::InstrumentedCallFunctionEx => 0,
            Self::InstrumentedLine => 0,
            // BuildTemplate: pops [strings_tuple, interpolations_tuple], pushes [template]
            Self::BuildTemplate => -1,
            // BuildInterpolation: pops [value, expr_str, format_spec?], pushes [interpolation]
            // has_format_spec is bit 0 of oparg
            Self::BuildInterpolation => {
                if oparg & 1 != 0 {
                    -2
                } else {
                    -1
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn fmt_dis(
        &self,
        arg: OpArg,
        f: &mut fmt::Formatter<'_>,
        ctx: &impl InstrDisplayContext,
        expand_code_objects: bool,
        pad: usize,
        level: usize,
    ) -> fmt::Result {
        todo!()
        /*
            macro_rules! w {
                ($variant:ident) => {
                    write!(f, stringify!($variant))
                };
                ($variant:ident, $map:ident = $arg_marker:expr) => {{
                    let arg = $arg_marker.get(arg);
                    write!(f, "{:pad$}({}, {})", stringify!($variant), arg, $map(arg))
                }};
                ($variant:ident, $arg_marker:expr) => {
                    write!(f, "{:pad$}({})", stringify!($variant), $arg_marker.get(arg))
                };
                ($variant:ident, ?$arg_marker:expr) => {
                    write!(
                        f,
                        "{:pad$}({:?})",
                        stringify!($variant),
                        $arg_marker.get(arg)
                    )
                };
            }

            let varname = |i: u32| ctx.get_varname(i as usize);
            let name = |i: u32| ctx.get_name(i as usize);
            let cell_name = |i: u32| ctx.get_cell_name(i as usize);

            let fmt_const =
                |op: &str, arg: OpArg, f: &mut fmt::Formatter<'_>, idx: &Arg<u32>| -> fmt::Result {
                    let value = ctx.get_constant(idx.get(arg) as usize);
                    match value.borrow_constant() {
                        BorrowedConstant::Code { code } if expand_code_objects => {
                            write!(f, "{op:pad$}({code:?}):")?;
                            code.display_inner(f, true, level + 1)?;
                            Ok(())
                        }
                        c => {
                            write!(f, "{op:pad$}(")?;
                            c.fmt_display(f)?;
                            write!(f, ")")
                        }
                    }
                };

            match self {
                Self::BinaryOp => write!(f, "{:pad$}({})", "BINARY_OP", op.get(arg)),
                Self::BuildList { size } => w!(BUILD_LIST, size),
                Self::BuildMap { size } => w!(BUILD_MAP, size),
                Self::BuildSet { size } => w!(BUILD_SET, size),
                Self::BuildSlice { argc } => w!(BUILD_SLICE, ?argc),
                Self::BuildString { size } => w!(BUILD_STRING, size),
                Self::BuildTuple { size } => w!(BUILD_TUPLE, size),
                Self::Call { nargs } => w!(CALL, nargs),
                Self::CallFunctionEx => w!(CALL_FUNCTION_EX),
                Self::CallKw { nargs } => w!(CALL_KW, nargs),
                Self::CallIntrinsic1 { func } => w!(CALL_INTRINSIC_1, ?func),
                Self::CallIntrinsic2 { func } => w!(CALL_INTRINSIC_2, ?func),
                Self::CheckEgMatch => w!(CHECK_EG_MATCH),
                Self::CheckExcMatch => w!(CHECK_EXC_MATCH),
                Self::CleanupThrow => w!(CLEANUP_THROW),
                Self::CompareOp => w!(COMPARE_OP, ?op),
                Self::ContainsOp(inv) => w!(CONTAINS_OP, ?inv),
                Self::ConvertValue { oparg } => write!(f, "{:pad$}{}", "CONVERT_VALUE", oparg.get(arg)),
                Self::Copy { index } => w!(COPY, index),
                Self::DeleteAttr { idx } => w!(DELETE_ATTR, name = idx),
                Self::DeleteDeref(idx) => w!(DELETE_DEREF, cell_name = idx),
                Self::DeleteFast(idx) => w!(DELETE_FAST, varname = idx),
                Self::DeleteGlobal(idx) => w!(DELETE_GLOBAL, name = idx),
                Self::DeleteName(idx) => w!(DELETE_NAME, name = idx),
                Self::DeleteSubscr => w!(DELETE_SUBSCR),
                Self::DictMerge { index } => w!(DICT_MERGE, index),
                Self::DictUpdate { index } => w!(DICT_UPDATE, index),
                Self::EndAsyncFor => w!(END_ASYNC_FOR),
                Self::EndSend => w!(END_SEND),
                Self::ExtendedArg => w!(EXTENDED_ARG, Arg::<u32>::marker()),
                Self::ForIter { target } => w!(FOR_ITER, target),
                Self::FormatSimple => w!(FORMAT_SIMPLE),
                Self::FormatWithSpec => w!(FORMAT_WITH_SPEC),
                Self::GetAIter => w!(GET_AITER),
                Self::GetANext => w!(GET_ANEXT),
                Self::GetAwaitable => w!(GET_AWAITABLE),
                Self::Reserved => w!(RESERVED),
                Self::GetIter => w!(GET_ITER),
                Self::GetLen => w!(GET_LEN),
                Self::ImportFrom { idx } => w!(IMPORT_FROM, name = idx),
                Self::ImportName { idx } => w!(IMPORT_NAME, name = idx),
                Self::IsOp(inv) => w!(IS_OP, ?inv),
                Self::JumpBackward { target } => w!(JUMP_BACKWARD, target),
                Self::JumpBackwardNoInterrupt { target } => w!(JUMP_BACKWARD_NO_INTERRUPT, target),
                Self::JumpForward { target } => w!(JUMP_FORWARD, target),
                Self::ListAppend { i } => w!(LIST_APPEND, i),
                Self::ListExtend { i } => w!(LIST_EXTEND, i),
                Self::LoadAttr { idx } => {
                    let encoded = idx.get(arg);
                    let (name_idx, is_method) = decode_load_attr_arg(encoded);
                    let attr_name = name(name_idx);
                    if is_method {
                        write!(
                            f,
                            "{:pad$}({}, {}, method=true)",
                            "LOAD_ATTR", encoded, attr_name
                        )
                    } else {
                        write!(f, "{:pad$}({}, {})", "LOAD_ATTR", encoded, attr_name)
                    }
                }
                Self::LoadBuildClass => w!(LOAD_BUILD_CLASS),
                Self::LoadFromDictOrDeref(i) => w!(LOAD_FROM_DICT_OR_DEREF, cell_name = i),
                Self::LoadConst { idx } => fmt_const("LOAD_CONST", arg, f, idx),
                Self::LoadSmallInt { idx } => w!(LOAD_SMALL_INT, idx),
                Self::LoadDeref(idx) => w!(LOAD_DEREF, cell_name = idx),
                Self::LoadFast(idx) => w!(LOAD_FAST, varname = idx),
                Self::LoadFastAndClear(idx) => w!(LOAD_FAST_AND_CLEAR, varname = idx),
                Self::LoadGlobal(idx) => w!(LOAD_GLOBAL, name = idx),
                Self::LoadName(idx) => w!(LOAD_NAME, name = idx),
                Self::LoadSpecial { method } => w!(LOAD_SPECIAL, method),
                Self::LoadSuperAttr { arg: idx } => {
                    let encoded = idx.get(arg);
                    let (name_idx, load_method, has_class) = decode_load_super_attr_arg(encoded);
                    let attr_name = name(name_idx);
                    write!(
                        f,
                        "{:pad$}({}, {}, method={}, class={})",
                        "LOAD_SUPER_ATTR", encoded, attr_name, load_method, has_class
                    )
                }
                Self::MakeFunction => w!(MAKE_FUNCTION),
                Self::MapAdd { i } => w!(MAP_ADD, i),
                Self::MatchClass(arg) => w!(MATCH_CLASS, arg),
                Self::MatchKeys => w!(MATCH_KEYS),
                Self::MatchMapping => w!(MATCH_MAPPING),
                Self::MatchSequence => w!(MATCH_SEQUENCE),
                Self::Nop => w!(NOP),
                Self::PopExcept => w!(POP_EXCEPT),
                Self::PopJumpIfFalse { target } => w!(POP_JUMP_IF_FALSE, target),
                Self::PopJumpIfTrue { target } => w!(POP_JUMP_IF_TRUE, target),
                Self::PopTop => w!(POP_TOP),
                Self::EndFor => w!(END_FOR),
                Self::PopIter => w!(POP_ITER),
                Self::PushExcInfo => w!(PUSH_EXC_INFO),
                Self::PushNull => w!(PUSH_NULL),
                Self::RaiseVarargs { kind } => w!(RAISE_VARARGS, ?kind),
                Self::Reraise { depth } => w!(RERAISE, depth),
                Self::Resume { arg } => w!(RESUME, arg),
                Self::ReturnValue => w!(RETURN_VALUE),
                Self::Send { target } => w!(SEND, target),
                Self::SetAdd { i } => w!(SET_ADD, i),
                Self::SetFunctionAttribute { attr } => w!(SET_FUNCTION_ATTRIBUTE, ?attr),
                Self::SetupAnnotations => w!(SETUP_ANNOTATIONS),
                Self::SetUpdate { i } => w!(SET_UPDATE, i),
                Self::StoreAttr { idx } => w!(STORE_ATTR, name = idx),
                Self::StoreDeref(idx) => w!(STORE_DEREF, cell_name = idx),
                Self::StoreFast(idx) => w!(STORE_FAST, varname = idx),
                Self::StoreFastLoadFast {
                    store_idx,
                    load_idx,
                } => {
                    write!(f, "STORE_FAST_LOAD_FAST")?;
                    write!(f, " ({}, {})", store_idx.get(arg), load_idx.get(arg))
                }
                Self::StoreGlobal(idx) => w!(STORE_GLOBAL, name = idx),
                Self::StoreName(idx) => w!(STORE_NAME, name = idx),
                Self::StoreSubscr => w!(STORE_SUBSCR),
                Self::Swap { index } => w!(SWAP, index),
                Self::ToBool => w!(TO_BOOL),
                Self::UnpackEx { args } => w!(UNPACK_EX, args),
                Self::UnpackSequence { size } => w!(UNPACK_SEQUENCE, size),
                Self::WithExceptStart => w!(WITH_EXCEPT_START),
                Self::UnaryInvert => w!(UNARY_INVERT),
                Self::UnaryNegative => w!(UNARY_NEGATIVE),
                Self::UnaryNot => w!(UNARY_NOT),
                Self::YieldValue { arg } => w!(YIELD_VALUE, arg),
                Self::GetYieldFromIter => w!(GET_YIELD_FROM_ITER),
                Self::BuildTemplate => w!(BUILD_TEMPLATE),
                Self::BuildInterpolation { oparg } => w!(BUILD_INTERPOLATION, oparg),
                _ => w!(RUSTPYTHON_PLACEHOLDER),
            }
        */
    }
}

/// Instructions used by the compiler. They are not executed by the VM.
#[derive(Clone, Copy, Debug)]
pub struct PseudoInstruction {
    opcode: PseudoOpcode,
    oparg: Option<AnyOparg>,
}

impl PseudoInstruction {
    fn stack_effect(&self) -> i32 {
        match self {
            Self::AnnotationsPlaceholder => 0,
            Self::Jump => 0,
            Self::JumpIfFalse => 0, // peek, don't pop: COPY + TO_BOOL + POP_JUMP_IF_FALSE
            Self::JumpIfTrue => 0,  // peek, don't pop: COPY + TO_BOOL + POP_JUMP_IF_TRUE
            Self::JumpNoInterrupt => 0,
            Self::LoadClosure => 1,
            Self::PopBlock => 0,
            Self::SetupCleanup => 0,
            Self::SetupFinally => 0,
            Self::SetupWith => 0,
            Self::StoreFastMaybeNull => -1,
            Self::LoadAttrMethod => 1, // pop obj, push method + self_or_null
            Self::LoadSuperMethod => -3 + 2, // pop 3, push [method, self_or_null]
            Self::LoadZeroSuperAttr => -3 + 1, // pop 3, push [attr]
            Self::LoadZeroSuperMethod => -3 + 2, // pop 3, push [method, self_or_null]
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AnyInstruction {
    Real(Instruction),
    Pseudo(PseudoInstruction),
}

impl From<Instruction> for AnyInstruction {
    fn from(value: Instruction) -> Self {
        Self::Real(value)
    }
}

impl From<PseudoInstruction> for AnyInstruction {
    fn from(value: PseudoInstruction) -> Self {
        Self::Pseudo(value)
    }
}

macro_rules! inst_either {
    (fn $name:ident ( &self $(, $arg:ident : $arg_ty:ty )* ) -> $ret:ty ) => {
        fn $name(&self $(, $arg : $arg_ty )* ) -> $ret {
            match self {
                Self::Real(op) => op.$name($($arg),*),
                Self::Pseudo(op) => op.$name($($arg),*),
            }
        }
    };
}

impl InstructionMetadata for AnyInstruction {
    inst_either!(fn stack_effect(&self) -> i32);

    inst_either!(fn fmt_dis(
        &self,
        arg: OpArg,
        f: &mut fmt::Formatter<'_>,
        ctx: &impl InstrDisplayContext,
        expand_code_objects: bool,
        pad: usize,
        level: usize
    ) -> fmt::Result);
}

impl AnyInstruction {
    /// Gets the inner value of [`Self::Real`].
    pub const fn real(self) -> Option<Instruction> {
        match self {
            Self::Real(ins) => Some(ins),
            _ => None,
        }
    }

    /// Gets the inner value of [`Self::Pseudo`].
    pub const fn pseudo(self) -> Option<PseudoInstruction> {
        match self {
            Self::Pseudo(ins) => Some(ins),
            _ => None,
        }
    }

    /// Same as [`Self::real`] but panics if wasn't called on [`Self::Real`].
    ///
    /// # Panics
    ///
    /// If was called on something else other than [`Self::Real`].
    pub const fn expect_real(self) -> Instruction {
        self.real()
            .expect("Expected Instruction::Real, found Instruction::Pseudo")
    }

    /// Same as [`Self::pseudo`] but panics if wasn't called on [`Self::Pseudo`].
    ///
    /// # Panics
    ///
    /// If was called on something else other than [`Self::Pseudo`].
    pub const fn expect_pseudo(self) -> PseudoInstruction {
        self.pseudo()
            .expect("Expected Instruction::Pseudo, found Instruction::Real")
    }
}
