import inspect
import io
import json
import os
import pathlib
import pydoc
import sys
import types
import typing
import warnings
from importlib.machinery import EXTENSION_SUFFIXES, ExtensionFileLoader

if typing.TYPE_CHECKING:
    from collections.abc import Iterable

IGNORED_MODULES = {"this", "antigravity"}
IGNORED_ATTRS = {
    # "__abstractmethods__",
    # "__add__",
    "__annotations__",
    # "__await__",
    # "__class__",
    # "__class_getitem__",
    # "__delattr__",
    "__dict__",
    "__dir__",
    # "__div__",
    "__doc__",
    # "__eq__",
    "__file__",
    "__ge__",
    # "__getattribute__",
    # "__getitem__",
    # "__getstate__",
    # "__gt__",
    # "__le__",
    "__loader__",
    # "__lt__",
    "__module__",
    # "__mul__",
    "__name__",
    # "__ne__",
    "__objclass__",
    "__package__",
    # "__qualname__",
    # "__spec__",
    # "__sub__",
}

OUT_FILE = pathlib.Path(__file__).parent / "src" / f"{sys.platform}.rs"


class DocEntry(typing.NamedTuple):
    parts: tuple[str, ...]
    doc: str

    @property
    def rust_repr(self) -> str:
        key = json.dumps(".".join(self.parts))
        doc = json.dumps(self.doc)
        return f"({key}, r#{doc}#)"


def is_c_extension(module: types.ModuleType) -> bool:
    """
    Adapted from: https://stackoverflow.com/a/39304199
    """
    if isinstance(getattr(module, "__loader__", None), ExtensionFileLoader):
        return True

    try:
        module_filename = inspect.getfile(module)
    except TypeError:
        return True

    module_filetype = os.path.splitext(module_filename)[1]
    return module_filetype in EXTENSION_SUFFIXES


def is_child(obj: typing.Any, module: types.ModuleType) -> bool:
    return inspect.getmodule(obj) is module


def iter_modules() -> "Iterable[types.ModuleType]":
    """
    Yields
    ------
    :class:`types.Module`
        Modules that are written in C. (not pure python)
    """
    for module_name in sys.stdlib_module_names - IGNORED_MODULES:
        try:
            with warnings.catch_warnings():
                warnings.filterwarnings("ignore", category=DeprecationWarning)
                module = __import__(module_name)
        except ImportError:
            warnings.warn(f"Could not import {module_name}", category=ImportWarning)
            continue

        if not is_c_extension(module):
            continue

        yield module


def traverse(
    obj: typing.Any, module: types.ModuleType, name_parts: tuple[str, ...]
) -> "typing.Iterable[DocEntry]":
    has_doc = any(
        f(obj)
        for f in (
            inspect.ismodule,
            inspect.isclass,
            inspect.isbuiltin,
            # inspect.isfunction,
            # inspect.ismethod,
        )
    )

    if has_doc and isinstance(obj.__doc__, str):
        yield DocEntry(name_parts, pydoc.getdoc(obj))

    # if inspect.isfunction(obj) or inspect.ismethod(obj):
    #    return

    for name, attr in inspect.getmembers(obj):
        if name in IGNORED_ATTRS:
            continue

        if attr == obj:
            continue

        if (module is obj) and (not is_child(attr, module)):
            pass
        # continue

        if (not inspect.ismodule(attr)) and not inspect.ismodule(obj):
            continue

        new_name_parts = name_parts + (name,)

        attr_typ = type(attr)
        is_type_or_module = (attr_typ is type) or (attr_typ is type(__builtins__))

        if is_type_or_module:
            yield from traverse(attr, module, new_name_parts)
            continue

        if inspect.isbuiltin(attr) and (
            callable(attr)
            or not issubclass(attr_typ, type)
            or attr_typ.__name__ in ("getset_descriptor", "member_descriptor")
        ):
            yield DocEntry(new_name_parts, pydoc.getdoc(attr))


def find_doc_entires() -> "Iterable[DocEntry]":
    yield from (
        doc_entry
        for module in iter_modules()
        for doc_entry in traverse(module, module, (module.__name__,))
    )
    yield from (doc_entry for doc_entry in traverse(__builtins__, __builtins__, ("",)))

    builtin_types = [
        type(bytearray().__iter__()),
        type(bytes().__iter__()),
        type(dict().__iter__()),
        type(dict().values().__iter__()),
        type(dict().items().__iter__()),
        type(dict().values()),
        type(dict().items()),
        type(set().__iter__()),
        type(list().__iter__()),
        type(range(0).__iter__()),
        type(str().__iter__()),
        type(tuple().__iter__()),
        type(None),
        type(lambda: ...),
    ]
    for typ in builtin_types:
        name_parts = ("builtins", typ.__name__)
        # if not isinstance(typ.__doc__, str):
        yield DocEntry(name_parts, pydoc.getdoc(typ))
        yield from traverse(typ, __builtins__, name_parts)


def main():
    doc_entries = sorted(
        doc_entry.rust_repr for doc_entry in find_doc_entires() if doc_entry.doc
    )

    for doc_entry in doc_entries:
        # continue
        print(doc_entry)
        print()
    # __import__("pprint").pprint(docs)


if __name__ == "__main__":
    main()
