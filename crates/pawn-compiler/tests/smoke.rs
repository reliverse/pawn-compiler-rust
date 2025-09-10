use pawn_amx::AmxRuntime;
use pawn_compiler::compile;

#[test]
fn compiles_and_runs_minimal_printf_program() {
    let source = r#"
        main() {
            printf("hello from pawn");
        }
    "#;

    // Compile to bytecode
    let bytecode = compile(source).expect("compile should succeed");

    // Initialize AMX runtime and execute main
    let mut runtime = AmxRuntime::new();
    runtime
        .init(&bytecode)
        .expect("runtime init should succeed");

    // Register a dummy printf; current Sysreq handler ignores it in MVP
    runtime.register_native("printf".to_string(), |_amx, _params| 0);

    // Execute; should complete without error
    let result = runtime
        .exec(pawn_amx::AMX_EXEC_MAIN)
        .expect("exec should succeed");
    assert_eq!(result, 0);
}
