#!/usr/bin/env node
"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const _1 = require(".");
const commander_1 = require("commander");
const path = __importStar(require("path"));
const program = new commander_1.Command('trpc-cli');
program.allowExcessArguments();
program.allowUnknownOption();
program.enablePositionalOptions();
program.passThroughOptions();
program.helpOption(false);
program.argument('filepath', 'The filepath of the module with the trpc router');
program.option('-e, --export [export]', 'The name of the export to use from the module. If not provided, all exports will be checked for a trpc router.');
program.option('-r, --require [module]', 'A module (or comma-separated modules) to require before running the cli. Can be used to pass in options for the trpc router. e.g. --require dotenv/config');
program.option('-i, --import [module]', 'A module (or comma-separated modules) to import before running the cli. Can be used to pass in options for the trpc router. e.g. --import tsx/esm');
program.action(async () => {
    const [filepath, ...argv] = program.args;
    if (filepath === '-h' || filepath === '--help') {
        program.help();
        return;
    }
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
    const options = program.opts();
    for (const r of options.require?.split(',') || []) {
        // eslint-disable-next-line @typescript-eslint/no-require-imports
        require(r);
    }
    for (const m of options.import?.split(',') || []) {
        await import(m);
    }
    if (!options.require && !options.import) {
        try {
            // eslint-disable-next-line @typescript-eslint/no-require-imports
            require('tsx/cjs');
            // @ts-expect-error - this might not be available, that's why we're catching
            await import('tsx/esm');
        }
        catch {
            // don't worry
        }
    }
    const fullpath = path.resolve(process.cwd(), filepath);
    let importedModule = (await import(fullpath));
    while ('module.exports' in importedModule && importedModule?.['module.exports'] !== importedModule) {
        // this is a cjs-like module, possibly what tsx gives us
        importedModule = importedModule?.['module.exports'];
    }
    while ('default' in importedModule && importedModule?.default !== importedModule) {
        // depending on how it's loaded we can end up with weird stuff like `{default: {default: {myRouter: ...}}}`
        importedModule = importedModule?.default;
    }
    let router;
    const isTrpcRouterLike = (value) => Boolean(value?._def?.procedures);
    if (options.export) {
        router = importedModule[options.export];
        if (!isTrpcRouterLike(router)) {
            throw new Error(`Expected a trpc router in ${filepath}.${options.export}, got ${typeof router}`);
        }
    }
    else if (isTrpcRouterLike(importedModule)) {
        router = importedModule;
    }
    else {
        const routerExports = Object.values(importedModule).filter(isTrpcRouterLike);
        if (routerExports.length !== 1) {
            throw new Error(`Expected exactly one trpc router in ${filepath}, found ${routerExports.length}. Exports: ${Object.keys(importedModule).join(', ')}`);
        }
        router = routerExports[0];
    }
    const cli = (0, _1.createCli)({ router });
    await cli.run({ argv });
});
program.parse(process.argv);
