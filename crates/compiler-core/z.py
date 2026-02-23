import pathlib

p = pathlib.Path(__file__).parent / "instructions.toml"
with p.open() as fd:
    lines = fd.readlines()

out = ""
for line in lines:
    line = line.rstrip()
    if line.strip().startswith("["):
        name = line.strip().removeprefix("[").removesuffix("]")
        out += f"[Instruction.{name}]"
    else:
        out += line
    out += "\n"

print(out)
