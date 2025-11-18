#!/usr/bin/env python
"""
Generate the `Instruction` enum by using CPython internal tools.
"""

import abc
import collections
import enum
import functools
import itertools
import pathlib
import subprocess
import sys
import typing

if typing.TYPE_CHECKING:
    from collections.abc import Iterable, Iterator

CRATE_ROOT = pathlib.Path(__file__).parent
OUT_PATH = CRATE_ROOT / "src" / "bytecode" / "instructions.rs"

DERIVE = "#[derive(Clone, Copy, Debug, Eq, PartialEq)]"

CPYTHON_PATH = (
    pathlib.Path(__file__).parents[3] / "cpython"  # Local filesystem path of cpython
)
_cases_generator_path = CPYTHON_PATH / "Tools" / "cases_generator"
sys.path.append(str(_cases_generator_path))

import analyzer
from generators_common import DEFAULT_INPUT
from stack import StackOffset, get_stack_effect


def _var_size(var):
    """
    Adapted from https://github.com/python/cpython/blob/bcee1c322115c581da27600f2ae55e5439c027eb/Tools/cases_generator/stack.py#L24-L36
    """
    if var.condition:
        if var.condition == "0":
            return "0"
        elif var.condition == "1":
            return var.size
        elif var.condition == "oparg & 1" and var.size == "1":
            return f"({var.condition})"
        else:
            return f"(if {var.condition} {{ {var.size} }} else {{ 0 }})"
    else:
        return var.size


StackOffset.pop = lambda self, item: self.popped.append(_var_size(item))
StackOffset.push = lambda self, item: self.pushed.append(_var_size(item))


def detect_cpython_version(
    readme_path: pathlib.Path = (CPYTHON_PATH / "README.rst"),
) -> str:
    """
    Try to find cpython version based on given path.

    Parameters
    ----------
    readme_path : pathlib.Path
        Local file path to the `README.rst` file.

    Returns
    -------
    str
        CPython version.
    """
    with readme_path.open() as fd:
        header = fd.readline()
    return header.split()[-1].strip()


def rustfmt(code: str) -> str:
    return subprocess.check_output(["rustfmt", "--emit=stdout"], input=code, text=True)


@enum.unique
class Oparg(enum.StrEnum):
    NameIdx = enum.auto()
    ConstIdx = enum.auto()
    Delta = enum.auto()
    Raw = enum.auto()  # Alias for Oparg
    # Opcode specific
    Resume = enum.auto()
    Compare = enum.auto()
    BinaryOperator = enum.auto()
    RaiseVarArgs = enum.auto()
    CallIntrinsic1 = enum.auto()
    CallIntrinsic2 = enum.auto()
    UnpackEx = enum.auto()

    @classmethod
    def try_from_instruction(
        cls, inst: "analyzer.Instruction | analyzer.PseudoInstruction]"
    ) -> typing.Self | None:
        prop = inst.properties
        if not prop.oparg:
            return

        # Override oparg type for some opcodes
        match inst.name:
            case "BINARY_OP":
                return cls.BinaryOperator
            case "RESUME":
                return cls.Resume
            case "RAISE_VARARGS":
                return cls.RaiseVarArgs
            case "CALL_INTRINSIC_1":
                return cls.CallIntrinsic1
            case "CALL_INTRINSIC_2":
                return cls.CallIntrinsic2
            case "UNPACK_EX":
                return cls.UnpackEx

        # Should we have a different oparg when `prop.has_free or `prop.uses_locals`?
        if prop.jumps:
            return cls.Delta
        elif prop.uses_co_consts:
            return cls.ConstIdx
        elif prop.uses_co_names:
            return cls.NameIdx
        else:
            return cls.Raw

    def __format__(self, spec) -> str:
        prefix = ""
        if self.name != "Raw":
            prefix = self.name

        return f"crate::{prefix}Oparg"


