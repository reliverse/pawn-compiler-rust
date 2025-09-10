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
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.CliValidationError = exports.FailedToExitError = exports.trpcCli = exports.parseRouter = exports.Command = exports.trpcServer = exports.zod = exports.z = void 0;
exports.createCli = createCli;
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
const trpcServer11 = __importStar(require("@trpc/server"));
const commander_1 = require("commander");
const util_1 = require("util");
const completions_1 = require("./completions");
const errors_1 = require("./errors");
const json_1 = require("./json");
const json_schema_1 = require("./json-schema");
const logging_1 = require("./logging");
const parse_procedure_1 = require("./parse-procedure");
const prompts_1 = require("./prompts");
const errors_2 = require("./standard-schema/errors");
const utils_1 = require("./standard-schema/utils");
const trpc_compat_1 = require("./trpc-compat");
const util_2 = require("./util");
__exportStar(require("./types"), exports);
var v4_1 = require("zod/v4");
Object.defineProperty(exports, "z", { enumerable: true, get: function () { return v4_1.z; } });
exports.zod = __importStar(require("zod"));
exports.trpcServer = __importStar(require("@trpc/server"));
class Command extends commander_1.Command {
    /** @internal track the commands that have been run, so that we can find the `__result` of the last command */
    __ran = [];
    __input;
    /** @internal stash the return value of the underlying procedure on the command so to pass to `FailedToExitError` for use in a pinch */
    __result;
}
exports.Command = Command;
/**
 * @internal takes a trpc router and returns an object that you **could** use to build a CLI, or UI, or a bunch of other things with.
 * Officially, just internal for building a CLI. GLHF.
 */
// todo: maybe refactor to remove CLI-specific concepts like "positional parameters" and "options". Libraries like trpc-ui want to do basically the same thing, but here we handle lots more validation libraries and edge cases. We could share.
const parseRouter = ({ router, ...params }) => {
    if ((0, trpc_compat_1.isOrpcRouter)(router))
        return parseOrpcRouter({ router, ...params });
    return parseTrpcRouter({ router, ...params });
};
exports.parseRouter = parseRouter;
const parseTrpcRouter = ({ router, ...params }) => {
    const defEntries = Object.entries(router._def.procedures);
    return defEntries.map(([procedurePath, procedure]) => {
        const meta = getMeta(procedure);
        if (meta.jsonInput) {
            return [procedurePath, { meta, parsedProcedure: jsonProcedureInputs(), incompatiblePairs: [], procedure }];
        }
        const procedureInputsResult = (0, parse_procedure_1.parseProcedureInputs)(procedure._def.inputs, params);
        if (!procedureInputsResult.success) {
            const procedureInputs = jsonProcedureInputs(`procedure's schema couldn't be converted to CLI arguments: ${procedureInputsResult.error}`);
            return [procedurePath, { meta, parsedProcedure: procedureInputs, incompatiblePairs: [], procedure }];
        }
        const procedureInputs = procedureInputsResult.value;
        const incompatiblePairs = (0, json_schema_1.incompatiblePropertyPairs)(procedureInputs.optionsJsonSchema);
        return [procedurePath, { meta: getMeta(procedure), parsedProcedure: procedureInputs, incompatiblePairs, procedure }];
    });
};
// We're going to use eval to require some optional dependencies. It's hard-coded, so safe, but some bundlers like tsdown will emit warnings unless we disguise it.
const disguisedEval = eval;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const parseOrpcRouter = (params) => {
    const entries = [];
    const { traverseContractProcedures, isProcedure } = disguisedEval(`require('@orpc/server')`);
    const router = params.router;
    const lazyRoutes = traverseContractProcedures({ path: [], router }, ({ contract, path }) => {
        let procedure = params.router;
        for (const p of path)
            procedure = procedure[p];
        if (!isProcedure(procedure))
            return; // if it's contract-only, we can't run it via CLI (user may have passed an implemented contract router? should we tell them? it's undefined behaviour so kinda on them)
        const procedureInputsResult = (0, parse_procedure_1.parseProcedureInputs)([contract['~orpc'].inputSchema], {
            '@valibot/to-json-schema': params['@valibot/to-json-schema'],
            effect: params.effect,
        });
        const procedurePath = path.join('.');
        const procedureish = { _def: { meta: contract['~orpc'].meta } };
        const meta = getMeta(procedureish);
        if (meta.jsonInput) {
            entries.push([procedurePath, { meta, parsedProcedure: jsonProcedureInputs(), incompatiblePairs: [], procedure }]);
            return;
        }
        if (!procedureInputsResult.success) {
            const parsedProcedure = jsonProcedureInputs(`procedure's schema couldn't be converted to CLI arguments: ${procedureInputsResult.error}`);
            entries.push([procedurePath, { meta, parsedProcedure: parsedProcedure, incompatiblePairs: [], procedure }]);
            return;
        }
        const parsedProcedure = procedureInputsResult.value;
        const incompatiblePairs = (0, json_schema_1.incompatiblePropertyPairs)(parsedProcedure.optionsJsonSchema);
        entries.push([procedurePath, { procedure, meta, incompatiblePairs, parsedProcedure }]);
    });
    if (lazyRoutes.length) {
        const suggestion = `Please use \`import {unlazyRouter} from '@orpc/server'\` to unlazy the router before passing it to trpc-cli`;
        const routes = lazyRoutes.map(({ path }) => path.join('.')).join(', ');
        throw new Error(`Lazy routers are not supported. ${suggestion}. Lazy routes detected: ${routes}`);
    }
    return entries;
};
/** helper to create a "ParsedProcedure" that just accepts a JSON string - for when we failed to parse the input schema or the use set jsonInput: true */
const jsonProcedureInputs = (reason) => {
    let description = `Input formatted as JSON`;
    if (reason)
        description += ` (${reason})`;
    return {
        positionalParameters: [],
        optionsJsonSchema: {
            type: 'object',
            properties: {
                input: { description }, // omit `type` - this is json input, it could be anything
            },
        },
        getPojoInput: parsedCliParams => parsedCliParams.options.input,
    };
};
/**
 * Run a trpc router as a CLI.
 *
 * @param router A trpc router
 * @param context The context to use when calling the procedures - needed if your router requires a context
 * @param trpcServer The trpc server module to use. Only needed if using trpc v10.
 * @returns A CLI object with a `run` method that can be called to run the CLI. The `run` method will parse the command line arguments, call the appropriate trpc procedure, log the result and exit the process. On error, it will log the error and exit with a non-zero exit code.
 */
