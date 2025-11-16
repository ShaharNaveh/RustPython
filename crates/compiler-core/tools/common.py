import pathlib
import typing

ROOT = pathlib.Path(__file__).parents[3]
DEFAULT_OUTPUT_DIR = CRATE_ROOT / "crates" / "compiler-core" / "src" / "bytecode"


def root_relative_path(filename: str, *, root: pathlib.Path = ROOT) -> str:
    return pathlib.Path(filename).absolute().relative_to(root).as_posix()


def write_dne_comment(
    filename: str, outfile: typing.TextIO, comment: str = "//"
) -> None:
    """
    Writes the Do Not Edit comment.
    """
    gen_path = root_relative_path(filename)
    outfile.write(
        f"""
{comment} This file is genrrated by {gen_path}
{comment} Do not edit manually!
"""
    )
