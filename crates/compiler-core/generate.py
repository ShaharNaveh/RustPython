#!/usr/bin/env python
import abc
import dataclasses
import pathlib
import subprocess
import sys

import tomllib

CRATE_ROOT = pathlib.Path(__file__).parent
CONF_FILE = CRATE_ROOT / "instructions.toml"
OUTPUT_PATH = CRATE_ROOT / "src" / "bytecode" / "generated.rs"


CPYTHON_PATH = (
    pathlib.Path(__file__).parents[3] / "cpython"  # Local filesystem path of cpython
)
_cases_generator_path = CPYTHON_PATH / "Tools" / "cases_generator"
sys.path.append(str(_cases_generator_path))

import analyzer
from generators_common import DEFAULT_INPUT
from stack import get_stack_effect

type Instructions = tuple["Instr", ...]


def rustfmt(code: str) -> str:
    return subprocess.check_output(["rustfmt", "--emit=stdout"], input=code, text=True)


@dataclasses.dataclass(frozen=True, kw_only=True, slots=True)
class Instr:
    name: str
    cpython_name: str
    opcode: int
    properties: analyzer.Properties
    stack_effect: dict[str, str]
    oparg: dict[str, str] = dataclasses.field(default_factory=dict)
    placeholder: bool = False

    def __lt__(self, other) -> bool:
        return self.opcode < other.opcode


@dataclasses.dataclass(frozen=True, slots=True)
class OpcodeEnumBuilder:
    name: str
    instructions: Instructions

    def generate(self) -> str:
        """
        Generate an opcode enum from the given instructions.

        Will include every property that starts with `fn_` or `impl_`.
        """
        fns = "\n\n".join(
            getattr(self, attr).strip()
            for attr in sorted(dir(self))
            if attr.startswith("fn_")
        )
        impls = "\n\n".join(
            getattr(self, attr).strip()
            for attr in sorted(dir(self))
            if attr.startswith("impl_")
        )

        variants = "\n".join(
            f"{instr.name}," + ("// Placeholder" if instr.placeholder else "")
            for instr in self
        )
        return f"""
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum {self.name} {{
            {variants}
        }}

        impl {self.name} {{
            {fns}
        }}

        {impls}
        """

    def _build_has_attr_fn(self, fn_attr: str, prop_attr: str, doc_flag: str):
        arms = "|".join(
            f"Self::{instr.name}"
            for instr in self
            if getattr(instr.properties, prop_attr)
        )

        if arms:
            body = f"matches!(self, {arms})"
        else:
            body = "false"

        return f"""
        /// Whether opcode ID have '{doc_flag}' set.
        #[must_use]
        pub const fn has_{fn_attr}(self) -> bool {{
            {body}
        }}
        """

    @property
    def num_repr(self) -> str:
        return "u8" if next(iter(self)).opcode < 255 else "u16"

    @property
    def fn_has_attr(self) -> str:
        return "\n\n".join(
            self._build_has_attr_fn(*args)
            for args in sorted(
                (
                    ("arg", "oparg", "HAS_ARG_FLAG"),
                    ("const", "uses_co_consts", "HAS_CONST_FLAG"),
                    ("exc", "pure", "HAS_PURE_FLAG"),
                    ("jump", "jumps", "HAS_JUMP_FLAG"),
                    ("local", "uses_locals", "HAS_LOCAL_FLAG"),
                    ("name", "uses_co_names", "HAS_NAME_FLAG"),
                )
            )
        )

    @property
    def impl_into_numeric(self) -> str:
        arms = ",\n".join(
            f"{self.name}::{instr.name} => {instr.opcode}" for instr in self
        )

        return f"""
        impl From<{self.name}> for {self.num_repr} {{
            fn from(value: {self.name}) -> Self {{
                match value {{
                    {arms}
                }}
            }}
        }}
        """

    @property
    def impl_tryfrom_numeric(self) -> str:
        arms = ",\n".join(f"{instr.opcode} => Self::{instr.name}" for instr in self)

        return f"""
        impl TryFrom<{self.num_repr}> for {self.name} {{
            type Error = crate::marshal::MarshalError;

            fn try_from(value: {self.num_repr}) -> Result<Self, Self::Error> {{
                Ok(match value {{
                    {arms},
                    _ => return Err(Self::Error::InvalidBytecode)
                }}
                )
            }}
        }}
        """

    @property
    def impl_display(self) -> str:
        arms = ",\n".join(
            f'Self::{instr.name} => "{instr.cpython_name}"' for instr in self
        )
        return f"""
        impl fmt::Display for {self.name} {{
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
                let name = match self {{
                    {arms}
                }};
                write!(f, "{{name}}")
            }}
        }}
        """

    def __iter__(self):
        yield from self.instructions


