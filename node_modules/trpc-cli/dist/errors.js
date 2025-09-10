"use strict";
/** An error thrown when the trpc procedure results in a bad request */
Object.defineProperty(exports, "__esModule", { value: true });
exports.FailedToExitError = exports.CliValidationError = void 0;
class CliValidationError extends Error {
}
exports.CliValidationError = CliValidationError;
/** An error which is only thrown when a custom \`process\` parameter is used. Under normal circumstances, this should not be used, even internally. */
class FailedToExitError extends Error {
    exitCode;
    constructor(message, { exitCode, cause }) {
        const fullMessage = `${message}. The process was expected to exit with exit code ${exitCode} but did not. This may be because a custom \`process\` parameter was used. The exit reason is in the \`cause\` property.`;
        super(fullMessage, { cause });
        this.exitCode = exitCode;
    }
}
exports.FailedToExitError = FailedToExitError;
