"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.promptify = exports.createShadowCommand = void 0;
/* eslint-disable import-x/order */
const commander_1 = require("commander");
const parseUpstreamOptionInfo = (value) => {
    if (typeof value !== 'string' || !value.startsWith('{'))
        return null;
    try {
        const info = JSON.parse(value);
        if (info.typeName !== 'UpstreamOptionInfo')
            return null;
        return info;
    }
    catch {
        return null;
    }
};
const parseUpstreamArgumentInfo = (value) => {
    if (typeof value !== 'string' || !value.startsWith('{'))
        return null;
    try {
        const info = JSON.parse(value);
        if (info.typeName !== 'UpstreamArgumentInfo')
            return null;
        return info;
    }
    catch {
        return null;
    }
};
const getDefaultSubcommand = (command) => {
    // it'd be good if there was a better way to get the "default" subcommand
    const defaultChild = command.description().match(/Available subcommands:.* (\S+) \(default\)/)?.[1];
    return defaultChild ? command.commands.find(c => c.name() === defaultChild) : undefined;
};
const createShadowCommand = (command, onAnalyze) => {
    const shadow = new commander_1.Command(command.name());
    shadow.exitOverride();
    shadow.configureOutput({
        writeOut: () => { },
        writeErr: () => { },
    });
    const argumentsMap = new Map();
    const optionsMap = new Map();
    command.options.forEach(original => {
        const id = Date.now().toString() + Math.random().toString().slice(1);
        const shadowOption = new commander_1.Option(original.flags.replace('<', '[').replace('>', ']'), JSON.stringify([`id=${id}`, original.description]));
        const upstreamOptionInfo = { typeName: 'UpstreamOptionInfo', id, specified: false };
        shadowOption.default(JSON.stringify(upstreamOptionInfo));
        shadowOption.argParser(value => JSON.stringify({ ...upstreamOptionInfo, specified: true, value }));
        shadow.addOption(shadowOption);
        optionsMap.set(id, { shadow: shadowOption, original: original });
    });
    command.registeredArguments.forEach(original => {
        const shadowArgument = new commander_1.Argument(original.name(), original.description);
        const id = Date.now().toString() + Math.random().toString().slice(1);
        shadowArgument.argOptional();
        const upstreamArgumentInfo = { typeName: 'UpstreamArgumentInfo', id, specified: false };
        shadowArgument.default(JSON.stringify(upstreamArgumentInfo));
        shadowArgument.argParser(value => JSON.stringify({ ...upstreamArgumentInfo, specified: true, value }));
        shadow.addArgument(shadowArgument);
        argumentsMap.set(id, { shadow: shadowArgument, original: original });
    });
    const analysis = {
        command: { shadow, original: command },
        arguments: [],
        options: [],
    };
    shadow.action(async (...args) => {
        // the last arg is the Command instance itself, the second last is the options object, and the other args are positional
        const positionalValues = args.slice(0, -2);
        const options = shadow.opts();
        if (args.at(-2) !== options) {
            // This is a code bug and not recoverable. Will hopefully never happen but if commander totally changes their API this will break
            throw new Error(`Unexpected args format, second last arg is not the options object`, { cause: args });
        }
        if (args.at(-1) !== shadow) {
            // This is a code bug and not recoverable. Will hopefully never happen but if commander totally changes their API this will break
            throw new Error(`Unexpected args format, last arg is not the Command instance`, { cause: args });
        }
        positionalValues.forEach(value => {
            const argumentInfo = parseUpstreamArgumentInfo(value);
            if (argumentInfo) {
                analysis.arguments.push({
                    ...argumentsMap.get(argumentInfo.id),
                    value: argumentInfo.value,
                    specified: argumentInfo.specified,
                });
            }
        });
        Object.values(options).forEach(value => {
            const upstreamOptionInfo = parseUpstreamOptionInfo(value);
            if (upstreamOptionInfo) {
                analysis.options.push({
                    ...optionsMap.get(upstreamOptionInfo.id),
                    value: upstreamOptionInfo.value,
                    specified: upstreamOptionInfo.specified,
                });
            }
        });
        await onAnalyze(analysis);
    });
    command.commands.forEach(subcommand => {
        const shadowSubcommand = (0, exports.createShadowCommand)(subcommand, onAnalyze);
        shadow.addCommand(shadowSubcommand);
    });
    return shadow;
};
exports.createShadowCommand = createShadowCommand;
const inquirerPrompter = (prompts) => {
    return prompts; // the `satisfies` just makes sure it's safe to cast like this - if we accidentally add a method `@inquirer/prompts` doesn't have, this will fail.
};
const clackPrompter = (prompts) => {
    const clack = prompts;
    class ExitPromptError extends Error {
    } // we look for errors with this name specifically
    const throwOnCancel = (value) => {
        if (clack.isCancel(value))
            throw new ExitPromptError();
        return value;
    };
    return {
        input: async (params) => {
            return clack
                .text({
                message: params.message,
                initialValue: params.default,
                defaultValue: params.default,
                placeholder: params.default,
                validate: params.validate
                    ? input => {
                        const result = params.validate(input);
                        if (result === true)
                            return undefined;
                        if (result === false)
                            return `Invalid input`;
                        return result;
                    }
                    : undefined,
            })
                .then(throwOnCancel);
        },
        checkbox: async (params) => {
            return clack
                .multiselect({
                message: params.message,
                options: params.choices.map(c => ({
                    label: c.name,
                    value: c.value,
                })),
                initialValues: params.choices.flatMap(c => (c.checked ? [c.value] : [])),
            })
                .then(throwOnCancel);
        },
        confirm: async (params) => {
            return clack
                .confirm({
                message: params.message,
                initialValue: params.default,
            })
                .then(throwOnCancel);
        },
        select: async (params) => {
            return clack
                .select({
                message: params.message,
                options: params.choices.map(sorc => {
                    const c = typeof sorc === 'string' ? { name: sorc, value: sorc } : sorc;
                    return {
                        label: c.name,
                        value: c.value,
                        hint: c.description,
                    };
                }),
                initialValue: params.default,
            })
                .then(throwOnCancel);
        },
    };
};
const promptsPrompter = (prompts) => {
    const p = prompts;
    // weirdly prompts *demands* a name but doesn't show it anywhere, it just returns `{x: 'foo'}` instead of just returning `'foo'` 🤷
    function x() {
        return (value) => value.x;
    }
    return {
        input: async (params) => {
            return p({
                name: 'x',
                type: 'text',
                message: params.message,
                validate: params.validate,
                initial: params.default,
            }).then(x());
        },
        confirm: async (params) => {
            return p({
                name: 'x',
                type: 'confirm',
                message: params.message,
                active: params.default ? 'yes' : 'no',
            }).then(x());
        },
        select: async (params) => {
            const choicesObjects = params.choices.map(c => (typeof c === 'string' ? { name: c, value: c } : c));
            return p({
                name: 'x',
                type: 'select',
                message: params.message,
                active: params.default,
                choices: choicesObjects.map(c => ({
                    title: c.name || c.value,
                    value: c.value,
                })),
                initial: params.default ? choicesObjects.findIndex(c => c.value === params.default) : undefined,
            }).then(x());
        },
        checkbox: async (params) => {
            const choicesObjects = params.choices.map(c => (typeof c === 'string' ? { name: c, value: c } : c));
            return p({
                name: 'x',
                type: 'multiselect',
                message: params.message,
                choices: choicesObjects.map(c => ({
                    title: c.name || c.value,
                    value: c.value,
                    selected: c.checked,
                })),
            }).then(x());
        },
    };
};
const enquirerPrompter = (prompts) => {
    const enquirer = prompts;
    // weirdly enquirer *demands* a name but doesn't show it anywhere, it just returns `{x: 'foo'}` instead of just returning `'foo'` 🤷
    function x() {
        return (value) => value.x;
    }
    return {
        input: async (params) => {
            return enquirer
                .prompt({
                type: 'input',
                name: 'x',
                message: params.message,
                validate: params.validate,
                initial: params.default,
            })
                .then(x());
        },
        confirm: async (params) => {
            return enquirer
                .prompt({
                type: 'confirm',
                name: 'x',
                message: params.message,
                validate: params.validate,
                initial: params.default,
            })
                .then(x());
        },
        select: async (params) => {
            return enquirer
                .prompt({
                type: 'select',
                name: 'x',
                message: params.message,
                // @ts-expect-error not sure why this is an error, in the IDE it infers the type correctly
                choices: params.choices.slice(),
                validate: params.validate,
                initial: params.default,
            })
                .then(x());
        },
        checkbox: async (params) => {
            return enquirer
                .prompt({
                type: 'multiselect',
                name: 'x',
                message: params.message,
                // @ts-expect-error not sure why this is an error, in the IDE it infers the type correctly
                choices: params.choices.slice().map(c => ({
                    name: c.name,
                    value: c.value,
                })),
                // validate: params.validate ? v => params.validate!([{value: v}]) : undefined,
                initial: params.choices.flatMap((c, i) => (c.checked ? [i] : [])),
            })
                .then(x());
        },
    };
};
const promptify = (program, prompts) => {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any
    let promptsInput = prompts;
    if (promptsInput?.default)
        promptsInput = promptsInput.default;
    let prompter;
    if (typeof promptsInput === 'function' && typeof promptsInput.inject === 'function') {
        prompter = promptsPrompter(promptsInput);
    }
    else if (promptsInput?.name === 'Enquirer') {
        prompter = enquirerPrompter(promptsInput);
    }
    else if (typeof promptsInput?.rawlist === 'function') {
        prompter = inquirerPrompter(promptsInput);
    }
    else if (typeof promptsInput?.intro === 'function') {
        prompter = clackPrompter(promptsInput);
    }
    else if (typeof promptsInput === 'function') {
        prompter = promptsInput(program); // some kind of custom prompter-getter
    }
    else {
        prompter = promptsInput; // some kind of custom prompter
    }
    const command = program;
    const analyseThenParse = async (argv, parseOptions) => {
        // eslint-disable-next-line no-console
        if (parseOptions?.from === 'electron') {
            // eslint-disable-next-line no-console
            console.warn(`Warning: using prompts in electron mode is untested. The first two args of $0 are not available in electron mode. Assuming that the first two args of ${JSON.stringify(argv)} are electron-related and not intended for the CLI.`);
        }
        if (parseOptions?.from !== 'user') {
            argv = argv.slice(2);
            parseOptions = { from: 'user' };
        }
        const f = { command, args: [...argv] }; // await figureOutCommandAndArgs({command, args: [...argv]})
        const nextArgv = [...f.args];
        let analysis = undefined;
        const maxAttempts = 100;
        for (let i = maxAttempts; i >= 0 && !analysis; i--) {
            // try to get analysis - we should epxect this to remain undefined only if the user hasn't specified a command. i.e. they're just running the root program
            // and are expecting to be prompted for a command.
            analysis = await new Promise((resolve, reject) => {
                const shadow = (0, exports.createShadowCommand)(f.command, async (an) => {
                    if (an.command.original.commands.length === 0) {
                        // no subcommands, let's pass this on straight to the original
                        resolve(an);
                        return;
                    }
                    const defaultSubcommand = getDefaultSubcommand(an.command.original);
                    if (defaultSubcommand) {
                        // there's a default subcommand, the original should have an action which passes through args to the default child. So we're done here.
                        // note that this means no prompting if you build `yarn` with this library, and then the end-user just runs `yarn` (because there's a default subcommand)
                        resolve(an);
                        return;
                    }
                    // ok, the user hasn't actually specified a subcommand, let's prompt them for one, add it on to the args
                    const name = await prompter.select({
                        message: `Select a ${an.command.original.name() || ''} subcommand`.replace('  ', ' '),
                        choices: an.command.original.commands.map(c => ({
                            name: c.name(),
                            value: c.name(),
                            description: c.description(),
                        })),
                    }, {});
                    // push onto the _end_ of the args, because at this point we know there are no subcommands, so hopefully there are no ambiguous option/flags interfering with parsing.
                    nextArgv.push(name);
                    // resolve with undefined - we'll have to re-parse now that we've got a subcommand added to the args
                    resolve(undefined);
                });
                shadow.parseAsync(nextArgv, parseOptions).catch(e => {
                    if (e?.constructor?.name === 'CommanderError') {
                        // CommanderError is thrown when user passes `--help`, or tries to use an unknown option.
                        // We want to suppress the "shadow" version of this error, so just pass on no analysis and the rest of the flow will call the original program
                        // with the same argument, and presumably get the same error but with more helpful output from the "real" program.
                        resolve({
                            command: { shadow: f.command, original: f.command },
                            arguments: [],
                            options: [],
                        });
                    }
                    else {
                        reject(e);
                    }
                });
            });
        }
        if (!analysis) {
            const message = `Failed to find a subcommand after ${maxAttempts} attempts - failing to avoid an infinite loop. This is probably a bug in trpc-cli.`;
            throw new Error(message);
        }
        const getMessage = (argOrOpt) => {
            const name = 'long' in argOrOpt ? argOrOpt.flags : `[${argOrOpt.name()}]`;
            const parts = [
                name,
                argOrOpt.description,
                argOrOpt.defaultValue && `(default: ${argOrOpt.defaultValue})`,
                !argOrOpt.defaultValue && !argOrOpt.required && '(optional)',
            ];
            return parts.filter(Boolean).join(' ').trim() + ':';
        };
        const baseContext = {
            command: analysis.command.original,
            inputs: {
                argv,
                arguments: analysis.arguments.map(a => ({ name: a.original.name(), specified: a.specified, value: a.value })),
                options: analysis.options.map(o => ({ name: o.original.name(), specified: o.specified, value: o.value })),
            },
        };
        await prompter.setup?.(baseContext);
        // one day maybe consider making this more configurable by the user (or rather the package user -- the developer)
        // it's kinda complicated. rn 'necessary' means only prompt if something's unspecified and required.
        // but it'll then prompt for all unspecified args, even if they're not required.
        const promptConfig = 'necessary';
        let shouldPrompt;
        if (promptConfig === 'always') {
            shouldPrompt = true;
        }
        else if (promptConfig === 'never') {
            shouldPrompt = false;
        }
        else {
            promptConfig;
            const someRequiredArgsUnspecified = analysis.arguments.some(a => a.original.required && !a.specified);
            const someRequiredOptionsUnspecified = analysis.options.some(o => o.original.required && !o.specified);
            shouldPrompt = someRequiredArgsUnspecified || someRequiredOptionsUnspecified;
        }
        if (shouldPrompt) {
            for (const arg of analysis.arguments) {
                const ctx = { ...baseContext, argument: arg.original };
                if (!arg.specified) {
                    const parseArg = 'parseArg' in arg.original && typeof arg.original.parseArg === 'function'
                        ? arg.original.parseArg
                        : undefined;
                    const promptedValue = await prompter.input({
                        message: getMessage(arg.original),
                        required: arg.original.required,
                        default: arg.value,
                        validate: input => {
                            try {
                                parseArg?.(input);
                                return true;
                            }
                            catch (e) {
                                return `${e?.message || e}`;
                            }
                        },
                    }, ctx);
                    nextArgv.push(promptedValue);
                }
            }
            for (const option of analysis.options) {
                const ctx = { ...baseContext, option: option.original };
                if (!option.specified) {
                    const fullFlag = option.original.long || `--${option.original.name()}`;
                    const isBoolean = option.original.isBoolean() || option.original.flags.includes('[boolean]');
                    if (isBoolean) {
                        const promptedValue = await prompter.confirm({
                            message: getMessage(option.original),
                            default: option.original.defaultValue ?? false,
                        }, ctx);
                        if (promptedValue)
                            nextArgv.push(fullFlag);
                    }
                    else if (option.original.variadic && option.original.argChoices) {
                        const choices = option.original.argChoices.slice();
                        const results = await prompter.checkbox({
                            message: getMessage(option.original),
                            choices: choices.map(choice => ({
                                value: choice,
                                name: choice,
                                checked: true,
                            })),
                        }, ctx);
                        results.forEach(result => {
                            if (typeof result === 'string')
                                nextArgv.push(fullFlag, result);
                        });
                    }
                    else if (option.original.argChoices) {
                        const choices = option.original.argChoices.slice();
                        const set = new Set(choices);
                        const promptedValue = await prompter.select({
                            message: getMessage(option.original),
                            choices,
                            default: option.original.defaultValue,
                            // required: option.original.required,
                        }, ctx);
                        if (set.has(promptedValue)) {
                            nextArgv.push(fullFlag, promptedValue);
                        }
                    }
                    else if (option.original.variadic) {
                        const values = [];
                        do {
                            const promptedValue = await prompter.input({
                                message: getMessage(option.original),
                                default: option.original.defaultValue?.[values.length],
                            }, ctx);
                            if (!promptedValue)
                                break;
                            values.push(fullFlag, promptedValue);
                        } while (values);
                        nextArgv.push(...values);
                    }
                    else {
                        // let's handle this as a string - but the `parseArg` function could turn it into a number or boolean or whatever
                        const getParsedValue = (input) => {
                            return option.original.parseArg ? option.original.parseArg(input, undefined) : input;
                        };
                        const promptedValue = await prompter.input({
                            message: getMessage(option.original),
                            default: option.value,
                            required: option.original.required,
                            validate: input => {
                                const parsed = getParsedValue(input);
                                if (parsed == null && input != null)
                                    return 'Invalid value';
                                return true;
                            },
                        }, ctx);
                        if (promptedValue)
                            nextArgv.push(fullFlag, getParsedValue(promptedValue) ?? promptedValue);
                    }
                }
            }
        }
        await prompter.teardown?.(baseContext);
        return f.command.parseAsync(nextArgv, parseOptions);
    };
    const parseAsync = (args, parseOptions) => analyseThenParse(args, parseOptions).catch(e => {
        if (e?.constructor?.name === 'ExitPromptError')
            return; // https://github.com/SBoudrias/Inquirer.js?tab=readme-ov-file#handling-ctrlc-gracefully
        throw e;
    });
    return new Proxy(program, {
        get(target, prop, receiver) {
            if (prop === 'parseAsync')
                return parseAsync;
            return Reflect.get(target, prop, receiver);
        },
    });
};
exports.promptify = promptify;
