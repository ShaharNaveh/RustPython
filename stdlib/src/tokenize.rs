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
    use ruff_python_trivia::{SimpleToken, SimpleTokenKind, first_non_trivia_token};
    use ruff_source_file::{LineIndex, OneIndexed};
    use ruff_text_size::{Ranged, TextLen, TextRange, TextSize};
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

            let token = match state.next() {
                Some(v) => v,
                None => {
                    let next_line = &zelf.advance_readline(vm, &zelf.encoding)?;
                    if next_line.is_empty() {
                        return Ok(PyIterReturn::StopIteration(None));
                    }
                    state.push(next_line);
                    state.push("\n");
                    state.next().unwrap()
                }
            };

            *zelf.state.write() = state.clone();

            let line_index = LineIndex::from_source_text(&state.buffer);
            let token_range = token.range();

            let line_column_start = line_index.line_column(token_range.start(), &state.buffer);
            let line_column_end = line_index.line_column(token_range.end(), &state.buffer);

            let current_line = state.buffer.rsplit_once("\n").unwrap().1;
            let out = vm
                .ctx
                .new_tuple(vec![
                    state.token_numeric_value().to_pyobject(vm),
                    vm.ctx.new_str(state.token_name()).into(),
                    vm.ctx
                        .new_tuple(vec![
                            line_column_start.line.get().to_pyobject(vm),
                            line_column_start.column.to_zero_indexed().to_pyobject(vm),
                        ])
                        .into(),
                    vm.ctx
                        .new_tuple(vec![
                            line_column_end.line.get().to_pyobject(vm),
                            line_column_end.column.to_zero_indexed().to_pyobject(vm),
                        ])
                        .into(),
                    vm.ctx.new_str(current_line).into(),
                ])
                .into();

            println!("");
            println!("");
            println!("");
            return Ok(PyIterReturn::Return(out));
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

    #[derive(Clone, Debug, Default)]
    pub struct PyTokenizerIterState {
        buffer: String,
        token: Option<SimpleTokenWrapper>,
    }

    impl PyTokenizerIterState {
        pub fn push(&mut self, s: &str) {
            self.buffer += s;
        }

        #[must_use]
        pub fn next(&mut self) -> Option<SimpleTokenWrapper> {
            let offset = self
                .token
                .clone()
                .map_or(TextSize::default(), |t| t.range.end());
            let mut tokenizer = SimpleTokenizer::starts_at(offset, &self.buffer);
            self.token = tokenizer.next().map(SimpleTokenWrapper);
            self.token.clone()
        }

        /// Get current token name repr.
        ///
        /// # Panics
        ///
        /// If token is None.
        #[must_use]
        pub fn token_name(&self) -> String {
            let token = &self.token.clone().unwrap();
            dbg!(&token.kind());
            match &token.kind() {
                SimpleTokenKind::Name | SimpleTokenKind::Other => {
                    self.buffer[token.range()].to_owned()
                }
                _ => format!("{token}"),
            }
        }

        /// Get current token numeric value.
        ///
        /// # Panics
        ///
        /// If token is None.
        #[must_use]
        pub fn token_numeric_value(&self) -> u8 {
            // TODO: Maybe we need to write our own tokenizer :/
            // Ruff doesnt have Indent & Dedent & String :(

            let token = self.token.clone().unwrap();
            if matches!(token.kind(), SimpleTokenKind::Name) {
                let token_name = &self.token_name();
                if token_name.contains('"') || token_name.contains('\'') {
                    return 3; // token.STRING
                } else if token_name.parse::<f32>().is_ok() {
                    return 2; // token.NUMBER
                };
            };
            u8::from(token)
        }
    }

    /// Simple wrapper to have some extra functionlity for `SimpleTokenKind`.
    #[derive(Clone, Debug)]
    pub(crate) struct SimpleTokenWrapper(SimpleToken);

    impl From<SimpleToken> for SimpleTokenWrapper {
        fn from(raw: SimpleToken) -> Self {
            Self(raw)
        }
    }

    impl From<SimpleTokenWrapper> for u8 {
        fn from(raw: SimpleTokenWrapper) -> Self {
            match raw.kind() {
                SimpleTokenKind::Pass
                | SimpleTokenKind::For
                | SimpleTokenKind::Name
                | SimpleTokenKind::In => 1,
                SimpleTokenKind::LParen => 7,
                SimpleTokenKind::RParen => 8,
                SimpleTokenKind::Colon => 11,
                SimpleTokenKind::Equals => 22,
                SimpleTokenKind::Newline => 63,
                SimpleTokenKind::Other => Self::MAX,
                other => unimplemented!("for {other:?}"),
            }
        }
    }

    impl fmt::Display for SimpleTokenWrapper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.kind() {
                SimpleTokenKind::LParen => write!(f, "("),
                SimpleTokenKind::RParen => write!(f, ")"),
                SimpleTokenKind::LBrace => write!(f, "{{"),
                SimpleTokenKind::RBrace => write!(f, "}}"),
                SimpleTokenKind::LBracket => write!(f, "["),
                SimpleTokenKind::RBracket => write!(f, "["),
                SimpleTokenKind::Comma => write!(f, ","),
                SimpleTokenKind::Colon => write!(f, ":"),
                SimpleTokenKind::Semi => write!(f, ";"),
                SimpleTokenKind::Slash => write!(f, "/"),
                other => write!(f, "{}", format!("{:?}", other).to_lowercase()),
            }
        }
    }

    impl Deref for SimpleTokenWrapper {
        type Target = SimpleToken;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
