#!/usr/bin/env python
import pathlib
import subprocess
import sys

CRATE_ROOT = pathlib.Path(__file__).parent
OUTPUT_PATH = CRATE_ROOT / "src" / "bytecode" / "generated.rs"

DERIVE = "#[derive(Clone, Copy, Debug, Eq, PartialEq)]"

CPYTHON_PATH = (
    pathlib.Path(__file__).parents[3] / "cpython"  # Local filesystem path of cpython
)
_cases_generator_path = CPYTHON_PATH / "Tools" / "cases_generator"
sys.path.append(str(_cases_generator_path))

import analyzer
from generators_common import DEFAULT_INPUT


def rustfmt(code: str) -> str:
    return subprocess.check_output(["rustfmt", "--emit=stdout"], input=code, text=True)

def enum_def(opmap: dict[str,int]) -> str:
    real, pseudo = 
def build_rust_code(analysis: analyzer.Analysis) -> str:

    for i in analysis.instructions.values():
        print(i.opcode)
    return ""


def main():
    analysis = analyzer.analyze_files([DEFAULT_INPUT])
    code = rustfmt(build_rust_code(analysis))
    OUTPUT_PATH.write_text(code, encoding="utf-8")


if __name__ == "__main__":
    main()
