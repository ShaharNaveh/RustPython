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


@dataclasses.dataclass
class Instr:
    _inner: "analyzer.Instruction | analyzer.PseudoInstruction"

    @property
    def name(self) -> str:
        return self._inner.name.title().replace("_", "")

    def __lt__(self, other) -> bool:
        return self._inner.opcode < other._inner.opcode


@dataclasses.dataclass
class OpcodeMeta(metaclass=abc.ABCMeta):
    _analysis: analyzer.Analysis

    @abc.abstractmethod
    def __iter__(self) -> "Iterator[Instr]": ...

    @property
    def rust_code(self) -> str:
        enum_name = type(self).__name__
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
        pub enum {enum_name} {{
            {variants}
        }}

        impl {enum_name} {{
            {methods}
        }}

        {impls}
        """.strip()


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
