use crate::bytecode::generated::{Instruction, Opcode, PseudoInstruction, PseudoOpcode};

impl Instruction {
    const fn is_unconditional_jump(self) -> bool {
        matches!(
            self.opcode(),
            Opcode::JumpForward | Opcode::JumpBackward | Opcode::JumpBackwardNoInterrupt
        )
    }

    const fn is_scope_exit(self) -> bool {
        matches!(
            self.opcode(),
            Opcode::ReturnValue | Opcode::RaiseVarargs | Opcode::Reraise
        )
    }
            Self::BinaryOpSubtractFloat => (1, 2),
            Self::BinaryOpSubtractInt => (1, 2),
            Self::BinarySlice { .. } => (1, 3),
            Self::BuildInterpolation { .. } => (1, 2 + (oparg & 1)),
            Self::BuildList { .. } => (1, oparg),
            Self::BuildMap { .. } => (1, oparg * 2),
            Self::BuildSet { .. } => (1, oparg),
            Self::BuildSlice { .. } => (1, oparg),
            Self::BuildString { .. } => (1, oparg),
            Self::BuildTemplate { .. } => (1, 2),
            Self::BuildTuple { .. } => (1, oparg),
            Self::Cache => (0, 0),
            Self::Call { .. } => (1, 2 + oparg),
            Self::CallAllocAndEnterInit => (0, 2 + oparg),
            Self::CallBoundMethodExactArgs => (0, 2 + oparg),
            Self::CallBoundMethodGeneral => (0, 2 + oparg),
            Self::CallBuiltinClass => (1, 2 + oparg),
            Self::CallBuiltinFast => (1, 2 + oparg),
            Self::CallBuiltinFastWithKeywords => (1, 2 + oparg),
            Self::CallBuiltinO => (1, 2 + oparg),
            Self::CallFunctionEx => (1, 4),
            Self::CallIntrinsic1 { .. } => (1, 1),
            Self::CallIntrinsic2 { .. } => (1, 2),
            Self::CallIsinstance => (1, 2 + oparg),
            Self::CallKw { .. } => (1, 3 + oparg),
            Self::CallKwBoundMethod => (0, 3 + oparg),
            Self::CallKwNonPy => (1, 3 + oparg),
            Self::CallKwPy => (0, 3 + oparg),
            Self::CallLen => (1, 3),
            Self::CallListAppend => (0, 3),
            Self::CallMethodDescriptorFast => (1, 2 + oparg),
            Self::CallMethodDescriptorFastWithKeywords => (1, 2 + oparg),
            Self::CallMethodDescriptorNoargs => (1, 2 + oparg),
            Self::CallMethodDescriptorO => (1, 2 + oparg),
            Self::CallNonPyGeneral => (1, 2 + oparg),
            Self::CallPyExactArgs => (0, 2 + oparg),
            Self::CallPyGeneral => (0, 2 + oparg),
            Self::CallStr1 => (1, 3),
            Self::CallTuple1 => (1, 3),
            Self::CallType1 => (1, 3),
            Self::CheckEgMatch => (2, 2),
            Self::CheckExcMatch => (2, 2),
            Self::CleanupThrow => (2, 3),
            Self::CompareOp { .. } => (1, 2),
            Self::CompareOpFloat => (1, 2),
            Self::CompareOpInt => (1, 2),
            Self::CompareOpStr => (1, 2),
            Self::ContainsOp(_) => (1, 2),
            Self::ContainsOpDict => (1, 2),
            Self::ContainsOpSet => (1, 2),
            Self::ConvertValue { .. } => (1, 1),
            Self::Copy { .. } => (2 + (oparg - 1), 1 + (oparg - 1)),
            Self::CopyFreeVars { .. } => (0, 0),
            Self::DeleteAttr { .. } => (0, 1),
            Self::DeleteDeref(_) => (0, 0),
            Self::DeleteFast(_) => (0, 0),
            Self::DeleteGlobal(_) => (0, 0),
            Self::DeleteName(_) => (0, 0),
            Self::DeleteSubscr => (0, 2),
            Self::DictMerge { .. } => (4 + (oparg - 1), 5 + (oparg - 1)),
            Self::DictUpdate { .. } => (1 + (oparg - 1), 2 + (oparg - 1)),
            Self::EndAsyncFor => (0, 2),
            Self::EndFor => (0, 1),
            Self::EndSend => (1, 2),
            Self::EnterExecutor => (0, 0),
            Self::ExitInitCheck => (0, 1),
            Self::ExtendedArg => (0, 0),
            Self::ForIter { .. } => (2, 1),
            Self::ForIterGen => (1, 1),
            Self::ForIterList => (2, 1),
            Self::ForIterRange => (2, 1),
            Self::ForIterTuple => (2, 1),
            Self::FormatSimple => (1, 1),
            Self::FormatWithSpec => (1, 2),
            Self::GetAIter => (1, 1),
            Self::GetANext => (2, 1),
            Self::GetAwaitable { .. } => (1, 1),
            Self::GetIter => (1, 1),
            Self::GetLen => (2, 1),
            Self::GetYieldFromIter => (1, 1),
            Self::ImportFrom { .. } => (2, 1),
            Self::ImportName { .. } => (1, 2),
            Self::InstrumentedCall => (1, 2 + oparg),
            Self::InstrumentedCallFunctionEx => (1, 4),
            Self::InstrumentedCallKw => (1, 3 + oparg),
            Self::InstrumentedEndAsyncFor => (0, 2),
            Self::InstrumentedEndFor => (1, 2),
            Self::InstrumentedEndSend => (1, 2),
            Self::InstrumentedForIter => (2, 1),
            Self::InstrumentedInstruction => (0, 0),
            Self::InstrumentedJumpBackward => (0, 0),
            Self::InstrumentedJumpForward => (0, 0),
            Self::InstrumentedLine => (0, 0),
            Self::InstrumentedLoadSuperAttr => (1 + (oparg & 1), 3),
            Self::InstrumentedNotTaken => (0, 0),
            Self::InstrumentedPopIter => (0, 1),
            Self::InstrumentedPopJumpIfFalse => (0, 1),
            Self::InstrumentedPopJumpIfNone => (0, 1),
            Self::InstrumentedPopJumpIfNotNone => (0, 1),
            Self::InstrumentedPopJumpIfTrue => (0, 1),
            Self::InstrumentedResume => (0, 0),
            Self::InstrumentedReturnValue => (1, 1),
            Self::InstrumentedYieldValue => (1, 1),
            Self::InterpreterExit => (0, 1),
            Self::IsOp(_) => (1, 2),
            Self::JumpBackward { .. } => (0, 0),
            Self::JumpBackwardJit => (0, 0),
            Self::JumpBackwardNoInterrupt { .. } => (0, 0),
            Self::JumpBackwardNoJit => (0, 0),
            Self::JumpForward { .. } => (0, 0),
            Self::ListAppend { .. } => (1 + (oparg - 1), 2 + (oparg - 1)),
            Self::ListExtend { .. } => (1 + (oparg - 1), 2 + (oparg - 1)),
            Self::LoadAttr { .. } => (1 + (oparg & 1), 1),
            Self::LoadAttrClass => (1 + (oparg & 1), 1),
            Self::LoadAttrClassWithMetaclassCheck => (1 + (oparg & 1), 1),
            Self::LoadAttrGetattributeOverridden => (1, 1),
            Self::LoadAttrInstanceValue => (1 + (oparg & 1), 1),
            Self::LoadAttrMethodLazyDict => (2, 1),
            Self::LoadAttrMethodNoDict => (2, 1),
            Self::LoadAttrMethodWithValues => (2, 1),
            Self::LoadAttrModule => (1 + (oparg & 1), 1),
            Self::LoadAttrNondescriptorNoDict => (1, 1),
            Self::LoadAttrNondescriptorWithValues => (1, 1),
            Self::LoadAttrProperty => (0, 1),
            Self::LoadAttrSlot => (1 + (oparg & 1), 1),
            Self::LoadAttrWithHint => (1 + (oparg & 1), 1),
            Self::LoadBuildClass => (1, 0),
            Self::LoadCommonConstant { .. } => (1, 0),
            Self::LoadConst { .. } => (1, 0),
            Self::LoadConstImmortal => (1, 0),
            Self::LoadConstMortal => (1, 0),
            Self::LoadDeref(_) => (1, 0),
            Self::LoadFast(_) => (1, 0),
            Self::LoadFastAndClear(_) => (1, 0),
            Self::LoadFastBorrow(_) => (1, 0),
            Self::LoadFastBorrowLoadFastBorrow { .. } => (2, 0),
            Self::LoadFastCheck(_) => (1, 0),
            Self::LoadFastLoadFast { .. } => (2, 0),
            Self::LoadFromDictOrDeref(_) => (1, 1),
            Self::LoadFromDictOrGlobals(_) => (1, 1),
            Self::LoadGlobal(_) => (
                1, // TODO: Differs from CPython `1 + (oparg & 1)`
                0,
            ),
            Self::LoadGlobalBuiltin => (1 + (oparg & 1), 0),
            Self::LoadGlobalModule => (1 + (oparg & 1), 0),
            Self::LoadLocals => (1, 0),
            Self::LoadName(_) => (1, 0),
            Self::LoadSmallInt { .. } => (1, 0),
            Self::LoadSpecial { .. } => (1, 1),
            Self::LoadSuperAttr { .. } => (1 + (oparg & 1), 3),
            Self::LoadSuperAttrAttr => (1, 3),
            Self::LoadSuperAttrMethod => (2, 3),
            Self::MakeCell(_) => (0, 0),
            Self::MakeFunction { .. } => (1, 1),
            Self::MapAdd { .. } => (1 + (oparg - 1), 3 + (oparg - 1)),
            Self::MatchClass { .. } => (1, 3),
            Self::MatchKeys { .. } => (3, 2),
            Self::MatchMapping => (2, 1),
            Self::MatchSequence => (2, 1),
            Self::Nop => (0, 0),
            Self::NotTaken => (0, 0),
            Self::PopExcept => (0, 1),
            Self::PopIter => (0, 1),
            Self::PopJumpIfFalse { .. } => (0, 1),
            Self::PopJumpIfNone { .. } => (0, 1),
            Self::PopJumpIfNotNone { .. } => (0, 1),
            Self::PopJumpIfTrue { .. } => (0, 1),
            Self::PopTop => (0, 1),
            Self::PushExcInfo => (2, 1),
            Self::PushNull => (1, 0),
            Self::RaiseVarargs { kind } => (
                0,
                // TODO: Differs from CPython: `oparg`
                match kind.get((oparg as u32).into()) {
                    RaiseKind::BareRaise => 0,
                    RaiseKind::Raise => 1,
                    RaiseKind::RaiseCause => 2,
                    RaiseKind::ReraiseFromStack => 1,
                },
            ),
            Self::Reraise { .. } => (
                1 + oparg, // TODO: Differs from CPython: `oparg`
                1 + oparg,
            ),
            Self::Reserved => (0, 0),
            Self::Resume { .. } => (0, 0),
            Self::ResumeCheck => (0, 0),
            Self::ReturnGenerator => (1, 0),
            Self::ReturnValue => (
                0, // TODO: Differs from CPython: `1`
                1,
            ),
            Self::Send { .. } => (2, 2),
            Self::SendGen => (1, 2),
            Self::SetAdd { .. } => (1 + (oparg - 1), 2 + (oparg - 1)),
            Self::SetFunctionAttribute { .. } => (1, 2),
            Self::SetUpdate { .. } => (1 + (oparg - 1), 2 + (oparg - 1)),
            Self::SetupAnnotations => (0, 0),
            Self::StoreAttr { .. } => (0, 2),
            Self::StoreAttrInstanceValue => (0, 2),
            Self::StoreAttrSlot => (0, 2),
            Self::StoreAttrWithHint => (0, 2),
            Self::StoreDeref(_) => (0, 1),
            Self::StoreFast(_) => (0, 1),
            Self::StoreFastLoadFast { .. } => (1, 1),
            Self::StoreFastStoreFast { .. } => (0, 2),
            Self::StoreGlobal(_) => (0, 1),
            Self::StoreName(_) => (0, 1),
            Self::StoreSlice => (0, 4),
            Self::StoreSubscr => (0, 3),
            Self::StoreSubscrDict => (0, 3),
            Self::StoreSubscrListInt => (0, 3),
            Self::Swap { .. } => (2 + (oparg - 2), 2 + (oparg - 2)),
            Self::ToBool => (1, 1),
            Self::ToBoolAlwaysTrue => (1, 1),
            Self::ToBoolBool => (1, 1),
            Self::ToBoolInt => (1, 1),
            Self::ToBoolList => (1, 1),
            Self::ToBoolNone => (1, 1),
            Self::ToBoolStr => (1, 1),
            Self::UnaryInvert => (1, 1),
            Self::UnaryNegative => (1, 1),
            Self::UnaryNot => (1, 1),
            Self::UnpackEx { .. } => (1 + (oparg & 0xFF) + (oparg >> 8), 1),
            Self::UnpackSequence { .. } => (oparg, 1),
            Self::UnpackSequenceList => (oparg, 1),
            Self::UnpackSequenceTuple => (oparg, 1),
            Self::UnpackSequenceTwoTuple => (2, 1),
            Self::WithExceptStart => (6, 5),
            Self::YieldValue { .. } => (1, 1),
        };

        debug_assert!((0..=i32::MAX).contains(&pushed));
        debug_assert!((0..=i32::MAX).contains(&popped));

        StackEffect::new(pushed as u32, popped as u32)
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
            Self::BinaryOp { op } => write!(f, "{:pad$}({})", "BINARY_OP", op.get(arg)),
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
            Self::CompareOp { op } => w!(COMPARE_OP, ?op),
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
            Self::GetAwaitable { arg } => w!(GET_AWAITABLE, arg),
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
                let oparg = idx.get(arg);
                let oparg_u32 = u32::from(oparg);
                let attr_name = name(oparg.name_idx());
                if oparg.is_method() {
                    write!(
                        f,
                        "{:pad$}({}, {}, method=true)",
                        "LOAD_ATTR", oparg_u32, attr_name
                    )
                } else {
                    write!(f, "{:pad$}({}, {})", "LOAD_ATTR", oparg_u32, attr_name)
                }
            }
            Self::LoadBuildClass => w!(LOAD_BUILD_CLASS),
            Self::LoadFromDictOrDeref(i) => w!(LOAD_FROM_DICT_OR_DEREF, cell_name = i),
            Self::LoadConst { idx } => fmt_const("LOAD_CONST", arg, f, idx),
            Self::LoadSmallInt { idx } => w!(LOAD_SMALL_INT, idx),
            Self::LoadDeref(idx) => w!(LOAD_DEREF, cell_name = idx),
            Self::LoadFast(idx) => w!(LOAD_FAST, varname = idx),
            Self::LoadFastAndClear(idx) => w!(LOAD_FAST_AND_CLEAR, varname = idx),
            Self::LoadFastBorrow(idx) => w!(LOAD_FAST_BORROW, varname = idx),
            Self::LoadFastCheck(idx) => w!(LOAD_FAST_CHECK, varname = idx),
            Self::LoadFastLoadFast { arg: packed } => {
                let oparg = packed.get(arg);
                let idx1 = oparg >> 4;
                let idx2 = oparg & 15;
                let name1 = varname(idx1);
                let name2 = varname(idx2);
                write!(f, "{:pad$}({}, {})", "LOAD_FAST_LOAD_FAST", name1, name2)
            }
            Self::LoadFastBorrowLoadFastBorrow { arg: packed } => {
                let oparg = packed.get(arg);
                let idx1 = oparg >> 4;
                let idx2 = oparg & 15;
                let name1 = varname(idx1);
                let name2 = varname(idx2);
                write!(
                    f,
                    "{:pad$}({}, {})",
                    "LOAD_FAST_BORROW_LOAD_FAST_BORROW", name1, name2
                )
            }
            Self::LoadFromDictOrGlobals(idx) => w!(LOAD_FROM_DICT_OR_GLOBALS, name = idx),
            Self::LoadGlobal(idx) => w!(LOAD_GLOBAL, name = idx),
            Self::LoadName(idx) => w!(LOAD_NAME, name = idx),
            Self::LoadSpecial { method } => w!(LOAD_SPECIAL, method),
            Self::LoadSuperAttr { arg: idx } => {
                let oparg = idx.get(arg);
                write!(
                    f,
                    "{:pad$}({}, {}, method={}, class={})",
                    "LOAD_SUPER_ATTR",
                    u32::from(oparg),
                    name(oparg.name_idx()),
                    oparg.is_load_method(),
                    oparg.has_class()
                )
            }
            Self::MakeFunction => w!(MAKE_FUNCTION),
            Self::MapAdd { i } => w!(MAP_ADD, i),
            Self::MatchClass(arg) => w!(MATCH_CLASS, arg),
            Self::MatchKeys => w!(MATCH_KEYS),
            Self::MatchMapping => w!(MATCH_MAPPING),
            Self::MatchSequence => w!(MATCH_SEQUENCE),
            Self::Nop => w!(NOP),
            Self::NotTaken => w!(NOT_TAKEN),
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
            Self::ReturnGenerator => w!(RETURN_GENERATOR),
            Self::Send { target } => w!(SEND, target),
            Self::SetAdd { i } => w!(SET_ADD, i),
            Self::SetFunctionAttribute { attr } => w!(SET_FUNCTION_ATTRIBUTE, ?attr),
            Self::SetupAnnotations => w!(SETUP_ANNOTATIONS),
            Self::SetUpdate { i } => w!(SET_UPDATE, i),
            Self::StoreAttr { idx } => w!(STORE_ATTR, name = idx),
            Self::StoreDeref(idx) => w!(STORE_DEREF, cell_name = idx),
            Self::StoreFast(idx) => w!(STORE_FAST, varname = idx),
            Self::StoreFastLoadFast { var_nums } => {
                let oparg = var_nums.get(arg);
                let store_idx = oparg.store_idx();
                let load_idx = oparg.load_idx();
                write!(f, "STORE_FAST_LOAD_FAST")?;
                write!(f, " ({}, {})", store_idx, load_idx)
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
    }
}

