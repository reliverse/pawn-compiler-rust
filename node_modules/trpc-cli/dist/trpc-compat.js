"use strict";
/* eslint-disable @typescript-eslint/no-explicit-any */
Object.defineProperty(exports, "__esModule", { value: true });
exports.isOrpcRouter = exports.isTrpc11Router = exports.isTrpc11Procedure = void 0;
const isTrpc11Procedure = (procedure) => {
    return 'type' in procedure._def && typeof procedure._def.type === 'string';
};
exports.isTrpc11Procedure = isTrpc11Procedure;
const isTrpc11Router = (router) => {
    if ((0, exports.isOrpcRouter)(router))
        return false;
    const procedure = Object.values(router._def.procedures)[0];
    return Boolean(procedure && (0, exports.isTrpc11Procedure)(procedure));
};
exports.isTrpc11Router = isTrpc11Router;
const isOrpcRouter = (router) => {
    // this could fall down if someone tries to pass a trpc router which doesn't use t.router(...) - but you're not allowed to do that ok!
    return !('_def' in router) || (router._def && '~orpc' in router._def);
};
exports.isOrpcRouter = isOrpcRouter;