class Inst:
    def __init__(
        self, inst: "analyzer.Instruction | analyzer.PseudoInstruction]"
    ) -> None:
        self._inner = inst

    @property
    def name(self) -> str:
        return self._inner.name.title().replace("_", "")

    @property
    def deopt(self) -> str:
        if (family := self.family) is None:
            return self.name

        return family.name.title().replace("_", "")

    def oparg(self, table: dict[str, typing.Self]) -> Oparg | None:
        """
        Parameters
        ----------
        table : dict
            Deopts table.
        """
        # We might have a specilized opcode that needs to have the oparg the same as what it would deopt into
        if (deopt_inst := table.get(self.name)) is not None:
            return deopt_inst.oparg(table)

        return Oparg.try_from_instruction(self._inner)

    @property
    def properties(self):
        return self._inner.properties

    @property
    def family(self):
        return getattr(self._inner, "family", None)

    @property
    def parts(self):
        return self._inner.parts

    def as_match_arm(self, table: dict[str, typing.Self], value: str = "_") -> str:
        """
        Parameters
        ----------
        table : dict
            Deopts table.
        """
        out = self.name
        if self.oparg(table):
            out += f"({value})"
        return f"Self::{out}"


class InstructionsMeta(metaclass=abc.ABCMeta):
    def __init__(self, analysis: analyzer.Analysis) -> None:
        self._analysis = analysis

    @abc.abstractmethod
    def __iter__(self) -> "Iterator[Inst]": ...

    @property
    @abc.abstractmethod
    def opcode_size(self) -> str:
        """
        Opcode numeric type (u8/u16/u32/etc)
        """
        ...

    @property
    @abc.abstractmethod
    def enum_name(self) -> str: ...

    @functools.cached_property
    def name_table(self) -> dict[str, Inst]:
        return {inst.name: inst for inst in self}

    @functools.cached_property
    def deopt_table(self) -> dict[str, Inst]:
        return {
            inst.name: self.name_table[inst.deopt]
            for inst in self
            if inst.name != inst.deopt
        }

    @property
    def rust_code(self) -> str:
        enum_variants = []

        for inst in self:
            name = inst.name
            opcode = self._analysis.opmap[inst._inner.name]
            oparg = inst.oparg(self.deopt_table)

            if oparg is None:
                enum_variants.append(f"{name} = {opcode}")
            else:
                enum_variants.append(f"{name}(OpargType<{oparg}>) = {opcode}")

        enum_variant_defs = ",\n".join(enum_variants)
        funcs = "\n\n".join(
            getattr(self, attr).strip()
            for attr in sorted(dir(self))
            if attr.startswith("fn_")
        )
        impls = "\n\n".join(
            getattr(self, attr).strip()
            for attr in sorted(dir(self))
            if attr.startswith("impl_")
        )

        return f"""
{DERIVE}
#[repr({self.opcode_size})]
pub enum {self.enum_name} {{
    {enum_variant_defs}
}}

impl {self.enum_name} {{
    {funcs}
}}

{impls}
        """.strip()

    def build_has_attr_fn(self, fn_attr: str, prop_attr: str, doc_flag: str):
        arms = "|".join(
            inst.as_match_arm(self.deopt_table)
            for inst in self
            if getattr(inst.properties, prop_attr)
        )
        if arms:
            inner = f"matches!(self, {arms})"
        else:
            inner = "false"

        return f"""
/// Whether opcode ID have '{doc_flag}' set.
#[must_use]
pub const fn has_{fn_attr}(self) -> bool {{
    {inner}
}}
        """

    fn_has_arg = property(
        lambda self: self.build_has_attr_fn("arg", "oparg", "HAS_ARG_FLAG")
    )
    fn_has_const = property(
        lambda self: self.build_has_attr_fn("const", "uses_co_consts", "HAS_CONST_FLAG")
    )
    fn_has_name = property(
        lambda self: self.build_has_attr_fn("name", "uses_co_names", "HAS_NAME_FLAG")
    )
    fn_has_jump = property(
        lambda self: self.build_has_attr_fn("jump", "jumps", "HAS_JUMP_FLAG")
    )
    fn_has_free = property(
        lambda self: self.build_has_attr_fn("free", "has_free", "HAS_FREE_FLAG")
    )
    fn_has_local = property(
        lambda self: self.build_has_attr_fn("local", "uses_locals", "HAS_LOCAL_FLAG")
    )
    fn_has_exc = property(
        lambda self: self.build_has_attr_fn("exc", "pure", "HAS_PURE_FLAG")
    )


