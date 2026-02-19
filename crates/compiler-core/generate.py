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


def rustfmt(code: str) -> str:
    return subprocess.check_output(["rustfmt", "--emit=stdout"], input=code, text=True)


@dataclasses.dataclass(frozen=True, kw_only=True, slots=True)
class Instr:
    name: str
    cpython_name: str
    opcode: int
    properties: analyzer.Properties
    oparg: dict[str, str] | None = None
    placeholder: bool = False

    def __lt__(self, other) -> bool:
        return self.opcode < other.opcode


def build_has_attr_fn(
    instructions: tuple[Instr, ...], fn_attr: str, prop_attr: str, doc_flag: str
):
    arms = "|".join(
        f"Self::{instr.name}"
        for instr in instructions
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


def generate_opcode_enum(name: str, instructions: tuple[Instr, ...]) -> str:
    variants = ",\n".join(instr.name for instr in instructions)
    enum_def = f"""
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum {name} {{
            {variants}
        }}
    """

    num_repr = "u8" if instructions[0].opcode < 255 else "u16"
    try_from_arms = ",\n".join(
        f"{instr.opcode} => Self::{instr.name}" for instr in instructions
    )

    try_from = f"""
    impl TryFrom<{num_repr}> for {name} {{
        type Error = crate::marshal::MarshalError;

        fn try_from(value: {num_repr}) -> Result<Self, Self::Error> {{
            Ok(match value {{
                {try_from_arms},
                _ => return Err(Self::Error::InvalidBytecode)
            }}
            )
        }}
    }}
    """

    display_arms = ",\n".join(
        f'Self::{instr.name} => "{instr.cpython_name}"' for instr in instructions
    )
    display = f"""
    impl fmt::Display for {name} {{
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
            let name = match self {{
                {display_arms}
            }};
            write!(f, "{{name}}")
        }}
    }}
    """

    has_attr_fns = "\n\n".join(
        build_has_attr_fn(instructions, *args)
        for args in (
            ("arg", "oparg", "HAS_ARG_FLAG"),
            ("const", "uses_co_consts", "HAS_CONST_FLAG"),
            ("exc", "pure", "HAS_PURE_FLAG"),
            ("jump", "jumps", "HAS_JUMP_FLAG"),
            ("local", "uses_locals", "HAS_LOCAL_FLAG"),
            ("name", "uses_co_names", "HAS_NAME_FLAG"),
        )
    )

    return f"""
    {enum_def}


    impl {name} {{
        {has_attr_fns}
    }}

    {display}

    {try_from}
    """


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
            properties = analysis.pseudos[cpython_name].properties
        else:
            properties = analysis.instructions[cpython_name].properties

        instr = Instr(name=name, **opts, properties=properties)

        if is_pseudo:
            pseudo_instructions.append(instr)
        else:
            instructions.append(instr)

    instructions = tuple(sorted(instructions))
    pseudo_instructions = tuple(sorted(pseudo_instructions))

    opcodes_code = generate_opcode_enum("Opcode", instructions)
    pseudo_opcodes_code = generate_opcode_enum("PseudoOpcode", pseudo_instructions)

    output = f"""
    use core::fmt;

    {opcodes_code}

    {pseudo_opcodes_code}
    """

    OUTPUT_PATH.write_text(rustfmt(output), encoding="utf-8")


if __name__ == "__main__":
    main()
