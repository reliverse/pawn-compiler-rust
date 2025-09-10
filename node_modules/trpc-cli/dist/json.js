"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.commandToJSON = void 0;
/**
 * Convert a commander `Command` instance to a JSON object.
 *
 * Note: in theory you could use this with any `Command` instance, it doesn't have
 * to be one built by `trpc-cli`. Implementing here because it's pretty simple to do and `commander` doesn't seem to provide a way to do it.
 *
 * Note: falsy values for strings are replaced with `undefined` in the output - e.g. if there's an empty description, it will be `undefined` in the output.
 */
const commandToJSON = (command) => {
    const json = {};
    const name = command.name();
    if (name)
        json.name = name;
    const version = command.version();
    if (version)
        json.version = version;
    const description = command.description();
    if (description)
        json.description = description;
    const usage = command.usage();
    if (usage)
        json.usage = usage;
    json.arguments = command.registeredArguments.map(arg => {
        const result = { name: arg.name() };
        result.variadic = arg.variadic;
        result.required = arg.required;
        if (arg.description)
            result.description = arg.description;
        if (arg.defaultValue)
            result.defaultValue = arg.defaultValue;
        if (arg.defaultValueDescription)
            result.defaultValueDescription = arg.defaultValueDescription;
        if (arg.argChoices)
            result.choices = arg.argChoices;
        return result;
    });
    json.options = command.options.map(o => {
        const result = { name: o.name() };
        result.required = o.required;
        result.optional = o.optional;
        result.negate = o.negate;
        result.variadic = o.variadic;
        if (o.flags)
            result.flags = o.flags;
        if (o.short)
            result.short = o.short;
        if (o.description)
            result.description = o.description;
        if (o.argChoices)
            result.choices = o.argChoices;
        const attributeName = o.attributeName();
        if (attributeName)
            result.attributeName = attributeName;
        if (o.defaultValue)
            result.defaultValue = o.defaultValue;
        if (o.defaultValueDescription)
            result.defaultValueDescription = o.defaultValueDescription;
        return result;
    });
    json.commands = command.commands.map(c => (0, exports.commandToJSON)(c));
    return json;
};
exports.commandToJSON = commandToJSON;
