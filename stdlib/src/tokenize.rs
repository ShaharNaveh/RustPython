pub(crate) use _tokenize::make_module;

#[pymodule]
mod _tokenize {
    use crate::{
        common::lock::PyRwLock,
        vm::{
            Py,
            PyPayload,
            PyResult,
            VirtualMachine,
            // PyObjectRef,
            builtins::PyTypeRef,
            function::ArgCallable,
            protocol::PyIterReturn,
            types::{Constructor, IterNext, Iterable, SelfIter},
        },
    };

    //#[derive(Debug, PyPayload)]
    use ruff_python_trivia::SimpleTokenizer;

    #[pyattr]
    #[pyclass(name = "TokenizerIter", module = "_tokenize")]
    pub struct PyTokenizerIter<'a> {
        readline: ArgCallable,
        extra_tokens: bool,
        encoding: String,
        state: PyRwLock<PyTokenizerIterState<'a>>,
    }

    #[pyclass(with(Constructor, Iterable, IterNext))]
    impl PyTokenizerIter<'_> {}

    impl<'_> Constructor for PyTokenizerIter<'_> {
        type Args = PyTokenizerIterArgs;

        fn py_new(cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult {
            let Self::Args {
                readline,
                extra_tokens,
                encoding,
            } = args;
            Self {
                readline,
                extra_tokens,
                encoding,
                state: PyRwLock::new(PyTokenizerIterState::default()),
            }
            .into_ref_with_type(vm, cls)
            .map(Into::into)
        }
    }

    impl<'a> SelfIter for PyTokenizerIter<'a> {}

    impl<'a> IterNext for PyTokenizerIter<'a> {
        fn next(zelf: &Py<Self>, vm: &VirtualMachine) -> PyResult<PyIterReturn> {
            println!("{:#?}", zelf.readline.invoke((), vm));

            return Ok(PyIterReturn::StopIteration(None));
        }
    }

    #[derive(FromArgs)]
    pub struct PyTokenizerIterArgs {
        #[pyarg(positional)]
        readline: ArgCallable,

        #[pyarg(named)]
        extra_tokens: bool,

        #[pyarg(named, default = String::from("utf-8"))]
        encoding: String,
    }

    #[derive(Debug, Default)]
    pub struct PyTokenizerIterState<'a> {
        tokenizer: Option<SimpleTokenizer<'a>>,
        last_line: Option<&'a str>,
        last_lineno: u32,
        last_end_lineno: u32,
        byte_col_offset_diff: Option<u32>,
    }
}
