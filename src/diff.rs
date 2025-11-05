pub struct DiffLine {
    pub kind: DiffKind,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffKind {
    Header,
    FileHeader,
    Hunk,
    Context,
    Added,
    Removed,
}

pub struct DiffParser;

impl DiffParser {
    pub fn parse(text: &str) -> Vec<DiffLine> {
        text.lines().map(Self::parse_line).collect()
    }

    fn parse_line(line: &str) -> DiffLine {
        if line.starts_with("---") || line.starts_with("+++") {
            DiffLine {
                kind: DiffKind::FileHeader,
                content: line.to_string(),
            }
        } else if line.starts_with("@@") {
            DiffLine {
                kind: DiffKind::Hunk,
                content: line.to_string(),
            }
        } else if line.starts_with('+') {
            DiffLine {
                kind: DiffKind::Added,
                content: line.to_string(),
            }
        } else if line.starts_with('-') {
            DiffLine {
                kind: DiffKind::Removed,
                content: line.to_string(),
            }
        } else {
            DiffLine {
                kind: DiffKind::Context,
                content: line.to_string(),
            }
        }
    }

    pub fn to_styled(lines: &[DiffLine]) -> Vec<String> {
        lines.iter().map(Self::style_line).collect()
    }

    fn style_line(line: &DiffLine) -> String {
        match line.kind {
            DiffKind::FileHeader => {
                format!("\u{1b}[36m{}\u{1b}[0m", line.content)
            }
            DiffKind::Hunk => {
                format!("\u{1b}[35m{}\u{1b}[0m", line.content)
            }
            DiffKind::Added => {
                format!("\u{1b}[32m{}\u{1b}[0m", line.content)
            }
            DiffKind::Removed => {
                format!("\u{1b}[31m{}\u{1b}[0m", line.content)
            }
            DiffKind::Context => line.content.clone(),
            DiffKind::Header => line.content.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_unified_diff() {
        let diff = "--- a/file.rs\n+++ b/file.rs\n@@ -1,5 +1,6 @@\n fn main() {\n-    println!(\"old\");\n+    println!(\"new\");\n     // context\n }\n";

        let lines = DiffParser::parse(diff);
        assert_eq!(lines.len(), 8);
        assert_eq!(lines[0].kind, DiffKind::FileHeader);
        assert_eq!(lines[2].kind, DiffKind::Hunk);
        assert_eq!(lines[3].kind, DiffKind::Context);
        assert_eq!(lines[4].kind, DiffKind::Removed);
        assert_eq!(lines[5].kind, DiffKind::Added);
    }

    #[test]
    fn style_generates_ansi() {
        let lines = vec![
            DiffLine {
                kind: DiffKind::Added,
                content: "+hello".to_string(),
            },
            DiffLine {
                kind: DiffKind::Removed,
                content: "-goodbye".to_string(),
            },
        ];

        let styled = DiffParser::to_styled(&lines);
        assert!(styled[0].contains("\u{1b}[32m"));
        assert!(styled[1].contains("\u{1b}[31m"));
    }
}
