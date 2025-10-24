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
    use ruff_python_trivia::{
        SimpleToken,
        SimpleTokenKind,
        SimpleTokenizer,
        // first_non_trivia_token,
    };
    use ruff_source_file::LineIndex;
    use ruff_text_size::{
        Ranged,
        //TextLen,
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

            let token = loop {
                match state.next() {
                    Some(v) => break v,
                    None => {
                        let next_line = &zelf.advance_readline(vm, &zelf.encoding)?;
                        if next_line.is_empty() {
                            return Ok(PyIterReturn::StopIteration(None));
                        }
                        state.push(next_line);
                    }
                }
            };

            *zelf.state.write() = state.clone();

            let line_index = LineIndex::from_source_text(&state.buffer);
            let token_range = token.range();

            let line_column_start = line_index.line_column(token_range.start(), &state.buffer);
            let line_column_end = line_index.line_column(token_range.end(), &state.buffer);

            let current_line = match &state.buffer.trim_end().rsplit_once('\n') {
                Some(tup) => tup.1.to_owned() + "\n",
                None => state.buffer.clone(),
            };

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
        offset: TextSize,
        done: bool,
    }

    impl PyTokenizerIterState {
        pub fn push(&mut self, s: &str) {
            self.buffer += s;
        }

        #[must_use]
        pub fn next(&mut self) -> Option<SimpleTokenWrapper> {
            self.token = SimpleTokenizer::starts_at(self.offset, &self.buffer)
                .skip_while(|token| {
                    matches!(
                        token.kind(),
                        SimpleTokenKind::Whitespace | SimpleTokenKind::Bogus
                    )
                })
                .next()
                .map(SimpleTokenWrapper);
            self.offset = match &self.token {
                Some(tok) => tok.range.end(),
                None => self.offset, // We want to preserve our offset if we reached a bogus token
            };
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
            // Ruff doesnt have:
            // - Indent
            // - Dedent
            // - String
            // - Number

            let token = self.token.clone().unwrap();
            /*
            if matches!(token.kind(), SimpleTokenKind::Other) {
                let token_name = &self.token_name();
                if token_name.contains('"') || token_name.contains('\'') {
                    return 3; // token.STRING
                } else if token_name.parse::<u8>().is_ok() {
                    return 2; // token.NUMBER
                };
            };
            */
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
                SimpleTokenKind::Newline => 4,
                SimpleTokenKind::LParen => 7,
                SimpleTokenKind::RParen => 8,
                SimpleTokenKind::Colon => 11,
                SimpleTokenKind::Plus => 14,
                SimpleTokenKind::Equals => 22,
                SimpleTokenKind::Other => Self::MAX,
                other => unimplemented!("for {other:?}"),
            }
        }
    }

    impl fmt::Display for SimpleTokenWrapper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let out = match self.kind() {
                SimpleTokenKind::LParen => "(",
                SimpleTokenKind::RParen => ")",
                SimpleTokenKind::LBrace => "{",
                SimpleTokenKind::RBrace => "}",
                SimpleTokenKind::LBracket => "[",
                SimpleTokenKind::RBracket => "[",
                SimpleTokenKind::Comma => ",",
                SimpleTokenKind::Colon => ":",
                SimpleTokenKind::Semi => ";",
                SimpleTokenKind::Slash => "/",
                SimpleTokenKind::Equals => "=",
                SimpleTokenKind::Plus => "+",
                SimpleTokenKind::Newline => "",
                other => &format!("{other:?}").to_lowercase(),
            };

            write!(f, "{out}")
        }
    }

    impl Deref for SimpleTokenWrapper {
        type Target = SimpleToken;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
