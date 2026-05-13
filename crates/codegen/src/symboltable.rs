//! Python code is pre-scanned for symbols in the ast.
//! This ensures that global and nonlocal keywords are picked up.
//! Then the compiler can use the symbol table to generate proper
//! load and store instructions for names.
//!
//! Inspirational file: https://github.com/python/cpython/blob/main/Python/symtable.c

use ruff_python_ast as ast;

use rustpython_compiler_core::{PositionEncoding, SourceFile, SourceLocation};

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct FutureFlags: u32 {
        const CO_FUTURE_DIVISION         = 0x20000;
        /// Do absolute imports by default.
        const CO_FUTURE_ABSOLUTE_IMPORT  = 0x40000;
        const CO_FUTURE_WITH_STATEMENT   = 0x80000;
        const CO_FUTURE_PRINT_FUNCTION   = 0x100000;
        const CO_FUTURE_UNICODE_LITERALS = 0x200000;
        const CO_FUTURE_BARRY_AS_BDFL    = 0x400000;
        const CO_FUTURE_GENERATOR_STOP   = 0x800000;
        const CO_FUTURE_ANNOTATIONS      = 0x1000000;
    }
}

#[derive(Clone, Copy, Debug)]
struct FutureFeatures {
    /// Flags set by future statements.
    features: FutureFlags,
    /// Location of last future statement.
    location: SourceLocation,
}

#[derive(Debug)]
struct FutureErrror {
    msg: String,
}

impl From<String> for FutureErrror {
    fn from(msg: String) -> Self {
        Self::new(msg)
    }
}

impl From<&str> for FutureErrror {
    fn from(msg: &str) -> Self {
        String::from(msg).into()
    }
}

impl FutureErrror {
    #[must_use]
    const fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl FutureFeatures {
    fn new(
        mod_module: &ast::ModModule,
        source_location: SourceLocation,
    ) -> Result<Self, FutureError> {
        let mut features = FutureFeatures::empty();

        for stmt in &mod_module.body {
            let Some(node) = stmt.import_from_stmt() else {
                break;
            };

            let ast::ImportFrom {
                level,
                module,
                names,
                ..
            } = node;

            if level != 0 {
                break;
            }

            if module.id() != "__future__" {
                break;
            }

            for name in &names {
                features |= match name {
                    "nested_scopes" => continue,
                    "generators" => continue,
                    "division" => continue,
                    "absolute_import" => continue,
                    "with_statement" => continue,
                    "unicode_literals" => continue,
                    "barry_as_FLUFL" => FutureFlags::CO_FUTURE_BARRY_AS_BDFL,
                    "generator_stop" => continue,
                    "annotations" => FutureFlags::CO_FUTURE_ANNOTATIONS,
                    "braces" => return Err("not a chance".into()),
                    _ => {
                        return Err(format!("future feature {name} is not defined").into());
                    }
                };
            }

            // TODO: Get line number from module.range somehow
        }

        Ok(Self {
            features,
            source_location,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BlockType {
    Function,
    Class,
    Module,
    /// Used for annotations. If 'from __future__ import annotations' is active,
    /// annotation blocks cannot bind names and are not evaluated. Otherwise, they
    /// are lazily evaluated (see PEP 649).
    Annotation,
    /// The block to enter when processing a "type" (PEP 695) construction,
    /// e.g., "type MyGeneric[T] = list[T]".
    TypeAlias,
    /// The block to enter when processing a "generic" (PEP 695) object,
    /// e.g., "def foo[T](): pass" or "class A[T]: pass".
    TypeParameters,
    /// The block to enter when processing the bound, the constraint tuple
    /// or the default value of a single "type variable" in the formal sense,
    /// i.e., a TypeVar, a TypeVarTuple or a ParamSpec object (the latter two
    /// do not support a bound or a constraint tuple).
    TypeVariable,
}

#[derive(Debug)]
struct SymbolTableEntry {
    /// Key in [`SymbolTable::blocks`].
    id: usize,
    /// Variable names to flags.
    symbols: HashMap,
    /// Name of current block.
    name: String,
    /// List of function parameters.
    varnames: Option<Vec>,
    /// List of child blocks.
    children: Vec,
    /// Locations of global and nonlocal statements .
    directives: Option,
    /// Set of names for which mangling should be applied.
    mangled_names: Option<HashSet>,
    typ: BlockType,
    /// Used when reporting errors.
    /// The content of that string is a description of the current "context".
    ///
    /// For instance, if we are processing the default value of the type
    /// variable "T" in "def foo[T = int](): pass", `sscope_info` is
    /// set to `Some("a TypeVar default")`.
    scope_info: Option<String>,
    /// True if block is nested.
    is_nested: bool,
}

impl SymbolTableEntry {
    fn new(typ: BlockType, loc: SourceLocation) -> Self {
        todo!()
    }
}

#[derive(Debug)]
struct SymbolTable {
    /// Name of file being compiled, decoded from the filesystem encoding.
    filename: String,
    /// Current symbol table entry.
    cur: Option<SymbolTableEntry>,
    /// Symbol table entry for module.
    top: Option<SymbolTableEntry>,
    /// AST node addresses to symbol table entries.
    blocks: HashMap,
    /// Stack of namespace info.
    stack: Vec,
    /// Name of current class.
    private: Option<String>,
    /// Module's future features that affect the symbol table.
    future: FutureFeatures,
}

#[derive(Debug)]
struct SymbolTableBuilder {
    table: SymbolTable,
}
