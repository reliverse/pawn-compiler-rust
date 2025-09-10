"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.looksLikeStandardSchema = exports.looksLikeStandardSchemaFailure = void 0;
const looksLikeStandardSchemaFailure = (error) => {
    return !!error && typeof error === 'object' && 'issues' in error && Array.isArray(error.issues);
};
exports.looksLikeStandardSchemaFailure = looksLikeStandardSchemaFailure;
const looksLikeStandardSchema = (thing) => {
    return !!thing && typeof thing === 'object' && '~standard' in thing && typeof thing['~standard'] === 'object';
};
exports.looksLikeStandardSchema = looksLikeStandardSchema;
