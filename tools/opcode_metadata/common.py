import pathlib
import re

ROOT = pathlib.Path(__file__).parents[2].resolve()
DEFAULT_INPUT = ROOT / "crates/compiler-core/src/instruction.rs"


def to_upper_snake_case(s: str) -> str:
    """
    Converts a PascalCaseString to be SNAKE_CASE

    Parameters
    ----------
    s : str
        Pascal cased string to convert.

    Returns
    -------
    str
        Uppercased snake case string.

    Examples
    --------
    >>> to_upper_snake_case("LoadAttr")
    LOAD_ATTR
    >>> to_upper_snake_case("CallIntrinsic1")
    CALL_INTRINSIC_1
    """
    res = re.sub(r"(?<=[a-z0-9])([A-Z])", r"_\1", s)
    return re.sub(r"(\D)(\d+)$", r"\1_\2", res).upper()


def extract_enum_body(name: str, text: str) -> str | None:
    """
    Extract the rust enum body from a raw rust source code.

    Parameters
    ----------
    name : str
        Enum name to extract its body.
    text : str
        Rust source code containing the enum body.

    Returns
    -------
    str
        If the enum body was found. None otherwise.
    """
    # Find the start of the enum block
    start_match = re.search(rf"enum\s+{name}\s*\{{", text)
    if not start_match:
        return None

    # Manually track brace depth from that point
    depth = 0
    start = start_match.end() - 1  # position of opening '{'
    for i, ch in enumerate(text[start:], start):
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                # Return only the inner content (excluding outer braces)
                return text[start + 1 : i]
