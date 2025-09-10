# Pawn Community Compiler Rust Rewrite

This document mainly outlines the plan for rewriting the original Pawn compiler in Rust, based on the analysis of the original C implementation. This TODO list will be updated as the project progresses and new requirements are discovered.

## Project Overview

The Pawn compiler consists of two main components:

1. **Compiler** (`source/compiler/`) - Parses Pawn source code and generates AMX bytecode
2. **AMX Runtime** (`source/amx/`) - Abstract Machine eXecutor for running compiled Pawn bytecode

## Phase 1: Project Setup and Foundation

### 1.1 Project Structure

- [x] Set up proper Rust project structure with workspace
- [x] Create separate crates for compiler and AMX runtime (+ CLI)
- [x] Set up Cargo.toml with core dependencies
- [ ] Configure build system and CI/CD
- [ ] Set up testing framework

### 1.2 Core Dependencies

- [x] Choose lexer/parser library (hand-written for MVP)
- [x] Set up error handling (thiserror)
- [ ] Choose logging framework (log, tracing)
- [ ] Set up CLI argument parsing (clap)
- [x] Choose file I/O and path handling libraries (std)

## Phase 2: AMX Runtime Implementation

- ### 2.1 Core AMX Types and Structures

- [x] Implement AMX header structure (basic read/write, validate)
- [x] Implement cell types and operations (core types)
- [x] Implement AMX state management (minimal runtime state)
- [ ] Port AMX core functionality from `amx.c`
- [ ] Implement AMX instruction set

### 2.2 AMX Core Modules

- [ ] **AMX Core** (`amxcore.c` → `amx_core.rs`)
  - [ ] AMX initialization and cleanup
  - [ ] Memory management
  - [ ] Stack operations
- [ ] **AMX Execution** (`amxexec.asm` → `amx_exec.rs`)
  - [ ] Instruction execution engine
  - [ ] JIT compilation support
  - [ ] Performance optimizations
- [ ] **AMX Debug** (`amxdbg.c` → `amx_debug.rs`)
  - [ ] Debug information handling
  - [ ] Breakpoint support
  - [ ] Variable inspection

### 2.3 AMX Extensions

- [ ] **Console I/O** (`amxcons.c` → `amx_console.rs`)
- [ ] **File Operations** (`amxfile.c` → `amx_file.rs`)
- [ ] **String Operations** (`amxstring.c` → `amx_string.rs`)
- [ ] **Time Functions** (`amxtime.c` → `amx_time.rs`)
- [ ] **Process Management** (`amxprocess.c` → `amx_process.rs`)
- [ ] **Floating Point** (`amxfloat.c` → `amx_float.rs`)
- [ ] **Fixed Point** (`fixed.c` → `amx_fixed.rs`)

### 2.4 AMX Utilities

- [ ] **AMX Args** (`amxargs.c` → `amx_args.rs`)
- [ ] **AMX Aux** (`amxaux.c` → `amx_aux.rs`)
- [ ] **AMX GC** (`amxgc.c` → `amx_gc.rs`)
- [ ] **AMX DGram** (`amxdgram.c` → `amx_dgram.rs`)

### 2.5 AMX Additional Components

- [ ] **Pattern Matching** (`fpattern.c` → `amx_pattern.rs`)
  - [ ] Filename pattern matching utilities
  - [ ] Wildcard support
- [ ] **INI File Support** (`minIni.c` → `amx_ini.rs`)
  - [ ] Configuration file parsing
  - [ ] Multi-platform INI support
- [ ] **Terminal Support** (`term_ga.c`, `termwin.c` → `amx_terminal.rs`)
  - [ ] GraphApp terminal interface
  - [ ] Windows terminal interface
  - [ ] Cross-platform terminal abstraction
- [ ] **JIT Compiler** (`amxjitr.asm`, `amxjits.asm` → `amx_jit.rs`)
  - [ ] Just-in-time compilation
  - [ ] Assembly code generation
  - [ ] Performance optimization

## Phase 3: Compiler Frontend

### 3.1 Lexical Analysis

- [x] Implement minimal Pawn lexer (ints, floats, strings, identifiers, operators)
- [ ] Handle Pawn keywords and operators
- [ ] Support for different character encodings
- [x] Handle preprocessor directives (MVP: ignore `#include` lines in CLI)
- [ ] Implement string and character literal parsing
- [ ] **Internationalization Support** (`sci18n.c` → `compiler_i18n.rs`)
  - [ ] Unicode and UTF-8 support
  - [ ] Codepage translation
  - [ ] Multi-byte character handling
  - [ ] Character set mapping

