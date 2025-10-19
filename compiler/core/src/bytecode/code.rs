use super::{
    // AsBag,
    Bag,
    BorrowedConstant,
    Constant,
    ConstantBag,
    ConstantData,
    Oparg,
    OpargByte,
};
use crate::{OneIndexed, SourceLocation};
use bitflags::bitflags;
use itertools::Itertools;
use std::{fmt, ops::Deref};

/// CPython 3.11+ linetable location info codes
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PyCodeLocationInfoKind {
    // Short forms are 0 to 9
    Short0 = 0,
    Short1 = 1,
    Short2 = 2,
    Short3 = 3,
    Short4 = 4,
    Short5 = 5,
    Short6 = 6,
    Short7 = 7,
    Short8 = 8,
    Short9 = 9,
    // One line forms are 10 to 12
    OneLine0 = 10,
    OneLine1 = 11,
    OneLine2 = 12,
    NoColumns = 13,
    Long = 14,
    None = 15,
}

impl PyCodeLocationInfoKind {
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Short0),
            1 => Some(Self::Short1),
            2 => Some(Self::Short2),
            3 => Some(Self::Short3),
            4 => Some(Self::Short4),
            5 => Some(Self::Short5),
            6 => Some(Self::Short6),
            7 => Some(Self::Short7),
            8 => Some(Self::Short8),
            9 => Some(Self::Short9),
            10 => Some(Self::OneLine0),
            11 => Some(Self::OneLine1),
            12 => Some(Self::OneLine2),
            13 => Some(Self::NoColumns),
            14 => Some(Self::Long),
            15 => Some(Self::None),
            _ => Option::None,
        }
    }

    pub fn is_short(&self) -> bool {
        (*self as u8) <= 9
    }

    pub fn short_column_group(&self) -> Option<u8> {
        if self.is_short() {
            Some(*self as u8)
        } else {
            Option::None
        }
    }

    pub fn one_line_delta(&self) -> Option<i32> {
        match self {
            Self::OneLine0 => Some(0),
            Self::OneLine1 => Some(1),
            Self::OneLine2 => Some(2),
            _ => Option::None,
        }
    }
}

/// Primary container of a single code object. Each python function has
/// a code object. Also a module has a code object.
#[derive(Clone)]
pub struct CodeObject<C: Constant = ConstantData> {
    pub instructions: Box<[CodeUnit]>,
    pub locations: Box<[SourceLocation]>,
    pub flags: CodeFlags,
    /// Number of positional-only arguments
    pub posonlyarg_count: u32,
    pub arg_count: u32,
    pub kwonlyarg_count: u32,
    pub source_path: C::Name,
    pub first_line_number: Option<OneIndexed>,
    pub max_stackdepth: u32,
    /// Name of the object that created this code object
    pub obj_name: C::Name,
    /// Qualified name of the object (like CPython's co_qualname)
    pub qualname: C::Name,
    pub cell2arg: Option<Box<[i32]>>,
    pub constants: Box<[C]>,
    pub names: Box<[C::Name]>,
    pub varnames: Box<[C::Name]>,
    pub cellvars: Box<[C::Name]>,
    pub freevars: Box<[C::Name]>,
    /// Line number table (CPython 3.11+ format)
    pub linetable: Box<[u8]>,
    /// Exception handling table
    pub exceptiontable: Box<[u8]>,
}

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct CodeFlags: u16 {
        const NEW_LOCALS = 0x01;
        const IS_GENERATOR = 0x02;
        const IS_COROUTINE = 0x04;
        const HAS_VARARGS = 0x08;
        const HAS_VARKEYWORDS = 0x10;
        const IS_OPTIMIZED = 0x20;
    }
}

impl CodeFlags {
    pub const NAME_MAPPING: &'static [(&'static str, Self)] = &[
        ("GENERATOR", Self::IS_GENERATOR),
        ("COROUTINE", Self::IS_COROUTINE),
        (
            "ASYNC_GENERATOR",
            Self::from_bits_truncate(Self::IS_GENERATOR.bits() | Self::IS_COROUTINE.bits()),
        ),
        ("VARARGS", Self::HAS_VARARGS),
        ("VARKEYWORDS", Self::HAS_VARKEYWORDS),
    ];
}

/// Argument structure
pub struct Arguments<'a, N: AsRef<str>> {
    pub posonlyargs: &'a [N],
    pub args: &'a [N],
    pub vararg: Option<&'a N>,
    pub kwonlyargs: &'a [N],
    pub varkwarg: Option<&'a N>,
}

