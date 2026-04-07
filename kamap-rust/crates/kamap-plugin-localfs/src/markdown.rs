/// 从 Markdown 内容中提取指定 heading 的 section
pub fn extract_heading_section(content: &str, heading_slug: &str) -> Option<HeadingSection> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;
    let mut section_level = 0;
    let mut start_line = 0;
    let mut section_lines = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(level) = heading_level(trimmed) {
            let slug = slugify(trimmed.trim_start_matches('#').trim());
            if slug == heading_slug && !in_section {
                in_section = true;
                section_level = level;
                start_line = i + 1; // 1-based
                section_lines.push((*line).to_string());
                continue;
            }
            // 遇到同级或更高级标题时结束
            if in_section && level <= section_level {
                break;
            }
        }
        if in_section {
            section_lines.push((*line).to_string());
        }
    }

    if in_section {
        Some(HeadingSection {
            start_line: start_line as u32,
            end_line: (start_line + section_lines.len() - 1) as u32,
            content: section_lines.join("\n"),
        })
    } else {
        None
    }
}

pub struct HeadingSection {
    pub start_line: u32,
    pub end_line: u32,
    pub content: String,
}

fn heading_level(line: &str) -> Option<usize> {
    if line.starts_with('#') {
        let count = line.chars().take_while(|c| *c == '#').count();
        if count <= 6 && line.len() > count && line.chars().nth(count) == Some(' ') {
            return Some(count);
        }
    }
    None
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_heading() {
        let content = "# Title\n\nIntro.\n\n## Login Flow\n\nLogin steps here.\n\n## Logout\n\nLogout steps.\n";
        let section = extract_heading_section(content, "login-flow").unwrap();
        assert_eq!(section.start_line, 5);
        assert!(section.content.contains("Login steps here."));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Login Flow"), "login-flow");
        assert_eq!(slugify("Hello World!"), "hello-world");
    }
}