### 3.2 Preprocessor

- [x] Implement `#include` directive (MVP: ignore-only in CLI)
- [ ] Implement `#define` and macro expansion
- [ ] Implement `#if`, `#ifdef`, `#ifndef`, `#else`, `#endif`
- [ ] Implement `#pragma` directives
- [ ] Handle conditional compilation

### 3.3 Syntax Analysis

- [x] Implement minimal parser to support `main(){ printf("..."); }` and single-line `main()` body
- [x] Build Abstract Syntax Tree (AST)
- [ ] Handle Pawn-specific constructs:
  - [ ] Function definitions and declarations
  - [ ] Variable declarations with tags
  - [ ] Array declarations and indexing
  - [ ] String handling
  - [ ] Native function declarations
  - [ ] Public function declarations

### 3.4 Semantic Analysis

- [x] Symbol table management (minimal, built-in `printf`)
- [ ] Type checking and tag system
- [ ] Variable scope resolution
- [ ] Function signature validation
- [ ] Array dimension checking
- [ ] Constant folding and optimization

### 3.5 Compiler Utilities

- [ ] **String Handling** (`lstring.c` → `compiler_string.rs`)
  - [ ] Dynamic string management
  - [ ] String pool implementation
- [ ] **Memory File** (`memfile.c` → `compiler_memfile.rs`)
  - [ ] In-memory file operations
  - [ ] Virtual file system
- [ ] **Hash Table** (`hashtable/` → `compiler_hashtable.rs`)
  - [ ] Symbol table implementation
  - [ ] Fast symbol lookup
- [ ] **State Management** (`scstate.c` → `compiler_state.rs`)
  - [ ] Automaton state handling
  - [ ] State machine management
- [ ] **List Management** (`sclist.c` → `compiler_list.rs`)
  - [ ] Dynamic list operations
  - [ ] Symbol list management
- [ ] **Variable Management** (`scvars.c` → `compiler_vars.rs`)
  - [ ] Variable scope tracking
  - [ ] Variable lifetime management
- [ ] **Stub Generation** (`scstub.c` → `compiler_stub.rs`)
  - [ ] Function stub generation
  - [ ] State selector stubs

## Phase 4: Compiler Backend

### 4.1 Code Generation

- [x] Generate AMX bytecode from AST (MVP: CONST/PUSH/ALU/printf Sysreq + HALT)
- [ ] Implement instruction selection
- [ ] Handle function calls and returns
- [ ] Implement control flow (if, while, for, switch)
- [ ] Handle array operations
- [ ] Implement string operations

### 4.2 Optimization

- [ ] Dead code elimination
- [ ] Constant propagation
- [ ] Register allocation
- [ ] Loop optimization
- [ ] Function inlining

### 4.3 Output Generation

- [ ] Generate AMX files (.amx)
- [ ] Generate assembly listings
- [ ] Generate debug information
- [ ] Support for different output formats

## Phase 5: Compiler Driver and Tools

### 5.1 Main Compiler Driver

- [x] Implement `pawnc` MVP CLI
- [ ] Command-line argument parsing
- [ ] File compilation pipeline
- [ ] Error reporting and diagnostics
- [ ] Integration with build systems
- [ ] **Library Interface** (`libpawnc.c` → `pawnc_lib.rs`)
  - [ ] Shared library interface
  - [ ] DLL export functions
  - [ ] C API compatibility

### 5.2 Disassembler

- [ ] Implement `pawndisasm` equivalent
- [ ] AMX bytecode disassembly
- [ ] Debug information display
- [ ] Cross-reference generation

### 5.3 Runtime Tools

- [ ] Implement `pawnrun` equivalent
- [ ] AMX file execution
- [ ] Debug mode support
- [ ] Performance profiling
- [ ] **PawnRun Variants** (`pawnrun/` → `pawnrun_variants.rs`)
  - [ ] Basic runtime (`prun1.c`)
  - [ ] Debug runtime (`prun2.c`)
  - [ ] Virtual memory runtime (`prun3.c`)
  - [ ] Function call runtime (`prun4.c`)
  - [ ] Garbage collection runtime (`prun5.c`)
  - [ ] JIT runtime (`prun_jit.c`)

### 5.4 Additional Tools

- [ ] **PawnRun Server** (`pawnruns.c` → `pawnrun_server.rs`)
  - [ ] Server mode execution
  - [ ] Multiple script handling
