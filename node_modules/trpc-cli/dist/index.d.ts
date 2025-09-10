import { Command as BaseCommand } from 'commander';
import { AnyRouter } from './trpc-compat';
import { ParsedProcedure, TrpcCli, TrpcCliMeta, TrpcCliParams } from './types';
export * from './types';
export { z } from 'zod/v4';
export * as zod from 'zod';
export * as trpcServer from '@trpc/server';
declare module 'zod/v4' {
    interface GlobalMeta {
        /**
         * If true, this property will be mapped to a positional CLI argument by trpc-cli. Only valid for string, number, or boolean types (or arrays of these types).
         * Note: the order of positional arguments is determined by the order of properties in the schema.
         * For example, the following are different:
         * - `z.object({abc: z.string().meta({positional: true}), xyz: z.string().meta({positional: true})})`
         * - `z.object({xyz: z.string().meta({positional: true}), abc: z.string().meta({positional: true})})`
         */
        positional?: boolean;
        /**
         * If set, this value will be used an alias for the option.
         * Note: this is only valid for options, not positional arguments.
         */
        alias?: string;
    }
}
export declare class Command extends BaseCommand {
    /** @internal track the commands that have been run, so that we can find the `__result` of the last command */
    __ran: Command[];
    __input?: unknown;
    /** @internal stash the return value of the underlying procedure on the command so to pass to `FailedToExitError` for use in a pinch */
    __result?: unknown;
}
/** re-export of the @trpc/server package, just to avoid needing to install manually when getting started */
export { type AnyRouter, type AnyProcedure } from './trpc-compat';
/**
 * @internal takes a trpc router and returns an object that you **could** use to build a CLI, or UI, or a bunch of other things with.
 * Officially, just internal for building a CLI. GLHF.
 */
export declare const parseRouter: <R extends AnyRouter>({ router, ...params }: TrpcCliParams<R>) => [string, ProcedureInfo][];
type ProcedureInfo = {
    meta: TrpcCliMeta;
    parsedProcedure: ParsedProcedure;
    incompatiblePairs: [string, string][];
    procedure: {};
};
/**
 * Run a trpc router as a CLI.
 *
 * @param router A trpc router
 * @param context The context to use when calling the procedures - needed if your router requires a context
 * @param trpcServer The trpc server module to use. Only needed if using trpc v10.
 * @returns A CLI object with a `run` method that can be called to run the CLI. The `run` method will parse the command line arguments, call the appropriate trpc procedure, log the result and exit the process. On error, it will log the error and exit with a non-zero exit code.
 */
export declare function createCli<R extends AnyRouter>({ router, ...params }: TrpcCliParams<R>): TrpcCli;
/** @deprecated renamed to `createCli` */
export declare const trpcCli: typeof createCli;
export { FailedToExitError, CliValidationError } from './errors';