@dataclasses.dataclass(frozen=True, slots=True)
class InstructionEnumBuilder:
    name: str
    instructions: Instructions

    def generate(self) -> str:
        """
        Generate an instruction enum from the given instructions.

        Will include every property that starts with `fn_` or `impl_`.
        """

        enum_variants = []
        for instr in self:
            variant = instr.name
            oparg = instr.oparg
            oparg_name = oparg.get("name")
            oparg_type = oparg.get("type")

            if oparg_name and (not oparg_type):
                raise ValueError(
                    f"{instr.name} has defined an oparg name without a type"
                )

            if oparg_name and oparg_type:
                variant += f"{{ {oparg_name}: {oparg_type}}}"
            elif oparg_type:
                variant += f"({oparg_type})"

            variant += ","

            if instr.placeholder:
                variant += "// Placeholder"

            enum_variants.append(variant)

        variants = "\n".join(enum_variants)

        fns = "\n\n".join(
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
        #[derive(Clone, Copy, Debug)]
        pub enum {self.name} {{
            {variants}
        }}

        impl {self.name} {{
            {fns}
        }}

        {impls}
        """

    @property
    def target_opcode(self) -> str:
        return "PseudoOpcode" if next(iter(self)).opcode > 255 else "Opcode"

    @property
    def fn_stack_effect(self) -> str:
        arms = ""
        for instr in self:
            body = f"Self::{instr.name}"
            stack_effect = instr.stack_effect
            popped, pushed = stack_effect["popped"], stack_effect["pushed"]
            needs_oparg = any("oparg" in expr for expr in (popped, pushed))
            oparg = instr.oparg
            oparg_name = oparg.get("name")
            oparg_type = oparg.get("type")
            stores_oparg = oparg_type is not None
            if stores_oparg:
                if oparg_name and oparg_type:
                    if needs_oparg:
                        body += f"{{ {oparg_name} }}"
                    else:
                        body += "{ .. }"
                elif oparg_type:
                    if needs_oparg:
                        oparg_name = "oparg"
                        body += f"({oparg_name})"
                    else:
                        body += "(_)"

            body += " => {\n"
            if needs_oparg and instr.placeholder:
                body += "let oparg = 0; // Placeholder"
            elif needs_oparg:
                body += f"""
                let oparg = i32::try_from(
                    u32::from({oparg_name})
                ).expect("oparg does not fit in an i32");
                """.strip()

            body += f"""\n(
            {pushed},
            {popped}
            )"""

            body += "},\n"

            arms += body

        return f"""
        pub fn stack_effect(self) -> StackEffect {{
            let (pushed, popped) = match self {{
                {arms}
            }};

            StackEffect::new(pushed as u32, popped as u32)
        }}
        """

    @property
    def fn_opcode(self) -> str:
        variants = []
        for instr in self:
            variant = instr.name
            oparg = instr.oparg
            oparg_name = oparg.get("name")
            oparg_type = oparg.get("type")

            if oparg_name and oparg_type:
                variant += "{ .. }"
            elif oparg_type:
                variant += "(_)"

            variants.append(f"Self::{variant} => {self.target_opcode}::{instr.name}")

        arms = ",\n".join(variants)
        return f"""
        /// Instruction's opcode.
        #[must_use]
        pub const fn opcode(self) -> {self.target_opcode} {{
            match self {{
                {arms}
            }}
        }}
        """

    @property
    def impl_into_opcode(self) -> str:
        return f"""
        impl From<{self.name}> for {self.target_opcode} {{
            fn from(value: {self.name}) -> Self {{
                value.opcode()
            }}
        }}
        """

    def __iter__(self):
        yield from self.instructions


def main():
    analysis = analyzer.analyze_files([DEFAULT_INPUT])
    conf = tomllib.loads(CONF_FILE.read_text())

    instructions = []
    pseudo_instructions = []
    for name, opts in conf.items():
        opcode = opts["opcode"]
        is_pseudo = opcode > 255

        cpython_name = opts["cpython_name"]
        if is_pseudo:
            instruction = analysis.pseudos[cpython_name]
        else:
            instruction = analysis.instructions[cpython_name]

        stack_effect = opts.get("stack_effect", {})
        stack = get_stack_effect(instruction)
        stack_effect.setdefault("popped", (-stack.base_offset).to_c())
        stack_effect.setdefault("pushed", (stack.logical_sp - stack.base_offset).to_c())

        instr = Instr(
            name=name,
            **opts,
            properties=instruction.properties,
            stack_effect=stack_effect,
        )

        if is_pseudo:
            pseudo_instructions.append(instr)
        else:
            instructions.append(instr)

    instructions = tuple(sorted(instructions))
    pseudo_instructions = tuple(sorted(pseudo_instructions))

    opcodes_code = OpcodeEnumBuilder("Opcode", instructions).generate()
    pseudo_opcodes_code = OpcodeEnumBuilder(
        "PseudoOpcode", pseudo_instructions
    ).generate()

    instructions_code = InstructionEnumBuilder("Instruction", instructions).generate()
    pseudo_instructions_code = InstructionEnumBuilder(
        "PseudoInstruction", pseudo_instructions
    ).generate()

    output = f"""
    use core::fmt;

    use crate::bytecode::StackEffect;
    use super::oparg;

    {opcodes_code}

    {pseudo_opcodes_code}

    {instructions_code}

    {pseudo_instructions_code}
    """

    OUTPUT_PATH.write_text(rustfmt(output), encoding="utf-8")


if __name__ == "__main__":
    main()