impl<N: AsRef<str>> fmt::Debug for Arguments<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! fmt_slice {
            ($x:expr) => {
                format_args!("[{}]", $x.iter().map(AsRef::as_ref).format(", "))
            };
        }
        f.debug_struct("Arguments")
            .field("posonlyargs", &fmt_slice!(self.posonlyargs))
            .field("args", &fmt_slice!(self.posonlyargs))
            .field("vararg", &self.vararg.map(N::as_ref))
            .field("kwonlyargs", &fmt_slice!(self.kwonlyargs))
            .field("varkwarg", &self.varkwarg.map(N::as_ref))
            .finish()
    }
}

impl<C: Constant> CodeObject<C> {
    /// Get all arguments of the code object
    /// like inspect.getargs
    pub fn arg_names(&self) -> Arguments<'_, C::Name> {
        let nargs = self.arg_count as usize;
        let nkwargs = self.kwonlyarg_count as usize;
        let mut varargs_pos = nargs + nkwargs;
        let posonlyargs = &self.varnames[..self.posonlyarg_count as usize];
        let args = &self.varnames[..nargs];
        let kwonlyargs = &self.varnames[nargs..varargs_pos];

        let vararg = if self.flags.contains(CodeFlags::HAS_VARARGS) {
            let vararg = &self.varnames[varargs_pos];
            varargs_pos += 1;
            Some(vararg)
        } else {
            None
        };
        let varkwarg = if self.flags.contains(CodeFlags::HAS_VARKEYWORDS) {
            Some(&self.varnames[varargs_pos])
        } else {
            None
        };

        Arguments {
            posonlyargs,
            args,
            vararg,
            kwonlyargs,
            varkwarg,
        }
    }

    /// Return the labels targeted by the instructions of this CodeObject
    pub fn label_targets(&self) -> BTreeSet<Label> {
        let mut label_targets = BTreeSet::new();
        let mut arg_state = OpArgState::default();
        for instruction in &*self.instructions {
            let (instruction, arg) = arg_state.get(*instruction);
            if let Some(l) = instruction.label_arg() {
                label_targets.insert(l.get(arg));
            }
        }
        label_targets
    }

    fn display_inner(
        &self,
        f: &mut fmt::Formatter<'_>,
        expand_code_objects: bool,
        level: usize,
    ) -> fmt::Result {
        let label_targets = self.label_targets();
        let line_digits = (3).max(self.locations.last().unwrap().row.to_string().len());
        let offset_digits = (4).max(self.instructions.len().to_string().len());
        let mut last_line = OneIndexed::MAX;
        let mut arg_state = OpArgState::default();
        for (offset, &instruction) in self.instructions.iter().enumerate() {
            let (instruction, arg) = arg_state.get(instruction);
            // optional line number
            let line = self.locations[offset].row;
            if line != last_line {
                if last_line != OneIndexed::MAX {
                    writeln!(f)?;
                }
                last_line = line;
                write!(f, "{line:line_digits$}")?;
            } else {
                for _ in 0..line_digits {
                    write!(f, " ")?;
                }
            }
            write!(f, " ")?;

            // level indent
            for _ in 0..level {
                write!(f, "    ")?;
            }

            /*
            // arrow and offset
            let arrow = if label_targets.contains(&Label(offset as u32)) {
                ">>"
            } else {
                "  "
            };
            */
            let arrow = ">>";
            write!(f, "{arrow} {offset:offset_digits$} ")?;

            // instruction
            instruction.fmt_dis(arg, f, self, expand_code_objects, 21, level)?;
            writeln!(f)?;
        }
        Ok(())
    }

    /// Recursively display this CodeObject
    pub fn display_expand_code_objects(&self) -> impl fmt::Display + '_ {
        struct Display<'a, C: Constant>(&'a CodeObject<C>);
        impl<C: Constant> fmt::Display for Display<'_, C> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.display_inner(f, true, 1)
            }
        }
        Display(self)
    }

    /// Map this CodeObject to one that holds a Bag::Constant
    pub fn map_bag<Bag: ConstantBag>(self, bag: Bag) -> CodeObject<Bag::Constant> {
        let map_names = |names: Box<[C::Name]>| {
            names
                .into_vec()
                .into_iter()
                .map(|x| bag.make_name(x.as_ref()))
                .collect::<Box<[_]>>()
        };
        CodeObject {
            constants: self
                .constants
                .into_vec()
                .into_iter()
                .map(|x| bag.make_constant(x.borrow_constant()))
                .collect(),
            names: map_names(self.names),
            varnames: map_names(self.varnames),
            cellvars: map_names(self.cellvars),
            freevars: map_names(self.freevars),
            source_path: bag.make_name(self.source_path.as_ref()),
            obj_name: bag.make_name(self.obj_name.as_ref()),
            qualname: bag.make_name(self.qualname.as_ref()),

            instructions: self.instructions,
            locations: self.locations,
            flags: self.flags,
            posonlyarg_count: self.posonlyarg_count,
            arg_count: self.arg_count,
            kwonlyarg_count: self.kwonlyarg_count,
            first_line_number: self.first_line_number,
            max_stackdepth: self.max_stackdepth,
            cell2arg: self.cell2arg,
            linetable: self.linetable,
            exceptiontable: self.exceptiontable,
        }
    }

    /// Same as `map_bag` but clones `self`
    pub fn map_clone_bag<Bag: ConstantBag>(&self, bag: &Bag) -> CodeObject<Bag::Constant> {
        let map_names =
            |names: &[C::Name]| names.iter().map(|x| bag.make_name(x.as_ref())).collect();
        CodeObject {
            constants: self
                .constants
                .iter()
                .map(|x| bag.make_constant(x.borrow_constant()))
                .collect(),
            names: map_names(&self.names),
            varnames: map_names(&self.varnames),
            cellvars: map_names(&self.cellvars),
            freevars: map_names(&self.freevars),
            source_path: bag.make_name(self.source_path.as_ref()),
            obj_name: bag.make_name(self.obj_name.as_ref()),
            qualname: bag.make_name(self.qualname.as_ref()),

            instructions: self.instructions.clone(),
            locations: self.locations.clone(),
            flags: self.flags,
            posonlyarg_count: self.posonlyarg_count,
            arg_count: self.arg_count,
            kwonlyarg_count: self.kwonlyarg_count,
            first_line_number: self.first_line_number,
            max_stackdepth: self.max_stackdepth,
            cell2arg: self.cell2arg.clone(),
            linetable: self.linetable.clone(),
            exceptiontable: self.exceptiontable.clone(),
        }
    }
}

