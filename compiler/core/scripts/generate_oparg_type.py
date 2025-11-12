#!/usr/bin/env python
"""
Generate possible oparg types.
"""

import abc
import enum
import inspect
import pathlib
import pydoc
import textwrap
import typing

if typing.TYPE_CHECKING:
    from collections.abc import Iterable, Iterator

CRATE_ROOT = pathlib.Path(__file__).parents[1]
OUT_PATH = CRATE_ROOT / "src" / "bytecode" / "oparg_types.rs"

DERIVE = "#[derive(Clone, Copy, Debug, Eq, PartialEq)]"

REFS = {
    "CodeObject": "[crate::CodeObject]",
    "Instruction": "[crate::Instruction]",
    "PEP695": "https://peps.python.org/pep-0695/",
}


def make_doc(s: str) -> str:
    s = s.strip()

    doc = "\n".join(f"/// {line}" for line in inspect.cleandoc(s).splitlines())

    refs = "\n".join(f"/// [{ref}]: {value}" for ref, value in REFS.items() if ref in s)

    if refs:
        return f"""
{doc}
///
{refs}
""".strip()

    return doc


@enum.unique
class OpargCategory(enum.IntEnum):
    Alias = enum.auto()
    Named = enum.auto()


class OpargTypeMeta(metaclass=abc.ABCMeta):
    @property
    @abc.abstractmethod
    def category(self) -> OpargCategory:
        """
        Used to group generated opargs in the genertaed rust code.
        """
        ...

    @property
    @abc.abstractmethod
    def rust_code(self) -> str:
        """
        Rust code from class settings.
        """
        ...

    @property
    def name(self) -> str:
        """
        Gets oparg type name.
        """
        return type(self).__name__

    @property
    def doc(self) -> str:
        """
        Returns
        -------
        str
            Either the class docstring formatted for rustdoc.
            Otherwise an empty string if class has no docstring set.
        """
        if docstr := pydoc._getowndoc(type(self)):
            return docstr

        return ""


class AliasOpargType(OpargTypeMeta):
    """
    When an instruction doesn't have constraints on its oparg value.
    """

    category = OpargCategory.Alias

    @property
    def rust_code(self) -> str:
        return f"""
{make_doc(self.doc)}
{DERIVE}
pub struct {self.name}(crate::Oparg);

impl std::ops::Deref for {self.name} {{
    type Target = crate::Oparg;

    fn deref(&self) -> &Self::Target {{
        &self.0
    }}
}}
        """


class VarNum(AliasOpargType):
    """
    Used by `LoadFast*` [`Instruction`]s.
    """


class Count(AliasOpargType):
    """
    Used at instructions like:
        - [`Instruction::BuildList`]
        - [`Instruction::UnpackEx`]
    """


class NameIdx(AliasOpargType):
    """
    Index inside [`CodeObject.names`].
    """


class ConstIdx(AliasOpargType):
    """
    Index inside [`CodeObject.constants`].
    """


class Delta(AliasOpargType):
    """
    Used by `Jump*` [`Instruction`]s.
    """


@enum.unique
class DocEnum(enum.Enum):
    """
    An enum that lets you optionally specify a docstring for the enum variants.

    Examples
    --------
    >>> import enum
    >>>
    >>> class MyFlags(enum.IntEnum, DocEnum):
    >>>     Foo = enum.auto(), "Foo oparg"
    >>>     Bar = enum.auto()
    >>>     Baz = enum.auto(), ["can", "be", "anything", "really"]
    >>>
    >>> MyFlags.Foo.__doc__
    Foo oparg
    >>> MyFlags.Bar.__doc__ is None
    True
    >>> MyFlags.Baz.__doc__
    ['can', 'be', 'anything', 'really']
    """

    _ignore_ = ["_start"]
    _start = 0

    @staticmethod
    def aa_generate_next_value_(name, start, count, last_values):
        return count + _start

    def __new__(cls, value: int, doc: str | None = None):
        obj = int.__new__(cls, value)
        obj._value_ = value
        obj.__doc__ = doc
        return obj


class NamedOpargType(OpargTypeMeta):
    category = OpargCategory.Named

    _val_tpl = "{}"  # TODO(3.14): Use tstrings

    @property
    @abc.abstractmethod
    def flags(self) -> DocEnum:
        """
        Enum with variant values and optional docstrings.
        """
        ...

    @property
    def rust_code(self) -> str:
        lines = []
        for member in self.flags:
            doc = member.__doc__
            if doc:
                lines.append(make_doc(doc))
            # TODO: Make use of `_numeric_repr_`
            line = f"{member.name} = {self._val_tpl.format(member.value)},"
            lines.append(line)

        arms = "\n".join(lines)

        if not (doc := pydoc._getowndoc(type(self))):
            doc = ""
        doc = make_doc(doc)

        # TODO: Should we check if _enum_cls is IntFlag and genertae a bitflag struct instead?
        return f"""
{doc}
{DERIVE}
#[repr(u32)]
pub enum {self.name} {{
    {arms}
}}
        """


