import { StandardSchemaV1 } from './contract';
export declare const looksLikeStandardSchemaFailure: (error: unknown) => error is StandardSchemaV1.FailureResult;
export declare const looksLikeStandardSchema: (thing: unknown) => thing is StandardSchemaV1;
