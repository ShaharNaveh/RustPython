import pathlib
import re

TARGET = pathlib.Path(__file__).parent / "z.rs"
lines = TARGET.read_text().splitlines()

entries = []
it = iter(lines)
for line in it:
    buf = line
    while "=" not in buf:
        buf += next(it)
    entries.append(buf)

for a in entries:
    print("#" * 15)
    print(a)