class BuildSlice(NamedOpargType):
    """
    Specifies if a slice is built with either 2 or 3 arguments.
    """

    @property
    def flags(self):
        two_doc = """
        ```py
        x[5:10]
        ```
        """

        three_doc = """
        ```py
        x[5:10:2]
        ```
        """

        class Inner(enum.IntEnum, DocEnum):
            Two = 2, two_doc
            Three = 3, three_doc

        return Inner


class Resume(NamedOpargType):
    @property
    def flags(self):
        class Inner(enum.IntEnum, DocEnum):
            AtFuncStart = enum.auto()
            AfterYield = enum.auto()
            AfterYieldFrom = enum.auto()
            AfterAwait = enum.auto()

        return Inner


# TODO
"""
class Compare(NamedOpargType):
    names = ()  
"""


class BinOp(NamedOpargType):
    @property
    def flags(self):
        class Inner(enum.IntEnum, DocEnum):
            Add = enum.auto(), "`+`"
            And = enum.auto(), "`&`"
            FloorDivide = enum.auto(), "`//`"
            Lshift = enum.auto(), "`<<`"
            MatrixMultiply = enum.auto(), "`@`"
            Multiply = enum.auto(), "`*`"
            Remainder = enum.auto(), "`%`"
            Or = enum.auto(), "`|`"
            Power = enum.auto(), "`**`"
            Rshift = enum.auto(), "`>>`"
            Subtract = enum.auto(), "`-`"
            TrueDivide = enum.auto(), "`/`"
            Xor = enum.auto(), "`^`"
            InplaceAdd = enum.auto(), "`+`"
            InplaceAnd = enum.auto(), "`&=`"
            InplaceFloorDivide = enum.auto(), "`//=`"
            InplaceLshift = enum.auto(), "`<<=`"
            InplaceMatrixMultiply = enum.auto(), "`@=`"
            InplaceMultiply = enum.auto(), "`*=`"
            InplaceRemainder = enum.auto(), "`%=`"
            InplaceOr = enum.auto(), "`|=`"
            InplacePower = enum.auto(), "`**=`"
            InplaceRshift = enum.auto(), "`>>=`"
            InplaceSubtract = enum.auto(), "`-=`"
            InplaceTrueDivide = enum.auto(), "`/=`"
            InplaceXor = enum.auto(), "`^=`"

        return Inner


class CallIntrinsic1(NamedOpargType):
    """
    [`CALL_INTRINSIC_1`]

    [CALL_INTRINSIC_1]: https://docs.python.org/3.13/library/dis.html#opcode-CALL_INTRINSIC_1
    """  # TODO: Move to Instruction

    @property
    def flags(self):
        print_doc = "Prints the argument to standard out. Used in the REPL."
        stop_iterator_error_doc = """
        Extracts the return value from a `StopIteration` exception.
        """

        type_var_doc = """
        Creates a [`typing.TypeVar`].

        [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar,
        """

        param_spec_doc = """
        Crates a [`typing.ParamSpec`].

        [typing.ParamSpec]: https://docs.python.org/3.13/library/typing.html#typing.ParamSpec
        """

        type_var_tuple_doc = """
        Crates a [`typing.TypeVarTuple`]

        [typing.TypeVarTuple]: https://docs.python.org/3.13/library/typing.html#typing.TypeVarTuple
        """

        type_alias_doc = """
        Creates a [`typing.TypeAliasType`].

        Used in the [`type`] statement. The argument is a tuple of the type aliass name, type parameters, and value.

        [type]: https://docs.python.org/3.13/reference/simple_stmts.html#type
        [typing.TypeAliasType]: https://docs.python.org/3.13/library/typing.html#typing.TypeAliasType
        """

        class Inner(enum.IntEnum, DocEnum):
            Invalid = enum.auto(), "Not valid."
            Print = enum.auto(), print_doc
            ImportStar = enum.auto(), "Performs `import *` for the named module."
            StopIterationError = enum.auto(), stop_iterator_error_doc
            AsyncGenWrap = enum.auto(), "Wraps an async generator value."
            UnaryPositive = enum.auto(), "Performs the unary `+` operation."
            ListToTuple = enum.auto(), "Converts a list to a tuple."
            TypeVar = enum.auto(), type_var_doc
            ParamSpec = enum.auto(), param_spec_doc
            TypeVarTuple = enum.auto(), type_var_tuple_doc
            SubscriptGeneric = enum.auto(), "Generic subscript for [`PEP695`]."
            TypeAlias = enum.auto(), type_alias_doc

        return Inner


