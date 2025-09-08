//! CLI interface for Pawn compiler

use pawn_amx::*;
use pawn_compiler::{LintIssue, compile as compile_lib, format_source, lint_source, load_config};
use std::fs;
use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("pawnc")
        .about("Pawn Compiler (Rust MVP)")
        .arg(Arg::new("input").required(false))
        .arg(Arg::new("output").required(false))
        .arg(
            Arg::new("check")
                .long("check")
                .help("Run linter on input")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("fix")
                .long("fix")
                .help("Format input (writes back)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .num_args(1)
                .help("Path to rustpwn.json"),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").map(|s| s.to_string());
    let output_file = matches
        .get_one::<String>("output")
        .map(|s| s.as_str())
        .unwrap_or("output.amx");

    let cfg_path = matches
        .get_one::<String>("config")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("rustpwn.json"));

    let cfg = load_config(&cfg_path);

    let flag_check = matches.get_flag("check");
    let flag_fix = matches.get_flag("fix");

    if input_file.is_none() && (flag_check || flag_fix) {
        // Project-wide check/fix
        let root = std::env::current_dir()?;
        let files = collect_pawn_files(&root, &cfg);
        if files.is_empty() {
            println!("No Pawn files found.");
            return Ok(());
        }
        let mut had_issues = false;
        for path in files {
            let content = match fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if flag_check {
                let issues: Vec<LintIssue> = lint_source(&content, &cfg);
                for i in issues {
                    had_issues = true;
                    eprintln!("{}:{}: {} ({})", path.display(), i.line, i.message, i.rule);
                }
            } else if flag_fix {
                let formatted = format_source(&content, &cfg);
                if formatted != content {
                    let _ = fs::write(&path, formatted);
                    println!("Formatted {}", path.display());
                }
            }
        }
        if flag_check && had_issues {
            std::process::exit(1);
        }
        return Ok(());
    }

    let input_file = match input_file {
        Some(s) => s,
        None => {
            println!("Usage: pawnc [--check|--fix] [--config <path>] <input_file> [output_file]");
            return Ok(());
        }
    };

    // Read input file
    let source_code = fs::read_to_string(&input_file)?;

    if flag_check {
        let issues: Vec<LintIssue> = lint_source(&source_code, &cfg);
        if issues.is_empty() {
            println!("No issues found.");
            return Ok(());
        } else {
            for i in issues {
                eprintln!("{}:{}: {} ({})", &input_file, i.line, i.message, i.rule);
            }
            std::process::exit(1);
        }
    }

    if flag_fix {
        let formatted = format_source(&source_code, &cfg);
        if formatted != source_code {
            fs::write(&input_file, formatted)?;
            println!("Formatted {}", &input_file);
        } else {
            println!("Already formatted: {}", &input_file);
        }
        return Ok(());
    }

    // Compile
    println!("Compiling {} to {}", input_file, output_file);
    let preprocessed = preprocess(&source_code);
    match compile_lib(&preprocessed) {
        Ok(bytecode) => {
            // Write bytecode to file
            fs::write(output_file, &bytecode)?;
            println!("Compilation successful! Output written to {}", output_file);

            // For MVP, also try to run the bytecode
            if let Err(e) = run_bytecode(&bytecode) {
                println!("Warning: Could not run bytecode: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

// legacy usage function kept for reference; not used with clap
#[allow(dead_code)]
fn print_usage() {}

// Minimal preprocessor for MVP: drop lines starting with #include and strip trailing semicolonsless printf forms
fn preprocess(input: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#include") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn run_bytecode(bytecode: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // Create AMX runtime
    let mut runtime = AmxRuntime::new();

    // Initialize with bytecode
    runtime.init(bytecode)?;

    // Register printf native
    runtime.register_native("printf".to_string(), |_amx, params| {
        if let Some(format_string) = params.get(0) {
            // For MVP, just print the string
            println!("{}", format_string);
        }
        0
    });

    // Execute
    let result = runtime.exec(AMX_EXEC_MAIN)?;
    println!("Execution completed with result: {}", result);

    Ok(())
}

fn collect_pawn_files(
    root: &std::path::Path,
    cfg: &pawn_compiler::Config,
) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    let include_globs = &cfg.files.include_globs;
    let exclude_globs = &cfg.files.exclude_globs;
    while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if name == ".git"
                    || name == "node_modules"
                    || name == "dist"
                    || name.starts_with("dist-")
                    || name == "target"
                    || name == ".turbo"
                    || name == ".vercel"
                    || (name == "styles" && path.join("dist").is_dir())
                {
                    continue;
                }
                stack.push(path);
            } else {
                if file_matches(&path, include_globs, exclude_globs) {
                    out.push(path);
                }
            }
        }
    }
    out
}

fn file_matches(path: &std::path::Path, includes: &[String], excludes: &[String]) -> bool {
    // very rough glob matching supporting ** and suffix extension checks commonly used here
    let rel = path.to_string_lossy();
    if excludes.iter().any(|g| glob_match(&rel, g)) {
        return false;
    }
    if includes.iter().any(|g| glob_match(&rel, g)) {
        return true;
    }
    // default: allow only pawn extensions
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    ext == "p" || ext == "pwn" || ext == "inc"
}

fn glob_match(text: &str, pat: &str) -> bool {
    // minimal: "**" matches any, "*" matches within a segment. If pat has no wildcard and is a directory, check prefix
    if pat == "**" {
        return true;
    }
    if pat.contains("*") {
        // naive: replace ** with .* and * with [^/]*
        let mut regex = String::new();
        let mut chars = pat.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '.' => regex.push_str("\\."),
                '?' => regex.push('.'),
                '*' => {
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        regex.push_str(".*");
                    } else {
                        regex.push_str("[^/]*");
                    }
                }
                '/' | '\\' => regex.push_str("[/\\]"),
                _ => regex.push(c),
            }
        }
        return regex::Regex::new(&format!("^{}$", regex))
            .map(|r| r.is_match(text))
            .unwrap_or(false);
    }
    // No wildcard case: exact or prefix match
    if pat.ends_with('/') {
        text.replace('\\', "/")
            .starts_with(&pat.trim_end_matches('/'))
    } else {
        text.ends_with(pat)
    }
}
