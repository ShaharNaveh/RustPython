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


class NamedOpargType(OpargTypeMeta):
    category = OpargCategory.Named

    _enum_cls = enum.IntEnum
    _start = 0
    _val_tpl = "{}"  # TODO(3.14): Use tstrings

    @property
    @abc.abstractmethod
    def variants(self) -> dict[str, str]:
        """
        Typed oparg variants.

        Returns
        -------
        dict[str, str]
            Ordered dict where: attr_name => doc
        """
        ...

    @property
    def rust_code(self) -> str:
        lines = []
        for member, doc in self:
            if doc:
                lines.append(make_doc(doc))

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

    def __iter__(self):
        yield from zip(
            self._enum_cls(self.name, tuple(self.variants.keys()), start=self._start),
            self.variants.values(),
        )


class BuildSlice(NamedOpargType):
    """
    Specifies if a slice is built with either 2 or 3 arguments.
    """

    _start = 2
    variants = {
        "Two": """
    ```py
    x[5:10]
    ```
    """,
        "Three": """
    ```py
    x[5:10:2]`
    ```
        """,
    }


class Resume(NamedOpargType):
    variants = {
        "AtFuncStart": None,
        "AfterYield": None,
        "AfterYieldFrom": None,
        "AfterAwait": None,
    }


# TODO
"""
class Compare(NamedOpargType):
    names = ()  
"""


class BinOp(NamedOpargType):
    variants = {
        "Add": "`+`",
        "And": "`&`",
        "FloorDivide": "`//`",
        "Lshift": "`<<`",
        "MatrixMultiply": "`@`",
        "Multiply": "`*`",
        "Remainder": "`%`",
        "Or": "`|`",
        "Power": "`**`",
        "Rshift": "`>>`",
        "Subtract": "`-`",
        "TrueDivide": "`/`",
        "Xor": "`^`",
        "InplaceAdd": "`+`",
        "InplaceAnd": "`&=`",
        "InplaceFloorDivide": "`//=`",
        "InplaceLshift": "`<<=`",
        "InplaceMatrixMultiply": "`@=`",
        "InplaceMultiply": "`*=`",
        "InplaceRemainder": "`%=`",
        "InplaceOr": "`|=`",
        "InplacePower": "`**=`",
        "InplaceRshift": "`>>=`",
        "InplaceSubtract": "`-=`",
        "InplaceTrueDivide": "`/=`",
        "InplaceXor": "`^=`",
    }


class CallIntrinsic1(NamedOpargType):
    """
    [`CALL_INTRINSIC_1`]

    [CALL_INTRINSIC_1]: https://docs.python.org/3.13/library/dis.html#opcode-CALL_INTRINSIC_1
    """  # TODO: Move to Instruction

    variants = {
        # "Invalid": "Not valid",
        "Print": "Prints the argument to standard out. Used in the REPL.",
        "ImportStar": "Performs `import *` for the named module.",
        "StopIterationError": "Extracts the return value from a `StopIteration` exception.",
        "AsyncGenWrap": "Wraps an async generator value.",
        "UnaryPositive": "Performs the unary `+` operation.",
        "ListToTuple": "Converts a list to a tuple.",
        "TypeVar": """
        Creates a [`typing.TypeVar`].

        [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar,
        """,
        "ParamSpec": """
        Crates a [`typing.ParamSpec`].

        [typing.ParamSpec]: https://docs.python.org/3.13/library/typing.html#typing.ParamSpec
        """,
        "TypeVarTuple": """
        Crates a [`typing.TypeVarTuple`]

        [typing.TypeVarTuple]: https://docs.python.org/3.13/library/typing.html#typing.TypeVarTuple
        """,
        "SubscriptGeneric": "Generic subscript for [`PEP695`].",
        "TypeAlias": """
        Creates a [`typing.TypeAliasType`].

        Used in the [`type`] statement. The argument is a tuple of the type aliass name, type parameters, and value.

        [type]: https://docs.python.org/3.13/reference/simple_stmts.html#type
        [typing.TypeAliasType]: https://docs.python.org/3.13/library/typing.html#typing.TypeAliasType
        """,
    }


class CallIntrinsic2(NamedOpargType):
    """
    [`CALL_INTRINSIC_2`]

    [CALL_INTRINSIC_2]: https://docs.python.org/3.13/library/dis.html#opcode-CALL_INTRINSIC_2
    """  # TODO: Move to Instruction

    variants = {
        # "Invalid": "Not valid",
        "PrepReraiseStar": """
        Calculates the [`ExceptionGroup`] to raise from a `try-except*`.

        [ExceptionGroup]: https://docs.python.org/3.13/library/exceptions.html#ExceptionGroup
        """,
        "TypeVarWithBound": """
        Creates a [`typing.TypeVar`] with a bound.

        [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar
        """,
        "TypeVarWithConstraint": """
        Creates a [`typing.TypeVar`] with constraints.

        [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar
        """,
        "SetFunctionTypeParams": "Sets the `__type_params__` attribute of a function.",
    }


class RaiseVarArgs(NamedOpargType):
    """
    Raises an exception using one of the 3 forms of the `raise` statement.
    """

    variants = {
        "Reraise": """
    Re-Raise previous exception.

    ```py
    raise 
    ```
    """,
        "Raise": """
    Raise exception instance or type at `STACK[-1]`.

    ```py
    raise STACK[-1] 
    ```
    """,
        "RaiseCause": """
    Raise exception instance or type at `STACK[-2]` with `__cause__` set to `STACK[-1]`.

    ```py
    raise STACK[-2] from STACK[-1]
    ```
    """,
    }


class SetFunctionAttribute(NamedOpargType):
    """
    Determines which attribute to set.
    """

    _val_tpl = "{:#04x}"
    _start = 1
    _enum_cls = enum.IntFlag
    variants = {
        "Defaults": "A tuple of default values for positional-only and positional-or-keyword parameters in positional order.",
        "KwDefaults": "A dictionary of keyword-only parameters' default values.",
        "Annotations": "A tuple of strings containing parameters' annotations.",
        "Closure": "A tuple containing cells for free variables, making a closure.",
    }


class ConvertValue(NamedOpargType):
    _val_tpl = "{:#01x}"
    variants = {
        "None": "No conversion.",
        "Str": "Converts by calling `str(...)`.",
        "Repr": "Converts by calling `repr(...)`.",
        "Ascii": "Converts by calling `ascii(...)`.",
    }


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

    variants = {"No": None, "Yes": None}


class Where(NamedOpargType):
    """
    Indicates where the instruction occurs.
    """

    variants = {
        # "NoWhere": "Nowhere",
        "AfterAEnter": "After a call to `__aenter__`.",
        "AfterAExit": "After a call to `__aexit__`.",
    }


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
