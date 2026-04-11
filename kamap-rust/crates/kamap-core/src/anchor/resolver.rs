/// Anchor-based code block resolver.
///
/// Given file content and an anchor string, resolves the exact line range
/// of the code block that the anchor refers to. Supports brace-delimited
/// languages (Rust, Go, JS, Java, C, …) and indentation-delimited languages
/// (Python, YAML, …) automatically.

/// Result of anchor resolution
#[derive(Debug, Clone)]
pub struct AnchorResult {
    /// 1-based start line (inclusive)
    pub start_line: u32,
    /// 1-based end line (inclusive)
    pub end_line: u32,
}

/// Resolve an anchor to a line range in the given file content.
///
/// - `content`: full file text
/// - `anchor`: text to search for (plain substring match)
/// - `anchor_context`: optional outer scope text that must appear before the anchor
///   (e.g. `"impl Token"` to disambiguate `"fn new"` inside a specific impl block)
///
/// Returns `None` if anchor is not found.
pub fn resolve_anchor(
    content: &str,
    anchor: &str,
    anchor_context: Option<&str>,
) -> Option<AnchorResult> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return None;
    }

    // Find anchor line index (0-based)
    let anchor_line = find_anchor_line(&lines, anchor, anchor_context)?;

    // Expand upward to include leading comments / attributes / decorators
    let start = expand_upward(&lines, anchor_line);

    // Determine block end
    let end = find_block_end(&lines, anchor_line);

    Some(AnchorResult {
        start_line: (start + 1) as u32,
        end_line: (end + 1) as u32,
    })
}

/// Find the 0-based line index that contains the anchor text.
/// If `anchor_context` is given, only match anchors that appear within
/// the block started by the context line.
fn find_anchor_line(
    lines: &[&str],
    anchor: &str,
    anchor_context: Option<&str>,
) -> Option<usize> {
    if let Some(ctx) = anchor_context {
        // Find the context line first
        let ctx_line = lines.iter().position(|l| l.contains(ctx))?;
        let ctx_end = find_block_end(lines, ctx_line);
        // Search within context block
        for i in ctx_line..=ctx_end {
            if lines[i].contains(anchor) && i != ctx_line {
                return Some(i);
            }
        }
        // If anchor text matches the context line itself, allow it
        if lines[ctx_line].contains(anchor) {
            return Some(ctx_line);
        }
        None
    } else {
        lines.iter().position(|l| l.contains(anchor))
    }
}

/// Expand upward from anchor_line to include leading comments, attributes,
/// and decorators that belong to the code block.
fn expand_upward(lines: &[&str], anchor_line: usize) -> usize {
    let mut start = anchor_line;
    while start > 0 {
        let prev = lines[start - 1].trim();
        if prev.is_empty() {
            break;
        }
        if is_comment_or_attribute(prev) {
            start -= 1;
        } else {
            break;
        }
    }
    start
}

/// Check if a trimmed line is a comment, doc-comment, attribute, or decorator.
fn is_comment_or_attribute(trimmed: &str) -> bool {
    // Single-line comments
    trimmed.starts_with("//")
        || trimmed.starts_with('#')  // Python decorator / Rust attribute / shell comment
        || trimmed.starts_with("/*")
        || trimmed.starts_with("*/")
        || trimmed.starts_with('*')  // Middle of block comment
        || trimmed.starts_with("\"\"\"")  // Python docstring
        || trimmed.starts_with("'''")    // Python docstring
        || trimmed.starts_with('@')      // Java/Kotlin/TS annotation
}

/// Find the end of the code block starting at `start_line`.
///
/// Strategy:
/// 1. Look for opening brace `{` in the anchor line or the next few lines.
///    If found → brace-counting mode (C-like languages).
/// 2. If no brace found → indentation mode (Python-like languages).
fn find_block_end(lines: &[&str], start_line: usize) -> usize {
    // Look for opening brace within a small window (the anchor line + next 5 lines)
    let search_end = (start_line + 6).min(lines.len());
    let mut brace_line = None;

    for i in start_line..search_end {
        if lines[i].contains('{') {
            brace_line = Some(i);
            break;
        }
    }

    if let Some(bl) = brace_line {
        find_block_end_brace(lines, bl)
    } else {
        find_block_end_indent(lines, start_line)
    }
}

/// Brace-counting block end detection.
/// Counts `{` and `}` from the brace_line until depth returns to 0.
fn find_block_end_brace(lines: &[&str], brace_line: usize) -> usize {
    let mut depth: i32 = 0;

    for i in brace_line..lines.len() {
        let line = lines[i];
        for ch in line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth <= 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
    }

    // If we never balanced, return last line
    lines.len() - 1
}

