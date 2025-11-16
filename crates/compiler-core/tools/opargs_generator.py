#!/usr/bin/env python
"""
Generate possible oparg types.
"""

import abc
import enum
import inspect
import pathlib
import pydoc
import textwrap
import typing

from common import DEFAULT_OUTPUT_DIR

if typing.TYPE_CHECKING:
    from collections.abc import Iterable, Iterator


DERIVE = "#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]"
REFS = {
    "CodeObject": "[crate::CodeObject]",
    "Instruction": "[crate::Instruction]",
    "PEP695": "https://peps.python.org/pep-0695/",
}
BASE_ERR = "crate::MarshalError"
ERR = f"{BASE_ERR}::InvalidBytecode"


def make_doc(s: str | None) -> str:
    if s is None:
        s = ""

    s = s.strip()

    doc = "\n".join(f"/// {line}" for line in inspect.cleandoc(s).splitlines())

    refs = "\n".join(f"/// [{ref}]: {value}" for ref, value in REFS.items() if ref in s)

    if refs:
        return f"""
{doc}
///
{refs}
""".strip()

    return doc


@enum.unique
class DocEnum(metaclass=enum.EnumType):
    """
    An enum that lets you optionally specify a docstring for the enum variants.

    Examples
    --------
    >>> import enum
    >>>
    >>> class MyFlags(enum.IntEnum, DocEnum):
    >>>     Foo = enum.auto(), "Foo oparg"
    >>>     Bar = enum.auto()
    >>>     Baz = enum.auto(), ["can", "be", "anything", "really"]
    >>>
    >>> MyFlags.Foo.__doc__
    Foo oparg
    >>> MyFlags.Bar.__doc__ is None
    True
    >>> MyFlags.Baz.__doc__
    ['can', 'be', 'anything', 'really']
    """

    @staticmethod
    def _generate_next_value_(name, start, count, last_values):
        return count

    def __new__(cls, value: int, doc: str | None = None):
        obj = int.__new__(cls, value)
        obj._value_ = value
        obj.__doc__ = doc
        return obj


@enum.unique
class OpargCategory(enum.IntEnum):
    Named = enum.auto()
    Alias = enum.auto()


class OpargTypeMeta(metaclass=abc.ABCMeta):
    @property
    @abc.abstractmethod
    def category(self) -> OpargCategory:
        """
        Used to group generated opargs in the genertaed rust code.
        """

    @property
    @abc.abstractmethod
    def inner(self) -> str:
        """
        Inner value for enum/struct.
        """

    @property
    @abc.abstractmethod
    def rust_def(self) -> str:
        """
        Rust source code for defining said enum/struct.
        """

    @property
    def rust_fns(self) -> str:
        """
        Rust code that implements methods on said enum/struct.

        Will take any class attribute that matches 'fn_*'.
        """
        fns = "\n\n".join(
            getattr(self, attr).strip()
            for attr in sorted(dir(self))
            if attr.startswith("fn_")
        )

        if not fns:
            return ""

        return f"""
        impl {self.name} {{
            {fns}
        }}
        """

    @property
    def rust_traits(self) -> str:
        """
        Rust code that implements traits on said enum/struct.

        Will take any class attribute that matches 'fn_*'.
        """
        return "\n\n".join(
            getattr(self, attr).strip()
            for attr in sorted(dir(self))
            if attr.startswith("trait_")
        )

    @property
    def trait_any_oparg(self) -> str:
        return f"impl crate::AnyOparg for {self.name} {{}}"

    @property
    def rust_code(self) -> str:
        """
        Generated rust code.
        """
        return f"""
        {self.rust_def}

        {self.rust_traits}

        {self.rust_fns}
        """

    @property
    def name(self) -> str:
        """
        Enum/Struct name.
        """
        cls_name = type(self).__name__
        return f"{cls_name}Oparg"

    @property
    def doc(self) -> str:
        """
        Returns
        -------
        str
            Either the class docstring formatted for rustdoc.
            Otherwise an empty string if class has no docstring set.
        """
        return pydoc._getowndoc(type(self))


