pub(crate) use _tokenize::make_module;

#[pymodule]
mod _tokenize {
    use crate::{
        common::lock::{
            // PyMutex,
            PyRwLock,
        },
        vm::{
            Py,
            PyPayload,
            PyResult,
            VirtualMachine,
            // PyObjectRef,
            builtins::{
                PyStr,
                // PyStrRef,
                PyTypeRef,
            },
            convert::ToPyObject,
            function::ArgCallable,
            protocol::PyIterReturn,
            types::{Constructor, IterNext, Iterable, SelfIter},
        },
    };
    use ruff_python_trivia::{BackwardsTokenizer, SimpleToken, SimpleTokenKind, SimpleTokenizer};
    use ruff_source_file::{LineIndex, OneIndexed};
    use ruff_text_size::{
        // Ranged,
        // TextLen,
        TextRange,
        TextSize,
    };
    use std::{fmt, ops::Deref};

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
                .finish()
        }
    }

    #[pyclass(with(Constructor, Iterable, IterNext))]
    impl PyTokenizerIter {}

    impl PyTokenizerIter {
        fn advance_readline(&self, vm: &VirtualMachine, _encoding: &str) -> PyResult<String> {
            let raw_line = &self
                .readline
                .invoke((), vm)?
                .downcast::<PyStr>()
                .map_err(|_| vm.new_type_error("readline() returned a non-string object"))?;
            Ok(raw_line.as_str().to_owned())
        }
    }

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
            let mut state = {
                let guard = zelf.state.read();
                guard.clone()
            };

            /*
                          Potentially have something like

                       let mut current_line = state.text[-1]
                        match <> {
            None => {
            current_line = <advance>
            }
                        }
                        */
            let token = match state.next() {
                Some(v) => v,
                None => {
                    let next_line = &zelf.advance_readline(vm, &zelf.encoding)?;
                    if next_line.is_empty() {
                        return Ok(PyIterReturn::StopIteration(None));
                    }
                    let text = &state.text;
                    let nstate = state.with_text(format!("{text}\n"));
                    *zelf.state.write() = nstate.clone();
                    let ntoken = nstate.next_token().unwrap();

                    ntoken
                }
            };

            dbg!(&token);

            return Ok(PyIterReturn::Return(1.to_pyobject(vm)));
            /*
                        // NOTE: in future if we read EOL then set `last_line` to None
                        return Ok(PyIterReturn::StopIteration(None));
            */
            // #########
            // #########
            // #########
            // #########
            // #########

            /*
                        let state = match &zelf.state {
                            Some(inner) => inner,
                            None => &PyRwLock::new(
                            PyTokenizerIterState::new(
                                zelf.advance_readline(vm)?,
                                TextSize::default(),
                            )),
                        };
            */

            /*
            let last_line = match &state.last_line {
                Some(v) => v,
                None => &LastLine::new(zelf.advance_readline(vm)?, OneIndexed::MIN),
            };
            */

            //   dbg!(&last_line);

            /*
                        println!("line={:#?}", &line);
                        let offset = &state
                            .offset
                            .unwrap_or_else(|| TextRange::up_to(TextSize::of(&line)));
                        println!("offset={:#?}", &offset);

                        let mut tokenizer = SimpleTokenizer::new(&line, *offset);

                        for token in tokenizer {
                            println!("token={:#?}", &token);
                        }
            */
            /*
            let token = tokenizer.next().unwrap();
            println!("token={:#?}", &token);

            let token = tokenizer.next().unwrap();
            println!("token={:#?}", &token);

            let token = tokenizer.next().unwrap();
            println!("token={:#?}", &token);
            */

            /*
            let mut tokenizer: &mut SimpleTokenizer<'_> = match &mut state.tokenizer {
                Some(tokenizer) => tokenizer,
                None => &SimpleTokenizer::new(&line, TextRange::up_to(TextSize::of(&line))),
            };

            let idk = tokenizer.next().unwrap();
            println!("{:#?}", idk);
            */
            /*
            for tok in tokenizer. {
                println!("{:#?}", tok);
            }
            */
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

    #[derive(Clone, Debug)]
    pub struct PyTokenizerIterState {
        text: String,
        token: SimpleToken,
    }

    impl PyTokenizerIterState {
        #[must_use]
        pub fn with_text(&self, text: String) -> Self {
            Self {
                text,
                token: self.token.clone(),
            }
        }

        #[must_use]
        pub fn next(&mut self) -> Option<SimpleToken> {
            let mut tokenizer = SimpleTokenizer::starts_at(self.token.range.end(), &self.text);
            let token = tokenizer.next();
            if let Some(tok) = &token {
                self.token = &tok.clone();
            };
            token
        }
    }

    impl Default for PyTokenizerIterState {
        fn default() -> Self {
            Self {
                text: String::default(),
                token: SimpleToken {
                    kind: SimpleTokenKind::EndOfFile,
                    range: TextRange::default(),
                },
            }
        }
    }
}