/// Instructions used by the compiler. They are not executed by the VM.
///
/// CPython 3.14.2 aligned (256-266).
#[derive(Clone, Copy, Debug)]
#[repr(u16)]
pub enum PseudoInstruction {
    // CPython 3.14.2 pseudo instructions (256-266)
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
}

const _: () = assert!(mem::size_of::<PseudoInstruction>() == 2);

impl From<PseudoInstruction> for u16 {
    #[inline]
    fn from(ins: PseudoInstruction) -> Self {
        // SAFETY: there's no padding bits
        unsafe { mem::transmute::<PseudoInstruction, Self>(ins) }
    }
}

impl TryFrom<u16> for PseudoInstruction {
    type Error = MarshalError;

    #[inline]
    fn try_from(value: u16) -> Result<Self, MarshalError> {
        let start = u16::from(Self::AnnotationsPlaceholder);
        let end = u16::from(Self::StoreFastMaybeNull(Arg::marker()));

        if (start..=end).contains(&value) {
            Ok(unsafe { mem::transmute::<u16, Self>(value) })
        } else {
            Err(Self::Error::InvalidBytecode)
        }
    }
>>>>>>> upstream/main
}

impl PseudoInstruction {
    const fn is_scope_exit(&self) -> bool {
        false
    }

    const fn is_unconditional_jump(&self) -> bool {
        matches!(
            self.opcode(),
            PseudoOpcode::Jump | PseudoOpcode::JumpNoInterrupt
        )
    }

