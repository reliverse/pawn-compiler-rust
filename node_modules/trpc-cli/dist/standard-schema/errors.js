"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.StandardSchemaV1Error = exports.prettifyStandardSchemaError = void 0;
exports.toDotPath = toDotPath;
const utils_1 = require("./utils");
const prettifyStandardSchemaError = (error) => {
    if (!(0, utils_1.looksLikeStandardSchemaFailure)(error))
        return null;
    const issues = [...error.issues]
        .map(issue => {
        const path = issue.path || [];
        const primitivePathSegments = path.map(segment => {
            if (typeof segment === 'string' || typeof segment === 'number' || typeof segment === 'symbol')
                return segment;
            return segment.key;
        });
        const dotPath = toDotPath(primitivePathSegments);
        return {
            issue,
            path,
            primitivePathSegments,
            dotPath,
        };
    })
        .sort((a, b) => a.path.length - b.path.length);
    const lines = [];
    for (const { issue, dotPath } of issues) {
        let message = `✖ ${issue.message}`;
        if (dotPath)
            message += ` → at ${dotPath}`;
        lines.push(message);
    }
    return lines.join('\n');
};
exports.prettifyStandardSchemaError = prettifyStandardSchemaError;
function toDotPath(path) {
    const segs = [];
    for (const seg of path) {
        if (typeof seg === 'number')
            segs.push(`[${seg}]`);
        else if (typeof seg === 'symbol')
            segs.push(`[${JSON.stringify(String(seg))}]`);
        else if (/[^\w$]/.test(seg))
            segs.push(`[${JSON.stringify(seg)}]`);
        else {
            if (segs.length)
                segs.push('.');
            segs.push(seg);
        }
    }
    return segs.join('');
}
class StandardSchemaV1Error extends Error {
    issues;
    constructor(failure, options) {
        super('Standard Schema error - details in `issues`.', options);
        this.issues = failure.issues;
    }
}
exports.StandardSchemaV1Error = StandardSchemaV1Error;