function createCli({ router, ...params }) {
    const procedureEntries = (0, exports.parseRouter)({ router, ...params });
    function buildProgram(runParams) {
        const logger = { ...logging_1.lineByLineConsoleLogger, ...runParams?.logger };
        const program = new Command(params.name);
        if (params.version)
            program.version(params.version);
        if (params.description)
            program.description(params.description);
        if (params.usage)
            [params.usage].flat().forEach(usage => program.usage(usage));
        program.showHelpAfterError();
        program.showSuggestionAfterError();
        // Organize commands in a tree structure for nested subcommands
        const commandTree = {
            '': program, // Root level
        };
        // Keep track of default commands for each parent path
        const defaultCommands = {};
        const _process = runParams?.process || process;
        const configureCommand = (command, procedurePath, { meta, parsedProcedure, incompatiblePairs, procedure }) => {
            const optionJsonSchemaProperties = (0, json_schema_1.flattenedProperties)(parsedProcedure.optionsJsonSchema);
            command.exitOverride(ec => {
                _process.exit(ec.exitCode);
                throw new errors_1.FailedToExitError(`Command ${command.name()} exitOverride`, { exitCode: ec.exitCode, cause: ec });
            });
            command.configureOutput({
                writeOut: str => {
                    logger.info?.(str);
                },
                writeErr: str => {
                    logger.error?.(str);
                },
            });
            command.showHelpAfterError();
            if (meta.usage)
                command.usage([meta.usage].flat().join('\n'));
            if (meta.examples)
                command.addHelpText('after', `\nExamples:\n${[meta.examples].flat().join('\n')}`);
            meta?.aliases?.command?.forEach(alias => {
                command.alias(alias);
            });
            command.description(meta?.description || '');
            parsedProcedure.positionalParameters.forEach(param => {
                const descriptionParts = [
                    param.type === 'string' ? '' : param.type, // "string" is the default assumption, don't bother showing it
                    param.description,
                    param.required ? '(required)' : '',
                ];
                const argument = new commander_1.Argument(param.name, descriptionParts.filter(Boolean).join(' '));
                if (param.type === 'number') {
                    argument.argParser(value => {
                        const number = numberParser(value, { fallback: null });
                        if (number == null)
                            throw new commander_1.InvalidArgumentError(`Invalid number: ${value}`);
                        return value;
                    });
                }
                argument.required = param.required;
                argument.variadic = param.array;
                command.addArgument(argument);
            });
            const unusedOptionAliases = { ...meta.aliases?.options };
            const addOptionForProperty = ([propertyKey, propertyValue]) => {
                const description = (0, json_schema_1.getDescription)(propertyValue);
                const longOption = `--${kebabCase(propertyKey)}`;
                let flags = longOption;
                const alias = propertyValue && 'alias' in propertyValue && typeof propertyValue.alias === 'string'
                    ? propertyValue.alias
                    : meta.aliases?.options?.[propertyKey];
                if (alias) {
                    let prefix = '-';
                    if (alias.startsWith('-'))
                        prefix = '';
                    else if (alias.length > 1)
                        prefix = '--';
                    flags = `${prefix}${alias}, ${flags}`;
                    delete unusedOptionAliases[propertyKey];
                }
                const allowedSchemas = (0, json_schema_1.getAllowedSchemas)(propertyValue);
                const firstSchemaWithDefault = allowedSchemas.find(subSchema => 'default' in subSchema);
                const defaultValue = firstSchemaWithDefault
                    ? { exists: true, value: firstSchemaWithDefault.default }
                    : { exists: false };
                const rootTypes = (0, json_schema_1.getSchemaTypes)(propertyValue).sort();
                const propertyType = rootTypes[0];
                const isValueRequired = 'required' in parsedProcedure.optionsJsonSchema &&
                    parsedProcedure.optionsJsonSchema.required?.includes(propertyKey);
                const isCliOptionRequired = isValueRequired && propertyType !== 'boolean' && !defaultValue.exists;
                function negate() {
                    const shouldNegate = 'negatable' in propertyValue ? propertyValue.negatable : meta.negateBooleans;
                    if (shouldNegate) {
                        const negation = new commander_1.Option(longOption.replace('--', '--no-'), `Negate \`${longOption}\` option.`.trim());
                        command.addOption(negation);
                    }
                }
                const bracketise = (name) => (isCliOptionRequired ? `<${name}>` : `[${name}]`);
                if (allowedSchemas.length > 1) {
                    const option = new commander_1.Option(`${flags} [value]`, description);
                    if (defaultValue.exists)
                        option.default(defaultValue.value);
                    else if (rootTypes.includes('boolean'))
                        option.default(false);
                    option.argParser(getOptionValueParser(propertyValue));
                    command.addOption(option);
                    if (rootTypes.includes('boolean'))
                        negate();
                    return;
                }
                if (rootTypes.length !== 1) {
                    const option = new commander_1.Option(`${flags} ${bracketise('json')}`, description);
                    option.argParser(getOptionValueParser(propertyValue));
                    command.addOption(option);
                    return;
                }
                if (propertyType === 'boolean') {
                    const option = new commander_1.Option(`${flags} [boolean]`, description);
                    option.argParser(value => booleanParser(value));
                    // don't set a default value of `false`, because `undefined` is accepted by the procedure
                    if (isValueRequired)
                        option.default(false);
                    else if (defaultValue.exists)
                        option.default(defaultValue.value);
                    command.addOption(option);
                    negate();
                    return;
                }
                let option = null;
                // eslint-disable-next-line unicorn/prefer-switch
                if (propertyType === 'string') {
                    option = new commander_1.Option(`${flags} ${bracketise('string')}`, description);
                }
                else if (propertyType === 'boolean') {
                    option = new commander_1.Option(flags, description);
                }
                else if (propertyType === 'number' || propertyType === 'integer') {
                    option = new commander_1.Option(`${flags} ${bracketise('number')}`, description);
                    option.argParser(value => numberParser(value, { fallback: null }));
                }
                else if (propertyType === 'array') {
                    option = new commander_1.Option(`${flags} [values...]`, description);
                    if (defaultValue.exists)
                        option.default(defaultValue.value);
                    else if (isValueRequired)
                        option.default([]);
                    const itemsSchema = 'items' in propertyValue ? propertyValue.items : {};
                    const itemEnumTypes = (0, json_schema_1.getEnumChoices)(itemsSchema);
                    if (itemEnumTypes?.type === 'string_enum') {
                        option.choices(itemEnumTypes.choices);
                    }
                    const itemParser = getOptionValueParser(itemsSchema);
                    if (itemParser) {
                        option.argParser((value, previous) => {
                            const parsed = itemParser(value);
                            return Array.isArray(previous) ? [...previous, parsed] : [parsed];
                        });
                    }
                }
                if (!option) {
                    option = new commander_1.Option(`${flags} [json]`, description);
                    option.argParser(value => parseJson(value, commander_1.InvalidOptionArgumentError));
                }
                if (defaultValue.exists && option.defaultValue !== defaultValue.value) {
                    option.default(defaultValue.value);
                }
                if (option.flags.includes('<')) {
                    option.makeOptionMandatory();
                }
                const enumChoices = (0, json_schema_1.getEnumChoices)(propertyValue);
                if (enumChoices?.type === 'string_enum') {
                    option.choices(enumChoices.choices);
                }
                option.conflicts(incompatiblePairs.flatMap(pair => {
                    const filtered = pair.filter(p => p !== propertyKey);
                    if (filtered.length === pair.length)
                        return [];
                    return filtered;
                }));
                command.addOption(option);
                if (propertyType === 'boolean')
                    negate(); // just in case we refactor the code above and don't handle booleans as a special case
            };
            Object.entries(optionJsonSchemaProperties).forEach(addOptionForProperty);
            const invalidOptionAliases = Object.entries(unusedOptionAliases).map(([option, alias]) => `${option}: ${alias}`);
            if (invalidOptionAliases.length) {
                throw new Error(`Invalid option aliases: ${invalidOptionAliases.join(', ')}`);
            }
            // Set the action for this command
            command.action(async (...args) => {
                program.__ran ||= [];
                program.__ran.push(command);
                const options = command.opts();
                if (args.at(-2) !== options) {
                    // This is a code bug and not recoverable. Will hopefully never happen but if commander totally changes their API this will break
                    throw new Error(`Unexpected args format, second last arg is not the options object`, { cause: args });
                }
                if (args.at(-1) !== command) {
                    // This is a code bug and not recoverable. Will hopefully never happen but if commander totally changes their API this will break
                    throw new Error(`Unexpected args format, last arg is not the Command instance`, { cause: args });
                }
                // the last arg is the Command instance itself, the second last is the options object, and the other args are positional
                const positionalValues = args.slice(0, -2);
                const input = parsedProcedure.getPojoInput({ positionalValues, options });
                const resolvedTrpcServer = await (params.trpcServer || trpcServer11);
                let caller;
                const deprecatedCreateCaller = Reflect.get(params, 'createCallerFactory');
                if (deprecatedCreateCaller) {
                    const message = `Using deprecated \`createCallerFactory\` option. Use \`trpcServer\` instead. e.g. \`createCli({router: myRouter, trpcServer: import('@trpc/server')})\``;
                    logger.error?.(message);
                    caller = deprecatedCreateCaller(router)(params.context);
                }
                else if ((0, trpc_compat_1.isOrpcRouter)(router)) {
                    const { call } = disguisedEval(`require('@orpc/server')`);
                    // create an object which acts enough like a trpc caller to be used for this specific procedure
                    caller = { [procedurePath]: (_input) => call(procedure, _input, { context: params.context }) };
                }
                else {
                    const createCallerFactor = resolvedTrpcServer.initTRPC.create().createCallerFactory;
                    caller = createCallerFactor(router)(params.context);
                }
                const result = await caller[procedurePath](input).catch(err => {
                    throw transformError(err, command);
                });
                command.__result = result;
                if (result != null)
                    logger.info?.(result);
            });
        };
        // Process each procedure and add as a command or subcommand
        procedureEntries.forEach(([procedurePath, commandConfig]) => {
            const segments = procedurePath.split('.');
            // Create the command path and ensure parent commands exist
            let currentPath = '';
            for (let i = 0; i < segments.length - 1; i++) {
                const segment = segments[i];
                const parentPath = currentPath;
                currentPath = currentPath ? `${currentPath}.${segment}` : segment;
                // Create parent command if it doesn't exist
                if (!commandTree[currentPath]) {
                    const parentCommand = commandTree[parentPath];
                    const newCommand = new Command(kebabCase(segment));
                    newCommand.showHelpAfterError();
                    parentCommand.addCommand(newCommand);
                    commandTree[currentPath] = newCommand;
                }
            }
            // Create the actual leaf command
            const leafName = segments.at(-1);
            const parentPath = segments.length > 1 ? segments.slice(0, -1).join('.') : '';
            const parentCommand = commandTree[parentPath];
            const leafCommand = new Command(leafName && kebabCase(leafName));
            configureCommand(leafCommand, procedurePath, commandConfig);
            parentCommand.addCommand(leafCommand);
            // Check if this command should be the default for its parent
            const meta = commandConfig.meta;
            if (meta.default === true) {
                // the parent will pass on its args straight to the child, which will validate them. the parent just blindly accepts anything.
                parentCommand.allowExcessArguments();
                parentCommand.allowUnknownOption();
                parentCommand.addHelpText('after', leafCommand.helpInformation());
                parentCommand.action(async () => {
                    await leafCommand.parseAsync([...parentCommand.args], { from: 'user' });
                });
                // ancestors need to support positional options to pass through the positional args
                // for (let ancestor = parentCommand.parent, i = 0; ancestor && i < 10; ancestor = ancestor.parent, i++) {
                //   ancestor.enablePositionalOptions()
                // }
                // parentCommand.passThroughOptions()
                defaultCommands[parentPath] = {
                    procedurePath: procedurePath,
                    config: commandConfig,
                    command: leafCommand,
                };
            }
        });
        // After all commands are added, generate descriptions for parent commands
        Object.entries(commandTree).forEach(([path, command]) => {
            // Skip the root command and leaf commands (which already have descriptions)
            // if (path === '' || command.commands.length === 0) return
            if (command.commands.length === 0)
                return;
            // Get the names of all direct subcommands
            const subcommandNames = command.commands.map(cmd => cmd.name());
            // Check if there's a default command for this path
            const defaultCommand = defaultCommands[path]?.command.name();
            // Format the subcommand list, marking the default one
            const formattedSubcommands = subcommandNames
                .map(name => (name === defaultCommand ? `${name} (default)` : name))
                .join(', ');
            // Get the existing description (might have been set by a default command)
            const existingDescription = command.description() || '';
            // Only add the subcommand list if it's not already part of the description
            const descriptionParts = [existingDescription, `Available subcommands: ${formattedSubcommands}`];
            command.description(descriptionParts.filter(Boolean).join('\n'));
        });
        return program;
    }
    const run = async (runParams, program = buildProgram(runParams)) => {
        if (!(0, util_2.looksLikeInstanceof)(program, 'Command'))
            throw new Error(`program is not a Command instance`);
        const opts = runParams?.argv ? { from: 'user' } : undefined;
        const argv = [...(runParams?.argv || process.argv)];
        const _process = runParams?.process || process;
        const logger = { ...logging_1.lineByLineConsoleLogger, ...runParams?.logger };
        program.exitOverride(exit => {
            _process.exit(exit.exitCode);
            throw new errors_1.FailedToExitError('Root command exitOverride', { exitCode: exit.exitCode, cause: exit });
        });
        program.configureOutput({
            writeOut: str => logger.info?.(str),
            writeErr: str => logger.error?.(str),
        });
        if (runParams?.completion) {
            const completion = typeof runParams.completion === 'function' ? await runParams.completion() : runParams.completion;
            (0, completions_1.addCompletions)(program, completion);
        }
        const formatError = runParams?.formatError ||
            ((err) => {
                if (err instanceof errors_1.CliValidationError) {
                    return err.message;
                }
                return (0, util_1.inspect)(err);
            });
        if (runParams?.prompts) {
            program = (0, prompts_1.promptify)(program, runParams.prompts);
        }
        await program.parseAsync(argv, opts).catch(err => {
            if (err instanceof errors_1.FailedToExitError)
                throw err;
            const logMessage = (0, util_2.looksLikeInstanceof)(err, Error)
                ? formatError(err) || err.message
                : `Non-error of type ${typeof err} thrown: ${err}`;
            logger.error?.(logMessage);
            _process.exit(1);
            throw new errors_1.FailedToExitError(`Program exit after failure`, { exitCode: 1, cause: err });
        });
        _process.exit(0);
        throw new errors_1.FailedToExitError('Program exit after success', {
            exitCode: 0,
            cause: program.__ran.at(-1)?.__result,
        });
    };
    return { run, buildProgram, toJSON: (program = buildProgram()) => (0, json_1.commandToJSON)(program) };
}
function getMeta(procedure) {
    const meta = procedure._def.meta;
    return meta?.cliMeta || meta || {};
}
function kebabCase(propName) {
    return propName.replaceAll(/([A-Z])/g, '-$1').toLowerCase();
}
/** @deprecated renamed to `createCli` */
exports.trpcCli = createCli;
function transformError(err, command) {
    if ((0, util_2.looksLikeInstanceof)(err, Error) && err.message.includes('This is a client-only function')) {
        return new Error('Failed to create trpc caller. If using trpc v10, either upgrade to v11 or pass in the `@trpc/server` module to `createCli` explicitly');
    }
    if ((0, util_2.looksLikeInstanceof)(err, 'TRPCError')) {
        const cause = err.cause;
        if ((0, utils_1.looksLikeStandardSchemaFailure)(cause)) {
            const prettyMessage = (0, errors_2.prettifyStandardSchemaError)(cause);
            return new errors_1.CliValidationError(prettyMessage + '\n\n' + command.helpInformation());
        }
        if (err.code === 'BAD_REQUEST' &&
            (err.cause?.constructor?.name === 'TraversalError' || // arktype error
                err.cause?.constructor?.name === 'StandardSchemaV1Error') // valibot error
        ) {
            return new errors_1.CliValidationError(err.cause.message + '\n\n' + command.helpInformation());
        }
        if (err.code === 'INTERNAL_SERVER_ERROR') {
            return cause;
        }
    }
    return err;
}
var errors_3 = require("./errors");
Object.defineProperty(exports, "FailedToExitError", { enumerable: true, get: function () { return errors_3.FailedToExitError; } });
Object.defineProperty(exports, "CliValidationError", { enumerable: true, get: function () { return errors_3.CliValidationError; } });
const numberParser = (val, { fallback = val } = {}) => {
    const number = Number(val);
    return Number.isNaN(number) ? fallback : number;
};
const booleanParser = (val, { fallback = val } = {}) => {
    if (val === 'true')
        return true;
    if (val === 'false')
        return false;
    return fallback;
};
const getOptionValueParser = (schema) => {
    const allowedSchemas = (0, json_schema_1.getAllowedSchemas)(schema)
        .slice()
        .sort((a, b) => String((0, json_schema_1.getSchemaTypes)(a)[0]).localeCompare(String((0, json_schema_1.getSchemaTypes)(b)[0])));
    const typesArray = allowedSchemas.flatMap(json_schema_1.getSchemaTypes);
    const types = new Set(typesArray);
    return (value) => {
        const definitelyPrimitive = typesArray.every(t => t === 'boolean' || t === 'number' || t === 'integer' || t === 'string');
        if (types.size === 0 || !definitelyPrimitive) {
            // parse this as JSON - too risky to fall back to a string because that will probably do the wrong thing if someone passes malformed JSON like `{"foo": 1,}` (trailing comma)
            const hint = `Malformed JSON. If passing a string, pass it as a valid JSON string with quotes (${JSON.stringify(value)})`;
            const parsed = parseJson(value, commander_1.InvalidOptionArgumentError, hint);
            if (!types.size)
                return parsed; // if types is empty, it means any type is allowed - e.g. for json input
            const jsonSchemaType = Array.isArray(parsed) ? 'array' : parsed === null ? 'null' : typeof parsed;
            if (!types.has(jsonSchemaType)) {
                throw new commander_1.InvalidOptionArgumentError(`Got ${jsonSchemaType} but expected ${[...types].join(' or ')}`);
            }
            return parsed;
        }
        if (types.has('boolean')) {
            const parsed = booleanParser(value, { fallback: null });
            if (typeof parsed === 'boolean')
                return parsed;
        }
        if (types.has('number')) {
            const parsed = numberParser(value, { fallback: null });
            if (typeof parsed === 'number')
                return parsed;
        }
        if (types.has('integer')) {
            const parsed = numberParser(value, { fallback: null });
            if (typeof parsed === 'number' && Number.isInteger(parsed))
                return parsed;
        }
        if (types.has('string')) {
            return value;
        }
        throw new commander_1.InvalidOptionArgumentError(`Got ${JSON.stringify(value)} but expected ${[...types].join(' or ')}`);
    };
};
const parseJson = (value, ErrorClass = commander_1.InvalidArgumentError, hint = `Malformed JSON.`) => {
    try {
        return JSON.parse(value);
    }
    catch {
        throw new ErrorClass(hint);
    }
};
