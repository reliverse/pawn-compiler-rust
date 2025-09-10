import { Command } from 'commander';
/**
 * JSON representation of a commander `Command` instance
 * Note: this is not necessarily a _complete_ representation of the command - it aims to be a big enough subset to be useful for generating documentation etc.
 */
export type CommandJSON = {
    name?: string;
    version?: string;
    description?: string;
    usage?: string;
    commands?: CommandJSON[];
    arguments?: {
        name: string;
        description?: string;
        required: boolean;
        defaultValue?: {};
        defaultValueDescription?: string;
        variadic: boolean;
        choices?: string[];
    }[];
    options?: {
        name: string;
        description?: string;
        required: boolean;
        defaultValue?: {};
        defaultValueDescription?: string;
        variadic: boolean;
        attributeName?: string;
        flags?: string;
        short?: string;
        negate: boolean;
        optional: boolean;
        choices?: string[];
    }[];
};
/**
 * Convert a commander `Command` instance to a JSON object.
 *
 * Note: in theory you could use this with any `Command` instance, it doesn't have
 * to be one built by `trpc-cli`. Implementing here because it's pretty simple to do and `commander` doesn't seem to provide a way to do it.
 *
 * Note: falsy values for strings are replaced with `undefined` in the output - e.g. if there's an empty description, it will be `undefined` in the output.
 */
export declare const commandToJSON: (command: Command) => CommandJSON;
