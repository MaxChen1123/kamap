use std::path::Path;

/// 将路径转换为使用正斜杠的字符串（跨平台兼容）
///
/// Git 在所有平台上都使用正斜杠 `/`，glob 模式也使用正斜杠。
/// 在 Windows 上 `Path::to_string_lossy()` 会生成反斜杠 `\`，
/// 导致与 Git 路径和 glob 模式不匹配。
/// 此函数确保路径始终使用正斜杠，保持跨平台一致性。
pub fn to_forward_slash(path: &Path) -> String {
    let s = path.to_string_lossy();
    if cfg!(windows) {
        s.replace('\\', "/")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_forward_slash_unix_style() {
        let path = Path::new("src/auth/login.ts");
        assert_eq!(to_forward_slash(path), "src/auth/login.ts");
    }

    #[test]
    fn test_forward_slash_preserves_content() {
        let path = Path::new("docs/my-doc.md");
        assert_eq!(to_forward_slash(path), "docs/my-doc.md");
    }
}
