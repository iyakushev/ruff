use ruff_python_ast::Expr;

use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_text_size::Ranged;

use crate::Violation;
use crate::checkers::ast::Checker;
use crate::rules::pylint::helpers::in_dunder_method;

/// ## What it does
/// Checks for `__init__` methods that are turned into generators by the
/// inclusion of `yield` or `yield from` expressions.
///
/// ## Why is this bad?
/// The `__init__` method is the constructor for a given Python class,
/// responsible for initializing, rather than creating, new objects.
///
/// The `__init__` method has to return `None`. By including a `yield` or
/// `yield from` expression in an `__init__`, the method will return a
/// generator object when called at runtime, resulting in a runtime error.
///
/// ## Example
/// ```python
/// class InitIsGenerator:
///     def __init__(self, i):
///         yield i
/// ```
///
/// ## References
/// - [CodeQL: `py-init-method-is-generator`](https://codeql.github.com/codeql-query-help/python/py-init-method-is-generator/)
#[derive(ViolationMetadata)]
pub(crate) struct YieldInInit;

impl Violation for YieldInInit {
    #[derive_message_formats]
    fn message(&self) -> String {
        "`__init__` method is a generator".to_string()
    }
}

/// PLE0100
pub(crate) fn yield_in_init(checker: &Checker, expr: &Expr) {
    if in_dunder_method("__init__", checker.semantic(), checker.settings()) {
        checker.report_diagnostic(YieldInInit, expr.range());
    }
}
