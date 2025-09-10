#!/usr/bin/env node
import { trpcServer, TrpcCliMeta } from 'trpc-cli';

declare const router: trpcServer.TRPCBuiltRouter<{
    ctx: object;
    meta: TrpcCliMeta;
    errorShape: trpcServer.TRPCDefaultErrorShape;
    transformer: false;
}, trpcServer.TRPCDecorateCreateRouterOptions<{
    init: trpcServer.TRPCMutationProcedure<{
        input: {
            pm?: "npm" | "yarn" | "pnpm" | "bun" | "deno" | undefined;
            editors?: ("vscode" | "zed")[] | undefined;
            rules?: ("zed" | "vscode-copilot" | "cursor" | "windsurf" | "claude" | "codex" | "kiro" | "cline" | "amp" | "aider" | "firebase-studio" | "open-hands" | "gemini-cli" | "junie" | "augmentcode" | "kilo-code" | "goose")[] | undefined;
            integrations?: ("husky" | "lefthook" | "lint-staged")[] | undefined;
            removePrettier?: boolean | undefined;
            removeEslint?: boolean | undefined;
            skipInstall?: boolean | undefined;
        };
        output: void;
        meta: TrpcCliMeta;
    }>;
    check: trpcServer.TRPCQueryProcedure<{
        input: string[] | undefined;
        output: void;
        meta: TrpcCliMeta;
    }>;
    fix: trpcServer.TRPCMutationProcedure<{
        input: [string[] | undefined, {
            unsafe?: boolean | undefined;
        }];
        output: void;
        meta: TrpcCliMeta;
    }>;
    doctor: trpcServer.TRPCQueryProcedure<{
        input: void;
        output: void;
        meta: TrpcCliMeta;
    }>;
    lint: trpcServer.TRPCQueryProcedure<{
        input: string[] | undefined;
        output: void;
        meta: TrpcCliMeta;
    }>;
    format: trpcServer.TRPCMutationProcedure<{
        input: [string[] | undefined, {
            unsafe?: boolean | undefined;
        }];
        output: void;
        meta: TrpcCliMeta;
    }>;
}>>;

export { router };