    /// Returns true if this is a block push pseudo instruction
    /// (SETUP_FINALLY, SETUP_CLEANUP, or SETUP_WITH).
    pub const fn is_block_push(&self) -> bool {
        matches!(
            self.opcode(),
            PseudoOpcode::SetupCleanup | PseudoOpcode::SetupFinally | PseudoOpcode::SetupWith
        )
    }
}

#[derive(Clone, Copy)]
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
    // `const fn`
    ($vis:vis const fn $name:ident ( &self $(, $arg:ident : $arg_ty:ty )* ) -> $ret:ty ) => {
      #[must_use]
      $vis const fn $name(self $(, $arg : $arg_ty )* ) -> $ret {
            match self {
                Self::Real(instr) => instr.$name($($arg),*),
                Self::Pseudo(instr) => instr.$name($($arg),*),
            }
        }
    };

    // `fn`
    ($vis:vis fn $name:ident ( &self $(, $arg:ident : $arg_ty:ty )* ) -> $ret:ty ) => {
      #[must_use]
      $vis fn $name(self $(, $arg : $arg_ty )* ) -> $ret {
            match self {
                Self::Real(instr) => instr.$name($($arg),*),
                Self::Pseudo(instr) => instr.$name($($arg),*),
            }
        }
    };
}

