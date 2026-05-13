use ruff_python_ast as ast;

/// Extract the docstring from a [`&[ast::Stmt]`].
///
/// ## See Also
/// [_PyAST_GetDocString](https://github.com/python/cpython/blob/v3.14.5/Python/ast.c#L1076-L1091)
pub(crate) fn get_doc_string<'a>(body: &'a [ast::Stmt]) -> Option<String> {
    if let Some(ast::Stmt::Expr(expr)) = body.first() {
        if let ast::Expr::StringLiteral(inner) = expr {
            return Some(inner.value.into());
        }
    };
    None
}