- [ ] **Pawn Debugger** (`pawndbg.c` → `pawn_debugger.rs`)
  - [ ] Interactive debugging
  - [ ] Breakpoint management
  - [ ] Variable inspection

## Phase 6: Language Features

### 6.1 Core Language Features

- [ ] Variables and constants
- [ ] Data types (int, float, bool, string)
- [ ] Arrays (single and multi-dimensional)
- [ ] Functions and procedures
- [ ] Control structures (if, while, for, switch)
- [ ] Operators and expressions

### 6.2 Advanced Features

- [ ] Tag system for type safety
- [ ] Native function interface
- [ ] Public/private function visibility
- [ ] String manipulation
- [ ] File I/O operations
- [ ] Error handling

### 6.3 SA-MP Compatibility

- [ ] SA-MP and open.mp (OpenMultiplayer) specific natives
- [ ] Callback system
- [ ] Plugin interface
- [ ] Memory management compatibility

## Phase 7: Testing and Validation

### 7.1 Unit Tests

- [ ] Test individual compiler components
- [ ] Test AMX runtime modules
- [ ] Test error handling
- [ ] Test edge cases

### 7.2 Integration Tests

- [ ] Test complete compilation pipeline
- [ ] Test with existing Pawn code
- [ ] Test SA-MP compatibility
- [ ] Performance benchmarking

### 7.3 Regression Tests

- [ ] Test against original compiler output
- [ ] Test with example programs
- [ ] Test with real-world SA-MP scripts
- [ ] Validate AMX bytecode compatibility

### 7.4 Compiler Test Suite

- [ ] **Test Framework** (`tests/` → `test_framework.rs`)
  - [ ] Port existing test cases (94 `.pwn` files)
  - [ ] Test metadata handling (93 `.meta` files)
  - [ ] Include file testing (4 `.inc` files)
  - [ ] Automated test runner
- [ ] **Test Categories**
  - [ ] Error handling tests (`error_*.pwn`)
  - [ ] Warning tests (`warning_*.pwn`)
  - [ ] GitHub issue tests (`gh_*.pwn`)
  - [ ] Feature tests (`__emit_*.pwn`, `__nameof_*.pwn`)
  - [ ] Pragma tests (`__pragma_*.pwn`)
  - [ ] Static assertion tests (`__static_assert.pwn`)
  - [ ] Array and string tests
  - [ ] Tag and enum tests
  - [ ] Multi-dimensional array tests

## Phase 8: Documentation and Examples

### 8.1 Documentation

- [ ] API documentation (rustdoc)
- [ ] User manual
- [ ] Migration guide from original compiler
- [ ] Performance comparison

### 8.2 Examples

- [ ] Port example programs from `examples/`
  - [ ] Basic examples (`hello.p`, `hello2.p`, `fib.p`)
  - [ ] Algorithm examples (`gcd.p`, `hanoi.p`, `sieve.p`)
  - [ ] Data structure examples (`queue.p`, `stack.inc`, `set.p`)
  - [ ] String processing (`rot13.p`, `wcount.p`, `strtok.inc`)
  - [ ] File I/O examples (`readfile.p`, `rpn.p`)
  - [ ] GUI examples (`gtkcalc.p`, `traffic.p`, `traffic2.p`)
  - [ ] Advanced examples (`cards.p`, `turtle.p`, `quine.p`)
- [ ] Create new Rust-specific examples
- [ ] SA-MP integration examples
- [ ] Performance optimization examples

## Phase 9: Performance and Optimization

### 9.1 Compiler Performance

- [ ] Optimize compilation speed
- [ ] Memory usage optimization
- [ ] Parallel compilation support
- [ ] Incremental compilation

### 9.2 Runtime Performance

- [ ] AMX execution optimization
- [ ] JIT compilation improvements
- [ ] Memory management optimization
- [ ] Garbage collection tuning

## Phase 10: Distribution and Deployment

### 10.1 Packaging

- [ ] Create binary distributions
- [ ] Cross-platform builds
- [ ] Package managers (cargo, npm, etc.)
- [ ] Docker images

### 10.2 Integration

- [ ] IDE plugins and extensions
- [ ] Build system integration
- [ ] CI/CD pipeline support
- [ ] Community tools integration

## Implementation Notes

### Key Files to Port

**Compiler Core:**

- `sc.h` → Core compiler definitions and types
- `sc1.c` → Lexical analysis
- `sc2.c` → Preprocessor
- `sc3.c` → Parser
- `sc4.c` → Semantic analysis
- `sc5.c` → Code generation
- `sc6.c` → Optimization
- `sc7.c` → Output generation

**Compiler Utilities:**