class CallIntrinsic2(NamedOpargType):
    @property
    def flags(self):
        prep_reraise_star_doc = """
        Calculates the [`ExceptionGroup`] to raise from a `try-except*`.

        [ExceptionGroup]: https://docs.python.org/3.13/library/exceptions.html#ExceptionGroup
        """

        type_var_with_bound_doc = """
        Creates a [`typing.TypeVar`] with a bound.

        [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar
        """

        type_var_with_constraint_doc = """
        Creates a [`typing.TypeVar`] with constraints.

        [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar
        """

        set_function_type_params_doc = (
            "Sets the `__type_params__` attribute of a function."
        )

        class Inner(enum.IntEnum, DocEnum):
            Invalid = enum.auto(), "Not valid."
            PrepReraiseStar = enum.auto(), prep_reraise_star_doc
            TypeVarWithBound = enum.auto(), type_var_with_bound_doc
            TypeVarWithConstraint = enum.auto(), type_var_with_constraint_doc
            SetFunctionTypeParams = enum.auto(), set_function_type_params_doc

        return Inner


class RaiseVarArgs(NamedOpargType):
    """
    Raises an exception using one of the 3 forms of the `raise` statement.
    """

    @property
    def flags(self):
        reraise_doc = """
        Re-Raise previous exception.

        ```py
        raise 
        ```
        """

        raise_doc = """
        Raise exception instance or type at `STACK[-1]`.

        ```py
        raise STACK[-1] 
        ```
        """

        raise_cause_doc = """
        Raise exception instance or type at `STACK[-2]` with `__cause__` set to `STACK[-1]`.

        ```py
        raise STACK[-2] from STACK[-1]
        ```
        """

        class Inner(enum.IntEnum, DocEnum):
            Reraise = enum.auto(), reraise_doc
            Raise = enum.auto(), raise_doc
            RaiseCause = enum.auto(), raise_cause_doc

        return Inner


class SetFunctionAttribute(NamedOpargType):
    """
    Determines which attribute to set.
    """

    @property
    def flags(self):
        defaults_doc = "A tuple of default values for positional-only and positional-or-keyword parameters in positional order."
        kw_defaults_doc = "A dictionary of keyword-only parameters' default values."
        annotations_doc = "A tuple of strings containing parameters' annotations."
        closure_doc = "A tuple containing cells for free variables, making a closure."

        class Inner(enum.IntFlag, DocEnum):
            _numeric_repr_ = hex

            Defaults = enum.auto(), defaults_doc
            KwDefaults = enum.auto(), kw_defaults_doc
            Annotations = enum.auto(), annotations_doc
            Closure = enum.auto(), closure_doc

        return Inner


class ConvertValue(NamedOpargType):
    """
    Used for implementing formatted string literals (f-strings).
    """

    @property
    def flags(self):
        class Inner(enum.IntFlag, DocEnum):
            _numeric_repr_ = hex

            _None = enum.auto(), "No conversion."
            Str = enum.auto(), "Converts by calling `str(...)`."
            Repr = enum.auto(), "Converts by calling `repr(...)`."
            Ascii = enum.auto(), "Converts by calling `ascii(...)`."

        # Inner._None._name_ = "None"

        return Inner


class Invert(NamedOpargType):
    """
    When used in the context of:
    - [`Instruction::IsOp`]:
        * [`Invert::No`]: Performs `is` comparison.
        * [`Invert::Yes`]: Performs `is not` comparison.

    - [`Instruction::ContainsOp`]:
        * [`Invert::No`]: Performs `in` comparison.
        * [`Invert::Yes`]: Performs `not in` comparison.
    """  # TODO: Should be on the `Instruction::{IsOp, ContainsOp}` instead

    @property
    def flags(self):
        class Inner(enum.IntEnum, DocEnum):
            No = enum.auto()
            Yes = enum.auto()

        return Inner


class Where(NamedOpargType):
    """
    Indicates where the instruction occurs.
    """

    @property
    def flags(self):
        class Inner(enum.IntEnum, DocEnum):
            NoWhere = enum.auto(), "Nowhere"
            AfterAEnter = enum.auto(), "After a call to `__aenter__`."
            AfterAExit = enum.auto(), "After a call to `__aexit__`."

        return Inner


def main():
    script_path = pathlib.Path(__file__).absolute().relative_to(CRATE_ROOT).as_posix()

    code = "\n\n".join(
        oparg_type().rust_code.strip()
        for parent in OpargTypeMeta.__subclasses__()
        for oparg_type in sorted(
            parent.__subclasses__(), key=lambda cls: (cls.category, cls.__name__)
        )
    )

    out = f"""
//! OpargType definitions.

// This file is generated by {script_path}
// Do not edit!

{code}
    """.strip()

    OUT_PATH.write_text(out)


if __name__ == "__main__":
    main()
