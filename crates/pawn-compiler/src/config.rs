use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct FormatterConfig {
    pub enabled: bool,
    pub line_width: usize,
    pub trim_trailing_whitespace: bool,
    pub insert_final_newline: bool,
    pub add_missing_braces: bool,
}

#[derive(Debug, Clone, Default)]
pub struct LinterConfig {
    pub enabled: bool,
    pub check_trailing_whitespace: bool,
    pub check_duplicate_includes: bool,
    pub check_missing_braces: bool,
    pub check_newline_eof: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PawnConfig {
    pub globals: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub formatter: FormatterConfig,
    pub linter: LinterConfig,
    pub pawn: PawnConfig,
    pub files: FilesConfig,
}

#[derive(Debug, Clone, Default)]
pub struct FilesConfig {
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
}

pub fn load_config(path: &Path) -> Config {
    let text = fs::read_to_string(path).unwrap_or_default();
    // Minimal detection without regex/serde
    let enabled_formatter = text.contains("\"formatter\"") && text.contains("\"enabled\": true");
    let enabled_linter = text.contains("\"linter\"") && text.contains("\"enabled\": true");

    fn rule_off(text: &str, key: &str) -> bool {
        let a = format!("\"{}\": \"off\"", key);
        let b = format!("\"{}\":\"off\"", key);
        text.contains(&a) || text.contains(&b)
    }
    let check_missing_braces = if text.contains("\"addMissingBraces\"") {
        !rule_off(&text, "addMissingBraces")
    } else {
        true
    };
    let check_trailing_whitespace = !rule_off(&text, "noTrailingWhitespace");
    let check_duplicate_includes = !rule_off(&text, "duplicateInclude");
    let check_newline_eof = !rule_off(&text, "newlineAtEndOfFile");

    // Parse files.includes minimal support: collect entries and split into include/exclude by '!'
    let mut include_globs: Vec<String> = Vec::new();
    let mut exclude_globs: Vec<String> = Vec::new();
    if let Some(start) = text.find("\"includes\"") {
        if let Some(arr_start) = text[start..].find('[') {
            let rest = &text[start + arr_start + 1..];
            if let Some(arr_end) = rest.find(']') {
                let array = &rest[..arr_end];
                for raw in array.split(',') {
                    let s = raw.trim().trim_matches('"');
                    if s.is_empty() {
                        continue;
                    }
                    if s.starts_with('!') {
                        exclude_globs.push(s[1..].to_string());
                    } else {
                        include_globs.push(s.to_string());
                    }
                }
            }
        }
    }
    if include_globs.is_empty() {
        include_globs.push("**".to_string());
    }

    Config {
        formatter: FormatterConfig {
            enabled: enabled_formatter,
            line_width: 100,
            trim_trailing_whitespace: check_trailing_whitespace,
            insert_final_newline: check_newline_eof,
            add_missing_braces: check_missing_braces,
        },
        linter: LinterConfig {
            enabled: enabled_linter,
            check_trailing_whitespace: check_trailing_whitespace,
            check_duplicate_includes: check_duplicate_includes,
            check_missing_braces: check_missing_braces,
            check_newline_eof,
        },
        pawn: PawnConfig {
            globals: vec!["printf".into()],
        },
        files: FilesConfig {
            include_globs,
            exclude_globs,
        },
    }
}
