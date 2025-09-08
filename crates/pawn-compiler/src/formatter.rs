use crate::config::Config;

pub fn format_source(source: &str, cfg: &Config) -> String {
    if !cfg.formatter.enabled {
        return source.to_string();
    }

    // Optional pass to add missing braces for simple function bodies like:
    // main()\n    printf "Hello"\n -> becomes main(){\n    printf "Hello"\n}
    let mut text = source.to_string();
    if cfg.formatter.add_missing_braces {
        text = add_missing_braces(&text);
    }

    // Whitespace normalization
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let mut trimmed = line.to_string();
        if cfg.formatter.trim_trailing_whitespace {
            while trimmed.ends_with(' ') || trimmed.ends_with('\t') {
                trimmed.pop();
            }
        }
        out.push_str(&trimmed);
        out.push('\n');
    }
    if cfg.formatter.insert_final_newline && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn add_missing_braces(input: &str) -> String {
    #[allow(unused_mut)]
    let mut lines: Vec<&str> = input.lines().collect();
    let mut output: Vec<String> = Vec::with_capacity(lines.len() + 2);
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_end();
        // Detect function header without opening brace on same line
        let is_header = {
            let t = trimmed.trim_start();
            t.ends_with("()") || t.ends_with(")") && !t.contains('{') && !t.starts_with("#")
        };
        if is_header {
            // Lookahead: if next non-empty line is indented more than this line, wrap with braces
            let indent_curr = leading_whitespace(line);
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            if j < lines.len() {
                let next_line = lines[j];
                let indent_next = leading_whitespace(next_line);
                if indent_next > indent_curr && !trimmed.ends_with('{') {
                    // Insert opening brace at end of header line
                    output.push(format!("{}{{", trimmed));
                    // Emit body lines until indentation returns to header level or EOF
                    i += 1;
                    while i < lines.len() {
                        let body_line = lines[i];
                        let body_indent = leading_whitespace(body_line);
                        if !body_line.trim().is_empty() && body_indent <= indent_curr {
                            break;
                        }
                        output.push(body_line.to_string());
                        i += 1;
                    }
                    // Insert closing brace aligned with header
                    output.push(format!("{}{}", " ".repeat(indent_curr), "}"));
                    continue; // skip the regular push at loop end
                }
            }
        }
        output.push(trimmed.to_string());
        i += 1;
    }
    output.join("\n")
}

fn leading_whitespace(s: &str) -> usize {
    s.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}