class AliasOpargType(OpargTypeMeta):
    """
    When an instruction doesn't have constraints on its oparg value.
    """

    category = OpargCategory.Alias
    inner = "crate::Oparg"

    @property
    def rust_def(self) -> str:
        docstr = make_doc(self.doc)
        return f"""
        {docstr}
        {DERIVE}
        #[repr(transparent)]
        pub struct {self.name}({self.inner});
        """

    @property
    def trait_deref(self) -> str:
        return f"""
        impl std::ops::Deref for {self.name} {{
            type Target = {self.inner};

            fn deref(&self) -> &Self::Target {{
                &self.0
            }}
        }}
        """

    '''
    @property
    def trait_from_inner(self) -> str:
        return f"""
        impl From<{self.inner}> for {self.name} {{
            fn from(value: {self.inner}) -> Self {{
                Self::new(value)
            }}
        }}
        """

    @property
    def trait_try_from_inner(self) -> str:
        return f"""
        impl TryFrom<{self.inner}> for {self.name} {{
            type Error = {BASE_ERR};

            fn try_from(value: {self.inner}) -> Result<Self, Self::Error> {{
                Ok(Self::from(value))
            }}
        }}
        """
    '''

    @property
    def fn_new(self) -> str:
        return f"""
        #[must_use]
        pub const fn new(value: {self.inner}) -> Self {{
            Self(value)
        }}
        """


class NamedOpargType(OpargTypeMeta):
    category = OpargCategory.Named
    inner = "u32"

    @property
    @abc.abstractmethod
    def flags(self) -> DocEnum:
        """
        Enum with variant values and optional docstrings.
        """

    @property
    def rust_def(self) -> str:
        lines = []
        for member in self.flags:
            doc = member.__doc__
            if doc:
                lines.append(make_doc(doc))
            line = f"{member.name} = {member.value},"
            lines.append(line)

        arms = "\n".join(lines)

        if not (doc := pydoc._getowndoc(type(self))):
            doc = ""
        doc = make_doc(doc)

        # TODO: Should we check if `issubclass(IntFlag, self.flags)` and genertae a bitflag struct instead?
        return f"""
        {doc}
        {DERIVE}
        #[repr({self.inner})]
        pub enum {self.name} {{
            {arms}
        }}
        """

    @property
    def trait_try_from_inner(self) -> str:
        arms = ",\n".join(
            f"{member.value} => Self::{member.name}" for member in self.flags
        )

        return f"""
        impl TryFrom<{self.inner}> for {self.name} {{
            type Error = {BASE_ERR};

            fn try_from(value: {self.inner}) -> Result<Self, Self::Error> {{
                Ok(
                    match value {{
                        {arms},
                        _ => return Err({ERR}),
                    }}
                )
            }}
        }}
        """

    @property
    def trait_try_from_oparg(self) -> str:
        arms = ",\n".join(
            f"{member.value} => Self::{member.name}" for member in self.flags
        )

        target = "crate::Oparg"

        return f"""
        impl TryFrom<{target}> for {self.name} {{
            type Error = {BASE_ERR};

            fn try_from(value: {target}) -> Result<Self, Self::Error> {{
                Self::try_from({self.inner}::from(value))
            }}
        }}
        """


class NameIdx(AliasOpargType):
    """
    Index inside [`CodeObject.names`].
    """


class ConstIdx(AliasOpargType):
    """
    Index inside [`CodeObject.constants`].
    """


class CmpOp(NamedOpargType):
    name = "CmpOp"

    @property
    def flags(self):
        class Cmp(enum.IntFlag):
            # https://github.com/python/cpython/blob/8183fa5e3f78ca6ab862de7fb8b14f3d929421e0/Include/internal/pycore_code.h#L579-L585

            Unordered = enum.auto()
            LessThan = enum.auto()
            GreaterThan = enum.auto()
            Equals = enum.auto()
            NotEquals = Unordered | LessThan | GreaterThan

        class Inner(enum.IntEnum, DocEnum):
            # https://github.com/python/cpython/blob/8183fa5e3f78ca6ab862de7fb8b14f3d929421e0/Python/compile.c#L2864-L2871

            Lt = Cmp.LessThan, "`<`"
            Le = Cmp.LessThan | Cmp.Equals, "`<=`"
            Eq = Cmp.Equals, "`==`"
            Ne = Cmp.NotEquals, "`!=`"
            Gt = Cmp.GreaterThan, "`>`"
            Ge = Cmp.GreaterThan | Cmp.Equals, "`>=`"

        return Inner

    @property
    def trait_try_from_compare(self) -> str:
        target = Compare().name

        return f"""
        impl TryFrom<{target}> for {self.name} {{
            type Error = {BASE_ERR};

            fn try_from(value: {target}) -> Result<Self, Self::Error> {{
                Self::try_from({self.inner}::from(value) >> 5)
            }}
        }}
        """

    @property
    def trait_any_oparg(self) -> str:
        return ""