- `lstring.c` → String handling
- `memfile.c` → Memory file operations
- `hashtable/` → Hash table implementation
- `scstate.c` → State management
- `sclist.c` → List management
- `scvars.c` → Variable management
- `scstub.c` → Stub generation
- `sci18n.c` → Internationalization
- `scmemfil.c` → Memory file integration

**AMX Runtime:**

- `amx.h` → AMX core definitions
- `amx.c` → AMX core implementation
- `amxcore.c` → AMX core operations
- `amxexec.asm` → Instruction execution
- `amxdbg.c` → Debug support

**AMX Extensions:**

- `amxcons.c` → Console I/O
- `amxfile.c` → File operations
- `amxstring.c` → String operations
- `amxtime.c` → Time functions
- `amxprocess.c` → Process management
- `amxfloat.c` → Floating point
- `fixed.c` → Fixed point arithmetic
- `amxargs.c` → Argument handling
- `amxaux.c` → Auxiliary functions
- `amxgc.c` → Garbage collection
- `amxdgram.c` → Datagram support

**AMX Additional:**

- `fpattern.c` → Pattern matching
- `minIni.c` → INI file support
- `term_ga.c` → GraphApp terminal
- `termwin.c` → Windows terminal
- `amxjitr.asm` → JIT compiler
- `amxjits.asm` → JIT support

**Tools:**

- `pawncc.c` → Main compiler driver
- `libpawnc.c` → Library interface
- `pawndisasm.c` → Disassembler
- `pawnrun.c` → Runtime executor
- `pawnruns.c` → Runtime server
- `pawndbg.c` → Debugger
- `pawnrun/` → Runtime variants

**Include Files:**

- `include/*.inc` → Standard library headers
- `examples/*.p` → Example programs
- `examples/*.inc` → Example includes

### Rust-Specific Considerations

1. **Memory Safety**: Leverage Rust's ownership system for safe memory management
2. **Error Handling**: Use Result types for proper error propagation
3. **Performance**: Utilize Rust's zero-cost abstractions
4. **Concurrency**: Explore parallel compilation opportunities
5. **C Interop**: Use FFI for compatibility with existing C libraries if needed

### Testing Strategy

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test complete compilation pipeline
3. **Property Tests**: Use quickcheck for random testing
4. **Fuzz Testing**: Test with malformed input
5. **Regression Tests**: Compare output with original compiler

## Success Criteria

- [ ] Successfully compile all example programs from `original-pawn-compiler/examples/`
- [x] MVP: parse and compile `examples/hello.p` and `examples/hello2.p` to AMX or emit clear error

## MVP To-Do

- [x] Start AMX execution at header.cod (runtime fix)
- [x] Wire CLI to library `compile()` and add minimal preprocessor (ignore `#include`)
- [x] Parser: support `main()` single-statement body across newlines
- [x] Codegen: set header.cip to entry point (beginning of code section)
- [x] Add a tiny smoke test that compiles and runs a minimal `printf` program
  - [ ] Ensure publics/natives tables minimal but valid
- [ ] Generate identical AMX bytecode to original compiler for test cases
- [ ] Maintain SA-MP compatibility
- [ ] Achieve comparable or better performance than original compiler
- [ ] Pass all existing test suites (94 test cases in `tests/`)
- [ ] Provide comprehensive documentation and examples
- [ ] Support all standard library includes (`include/*.inc`)
- [ ] Implement all AMX runtime extensions
- [ ] Support all compiler features and pragmas
- [ ] Maintain full compatibility with original compiler API

## Formatter and Linter for Pawn

This rewrite includes a built-in, Biome-like formatter and linter for Pawn source files:

- Supported file types: `.p`, `.pwn`, `.inc`
- Configuration lives in `rustpwn.json` at the repository root
- Defaults: 100 character line width, spaces for indentation, trailing whitespace trimmed, final newline ensured
- Linting: sensible defaults enabled (trailing whitespace, unreachable code, duplicate includes, etc.)

### Usage

- CLI integration (planned):
  - `pawnc --check` to run the linter
  - `pawnc --fix` to apply formatting
  - `pawnc --config rustpwn.json` to point to a custom config

### Configuration

Adjust rules and formatting options in `rustpwn.json`. Example keys:

- `formatter.indentStyle`, `formatter.lineWidth`, `formatter.trimTrailingWhitespace`, `formatter.insertFinalNewline`
- `linter.rules.style.noTrailingWhitespace`, `linter.rules.suspicious.duplicateInclude`
- `pawn.globals` to add known global symbols
