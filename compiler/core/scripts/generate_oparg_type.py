#!/usr/bin/env python
"""
Generate possible oparg types.
"""

import abc
import enum
import inspect
import pathlib
import pydoc
import typing

if typing.TYPE_CHECKING:
    from collections.abc import Iterable, Iterator

CRATE_ROOT = pathlib.Path(__file__).parents[1]
OUT_PATH = CRATE_ROOT / "src" / "bytecode" / "oparg_types.rs"

DERIVE = "#[derive(Clone, Copy, Debug)]"


def make_doc(s: str) -> str:
    return "\n".join(f"/// {line}" for line in inspect.cleandoc(s).splitlines())


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
            return make_doc(docstr)

        return ""


class AliasOpargType(OpargTypeMeta):
    """
    When an instruction doesn't have constraints on its oparg value.
    """

    category = OpargCategory.Alias

    @property
    def rust_code(self) -> str:
        return f"""
{self.doc}
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
    Used by `LoadFast*` [Instruction](crate::Instruction)s.
    """


class Count(AliasOpargType):
    """
    Used for Instructions like:
        - [`Instructions::BuildList`]
        - [`Instructions::UnpackEx`]
    """


class NameIdx(AliasOpargType):
    """
    Index inside [`CodeObject.names`](crate::CodeObject.names).
    """


class ConstIdx(AliasOpargType):
    """
    Index inside [`CodeObject.constants`](crate::CodeObject.constants).
    """


class Delta(AliasOpargType):
    """
    Used for `Jump` instructions.
    """


class NamedOpargType(OpargTypeMeta):
    category = OpargCategory.Named

    _enum_cls = enum.IntEnum
    _start = 0
    _val_tpl = "{}"  # TODO(3.14): Use tstrings
    _doc_suffix = ""

    @property
    @abc.abstractmethod
    def names(self) -> tuple[str, ...]:
        """
        Oparg names.
        """
        ...

    @property
    def doc(self) -> str:
        return make_doc(
            f"Used for [Instruction::{self.name}](crate::Instruction::{self.name}).{self._doc_suffix}"
        )

    @property
    def rust_code(self) -> str:
        arms = ",\n".join(
            f"{member.name} = {self._val_tpl.format(member.value)}" for member in self
        )

        # Should we check if _enum_cls is IntFlag and genertae a bitflag struct instead?

        return f"""
{self.doc}
{DERIVE}
#[repr(u32)]
pub enum {self.name} {{
    {arms}
}}
        """

    def __iter__(self):
        yield from self._enum_cls(self.name, self.names, start=self._start)


class BuildSlice(NamedOpargType):
    names = ("Two", "Three")
    _start = 2
    _doc_suffix = """
    Specifies if a slice was built with 2 or 3 arguments.

    For example:

    ```py
    [0:10] # BuildSlice::Two
    [0:10:2] # BuildSlice::Three
    ```
    """


class Resume(NamedOpargType):
    names = ("AtFuncStart", "AfterYield", "AfterYieldFrom", "AfterAwait")


class Compare(NamedOpargType):
    names = ()  # TODO


class BinOp(NamedOpargType):
    names = (
        "Add",
        "And",
        "FloorDivide",
        "Lshift",
        "MatrixMultiply",
        "Multiply",
        "Remainder",
        "Or",
        "Power",
        "Rshift",
        "Subtract",
        "TrueDivide",
        "Xor",
        "InplaceAdd",
        "InplaceAnd",
        "InplaceFloorDivide",
        "InplaceLshift",
        "InplaceMatrixMultiply",
        "InplaceMultiply",
        "InplaceRemainder",
        "InplaceOr",
        "InplacePower",
        "InplaceRshift",
        "InplaceSubtract",
        "InplaceTrueDivide",
        "InplaceXor",
    )


class CallIntrinsic1(NamedOpargType):
    names = (
        "Print",
        "ImportStar",
        "StopIterationError",
        "AsyncGenWrap",
        "UnaryPositive",
        "ListToTuple",
        "TypeVar",
        "ParamSpec",
        "TypeVarTuple",
        "SubscriptGeneric",
        "TypeAlias",
    )


class CallIntrinsic2(NamedOpargType):
    names = (
        "Invalid",
        "PrepReraiseStar",
        "TypeVarWithBound",
        "TypeVarWithConstraint",
        "SetFunctionTypeParams",
        "SetTypeparamDefault",
    )


class RaiseVarArgs(NamedOpargType):
    names = ("Reraise", "Raise", "RaiseCause")


class RaiseVarArgs(NamedOpargType):
    names = ("Reraise", "Raise", "RaiseCause")


class SetFunctionAttribute(NamedOpargType):
    names = ("Defaults", "KwDefaults", "Annotations", "Closure")

    _val_tpl = "{:#04x}"
    _start = 1
    _enum_cls = enum.IntFlag


class ConvertValue(NamedOpargType):
    names = ("None", "Str", "Repr", "Ascii")

    _val_tpl = "{:#01x}"


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