class Compare(AliasOpargType):
    @property
    def fn_cmp_op(self) -> str:
        target = CmpOp().name

        return f"""
        #[must_use]
        pub fn cmp_op(self) -> Result<{target}, {BASE_ERR}> {{
            {target}::try_from(self)
        }}
        """

    @property
    def fn_coerce_bool(self) -> str:
        return """
        /// Indicated if the comparison result should be coerced to bool.
        #[must_use]
        pub const fn coerce_bool(self) -> bool {
            (self & 16) != 0
        }
        """


class Count(AliasOpargType): ...


class Delta(AliasOpargType): ...


class VarNum(AliasOpargType): ...


class BuildSlice(NamedOpargType):
    """
    Specifies if a slice is built with either 2 or 3 arguments.
    """

    @property
    def flags(self):
        two_doc = """
        ```py
        x[5:10]
        ```
        """

        three_doc = """
        ```py
        x[5:10:2]
        ```
        """

        class Inner(enum.IntEnum, DocEnum):
            Two = 2, two_doc
            Three = 3, three_doc

        return Inner


class Resume(NamedOpargType):
    @property
    def flags(self):
        class Inner(enum.IntEnum, DocEnum):
            AtFuncStart = enum.auto()
            AfterYield = enum.auto()
            AfterYieldFrom = enum.auto()
            AfterAwait = enum.auto()

        return Inner


class BinOp(NamedOpargType):
    @property
    def flags(self):
        class Inner(enum.IntEnum, DocEnum):
            Add = enum.auto(), "`+`"
            And = enum.auto(), "`&`"
            FloorDivide = enum.auto(), "`//`"
            Lshift = enum.auto(), "`<<`"
            MatrixMultiply = enum.auto(), "`@`"
            Multiply = enum.auto(), "`*`"
            Remainder = enum.auto(), "`%`"
            Or = enum.auto(), "`|`"
            Power = enum.auto(), "`**`"
            Rshift = enum.auto(), "`>>`"
            Subtract = enum.auto(), "`-`"
            TrueDivide = enum.auto(), "`/`"
            Xor = enum.auto(), "`^`"
            InplaceAdd = enum.auto(), "`+`"
            InplaceAnd = enum.auto(), "`&=`"
            InplaceFloorDivide = enum.auto(), "`//=`"
            InplaceLshift = enum.auto(), "`<<=`"
            InplaceMatrixMultiply = enum.auto(), "`@=`"
            InplaceMultiply = enum.auto(), "`*=`"
            InplaceRemainder = enum.auto(), "`%=`"
            InplaceOr = enum.auto(), "`|=`"
            InplacePower = enum.auto(), "`**=`"
            InplaceRshift = enum.auto(), "`>>=`"
            InplaceSubtract = enum.auto(), "`-=`"
            InplaceTrueDivide = enum.auto(), "`/=`"
            InplaceXor = enum.auto(), "`^=`"

        return Inner


class CallIntrinsic1(NamedOpargType):
    @property
    def flags(self):
        print_doc = "Prints the argument to standard out. Used in the REPL."
        stop_iterator_error_doc = """
        Extracts the return value from a `StopIteration` exception.
        """

        type_var_doc = """
        Creates a [`typing.TypeVar`].

        [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar,
        """

        param_spec_doc = """
        Crates a [`typing.ParamSpec`].

        [typing.ParamSpec]: https://docs.python.org/3.13/library/typing.html#typing.ParamSpec
        """

        type_var_tuple_doc = """
        Crates a [`typing.TypeVarTuple`]

        [typing.TypeVarTuple]: https://docs.python.org/3.13/library/typing.html#typing.TypeVarTuple
        """

        type_alias_doc = """
        Creates a [`typing.TypeAliasType`].

        Used in the [`type`] statement. The argument is a tuple of the type aliass name, type parameters, and value.

        [type]: https://docs.python.org/3.13/reference/simple_stmts.html#type
        [typing.TypeAliasType]: https://docs.python.org/3.13/library/typing.html#typing.TypeAliasType
        """

        class Inner(enum.IntEnum, DocEnum):
            Invalid = enum.auto(), "Not valid."
            Print = enum.auto(), print_doc
            ImportStar = enum.auto(), "Performs `import *` for the named module."
            StopIterationError = enum.auto(), stop_iterator_error_doc
            AsyncGenWrap = enum.auto(), "Wraps an async generator value."
            UnaryPositive = enum.auto(), "Performs the unary `+` operation."
            ListToTuple = enum.auto(), "Converts a list to a tuple."
            TypeVar = enum.auto(), type_var_doc
            ParamSpec = enum.auto(), param_spec_doc
            TypeVarTuple = enum.auto(), type_var_tuple_doc
            SubscriptGeneric = enum.auto(), "Generic subscript for [`PEP695`]."
            TypeAlias = enum.auto(), type_alias_doc

        return Inner


