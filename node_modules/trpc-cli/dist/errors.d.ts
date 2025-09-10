/** An error thrown when the trpc procedure results in a bad request */
export declare class CliValidationError extends Error {
}
/** An error which is only thrown when a custom \`process\` parameter is used. Under normal circumstances, this should not be used, even internally. */
export declare class FailedToExitError extends Error {
    readonly exitCode: number;
    constructor(message: string, { exitCode, cause }: {
        exitCode: number;
        cause: unknown;
    });
}
