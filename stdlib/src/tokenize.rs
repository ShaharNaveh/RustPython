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
            builtins::{PyStr, PyTypeRef},
            function::ArgCallable,
            protocol::PyIterReturn,
            types::{Constructor, IterNext, Iterable, SelfIter},
        },
    };
    use ruff_python_trivia::SimpleTokenizer;
    use std::fmt;

    #[pyattr]
    #[pyclass(name = "TokenizerIter")]
    #[derive(PyPayload)]
    pub struct PyTokenizerIter {
        readline: ArgCallable,
        extra_tokens: bool,
        encoding: String,
        state: PyRwLock<PyTokenizerIterState>,
    }

    impl fmt::Debug for PyTokenizerIter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("PyTokenizerIter")
                .field("readline", &self.readline)
                .field("extra_tokens", &self.extra_tokens)
                .field("encoding", &self.encoding)
                //.field("state", &self.state)
                .finish()
        }
    }

    #[pyclass(with(Constructor, Iterable, IterNext))]
    impl PyTokenizerIter {}

    impl Constructor for PyTokenizerIter {
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

    impl SelfIter for PyTokenizerIter {}

    impl IterNext for PyTokenizerIter {
        fn next(zelf: &Py<Self>, vm: &VirtualMachine) -> PyResult<PyIterReturn> {
            let state = zelf.state.read();
            let last_line = match &state.last_line {
                Some(line) => line,
                None => &zelf
                    .readline
                    .invoke((), vm)?
                    .downcast::<PyStr>()
                    .map_err(|_| vm.new_type_error("readline() returned a non-string object"))?,
            };

            dbg!("{:?}", last_line);
            /*
            let state = {
                let state_guard = zelf.state.read();
                state_guard.clone()
            };
            */

            //  println!("State: {:?}", state.last_line);

            // NOTE: in future if we read EOL then set `last_line` to None
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

    #[derive(Default)]
    pub struct PyTokenizerIterState {
        tokenizer: Option<SimpleTokenizer<'static>>,
        last_line: Option<PyStr>,
        last_lineno: u32,
        last_end_lineno: u32,
        byte_col_offset_diff: Option<u32>,
    }
}
