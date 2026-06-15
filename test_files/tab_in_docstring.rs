//! Demonstration of D206 (tabs disallowed in docstring content).

/// Summary line.
///
///	Detail paragraph indented with a tab — should trigger D206.
pub fn tabby_bad() {}

/// Summary line.
///
///     Detail paragraph indented with spaces — should NOT trigger D206.
pub fn tabby_good() {}