/// Indentation-based block end detection.
/// The block ends when we encounter a non-empty line with indentation
/// less than or equal to the anchor line's indentation.
fn find_block_end_indent(lines: &[&str], start_line: usize) -> usize {
    let base_indent = indent_level(lines[start_line]);
    let mut last_content_line = start_line;

    for i in (start_line + 1)..lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            continue;
        }
        let indent = indent_level(line);
        if indent <= base_indent {
            return last_content_line;
        }
        last_content_line = i;
    }

    last_content_line
}

/// Count leading whitespace (spaces; tabs count as 4 spaces).
fn indent_level(line: &str) -> usize {
    let mut level = 0;
    for ch in line.chars() {
        match ch {
            ' ' => level += 1,
            '\t' => level += 4,
            _ => break,
        }
    }
    level
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_function() {
        let code = r#"use std::io;

/// Login handler
pub fn login(user: &str) -> Result<Token> {
    let db = get_db();
    if let Some(r) = db.find(user) {
        if r.verify() {
            return Ok(Token::new(r));
        }
    }
    Err(AuthError::Invalid)
}

pub fn logout() {
    cleanup();
}"#;
        let result = resolve_anchor(code, "fn login", None).unwrap();
        // Should include the doc comment
        assert_eq!(result.start_line, 3);
        // Should end at the closing }
        assert_eq!(result.end_line, 12);
    }

    #[test]
    fn test_rust_function_after_insertion() {
        let code = r#"use std::io;

pub fn health_check() -> bool {
    true
}

/// Login handler
pub fn login(user: &str) -> Result<Token> {
    let db = get_db();
    Ok(Token::new(db))
}
"#;
        let result = resolve_anchor(code, "fn login", None).unwrap();
        assert_eq!(result.start_line, 7); // doc comment
        assert_eq!(result.end_line, 11);  // closing }
    }

    #[test]
    fn test_python_function() {
        let code = r#"import os

def login(user, password):
    db = get_db()
    record = db.find(user)
    if record and record.verify(password):
        return Token(record)
    raise AuthError("Invalid")

def logout(token):
    cleanup(token)
"#;
        let result = resolve_anchor(code, "def login", None).unwrap();
        assert_eq!(result.start_line, 3);
        assert_eq!(result.end_line, 8);
    }

    #[test]
    fn test_anchor_context() {
        let code = r#"struct Foo;

impl Foo {
    fn new() -> Self {
        Foo
    }
}

impl Bar {
    fn new() -> Self {
        Bar
    }
}
"#;
        // Without context: finds first "fn new"
        let r1 = resolve_anchor(code, "fn new", None).unwrap();
        assert_eq!(r1.start_line, 4);

        // With context "impl Bar": finds "fn new" inside Bar
        let r2 = resolve_anchor(code, "fn new", Some("impl Bar")).unwrap();
        assert_eq!(r2.start_line, 10);
        assert_eq!(r2.end_line, 12);
    }

    #[test]
    fn test_anchor_not_found() {
        let code = "fn main() {}\n";
        assert!(resolve_anchor(code, "fn nonexistent", None).is_none());
    }

    #[test]
    fn test_javascript_class() {
        let code = r#"// Auth module
class AuthService {
    constructor(db) {
        this.db = db;
    }

    async login(user, pass) {
        const record = await this.db.find(user);
        return record.verify(pass);
    }
}

module.exports = AuthService;
"#;
        let result = resolve_anchor(code, "class AuthService", None).unwrap();
        assert_eq!(result.start_line, 1); // includes comment
        assert_eq!(result.end_line, 11);  // closing }
    }

    #[test]
    fn test_single_line_function() {
        let code = "fn answer() -> u32 { 42 }\nfn other() {}\n";
        let result = resolve_anchor(code, "fn answer", None).unwrap();
        assert_eq!(result.start_line, 1);
        assert_eq!(result.end_line, 1);
    }

    #[test]
    fn test_multiline_signature() {
        let code = r#"pub fn complex_function(
    arg1: String,
    arg2: u32,
    arg3: bool,
) -> Result<()> {
    do_something();
    Ok(())
}
"#;
        let result = resolve_anchor(code, "fn complex_function", None).unwrap();
        assert_eq!(result.start_line, 1);
        assert_eq!(result.end_line, 8);
    }

    #[test]
    fn test_decorated_python() {
        let code = r#"import flask

@app.route("/login")
@requires_auth
def login():
    return handle_login()

def other():
    pass
"#;
        let result = resolve_anchor(code, "def login", None).unwrap();
        assert_eq!(result.start_line, 3); // includes decorators
        assert_eq!(result.end_line, 6);
    }
}