class RealInstructions(InstructionsMeta):
    enum_name = "RealInstruction"
    opcode_size = "u8"

    def __iter__(self) -> "Iterator[analyzer.Instruction | analyzer.PseudoInstruction]":
        yield from map(
            Inst,
            sorted(
                itertools.chain(
                    self._analysis.instructions.values(),
                    [analyzer.Instruction("INSTRUMENTED_LINE", [], None)],
                ),
                key=lambda inst: inst.name,
            ),
        )

    def _generate_stack_effect(self, direction: str) -> str:
        """
        Adapted from https://github.com/python/cpython/blob/bcee1c322115c581da27600f2ae55e5439c027eb/Tools/cases_generator/stack.py#L89-L111
        """

        lines = []
        for inst in self:
            stack = get_stack_effect(inst)
            if direction == "popped":
                val = -stack.base_offset
            elif direction == "pushed":
                val = stack.top_offset - stack.base_offset

            expr = val.to_c().replace("oparg", "*oparg")
            left_side = inst.as_match_arm(self.deopt_table)
            line = f"{left_side} => {expr}"
            lines.append(line)

        arms = ",\n".join(lines)
        doc = "from" if direction == "popped" else "on"
        return f"""
/// How many items should be {direction} {doc} the stack.
pub(crate) const fn num_{direction}(self, oparg: Oparg) -> u32 {{
    match self {{
        {arms}
    }}
}}
        """

    @property
    def fn_num_popped(self) -> str:
        return self._generate_stack_effect("popped")

    @property
    def fn_num_pushed(self) -> str:
        return self._generate_stack_effect("pushed")

    @property
    def fn_deopt(self) -> str:
        VALUE = "typ"
        lines = []
        for deopt, inst in self.deopt_table.items():
            left = self.name_table[deopt].as_match_arm(self.deopt_table, value=VALUE)
            right = inst.as_match_arm(self.deopt_table, value=VALUE)
            lines.append(f"{left} => {right}")

        arms = ",\n".join(sorted(lines))

        return f"""
pub const fn deopt(self) -> Option<Self> {{
    // Should we group these?
    Some(match self {{
        {arms},
        _ => return None,
    }})
}}
        """


class PseudoInstructions(InstructionsMeta):
    enum_name = "PseudoInstruction"
    opcode_size = "u16"

    def __iter__(self) -> "Iterator[analyzer.PseudoInstruction]":
        yield from map(
            Inst, sorted(self._analysis.pseudos.values(), key=lambda inst: inst.name)
        )


def main():
    analysis = analyzer.analyze_files([DEFAULT_INPUT])
    real_instructions = RealInstructions(analysis)
    pseudo_instructions = PseudoInstructions(analysis)

    cpython_version = detect_cpython_version()
    script_path = pathlib.Path(__file__).absolute().relative_to(CRATE_ROOT).as_posix()
    out = f"""
//! CPython {cpython_version} VM instructions.

// This file is generated by {script_path}
// Do not edit!

use crate::{{Oparg, OpargType}};

{real_instructions.rust_code}

{pseudo_instructions.rust_code}

const _: () = assert!(std::mem::size_of::<RealInstruction>() == 1);
const _: () = assert!(std::mem::size_of::<PseudoInstruction>() == 2);
    """.strip()

    OUT_PATH.write_text(rustfmt(out))


if __name__ == "__main__":
    main()
