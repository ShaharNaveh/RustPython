#!/usr/bin/env python
import abc
import dataclasses
import pathlib
import subprocess
import sys

CRATE_ROOT = pathlib.Path(__file__).parent
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


@dataclasses.dataclass(frozen=True, slots=True)
class Instr:
    _inner: "analyzer.Instruction | analyzer.PseudoInstruction"

    @property
    def name(self) -> str:
        return self._inner.name.title().replace("_", "")

    @property
    def opcode(self) -> int:
        return self._inner.opcode

    def __lt__(self, other) -> bool:
        return self._inner.opcode < other._inner.opcode


@dataclasses.dataclass(frozen=True, slots=True)
class OpcodeMeta(metaclass=abc.ABCMeta):
    _analysis: analyzer.Analysis

    @abc.abstractmethod
    def __iter__(self) -> "Iterator[Instr]": ...

    @property
    def enum_name(self) -> str:
        return type(self).__name__

    @property
    def rust_code(self) -> str:
        variants = ",\n".join(instr.name for instr in self)

        methods = "\n\n".join(
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
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum {self.enum_name} {{
            {variants}
        }}

        impl {self.enum_name} {{
            {methods}
        }}

        {impls}
        """.strip()

    @property
    def impl_tryfrom_numeric(self) -> str:
        num_repr = "u8" if next(iter(self)).opcode < 255 else "u16"
        arms = ",\n".join(f"{instr.opcode} => Self::{instr.name}" for instr in self)

        return f"""
        impl TryFrom<{num_repr}> for {self.enum_name} {{
            type Error = crate::marshal::MarshalError;

            fn try_from(value: {num_repr}) -> Result<Self, Self::Error> {{
                Ok(match value {{
                    {arms},
                    _ => return Err(Self::Error::InvalidBytecode)
                }}
                )
            }}
        }}
        """


@dataclasses.dataclass(frozen=True, slots=True)
class Opcode(OpcodeMeta):
    def __iter__(self) -> "Iterator[Instr]":
        yield from sorted(map(Instr, self._analysis.instructions.values()))

    @property
    def fn_foo(self):
        return ""


def main():
    analysis = analyzer.analyze_files([DEFAULT_INPUT])
    opcodes = Opcode(analysis)

    code = f"""
    {opcodes.rust_code}
    """

    OUTPUT_PATH.write_text(rustfmt(code), encoding="utf-8")


if __name__ == "__main__":
    main()
