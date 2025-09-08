use crate::config::Config;

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub rule: &'static str,
    pub message: String,
    pub line: usize,
}

pub fn lint_source(source: &str, cfg: &Config) -> Vec<LintIssue> {
    if !cfg.linter.enabled {
        return Vec::new();
    }
    let mut issues = Vec::new();
    let mut seen_includes = std::collections::HashSet::new();
    // Missing braces: detect function headers not followed by '{' while body is indented
    let mut previous_header: Option<(usize, usize)> = None; // (line_no, indent)
    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line;
        if cfg.linter.check_trailing_whitespace {
            if line.ends_with(' ') || line.ends_with('\t') {
                issues.push(LintIssue {
                    rule: "style.noTrailingWhitespace",
                    message: "Trailing whitespace".into(),
                    line: line_no,
                });
            }
        }

        if cfg.linter.check_duplicate_includes {
            let trimmed = line.trim_start();
            if trimmed.starts_with("#include") {
                // naive extract between quotes or after space
                let token = trimmed.split_whitespace().nth(1).unwrap_or("");
                if !token.is_empty() {
                    if !seen_includes.insert(token.to_string()) {
                        issues.push(LintIssue {
                            rule: "suspicious.duplicateInclude",
                            message: format!("Duplicate include: {}", token),
                            line: line_no,
                        });
                    }
                }
            }
        }

        if cfg.linter.check_missing_braces {
            let trimmed = line.trim_end();
            let tstart = trimmed.trim_start();
            let is_header = (tstart.ends_with("()") || tstart.ends_with(")"))
                && !tstart.contains('{')
                && !tstart.starts_with('#');
            if is_header {
                let indent = leading_whitespace(line);
                previous_header = Some((line_no, indent));
                continue;
            }
            if let Some((hdr_line, hdr_indent)) = previous_header {
                if !tstart.is_empty() {
                    let indent = leading_whitespace(line);
                    if indent > hdr_indent {
                        issues.push(LintIssue {
                            rule: "style.addMissingBraces",
                            message: "Function-like header without braces around body".into(),
                            line: hdr_line,
                        });
                    }
                    previous_header = None;
                }
            }
        }
    }
    // final newline check
    if cfg.linter.enabled
        && cfg.linter.check_newline_eof
        && !source.is_empty()
        && !source.ends_with('\n')
    {
        issues.push(LintIssue {
            rule: "style.newlineAtEndOfFile",
            message: "File should end with a newline".into(),
            line: source.lines().count(),
        });
    }
    issues
}

fn leading_whitespace(s: &str) -> usize {
    s.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}