impl AnyInstruction {
    inst_either!(pub const fn label_oparg(&self) -> Option<crate::bytecode::oparg::Label>);

    inst_either!(pub const fn is_unconditional_jump(&self) -> bool);

    inst_either!(pub const fn is_scope_exit(&self) -> bool);

    inst_either!(pub fn stack_effect(&self) -> i32);

    inst_either!(pub fn stack_effect_info(&self) -> StackEffect);

    inst_either!(pub fn fmt_dis(
        &self,
        f: &mut core::fmt::Formatter<'_>,
        ctx: &impl crate::bytecode::InstrDisplayContext,
        expand_code_objects: bool,
        pad: usize,
        level: usize
    ) -> core::fmt::Result);

    pub fn display(
        &self,
        ctx: &impl crate::bytecode::InstrDisplayContext,
    ) -> impl core::fmt::Display {
        match self {
            Self::Real(instr) => instr.display(ctx),
            Self::Pseudo(_) => unimplemented!(),
        }
    }
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

    /// Returns true if this is a block push pseudo instruction
    /// (SETUP_FINALLY, SETUP_CLEANUP, or SETUP_WITH).
    pub const fn is_block_push(self) -> bool {
        matches!(self, Self::Pseudo(instr) if instr.is_block_push())
    }

    /// Returns true if this is a POP_BLOCK pseudo instruction.
    pub const fn is_pop_block(self) -> bool {
        matches!(self, Self::Pseudo(PseudoInstruction::PopBlock))
    }
}

/// What effect the instruction has on the stack.
#[derive(Clone, Copy)]
pub struct StackEffect {
    /// How many items the instruction is pushing on the stack.
    pushed: u32,
    /// How many items the instruction is popping from the stack.
    popped: u32,
}

impl StackEffect {
    /// Creates a new [`Self`].
    pub const fn new(pushed: u32, popped: u32) -> Self {
        Self { pushed, popped }
    }

    /// Get the calculated stack effect as [`i32`].
    pub const fn effect(self) -> i32 {
        (self.pushed() as i32) - (self.popped() as i32)
    }

    /// Get the pushed count.
    pub const fn pushed(self) -> u32 {
        self.pushed
    }

    /// Get the popped count.
    pub const fn popped(self) -> u32 {
        self.popped
    }
}

impl From<StackEffect> for i32 {
    fn from(stack_effect: StackEffect) -> Self {
        stack_effect.effect()
    }
}