impl<C: Constant> fmt::Display for CodeObject<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_inner(f, false, 1)?;
        for constant in &*self.constants {
            if let BorrowedConstant::Code { code } = constant.borrow_constant() {
                writeln!(f, "\nDisassembly of {code:?}")?;
                code.fmt(f)?;
            }
        }
        Ok(())
    }
}

pub trait InstrDisplayContext {
    type Constant: Constant;

    fn get_constant(&self, i: usize) -> &Self::Constant;

    fn get_name(&self, i: usize) -> &str;

    fn get_varname(&self, i: usize) -> &str;

    fn get_cell_name(&self, i: usize) -> &str;
}

impl<C: Constant> InstrDisplayContext for CodeObject<C> {
    type Constant = C;

    fn get_constant(&self, i: usize) -> &C {
        &self.constants[i]
    }

    fn get_name(&self, i: usize) -> &str {
        self.names[i].as_ref()
    }

    fn get_varname(&self, i: usize) -> &str {
        self.varnames[i].as_ref()
    }

    fn get_cell_name(&self, i: usize) -> &str {
        self.cellvars
            .get(i)
            .unwrap_or_else(|| &self.freevars[i - self.cellvars.len()])
            .as_ref()
    }
}

impl fmt::Display for ConstantData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.borrow_constant().fmt_display(f)
    }
}

impl<C: Constant> fmt::Debug for CodeObject<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<code object {} at ??? file {:?}, line {}>",
            self.obj_name.as_ref(),
            self.source_path.as_ref(),
            self.first_line_number.map_or(-1, |x| x.get() as i32)
        )
    }
}

#[derive(Default, Copy, Clone)]
#[repr(transparent)]
pub struct OpargState(Oparg);

impl OpargState {
    #[inline(always)]
    pub fn get(&mut self, ins: CodeUnit) -> (RealInstruction, Oparg) {
        let arg = self.extend(ins.arg);
        if ins.op != RealInstruction::ExtendedArg {
            self.reset();
        }
        (ins.op, arg)
    }

    #[inline(always)]
    pub fn extend(&mut self, arg: OpargByte) -> Oparg {
        self.0 = (self.0 << 8) | u32::from(arg);
        self.0
    }

    #[inline(always)]
    pub const fn reset(&mut self) {
        self.0 = Oparg::NULL;
    }
}

impl Deref for OpargState {
    type Target = Oparg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