class CallIntrinsic2(NamedOpargType):
    @property
    def flags(self):
        prep_reraise_star_doc = """
        Calculates the [`ExceptionGroup`] to raise from a `try-except*`.

        [ExceptionGroup]: https://docs.python.org/3.13/library/exceptions.html#ExceptionGroup
        """

        type_var_with_bound_doc = """
        Creates a [`typing.TypeVar`] with a bound.

        [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar
        """

        type_var_with_constraint_doc = """
        Creates a [`typing.TypeVar`] with constraints.

        [typing.TypeVar]: https://docs.python.org/3.13/library/typing.html#typing.TypeVar
        """

        set_function_type_params_doc = (
            "Sets the `__type_params__` attribute of a function."
        )

        class Inner(enum.IntEnum, DocEnum):
            Invalid = enum.auto(), "Not valid."
            PrepReraiseStar = enum.auto(), prep_reraise_star_doc
            TypeVarWithBound = enum.auto(), type_var_with_bound_doc
            TypeVarWithConstraint = enum.auto(), type_var_with_constraint_doc
            SetFunctionTypeParams = enum.auto(), set_function_type_params_doc

        return Inner


class RaiseVarargs(NamedOpargType):
    """
    Raises an exception using one of the 3 forms of the `raise` statement.
    """

    @property
    def flags(self):
        reraise_doc = """
        Re-Raise previous exception.

        ```py
        raise 
        ```
        """

        raise_doc = """
        Raise exception instance or type at `STACK[-1]`.

        ```py
        raise STACK[-1] 
        ```
        """

        raise_cause_doc = """
        Raise exception instance or type at `STACK[-2]` with `__cause__` set to `STACK[-1]`.

        ```py
        raise STACK[-2] from STACK[-1]
        ```
        """

        class Inner(enum.IntEnum, DocEnum):
            Reraise = enum.auto(), reraise_doc
            Raise = enum.auto(), raise_doc
            RaiseCause = enum.auto(), raise_cause_doc

        return Inner


class SetFunctionAttribute(NamedOpargType):
    """
    Determines which attribute to set.
    """

    @property
    def flags(self):
        defaults_doc = "A tuple of default values for positional-only and positional-or-keyword parameters in positional order."
        kw_defaults_doc = "A dictionary of keyword-only parameters' default values."
        annotations_doc = "A tuple of strings containing parameters' annotations."
        closure_doc = "A tuple containing cells for free variables, making a closure."

        class Inner(enum.IntFlag, DocEnum):
            _generate_next_value_ = enum.IntFlag._generate_next_value_

            Defaults = enum.auto(), defaults_doc
            KwDefaults = enum.auto(), kw_defaults_doc
            Annotations = enum.auto(), annotations_doc
            Closure = enum.auto(), closure_doc

        return Inner


class ConvertValue(NamedOpargType):
    """
    Used for implementing formatted string literals (f-strings).
    """

    @property
    def flags(self):
        class Inner(enum.IntEnum, DocEnum):
            _None = enum.auto(), "No conversion."
            Str = enum.auto(), "Converts by calling `str(...)`."
            Repr = enum.auto(), "Converts by calling `repr(...)`."
            Ascii = enum.auto(), "Converts by calling `ascii(...)`."

        Inner._None._name_ = "None"

        return Inner


class Invert(NamedOpargType):
    @property
    def flags(self):
        class Inner(enum.IntEnum, DocEnum):
            No = enum.auto()
            Yes = enum.auto()

        return Inner


class Where(NamedOpargType):
    """
    Indicates where the instruction occurs.
    """

    @property
    def flags(self):
        class Inner(enum.IntEnum, DocEnum):
            NoWhere = enum.auto(), "Nowhere"
            AfterAEnter = enum.auto(), "After a call to `__aenter__`."
            AfterAExit = enum.auto(), "After a call to `__aexit__`."

        return Inner


def main():
    script_path = pathlib.Path(__file__).absolute().relative_to(CRATE_ROOT).as_posix()

    oparg_types = tuple(
        oparg_type()
        for parent in OpargTypeMeta.__subclasses__()
        for oparg_type in sorted(
            parent.__subclasses__(), key=lambda cls: (cls.category, cls.__name__)
        )
    )

    rust_code = "\n\n".join(typ.rust_code.strip() for typ in oparg_types)

    out = f"""
    //! Oparg definitions.

    // This file is generated by {script_path}
    // Do not edit!

    {rust_code}
    """.strip()

    OUT_PATH.write_text(out)


if __name__ == "__main__":
    main()
