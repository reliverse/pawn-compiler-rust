// Bun script: compile all example Pawn files and report results
// Usage: bun run scripts/commands/test-examples.ts

import { spawn } from "bun";
import { mkdirSync, readdirSync } from "fs";
import { join, basename, extname } from "path";

const repoRoot = process.cwd();
const examplesDir = join(repoRoot, "original-pawn-compiler", "examples");
const outputDir = join(repoRoot, "target", "test_outputs");
const pawncPath = join(repoRoot, "target", "debug", process.platform === "win32" ? "pawnc.exe" : "pawnc");

async function run() {
    console.log("Building compiler (cargo build)...");
    const build = spawn({ cmd: ["cargo", "build"], stdout: "inherit", stderr: "inherit" });
    const buildExit = await build.exited;
    if (buildExit !== 0) {
        console.error(`Build failed with code ${buildExit}`);
        process.exit(buildExit);
    }

    mkdirSync(outputDir, { recursive: true });

    const entries = readdirSync(examplesDir, { withFileTypes: true });
    const files = entries
        .filter((e) => e.isFile())
        .map((e) => e.name)
        .filter((n) => [".p", ".pwn"].includes(extname(n).toLowerCase()));

    if (files.length === 0) {
        console.warn("No example files found.");
        return;
    }

    console.log(`Found ${files.length} example(s). Compiling...`);

    let passed = 0;
    let failed = 0;
    const failures: { file: string; code: number }[] = [];

    for (const name of files) {
        const inputPath = join(examplesDir, name);
        const outName = basename(name, extname(name)) + ".amx";
        const outPath = join(outputDir, outName);
        const cmd = [pawncPath, inputPath, outPath];
        const child = spawn({ cmd, stdout: "pipe", stderr: "pipe" });
        const { stdout, stderr } = await child;
        const exitCode = await child.exited;
        const outStr = await new Response(stdout).text();
        const errStr = await new Response(stderr).text();
        const prefix = exitCode === 0 ? "PASS" : "FAIL";
        console.log(`[${prefix}] ${name}`);
        if (outStr.trim().length) console.log(outStr.trim());
        if (errStr.trim().length) console.error(errStr.trim());
        if (exitCode === 0) {
            passed++;
        } else {
            failed++;
            failures.push({ file: name, code: exitCode });
        }
    }

    console.log("");
    console.log(`Summary: ${passed} passed, ${failed} failed`);
    if (failed > 0) {
        for (const f of failures) {
            console.log(` - ${f.file} (exit ${f.code})`);
        }
        process.exit(1);
    }
}

run().catch((e) => {
    console.error(e);
    process.exit(1);
});


