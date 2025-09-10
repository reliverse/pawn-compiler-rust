import { StandardSchemaV1 } from './contract';
export declare const prettifyStandardSchemaError: (error: unknown) => string | null;
export declare function toDotPath(path: (string | number | symbol)[]): string;
export declare class StandardSchemaV1Error extends Error implements StandardSchemaV1.FailureResult {
    issues: StandardSchemaV1.FailureResult['issues'];
    constructor(failure: StandardSchemaV1.FailureResult, options?: {
        cause?: Error;
    });
}
