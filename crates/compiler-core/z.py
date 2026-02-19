import pathlib
import re

TARGET = pathlib.Path(__file__).parent / "z.rs"
lines = TARGET.read_text().splitlines()


def to_snake(s: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", s).upper()


entries = []
it = iter(lines)
for line in it:
    buf = line
    while "=" not in buf:
        buf += next(it)
    entries.append(buf)

res = []
for entry in entries:
    placeholder = "// Placeholder" in entry
    entry = entry.split("//")[0].strip()

    entry, opcode = entry.split("=")
    opcode = int(opcode.removesuffix(","))

    oparg_name = None
    oparg_type = None
    if "{" in entry:
        oparg_name, oparg_type = re.findall(r"(\w+): Arg<(.*)>", entry)[0]
    elif "(" in entry:
        oparg_type = re.findall(r"Arg<(.*)>", entry)[0]

    name = entry.split("(")[0].split()[0]
    cpython_name = to_snake(name)

    res.append(f"[{name}]")
    res.append(f"opcode = {opcode}")
    res.append(f'cpython_name = "{cpython_name}"')
    if placeholder:
        res.append(f"placeholder = true")

    if (oparg_type is not None) and (oparg_type != "u32"):
        oparg_type = f"oparg::{oparg_type}"

    if oparg_name and oparg_type:
        res.append(f'oparg = {{ name = "{oparg_name}", type = "{oparg_type}" }}')
    elif oparg_name:
        res.append(f'oparg = {{ name = "{oparg_name}" }}')
    elif oparg_type:
        res.append(f'oparg = {{ type = "{oparg_type}" }}')

    res.append("")

output = "\n".join(res)
print(output)
