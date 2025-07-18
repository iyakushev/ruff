use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_python_ast::{self as ast, Expr};
use ruff_text_size::Ranged;

use crate::{AlwaysFixableViolation, Fix};
use crate::{checkers::ast::Checker, fix::edits::add_argument};

/// ## What it does
/// Checks for `warnings.warn` calls without an explicit `stacklevel` keyword
/// argument.
///
/// ## Why is this bad?
/// The `warnings.warn` method uses a `stacklevel` of 1 by default, which
/// will output a stack frame of the line on which the "warn" method
/// is called. Setting it to a higher number will output a stack frame
/// from higher up the stack.
///
/// It's recommended to use a `stacklevel` of 2 or higher, to give the caller
/// more context about the warning.
///
/// ## Example
/// ```python
/// import warnings
///
/// warnings.warn("This is a warning")
/// ```
///
/// Use instead:
/// ```python
/// import warnings
///
/// warnings.warn("This is a warning", stacklevel=2)
/// ```
///
/// ## Fix safety
/// This rule's fix is marked as unsafe because it changes
/// the behavior of the code. Moreover, the fix will assign
/// a stacklevel of 2, while the user may wish to assign a
/// higher stacklevel to address the diagnostic.
///
/// ## References
/// - [Python documentation: `warnings.warn`](https://docs.python.org/3/library/warnings.html#warnings.warn)
#[derive(ViolationMetadata)]
pub(crate) struct NoExplicitStacklevel;

impl AlwaysFixableViolation for NoExplicitStacklevel {
    #[derive_message_formats]
    fn message(&self) -> String {
        "No explicit `stacklevel` keyword argument found".to_string()
    }

    fn fix_title(&self) -> String {
        "Set `stacklevel=2`".to_string()
    }
}

/// B028
pub(crate) fn no_explicit_stacklevel(checker: &Checker, call: &ast::ExprCall) {
    if !checker
        .semantic()
        .resolve_qualified_name(&call.func)
        .is_some_and(|qualified_name| matches!(qualified_name.segments(), ["warnings", "warn"]))
    {
        return;
    }

    // When prefixes are supplied, stacklevel is implicitly overridden to be `max(2, stacklevel)`.
    //
    // Signature as of Python 3.13 (https://docs.python.org/3/library/warnings.html#warnings.warn)
    // ```text
    //                  0       1               2            3                  4
    // warnings.warn(message, category=None, stacklevel=1, source=None, *, skip_file_prefixes=())
    // ```
    if call
        .arguments
        .find_argument_value("stacklevel", 2)
        .is_some()
        || is_skip_file_prefixes_param_set(&call.arguments)
        || call
            .arguments
            .args
            .iter()
            .any(ruff_python_ast::Expr::is_starred_expr)
        || call
            .arguments
            .keywords
            .iter()
            .any(|keyword| keyword.arg.is_none())
    {
        return;
    }
    let mut diagnostic = checker.report_diagnostic(NoExplicitStacklevel, call.func.range());

    let edit = add_argument(
        "stacklevel=2",
        &call.arguments,
        checker.comment_ranges(),
        checker.locator().contents(),
    );

    diagnostic.set_fix(Fix::unsafe_edit(edit));
}

/// Returns `true` if `skip_file_prefixes` is set to its non-default value.
/// The default value of `skip_file_prefixes` is an empty tuple.
fn is_skip_file_prefixes_param_set(arguments: &ast::Arguments) -> bool {
    arguments
        .find_keyword("skip_file_prefixes")
        .is_some_and(|keyword| match &keyword.value {
            Expr::Tuple(tuple) => !tuple.elts.is_empty(),
            _ => true,
        })
}
